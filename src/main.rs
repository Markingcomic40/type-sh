use std::io::Write;

use crossterm::{
    queue,
    style::{Print, ResetColor, SetForegroundColor},
};

use crate::{error::AppError, terminal::Terminal};

mod error;
mod terminal;

fn main() -> error::Result<()> {
    let mut term = Terminal::new()?;

    queue!(
        term.stdout(),
        SetForegroundColor(crossterm::style::Color::Cyan),
        Print("Grr \n"),
        ResetColor,
        Print("Small grr"),
    )?;

    term.stdout().flush()?;

    std::thread::sleep(std::time::Duration::from_secs(3));

    drop(term);

    println!("Hello, world!");

    Ok(())
}
