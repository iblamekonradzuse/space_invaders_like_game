use rand::Rng;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::{self, stdout, Write};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
use termion::color;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;

// Define game constants
const WIDTH: usize = 60;
const HEIGHT: usize = 30;
const LASER_HITBOX_WIDTH: usize = 3; // New constant for laser hitbox width

// Game struct to hold all game state
struct Game {
    player: usize,
    enemies: Vec<(usize, usize, char, usize)>, // (x, y, type, color)
    bullets: Vec<(usize, usize)>,
    powerups: Vec<(usize, usize, char)>, // (x, y, type)
    explosions: Vec<(usize, usize, u8)>, // (x, y, frame)
    score: u32,
    high_score: u32,
    level: usize,
    lives: usize,
    enemy_move_counter: usize,
    powerup_active: Option<char>,
    powerup_timer: u8,
    start_time: Instant,
    last_powerup_time: Instant,
    powerup_move_counter: usize,
    paused: bool,
}

impl Game {
    // Initialize a new game
    fn new() -> Self {
        let high_score = Game::load_high_score();
        Game {
            player: WIDTH / 2,
            enemies: Vec::new(),
            bullets: Vec::new(),
            powerups: Vec::new(),
            explosions: Vec::new(),
            score: 0,
            high_score,
            level: 1,
            lives: 3,
            enemy_move_counter: 0,
            powerup_active: None,
            powerup_timer: 0,
            start_time: Instant::now(),
            last_powerup_time: Instant::now(),
            powerup_move_counter: 0,
            paused: false,
        }
    }

    // Load the high score from a file
    fn load_high_score() -> u32 {
        if let Ok(mut file) = OpenOptions::new().read(true).open("high_score.txt") {
            let mut content = String::new();
            if file.read_to_string(&mut content).is_ok() {
                if let Ok(score) = content.trim().parse() {
                    return score;
                }
            }
        }
        0
    }

    // Save the high score to a file
    fn save_high_score(&self) {
        if self.score > self.high_score {
            if let Ok(mut file) = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open("high_score.txt")
            {
                let _ = write!(file, "{}", self.score);
            }
        }
    }

    // Create enemies based on the current level
    fn create_enemies(&self) -> Vec<(usize, usize, char, usize)> {
        let mut enemies = Vec::new();
        let mut rng = rand::thread_rng();
        let rows = 1 + self.level / 3;
        let cols = 3 + self.level / 3;

        for row in 0..rows {
            for col in 0..cols {
                let enemy_type = match rng.gen_range(0..4) {
                    0 => 'Z', // Zigzag
                    1 => 'W', // Wave
                    2 => 'D', // Diagonal
                    _ => 'H', // Health (new enemy type)
                };
                let color = rng.gen_range(1..5);
                enemies.push((
                    col * (WIDTH / (cols + 1)) + 5,
                    row * 2 + 3,
                    enemy_type,
                    color,
                ));
            }
        }
        enemies
    }

    // Create a powerup
    fn create_powerup(&mut self) {
        let mut rng = rand::thread_rng();
        // Only create a powerup if 30 seconds have passed and there are no existing powerups
        if self.last_powerup_time.elapsed() >= Duration::from_secs(30) && self.powerups.is_empty() {
            let powerup_type = match rng.gen_range(0..3) {
                0 => 'B', // Bigger Laser
                1 => 'M', // Multi-directional Laser
                _ => 'S', // Shield
            };
            self.powerups
                .push((rng.gen_range(0..WIDTH), 0, powerup_type));
            self.last_powerup_time = Instant::now();
        }
    }

