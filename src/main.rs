use std::path::Path;
use clap::Parser;

#[derive(Parser)]
struct Args {
    #[clap(short, long)]
    config: Option<Path>,
}

fn main() {
    let args = Args::parse();
    println!("Hello, world!");
}
