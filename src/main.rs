use clap::{self, Parser};

#[derive(Debug, clap::Parser)]
struct Args {
    #[clap(subcommand)]
    action: Action,
}

#[derive(Debug, clap::Subcommand)]
enum Action {
    Validator { index: u64 },
}

fn main() {
    let args = Args::parse();
    println!("{:?}", args);
}
