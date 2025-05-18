use simplelog::{Config, LevelFilter, WriteLogger};
use std::{fs::File, io};

mod app;

fn main() -> io::Result<()> {
    let _ = WriteLogger::init(
        LevelFilter::Info,
        Config::default(),
        File::create("info.log").unwrap(),
    );
    let terminal = ratatui::init();
    let result = app::App::new().run(terminal);
    ratatui::restore();
    result
}
