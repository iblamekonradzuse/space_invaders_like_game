use std::io::{self, Write};
use termion::color;
use termion::screen::AlternateScreen;

pub fn display_start_screen(
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
