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

mod display_game_over_screen;
mod display_pause_screen;
mod display_start_screen;
mod display_tutorial_screen;

use crate::display_game_over_screen::display_game_over_screen;
use crate::display_pause_screen::display_pause_screen;
use crate::display_start_screen::display_start_screen;
use crate::display_tutorial_screen::display_tutorial_screen;

// Define game constants
const WIDTH: usize = 60;
const HEIGHT: usize = 30;
const LASER_HITBOX_WIDTH: usize = 3;

// Game struct to hold all game state
struct Game {
    player: usize,
    enemies: Vec<Enemy>,
    bullets: Vec<(usize, usize, bool)>, // (x, y, is_enemy_bullet)
    powerups: Vec<(usize, usize, char)>,
    explosions: Vec<(usize, usize, u8)>,
    score: u32,
    high_score: u32,
    level: usize,
    lives: usize,
    enemy_move_counter: usize,
    powerup_active: Option<char>,
    powerup_timer: u8,
    start_time: Instant,
    last_powerup_time: Instant,
    last_health_enemy_time: Instant,
    powerup_move_counter: usize,
    paused: bool,
    boss: Option<Boss>,
}

struct Enemy {
    x: usize,
    y: usize,
    enemy_type: char,
    color: usize,
    health: u8,
    shoot_timer: u8,
}

struct Boss {
    x: usize,
    y: usize,
    health: u16,
    max_health: u16,
    phase: u8,
    shoot_timer: u8,
    direction: i8,
    move_timer: u32,
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
            level: 4,
            lives: 3,
            enemy_move_counter: 0,
            powerup_active: None,
            powerup_timer: 0,
            start_time: Instant::now(),
            last_powerup_time: Instant::now(),
            last_health_enemy_time: Instant::now(),
            powerup_move_counter: 0,
            paused: false,
            boss: None,
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
    fn create_enemies(&mut self) -> Vec<Enemy> {
        let mut enemies = Vec::new();
        let mut rng = rand::thread_rng();
        let rows = 1 + self.level / 3;
        let cols = 3 + self.level / 3;

        for row in 0..rows {
            for col in 0..cols {
                let enemy_type = match rng.gen_range(0..8) {
                    0 => 'Z', // Zigzag
                    1 => 'W', // Wave
                    2 => 'D', // Diagonal
                    3 => 'S', // Shooter
                    4 => 'T', // Teleporter
                    5 => 'F', // Fast
                    6 => 'B', // Bomber
                    _ => 'N', // Normal
                };
                let color = rng.gen_range(1..5);
                enemies.push(Enemy {
                    x: col * (WIDTH / (cols + 1)) + 5,
                    y: row * 2 + 3,
                    enemy_type,
                    color,
                    health: match enemy_type {
                        'S' | 'T' | 'F' => 2,
                        'B' => 3,
                        _ => 1,
                    },
                    shoot_timer: rng.gen_range(0..50),
                });
            }
        }

        // Add a health enemy if it's time
        if self.last_health_enemy_time.elapsed() >= Duration::from_secs(60) {
            enemies.push(Enemy {
                x: rng.gen_range(0..WIDTH),
                y: 0,
                enemy_type: 'H',
                color: 2,
                health: 1,
                shoot_timer: 0,
            });
            self.last_health_enemy_time = Instant::now();
        }

        enemies
    }

