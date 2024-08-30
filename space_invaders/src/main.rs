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
    enemies: Vec<(usize, usize)>,
    bullets: Vec<(usize, usize)>,
    score: u32,
    level: usize,
    lives: usize,
    enemy_move_counter: usize,
}

impl Game {
    fn new() -> Self {
        Game {
            player: WIDTH / 2,
            enemies: Vec::new(),
            bullets: Vec::new(),
            score: 0,
            level: 1,
            lives: 5,
            enemy_move_counter: 0,
        }
    }

    fn create_enemies(&self) -> Vec<(usize, usize)> {
        let mut enemies = Vec::new();
        let rows = 1 + self.level / 3;
        let cols = 3 + self.level / 3;
        for row in 0..rows {
            for col in 0..cols {
                enemies.push((col * (WIDTH / (cols + 1)) + 5, row * 2 + 3));
            }
        }
        enemies
    }

    fn update(&mut self) {
        // Move bullets
        self.bullets.retain_mut(|bullet| {
            bullet.1 = bullet.1.saturating_sub(1);
            bullet.1 > 0
        });

        // Check for collisions
        let initial_enemy_count = self.enemies.len();
        self.enemies
            .retain(|&enemy| !self.bullets.iter().any(|&bullet| bullet == enemy));
        let enemies_destroyed = initial_enemy_count - self.enemies.len();

        // Update score
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
                    enemy.1 += 1;
                    if enemy.1 >= HEIGHT - 1 {
                        self.lives = self.lives.saturating_sub(1);
                        self.enemies = self.create_enemies();
                        break;
                    }
                }
            }
        }
    }

    fn render(&self) -> String {
        let mut output = format!(
            "{}Score: {} | Level: {} | Lives: {}{}\r\n",
            color::Fg(color::Yellow),
            self.score,
            self.level,
            "♥".repeat(self.lives),
            color::Fg(color::Reset)
        );
        let mut screen = vec![vec![' '; WIDTH]; HEIGHT];

        // Draw player
        screen[HEIGHT - 1][self.player] = 'A';

        // Draw enemies
        for &(x, y) in &self.enemies {
            if y < HEIGHT {
                screen[y][x] = 'W';
            }
        }

        // Draw bullets
        for &(x, y) in &self.bullets {
            if y < HEIGHT {
                screen[y][x] = '|';
            }
        }

        // Convert screen to string with colors
        for (y, row) in screen.iter().enumerate() {
            for (x, &ch) in row.iter().enumerate() {
                match ch {
                    'A' => output.push_str(&format!("{}", color::Fg(color::Blue))),
                    'W' => output.push_str(&format!("{}", color::Fg(color::Red))),
                    '|' => output.push_str(&format!("{}", color::Fg(color::Green))),
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
                    self.bullets.push((self.player, HEIGHT - 2));
                }
            }
            _ => {}
        }
    }

    fn is_game_over(&self) -> bool {
        self.lives == 0
    }
}

fn display_start_screen(
    screen: &mut AlternateScreen<termion::raw::RawTerminal<std::io::Stdout>>,
) -> io::Result<()> {
    write!(screen, "{}", termion::clear::All)?;
    write!(
        screen,
        "{}{}{}Space Invaders!{}",
        termion::cursor::Goto(1, 1),
        termion::style::Bold,
        color::Fg(color::Cyan),
        color::Fg(color::Reset)
    )?;
    write!(
        screen,
        "{}{}Press 'S' to start the game",
        termion::cursor::Goto(1, 3),
        color::Fg(color::Green)
    )?;
    write!(
        screen,
        "{}{}Press 'Q' to quit",
        termion::cursor::Goto(1, 5),
        color::Fg(color::Red)
    )?;
    screen.flush()?;
    Ok(())
}

fn display_game_over_screen(
    screen: &mut AlternateScreen<termion::raw::RawTerminal<std::io::Stdout>>,
    score: u32,
    level: usize,
) -> io::Result<()> {
    write!(screen, "{}", termion::clear::All)?;
    write!(
        screen,
        "{}{}{}Game Over!{}",
        termion::cursor::Goto(1, 1),
        termion::style::Bold,
        color::Fg(color::Red),
        color::Fg(color::Reset)
    )?;
    write!(
        screen,
        "{}{}Final Score: {}",
        termion::cursor::Goto(1, 3),
        color::Fg(color::Yellow),
        score
    )?;
    write!(
        screen,
        "{}{}Levels Completed: {}",
        termion::cursor::Goto(1, 5),
        color::Fg(color::Yellow),
        level - 1
    )?;
    write!(
        screen,
        "{}{}Press 'R' to play again",
        termion::cursor::Goto(1, 7),
        color::Fg(color::Green)
    )?;
    write!(
        screen,
        "{}{}Press 'Q' to quit",
        termion::cursor::Goto(1, 9),
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

        display_game_over_screen(&mut screen, game.score, game.level)?;

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

