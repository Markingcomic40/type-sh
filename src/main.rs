//! So past this commit I still try to at least discuss and code the logic related stuff myself but consulting w and even perhaps looking at literal code AI generated (not always tho i have fallen victim to the command c command v quite a few times probs obvious where) and for visuals idk i just didnt think its worth bothering with just want to wrap it up so i can focus on my game that is a bit more proper cause this was like my first ever rust program, did my hellow owrld printing in the terminal iirc 

mod app;
mod core;
mod error;
mod screen;
mod settings;
mod terminal;
mod ui;

fn main() -> error::Result<()> {
    terminal::install_panic_hook();
    app::App::new()?.run()
}
