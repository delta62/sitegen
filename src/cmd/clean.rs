use crate::args::Args;
use crate::config::Config;
use tokio::fs;

pub async fn clean(_args: &Args, config: &Config) {
    fs::remove_dir_all(config.build.out_dir.as_str())
        .await
        .unwrap_or_default()
}
