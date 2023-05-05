use anyhow::Result;
use clap::Parser;

#[derive(Parser)]
struct Opts {
    /// Path to config file [Default: ./config.yaml]
    #[clap(short='c', long)]
    config: Option<String>,
    /// Path to backup file [Default: ./bkup]
    #[clap(short='o', long)]
    output: Option<String>,
    #[clap(short, long)]
    verbose: bool,
}

pub fn run() -> Result<()> {
    let _opts = Opts::parse();
    println!("Hello, world!");
    Ok(())
}
