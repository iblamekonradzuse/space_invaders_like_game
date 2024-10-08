use std::io::{self, Write};
use termion::color;
use termion::screen::AlternateScreen;

pub fn display_tutorial_screen(
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
        termion::cursor::Goto(2, 3),
        color::Fg(color::Yellow),
        color::Fg(color::Reset)
    )?;
    write!(screen, "{}N - Normal enemy", termion::cursor::Goto(4, 5))?;
    write!(screen, "{}Z - Zigzag enemy", termion::cursor::Goto(4, 6))?;
    write!(screen, "{}W - Wave enemy", termion::cursor::Goto(4, 7))?;
    write!(screen, "{}D - Diagonal enemy", termion::cursor::Goto(4, 8))?;
    write!(
        screen,
        "{}H - Health enemy (gives extra life when destroyed)",
        termion::cursor::Goto(4, 8)
    )?;

    write!(screen, "{}B - Bomber enemy", termion::cursor::Goto(4, 9))?;
    write!(screen, "{}S - Shooter enemy", termion::cursor::Goto(4, 10))?;
    write!(
        screen,
        "{}T - Teleporter enemy",
        termion::cursor::Goto(4, 7)
    )?;
    write!(screen, "{}F - Rusher enemy", termion::cursor::Goto(4, 11))?;

    write!(
        screen,
        "{}{}Powerups:{}",
        termion::cursor::Goto(2, 13),
        color::Fg(color::Yellow),
        color::Fg(color::Reset)
    )?;
    write!(
        screen,
        "{}B - Bigger Laser (3-wide shot)",
        termion::cursor::Goto(4, 15)
    )?;
    write!(
        screen,
        "{}M - Multi-directional Laser (3-way shot)",
        termion::cursor::Goto(4, 16)
    )?;
    write!(
        screen,
        "{}S - Shield (temporary invincibility)",
        termion::cursor::Goto(4, 17)
    )?;

    write!(
        screen,
        "{}{}Controls:{}",
        termion::cursor::Goto(2, 19),
        color::Fg(color::Yellow),
        color::Fg(color::Reset)
    )?;
    write!(
        screen,
        "{}Left/Right Arrow - Move ship",
        termion::cursor::Goto(4, 21)
    )?;
    write!(screen, "{}Space - Shoot", termion::cursor::Goto(4, 22))?;
    write!(screen, "{}P - Pause/Unpause", termion::cursor::Goto(4, 23))?;

    write!(
        screen,
        "{}{}Press 'B' to return to the main menu",
        termion::cursor::Goto(2, 28),
        color::Fg(color::Green)
    )?;
    screen.flush()?;
    Ok(())
}