    // Update game state
    fn update(&mut self) {
        if self.paused {
            return;
        }

        // Handle powerup timer
        if let Some(_) = self.powerup_active {
            if self.powerup_timer > 0 {
                self.powerup_timer -= 1;
            } else {
                self.powerup_active = None;
            }
        }

        // Move bullets and check for collisions with powerups
        self.bullets.retain_mut(|bullet| {
            bullet.1 = bullet.1.saturating_sub(1);

            // Check for collisions with powerups
            self.powerups.retain(|powerup| {
                if bullet.0 == powerup.0 && bullet.1 == powerup.1 {
                    self.powerup_active = Some(powerup.2);
                    self.powerup_timer = 100; // Lasts for a few seconds
                    false // Remove the powerup
                } else {
                    true // Keep the powerup
                }
            });

            bullet.1 > 0
        });

        // Move powerups
        self.powerup_move_counter += 1;
        if self.powerup_move_counter >= 20 - self.level.min(15) {
            self.powerup_move_counter = 0;
            for powerup in &mut self.powerups {
                powerup.1 += 1;
            }
            self.powerups.retain(|powerup| powerup.1 < HEIGHT);
        }

        // Check for collisions
        let initial_enemy_count = self.enemies.len();
        self.enemies.retain(|&enemy| {
            let mut hit = false;
            for bullet in &self.bullets {
                // Check if the enemy is within the bullet's hitbox
                if (bullet.0.saturating_sub(LASER_HITBOX_WIDTH / 2)..=bullet.0.saturating_add(LASER_HITBOX_WIDTH / 2))
                    .contains(&enemy.0) && bullet.1 == enemy.1
                {
                    hit = true;
                    self.explosions.push((enemy.0, enemy.1, 0));
                    if enemy.2 == 'H' {
                        self.lives = (self.lives + 1).min(5); // Cap lives at 5
                    }
                    break;
                }
            }
            !hit
        });
        let enemies_destroyed = initial_enemy_count - self.enemies.len();
        self.score += enemies_destroyed as u32 * 10;

        // Move enemies
        self.enemy_move_counter += 1;
        if self.enemy_move_counter >= 20 - self.level.min(15) {
            self.enemy_move_counter = 0;
            if self.enemies.is_empty() {
                self.level += 1;
                self.enemies = self.create_enemies();
            } else {
                for enemy in &mut self.enemies {
                    match enemy.2 {
                        'Z' => {
                            enemy.0 =
                                (enemy.0 + if enemy.1 % 4 < 2 { 1 } else { WIDTH - 1 }) % WIDTH;
                            enemy.1 += 1;
                        }
                        'W' => {
                            enemy.0 = (enemy.0 + (enemy.1 as f32 / 2.0).sin() as usize + 1) % WIDTH;
                            enemy.1 += 1;
                        }
                        'D' => {
                            enemy.0 = (enemy.0 + 1) % WIDTH;
                            enemy.1 += 1;
                        }
                        'H' => {
                            enemy.1 += 1;
                        }
                        _ => {}
                    }
                    if enemy.1 >= HEIGHT - 1 {
                        self.lives = self.lives.saturating_sub(1);
                        self.enemies = self.create_enemies();
                        break;
                    }
                }
            }
        }

        // Move explosions
        for explosion in &mut self.explosions {
            explosion.2 += 1;
        }
        self.explosions.retain(|explosion| explosion.2 < 3);

        // Create powerups
        self.create_powerup();
    }

