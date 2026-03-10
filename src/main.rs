use std::io::Write;

use crossterm::{
    queue,
    style::{Print, ResetColor, SetForegroundColor},
};

use crate::terminal::Terminal;

mod terminal;

fn main() {
    let mut term = Terminal::new().unwrap();

    queue!(
        term.stdout(),
        SetForegroundColor(crossterm::style::Color::Cyan),
        Print("Grr \n"),
        ResetColor,
        Print("Small grr"),
    )
    .unwrap();

    term.stdout().flush().unwrap();

    std::thread::sleep(std::time::Duration::from_secs(3));

    drop(term);

    println!("Hello, world!");
}
