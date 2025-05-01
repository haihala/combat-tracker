use std::io;

mod app;

fn main() -> io::Result<()> {
    let terminal = ratatui::init();
    let result = app::App::new().run(terminal);
    ratatui::restore();
    result
}