    // Render the game state as a string
    fn render(&self) -> String {
        let mut output = String::new();
        
        // Only display the top info bar if the game is not paused
        if !self.paused {
            let elapsed = self.start_time.elapsed();
            let minutes = elapsed.as_secs() / 60;
            let seconds = elapsed.as_secs() % 60;

            output.push_str(&format!(
                "{}Score: {} | High Score: {} | Level: {} | Lives: {} | Time: {:02}:{:02}{}\r\n",
                color::Fg(color::Yellow),
                self.score,
                self.high_score,
                self.level,
                "♥".repeat(self.lives),
                minutes,
                seconds,
                color::Fg(color::Reset)
            ));
        }

        let mut screen = vec![vec![' '; WIDTH]; HEIGHT];

        // Draw player
        if !self.paused {
            screen[HEIGHT - 1][self.player] = 'A';
        }

        // Draw enemies
        if !self.paused {
            for &(x, y, enemy_type, _color) in &self.enemies {
                if y < HEIGHT {
                    screen[y][x] = enemy_type;
                }
            }
        }

        // Draw bullets
        if !self.paused {
            for &(x, y) in &self.bullets {
                if y < HEIGHT {
                    screen[y][x] = '|';
                }
            }
        }

        // Draw powerups
        if !self.paused {
            for &(x, y, powerup_type) in &self.powerups {
                if y < HEIGHT {
                    screen[y][x] = powerup_type;
                }
            }
        }

        // Draw explosions
        if !self.paused {
            for &(x, y, frame) in &self.explosions {
                if y < HEIGHT {
                    screen[y][x] = match frame {
                        0 => '*',
                        1 => '+',
                        _ => ' ',
                    };
                }
            }
        }

        // Convert screen to string with colors
        for (_y, row) in screen.iter().enumerate() {
            for (_x, &ch) in row.iter().enumerate() {
                match ch {
                    'A' => output.push_str(&format!("{}", color::Fg(color::Blue))),
                    'Z' => output.push_str(&format!("{}", color::Fg(color::LightRed))),
                    'W' => output.push_str(&format!("{}", color::Fg(color::LightMagenta))),
                    'D' => output.push_str(&format!("{}", color::Fg(color::LightYellow))),
                    'H' => output.push_str(&format!("{}", color::Fg(color::Green))),
                    '|' => output.push_str(&format!("{}", color::Fg(color::Green))),
                    '*' | '+' => output.push_str(&format!("{}", color::Fg(color::Red))),
                    'B' => output.push_str(&format!("{}", color::Fg(color::Cyan))),
                    'M' => output.push_str(&format!("{}", color::Fg(color::LightGreen))),
                    'S' => output.push_str(&format!("{}", color::Fg(color::LightBlue))),
                    _ => output.push_str(&format!("{}", color::Fg(color::Reset))),
                }
                output.push(ch);
            }
            output.push_str(&format!("{}\r\n", color::Fg(color::Reset)));
        }

        output
    }

    // Handle user input
    fn handle_input(&mut self, key: Key) {
        match key {
            Key::Left => {
                if !self.paused {
                    self.player = self.player.saturating_sub(1)
                }
            }
            Key::Right => {
                if !self.paused {
                    self.player = (self.player + 1).min(WIDTH - 1)
                }
            }
            Key::Char(' ') => {
                if !self.paused && self.bullets.len() < 3 {
                    // Limit the number of bullets
                    match self.powerup_active {
                        Some('B') => {
                            // Bigger Laser
                            self.bullets.push((self.player, HEIGHT - 2));
                            self.bullets
                                .push((self.player.saturating_sub(1), HEIGHT - 2));
                            self.bullets
                                .push(((self.player + 1).min(WIDTH - 1), HEIGHT - 2));
                        }
                        Some('M') => {
                            // Multi-directional Laser
                            self.bullets.push((self.player, HEIGHT - 2));
                            self.bullets
                                .push((self.player.saturating_sub(1), HEIGHT - 2));
                            self.bullets.push((self.player + 1, HEIGHT - 2));
                        }
                        _ => self.bullets.push((self.player, HEIGHT - 2)),
                    }
                }
            }
            Key::Char('p') | Key::Char('P') => {
                self.paused = !self.paused;
            }
            _ => {}
        }

        // Check for powerup collection
        if !self.paused {
            self.powerups.retain(|&powerup| {
                if powerup.0 == self.player && powerup.1 == HEIGHT - 1 {
                    self.powerup_active = Some(powerup.2);
                    self.powerup_timer = 100; // Lasts for a few seconds
                    return false;
                }
                true
            });
        }
    }
// Check if the game is over
    fn is_game_over(&self) -> bool {
        self.lives == 0
    }
}

// Display the start screen
fn display_start_screen(
    screen: &mut AlternateScreen<termion::raw::RawTerminal<std::io::Stdout>>,
) -> io::Result<()> {
    write!(screen, "{}", termion::clear::All)?;
    write!(
        screen,
        "{}{}{}✰✰✰ S P A C E ✰✰✰  {}",
        termion::cursor::Goto(10, 8),
        termion::style::Bold,
        color::Fg(color::Cyan),
        color::Fg(color::Reset)
    )?;
    write!(
        screen,
        "{}{}{}✰✰ I N V A D E R S ✰✰{}",
        termion::cursor::Goto(8, 9),
        termion::style::Bold,
        color::Fg(color::Cyan),
        color::Fg(color::Reset)
    )?;
    write!(
        screen,
        "{}{}Arrow keys to move,",
        termion::cursor::Goto(10, 13),
        color::Fg(color::Yellow)
    )?;
    write!(
        screen,
        "{}{} Space to shoot!",
        termion::cursor::Goto(10, 14),
        color::Fg(color::Yellow)
    )?;
    write!(
        screen,
        "{}{}Press 'P' to pause/unpause",
        termion::cursor::Goto(10, 15),
        color::Fg(color::Yellow)
    )?;
    write!(
        screen,
        "{}{}Press 'S' to start the game",
        termion::cursor::Goto(6, 21),
        color::Fg(color::Green)
    )?;
    write!(
        screen,
        "{}{}Press 'T' for tutorial",
        termion::cursor::Goto(9, 22),
        color::Fg(color::Blue)
    )?;
    write!(
        screen,
        "{}{}Press 'Q' to quit",
        termion::cursor::Goto(11, 23),
        color::Fg(color::Red)
    )?;
    screen.flush()?;
    Ok(())
}

