use std::io::{self, Write};
use termion::color;
use termion::screen::AlternateScreen;

pub fn display_option_screen(
    screen: &mut AlternateScreen<termion::raw::RawTerminal<std::io::Stdout>>,
) -> io::Result<()> {
    write!(screen, "{}", termion::clear::All)?;
    write!(
        screen,
        "{}{}{}✰✰✰ O P T I O N S ✰✰✰{}",
        termion::cursor::Goto(10, 8),
        termion::style::Bold,
        color::Fg(color::Cyan),
        color::Fg(color::Reset)
    )?;
    write!(
        screen,
        "{}{}Background Music:",
        termion::cursor::Goto(8, 12),
        color::Fg(color::Yellow)
    )?;
    write!(
        screen,
        "{}{}[ / ]",
        termion::cursor::Goto(28, 12),
        color::Fg(color::Green)
    )?;
    write!(
        screen,
        "{}{}Laser Effects:",
        termion::cursor::Goto(8, 14),
        color::Fg(color::Yellow)
    )?;
    write!(
        screen,
        "{}{}- / +",
        termion::cursor::Goto(28, 14),
        color::Fg(color::Green)
    )?;
    write!(
        screen,
        "{}{}Press 'B' to go back",
        termion::cursor::Goto(10, 20),
        color::Fg(color::Red)
    )?;
    screen.flush()?;
    Ok(())
}
