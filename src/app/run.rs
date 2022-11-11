use super::AsyncRunner;
use crate::conf::ConfFromPath;
use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
pub struct RunSubcommand {
    #[clap(short = 'c', long = "conf", default_value = "conf.toml")]
    conf: ConfFromPath,
}

#[async_trait::async_trait]
impl AsyncRunner for RunSubcommand {
    async fn run(self) -> Result<()> {
        Ok(())
    }
}
