mod app;
mod core;
mod error;
mod preferences;
mod terminal;
mod ui;

use app::App;

fn main() -> error::Result<()> {
    terminal::install_panic_hook();

    let mut app = App::new()?;
    app.run()
}
