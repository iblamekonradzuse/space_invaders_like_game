use std::io::{self, Write};
use std::time::Duration;
use termion::color;
use termion::screen::AlternateScreen;

pub fn display_game_over_screen(
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
