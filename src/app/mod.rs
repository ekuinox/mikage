mod init;

use self::init::InitSubcommand;
use anyhow::Result;
use clap::Parser;

#[async_trait::async_trait]
pub trait AsyncRunner {
    async fn run(self) -> Result<()>;
}

#[derive(Parser, Debug)]
pub enum Subcommand {
    #[clap(name = "init")]
    Init(InitSubcommand),
}

#[async_trait::async_trait]
impl AsyncRunner for Subcommand {
    async fn run(self) -> Result<()> {
        match self {
            Subcommand::Init(s) => s.run().await,
        }
    }
}

#[derive(Parser, Debug)]
pub struct App {
    #[clap(subcommand)]
    subcommand: Subcommand,
}

#[async_trait::async_trait]
impl AsyncRunner for App {
    async fn run(self) -> Result<()> {
        self.subcommand.run().await
    }
}
