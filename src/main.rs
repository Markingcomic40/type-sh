mod app;
mod core;
mod error;
mod net;
mod screen;
mod settings;
mod terminal;
mod ui;

fn main() -> error::Result<()> {
    terminal::install_panic_hook();
    app::App::new()?.run()
}
