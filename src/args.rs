use clap::Parser;

#[derive(Debug, Parser)]
pub struct Args {
    #[arg(long)]
    pub logging: bool,
}
