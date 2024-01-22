use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent},
    terminal, QueueableCommand,
};
use std::io::{Result, Write};

fn main() -> Result<()> {
    let mut stdout = std::io::stdout();
    terminal::enable_raw_mode()?;
    loop {
        stdout
            .queue(terminal::Clear(terminal::ClearType::All))?
            .queue(cursor::MoveTo(0, 0))?
            .flush()?;
        let event = event::read()?;
        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Char('q'),
                ..
            }) => break,
            _ => println!("{:?}\r", event),
        }
        stdout.flush()?;
    }
    terminal::disable_raw_mode()?;
    Ok(())
}
