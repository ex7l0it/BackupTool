use anyhow::Result;
use cfgbkc::*;

fn main() -> Result<()> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();
    run()
}
