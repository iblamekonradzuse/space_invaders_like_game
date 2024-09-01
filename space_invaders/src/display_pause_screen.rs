use std::io::{self, Write};
use termion::color;
use termion::screen::AlternateScreen;

const WIDTH: usize = 80; // Example value, adjust as needed
const HEIGHT: usize = 24; // Example value, adjust as needed

pub fn display_pause_screen(
    screen: &mut AlternateScreen<termion::raw::RawTerminal<std::io::Stdout>>,
) -> io::Result<()> {
    write!(
        screen,
        "{}{}{}GAME PAUSED{}",
        termion::cursor::Goto(
            ((WIDTH / 2) - 5).try_into().unwrap(),
            (HEIGHT / 2).try_into().unwrap()
        ),
        termion::style::Bold,
        color::Fg(color::Yellow),
        color::Fg(color::Reset)
    )?;
    write!(
        screen,
        "{}{}Press 'P' to resume",
        termion::cursor::Goto(
            ((WIDTH / 2) - 9).try_into().unwrap(),
            ((HEIGHT / 2) + 2).try_into().unwrap()
        ),
        color::Fg(color::Green)
    )?;
    screen.flush()?;
    Ok(())
}