// Display the tutorial screen
fn display_tutorial_screen(
    screen: &mut AlternateScreen<termion::raw::RawTerminal<std::io::Stdout>>,
) -> io::Result<()> {
    write!(screen, "{}", termion::clear::All)?;
    write!(
        screen,
        "{}{}{}Tutorial{}",
        termion::cursor::Goto(25, 2),
        termion::style::Bold,
        color::Fg(color::Cyan),
        color::Fg(color::Reset)
    )?;

    write!(
        screen,
        "{}{}Enemies:{}",
        termion::cursor::Goto(2, 4),
        color::Fg(color::Yellow),
        color::Fg(color::Reset)
    )?;
    write!(screen, "{}Z - Zigzag enemy", termion::cursor::Goto(4, 5))?;
    write!(screen, "{}W - Wave enemy", termion::cursor::Goto(4, 6))?;
    write!(screen, "{}D - Diagonal enemy", termion::cursor::Goto(4, 7))?;
    write!(
        screen,
        "{}H - Health enemy (gives extra life when destroyed)",
        termion::cursor::Goto(4, 8)
    )?;

    write!(
        screen,
        "{}{}Powerups:{}",
        termion::cursor::Goto(2, 10),
        color::Fg(color::Yellow),
        color::Fg(color::Reset)
    )?;
    write!(
        screen,
        "{}B - Bigger Laser (3-wide shot)",
        termion::cursor::Goto(4, 11)
    )?;
    write!(
        screen,
        "{}M - Multi-directional Laser (3-way shot)",
        termion::cursor::Goto(4, 12)
    )?;
    write!(
        screen,
        "{}S - Shield (temporary invincibility)",
        termion::cursor::Goto(4, 13)
    )?;

    write!(
        screen,
        "{}{}Controls:{}",
        termion::cursor::Goto(2, 15),
        color::Fg(color::Yellow),
        color::Fg(color::Reset)
    )?;
    write!(
        screen,
        "{}Left/Right Arrow - Move ship",
        termion::cursor::Goto(4, 16)
    )?;
    write!(screen, "{}Space - Shoot", termion::cursor::Goto(4, 17))?;
    write!(screen, "{}P - Pause/Unpause", termion::cursor::Goto(4, 18))?;

    write!(
        screen,
        "{}{}Press 'B' to return to the main menu",
        termion::cursor::Goto(2, 24),
        color::Fg(color::Green)
    )?;
    screen.flush()?;
    Ok(())
}

// Display the game over screen
fn display_game_over_screen(
    screen: &mut AlternateScreen<termion::raw::RawTerminal<std::io::Stdout>>,
    score: u32,
    level: usize,
    high_score: u32,
    time_survived: Duration,
) -> io::Result<()> {
    write!(screen, "{}", termion::clear::All)?;
    write!(
        screen,
        "{}{}{}Game Over!{}",
        termion::cursor::Goto(4, 6),
        termion::style::Bold,
        color::Fg(color::Red),
        color::Fg(color::Reset)
    )?;
    write!(
        screen,
        "{}{}Final Score: {}",
        termion::cursor::Goto(4, 9),
        color::Fg(color::Yellow),
        score
    )?;
    write!(
        screen,
        "{}{}Levels Completed: {}",
        termion::cursor::Goto(4, 10),
        color::Fg(color::Yellow),
        level - 1
    )?;
    write!(
        screen,
        "{}{}Time Survived: {:02}:{:02}",
        termion::cursor::Goto(4, 11),
        color::Fg(color::Yellow),
        time_survived.as_secs() / 60,
        time_survived.as_secs() % 60
    )?;
    write!(
        screen,
        "{}{}High Score: {}",
        termion::cursor::Goto(4, 13),
        color::Fg(color::Cyan),
        high_score
    )?;
    write!(
        screen,
        "{}{}Press 'R' to play again",
        termion::cursor::Goto(4, 15),
        color::Fg(color::Green)
    )?;
    write!(
        screen,
        "{}{}Press 'Q' to quit",
        termion::cursor::Goto(4, 16),
        color::Fg(color::Red)
    )?;
    screen.flush()?;
    Ok(())
}

