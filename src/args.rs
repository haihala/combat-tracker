use clap::Parser;

#[derive(Debug, Parser)]
pub struct Args {
    #[arg(long)]
    pub logging: bool,
    #[arg(long)]
    pub init_test_creatures: bool,
}
