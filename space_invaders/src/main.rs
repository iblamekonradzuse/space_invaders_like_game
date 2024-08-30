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

const WIDTH: usize = 60;
const HEIGHT: usize = 30;

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
}

impl Game {
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
            lives: 1,
            enemy_move_counter: 0,
            powerup_active: None,
            powerup_timer: 0,
        }
    }

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

    fn create_enemies(&self) -> Vec<(usize, usize, char, usize)> {
        let mut enemies = Vec::new();
        let mut rng = rand::thread_rng();
        let rows = 1 + self.level / 3;
        let cols = 3 + self.level / 3;

        for row in 0..rows {
            for col in 0..cols {
                let enemy_type = match rng.gen_range(0..3) {
                    0 => 'Z', // Zigzag
                    1 => 'W', // Wave
                    _ => 'D', // Diagonal
                };
                let color = rng.gen_range(1..4);
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

    fn create_powerup(&mut self) {
        let mut rng = rand::thread_rng();
        if rng.gen_bool(0.1) {
            let powerup_type = match rng.gen_range(0..3) {
                0 => 'B', // Bigger Laser
                1 => 'M', // Multi-directional Laser
                _ => 'S', // Shield
            };
            self.powerups
                .push((rng.gen_range(0..WIDTH), 0, powerup_type));
        }
    }

    fn update(&mut self) {
        // Handle powerup timer
        if let Some(_) = self.powerup_active {
            if self.powerup_timer > 0 {
                self.powerup_timer -= 1;
            } else {
                self.powerup_active = None;
            }
        }

        // Move bullets
        self.bullets.retain_mut(|bullet| {
            bullet.1 = bullet.1.saturating_sub(1);
            bullet.1 > 0
        });

        // Move powerups
        for powerup in &mut self.powerups {
            powerup.1 += 1;
        }
        self.powerups.retain(|powerup| powerup.1 < HEIGHT);

        // Check for collisions
        let initial_enemy_count = self.enemies.len();
        self.enemies.retain(|&enemy| {
            let mut hit = false;
            for bullet in &self.bullets {
                if bullet.0 == enemy.0 && bullet.1 == enemy.1 {
                    hit = true;
                    self.explosions.push((enemy.0, enemy.1, 0));
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
                        'Z' => enemy.0 = (enemy.0 + 1) % WIDTH,
                        'W' => enemy.0 = (enemy.0 + 1) % WIDTH,
                        'D' => {
                            enemy.0 = (enemy.0 + 1) % WIDTH;
                            enemy.1 += 1;
                        }
                        _ => {}
                    }
                    enemy.1 += 1;
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
    }

    fn render(&self) -> String {
        let mut output = format!(
            "{}Score: {} | High Score: {} | Level: {} | Lives: {}{}\r\n",
            color::Fg(color::Yellow),
            self.score,
            self.high_score,
            self.level,
            "♥".repeat(self.lives),
            color::Fg(color::Reset)
        );
        let mut screen = vec![vec![' '; WIDTH]; HEIGHT];

        // Draw player
        screen[HEIGHT - 1][self.player] = 'A';

        // Draw enemies
        for &(x, y, enemy_type, color) in &self.enemies {
            if y < HEIGHT {
                screen[y][x] = enemy_type;
            }
        }

        // Draw bullets
        for &(x, y) in &self.bullets {
            if y < HEIGHT {
                screen[y][x] = '|';
            }
        }

        // Draw powerups
        for &(x, y, powerup_type) in &self.powerups {
            if y < HEIGHT {
                screen[y][x] = powerup_type;
            }
        }

        // Draw explosions
        for &(x, y, frame) in &self.explosions {
            if y < HEIGHT {
                screen[y][x] = match frame {
                    0 => '*',
                    1 => '+',
                    _ => ' ',
                };
            }
        }

        // Convert screen to string with colors
        for (y, row) in screen.iter().enumerate() {
            for (x, &ch) in row.iter().enumerate() {
                match ch {
                    'A' => output.push_str(&format!("{}", color::Fg(color::Blue))),
                    'Z' => output.push_str(&format!("{}", color::Fg(color::LightRed))),
                    'W' => output.push_str(&format!("{}", color::Fg(color::LightMagenta))),
                    'D' => output.push_str(&format!("{}", color::Fg(color::LightYellow))),
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

    fn handle_input(&mut self, key: Key) {
        match key {
            Key::Left => self.player = self.player.saturating_sub(1),
            Key::Right => self.player = (self.player + 1).min(WIDTH - 1),
            Key::Char(' ') => {
                if self.bullets.len() < 3 {
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
            _ => {}
        }

        // Check for powerup collection
        self.powerups.retain(|&powerup| {
            if powerup.0 == self.player && powerup.1 == HEIGHT - 1 {
                self.powerup_active = Some(powerup.2);
                self.powerup_timer = 100; // Lasts for a few seconds
                return false;
            }
            true
        });
    }

    fn is_game_over(&self) -> bool {
        self.lives == 0
    }
}
// ssad
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
        "{}{}Arow keys to move,",
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
        "{}{}Press 'S' to start the game",
        termion::cursor::Goto(6, 21),
        color::Fg(color::Green)
    )?;
    write!(
        screen,
        "{}{}Press 'Q' to quit",
        termion::cursor::Goto(10, 22),
        color::Fg(color::Red)
    )?;
    screen.flush()?;
    Ok(())
}

fn display_game_over_screen(
    screen: &mut AlternateScreen<termion::raw::RawTerminal<std::io::Stdout>>,
    score: u32,
    level: usize,
    high_score: u32,
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
        "{}{}High Score: {}",
        termion::cursor::Goto(4, 12),
        color::Fg(color::Cyan),
        high_score
    )?;
    write!(
        screen,
        "{}{}Press 'R' to play again",
        termion::cursor::Goto(4, 14),
        color::Fg(color::Green)
    )?;
    write!(
        screen,
        "{}{}Press 'Q' to quit",
        termion::cursor::Goto(4, 15),
        color::Fg(color::Red)
    )?;
    screen.flush()?;
    Ok(())
}

fn main() -> io::Result<()> {
    let mut screen = AlternateScreen::from(stdout().into_raw_mode()?);
    let (tx, rx) = mpsc::channel();

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
        display_start_screen(&mut screen)?;

        loop {
            if let Ok(key) = rx.recv() {
                match key {
                    Key::Char('s') | Key::Char('S') => break,
                    Key::Char('q') | Key::Char('Q') => break 'main_loop,
                    _ => {}
                }
            }
        }

        let mut game = Game::new();
        game.enemies = game.create_enemies();
        let mut last_update = Instant::now();

        'game_loop: loop {
            if last_update.elapsed() >= Duration::from_millis(50) {
                game.update();
                write!(screen, "{}{}", termion::clear::All, game.render())?;
                screen.flush()?;
                last_update = Instant::now();

                if game.is_game_over() {
                    game.save_high_score();
                    break 'game_loop;
                }
            }

            if let Ok(key) = rx.try_recv() {
                match key {
                    Key::Ctrl('c') => break 'main_loop,
                    key => game.handle_input(key),
                }
            }

            thread::sleep(Duration::from_millis(10));
        }

        display_game_over_screen(&mut screen, game.score, game.level, game.high_score)?;

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
