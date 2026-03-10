mod app;
mod core;
mod error;
mod terminal;

use app::App;

fn main() -> error::Result<()> {
    let mut app = App::new()?;
    app.run()?;

    Ok(())
}