// Display the pause screen
fn display_pause_screen(
    screen: &mut AlternateScreen<termion::raw::RawTerminal<std::io::Stdout>>,
) -> io::Result<()> {
    write!(
        screen,
        "{}{}{}GAME PAUSED{}",
        termion::cursor::Goto(((WIDTH / 2) - 5).try_into().unwrap(), (HEIGHT / 2).try_into().unwrap()),
        termion::style::Bold,
        color::Fg(color::Yellow),
        color::Fg(color::Reset)
    )?;
    write!(
        screen,
        "{}{}Press 'P' to resume",
        termion::cursor::Goto(((WIDTH / 2) - 9).try_into().unwrap(), ((HEIGHT / 2) + 2).try_into().unwrap()),
        color::Fg(color::Green)
    )?;
    screen.flush()?;
    Ok(())
}

// Main function to run the game
fn main() -> io::Result<()> {
    // Set up the terminal screen
    let mut screen = AlternateScreen::from(stdout().into_raw_mode()?);
    let (tx, rx) = mpsc::channel();

    // Spawn a thread to handle user input
    thread::spawn(move || {
        let stdin = io::stdin();
        for key in stdin.keys() {
            if let Ok(key) = key {
                if tx.send(key).is_err() {
                    return;
                }
            }
        }
    });

    'main_loop: loop {
        // Display the start screen
        display_start_screen(&mut screen)?;

        // Wait for the user to start the game, view tutorial, or quit
        loop {
            if let Ok(key) = rx.recv() {
                match key {
                    Key::Char('s') | Key::Char('S') => break,
                    Key::Char('t') | Key::Char('T') => {
                        display_tutorial_screen(&mut screen)?;
                        loop {
                            if let Ok(key) = rx.recv() {
                                if let Key::Char('b') | Key::Char('B') = key {
                                    break;
                                }
                            }
                        }
                        display_start_screen(&mut screen)?;
                    }
                    Key::Char('q') | Key::Char('Q') => break 'main_loop,
                    _ => {}
                }
            }
        }

        // Initialize the game
        let mut game = Game::new();
        game.enemies = game.create_enemies();
        let mut last_update = Instant::now();

        // Main game loop
        'game_loop: loop {
            // Update game state every 50ms
            if last_update.elapsed() >= Duration::from_millis(50) {
                game.update();
                write!(screen, "{}{}", termion::clear::All, game.render())?;
                if game.paused {
                    display_pause_screen(&mut screen)?;
                }
                screen.flush()?;
                last_update = Instant::now();

                // Check if the game is over
                if game.is_game_over() {
                    game.save_high_score();
                    break 'game_loop;
                }
            }

            // Handle user input
            if let Ok(key) = rx.try_recv() {
                match key {
                    Key::Ctrl('c') => break 'main_loop,
                    key => game.handle_input(key),
                }
            }

            // Small sleep to prevent CPU hogging
            thread::sleep(Duration::from_millis(10));
        }

        // Display game over screen
        let time_survived = game.start_time.elapsed();
        display_game_over_screen(
            &mut screen,
            game.score,
            game.level,
            game.high_score,
            time_survived,
        )?;

        // Wait for the user to restart or quit
        loop {
            if let Ok(key) = rx.recv() {
                match key {
                    Key::Char('r') | Key::Char('R') => break,
                    Key::Char('q') | Key::Char('Q') => break 'main_loop,
                    _ => {}
                }
            }
        }
    }

    Ok(())
}
