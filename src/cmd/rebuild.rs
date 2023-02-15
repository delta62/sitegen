use super::build;
use crate::args::Args;
use crate::config::Config;
use crate::error::Result;
use std::path::Path;
use tokio::runtime::Runtime;

pub fn rebuild<P: AsRef<Path>>(rt: &Runtime, path: P, args: &Args, config: &Config) -> Result<()> {
    log::info!("change: {}", path.as_ref().to_str().unwrap());

    rt.block_on(async { build(args, config).await })
}