    // Create a powerup
    fn create_powerup(&mut self) {
        let mut rng = rand::thread_rng();
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

        // Move bullets and check for collisions
        self.bullets.retain_mut(|bullet| {
            if bullet.2 {
                // Enemy bullet
                bullet.1 += 1;
            } else {
                // Player bullet
                bullet.1 = bullet.1.saturating_sub(1);
            }

            // Check for collisions with powerups
            self.powerups.retain(|powerup| {
                if bullet.0 == powerup.0 && bullet.1 == powerup.1 && !bullet.2 {
                    self.powerup_active = Some(powerup.2);
                    self.powerup_timer = 100;
                    false
                } else {
                    true
                }
            });

            // Check for collisions with player
            if bullet.2 && bullet.1 == HEIGHT - 1 && (bullet.0 == self.player || bullet.0 == self.player.saturating_sub(1) || bullet.0 == self.player + 1) {
                self.lives = self.lives.saturating_sub(1);
                false
            } else {
                bullet.1 > 0 && bullet.1 < HEIGHT
            }
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

        // Check for collisions and update enemies
        let mut rng = rand::thread_rng();
        self.enemies.retain_mut(|enemy| {
            let mut hit = false;
            for bullet in &self.bullets {
                if !bullet.2 && (bullet.0.saturating_sub(LASER_HITBOX_WIDTH / 2)
                    ..=bullet.0.saturating_add(LASER_HITBOX_WIDTH / 2))
                    .contains(&enemy.x)
                    && bullet.1 == enemy.y
                {
                    enemy.health -= 1;
                    if enemy.health == 0 {
                        hit = true;
                        self.explosions.push((enemy.x, enemy.y, 0));
                        if enemy.enemy_type == 'H' {
                            self.lives = (self.lives + 1).min(5);
                        }
                        self.score += match enemy.enemy_type {
                            'S' | 'T' | 'F' => 20,
                            'B' => 30,
                            'H' => 50,
                            _ => 10,
                        };
                    }
                    break;
                }
            }

            // Enemy shooting
            if enemy.enemy_type == 'S' || enemy.enemy_type == 'B' {
                enemy.shoot_timer += 1;
                if enemy.shoot_timer >= 50 {
                    enemy.shoot_timer = 0;
                    if self.bullets.len() < 10 {
                        self.bullets.push((enemy.x, enemy.y + 1, true));
                        if enemy.enemy_type == 'B' {
                            // Bomber shoots in 3 directions
                            self.bullets.push((enemy.x.saturating_sub(1), enemy.y + 1, true));
                            self.bullets.push((enemy.x + 1, enemy.y + 1, true));
                        }
                    }
                }
            }

            !hit
        });

        // Move enemies
        self.enemy_move_counter += 1;
        if self.enemy_move_counter >= 20 - self.level.min(15) {
            self.enemy_move_counter = 0;
            if self.enemies.is_empty() && self.boss.is_none() {
                self.level += 1;
                self.lives = (self.lives + 1).min(5); // Give player an extra life after beating a level
                if self.level >= 5 && self.level % 5 == 0 {
                    self.spawn_boss();
                } else {
                    self.enemies = self.create_enemies();
                }
            } else {
                for enemy in &mut self.enemies {
                    match enemy.enemy_type {
                        'Z' => {
                            enemy.x = (enemy.x + if enemy.y % 4 < 2 { 1 } else { WIDTH - 1 }) % WIDTH;
                            enemy.y += 1;
                        }
                        'W' => {
                            enemy.x = (enemy.x + (enemy.y as f32 / 2.0).sin() as usize + 1) % WIDTH;
                            enemy.y += 1;
                        }
                        'D' => {
                            enemy.x = (enemy.x + 1) % WIDTH;
                            enemy.y += 1;
                        }
                        'T' => {
                            if rng.gen_bool(0.1) {
                                enemy.x = rng.gen_range(0..WIDTH);
                                enemy.y = rng.gen_range(0..HEIGHT / 2);
                            } else {
                                enemy.y += 1;
                            }
                        }
                        'F' => {
                            enemy.y += 2;
                        }
                        'H' | 'S' | 'B' | 'N' => {
                            enemy.y += 1;
                        }
                        _ => {}
                    }
                    if enemy.y >= HEIGHT - 1 {
                        self.lives = self.lives.saturating_sub(1);
                        self.enemies = self.create_enemies();
                        break;
                    }
                }
            }
        }

// Update boss
if let Some(boss) = &mut self.boss {
    boss.shoot_timer += 1;
    if boss.shoot_timer >= 20 { // Changed from 20 to 10 for faster shooting
        boss.shoot_timer = 0;
        if self.bullets.len() < 15 { // Increased max bullets from 10 to 15
            self.bullets.push((boss.x, boss.y + 1, true));
            if boss.phase >= 2 {
                self.bullets.push((boss.x.saturating_sub(2), boss.y + 1, true));
                self.bullets.push((boss.x + 2, boss.y + 1, true));
            }
            if boss.phase >= 3 {
                self.bullets.push((boss.x.saturating_sub(4), boss.y + 1, true));
                self.bullets.push((boss.x + 4, boss.y + 1, true));
            }
        }
    }
    // Boss movement
    boss.move_timer += 1;
    if boss.move_timer >= 12 { // Move every 12 frames instead of 3 (75% slower)
        boss.move_timer = 0;
        if boss.x == 0 || boss.x == WIDTH - 1 {
            boss.direction *= -1;
        }
        boss.x = (boss.x as i32 + boss.direction as i32).max(0).min(WIDTH as i32 - 1) as usize;
    }
    // Check for collisions with boss
    for bullet in &self.bullets {
        if !bullet.2 && (bullet.0.saturating_sub(2)..=bullet.0.saturating_add(2)).contains(&boss.x)
            && bullet.1 == boss.y
        {
            boss.health = boss.health.saturating_sub(1);
            if boss.health == 0 {
                self.score += 1000;
                self.explosions.push((boss.x, boss.y, 0));
                self.boss = None;
                break;
            } else if boss.health == boss.max_health * 2 / 3 {
                boss.phase = 2;
            } else if boss.health == boss.max_health / 3 {
                boss.phase = 3;
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
// Spawn a boss
    fn spawn_boss(&mut self) {
        let max_health = 50 + self.level as u16 * 10;
        self.boss = Some(Boss {
            x: WIDTH / 2,
            y: 3,
            health: max_health,
            max_health,
            phase: 1,
            shoot_timer: 0,
            direction: 1,
            move_timer: 0,
        });
    }

    // Render the game state as a string
    fn render(&self) -> String {
        let mut output = String::new();

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
            for enemy in &self.enemies {
                if enemy.y < HEIGHT {
                    screen[enemy.y][enemy.x] = enemy.enemy_type;
                }
            }
        }

        // Draw boss
        if let Some(boss) = &self.boss {
            screen[boss.y][boss.x] = 'B';
            // Draw boss health bar
            let health_bar_width = 20;
            let health_percentage = boss.health as f32 / boss.max_health as f32;
            let filled_width = (health_percentage * health_bar_width as f32) as usize;
            for i in 0..health_bar_width {
                screen[1][i + (WIDTH - health_bar_width) / 2] = if i < filled_width { '█' } else { '░' };
            }
        }

        // Draw bullets
        if !self.paused {
            for &(x, y, is_enemy) in &self.bullets {
                if y < HEIGHT {
                    screen[y][x] = if is_enemy { '↓' } else { '|' };
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
                    'S' => output.push_str(&format!("{}", color::Fg(color::Red))),
                    'T' => output.push_str(&format!("{}", color::Fg(color::Cyan))),
                    'F' => output.push_str(&format!("{}", color::Fg(color::LightBlue))),
                    'B' => output.push_str(&format!("{}", color::Fg(color::LightCyan))),
                    'N' => output.push_str(&format!("{}", color::Fg(color::White))),
                    '|' => output.push_str(&format!("{}", color::Fg(color::Green))),
                    '↓' => output.push_str(&format!("{}", color::Fg(color::Red))),
                    '*' | '+' => output.push_str(&format!("{}", color::Fg(color::Red))),
                    'B' => output.push_str(&format!("{}", color::Fg(color::Magenta))),
                    'M' => output.push_str(&format!("{}", color::Fg(color::LightGreen))),
                    'S' => output.push_str(&format!("{}", color::Fg(color::LightBlue))),
                    '█' => output.push_str(&format!("{}", color::Fg(color::Green))),
                    '░' => output.push_str(&format!("{}", color::Fg(color::Red))),
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
                if !self.paused && self.bullets.iter().filter(|&b| !b.2).count() < 3 {
                    match self.powerup_active {
                        Some('B') => {
                            // Bigger Laser
                            self.bullets.push((self.player, HEIGHT - 2, false));
                            self.bullets.push((self.player.saturating_sub(1), HEIGHT - 2, false));
                            self.bullets.push(((self.player + 1).min(WIDTH - 1), HEIGHT - 2, false));
                        }
                        Some('M') => {
                            // Multi-directional Laser
                            self.bullets.push((self.player, HEIGHT - 2, false));
                            self.bullets.push((self.player.saturating_sub(1), HEIGHT - 2, false));
                            self.bullets.push((self.player + 1, HEIGHT - 2, false));
                        }
                        _ => self.bullets.push((self.player, HEIGHT - 2, false)),
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
