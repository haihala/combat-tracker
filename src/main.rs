use clap::Parser;
use simplelog::{Config, LevelFilter, WriteLogger};
use std::{fs::File, io};

mod app;
mod args;

fn main() -> io::Result<()> {
    let parsed_args = args::Args::parse();

    if parsed_args.logging {
        let _ = WriteLogger::init(
            LevelFilter::Info,
            Config::default(),
            File::create("info.log").unwrap(),
        );
    }

    let terminal = ratatui::init();
    let result = app::App::new(parsed_args.init_test_creatures).run(terminal);
    ratatui::restore();
    result
}
