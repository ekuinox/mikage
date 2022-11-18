mod routes;

use self::routes::frontend;
use actix_web::{App, HttpServer};
use actix_web_static_files::ResourceFiles;
use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
pub struct ServerApp {
    #[clap(long = "host", default_value = "0.0.0.0")]
    host: String,

    #[clap(short = 'p', long = "port", default_value = "10092")]
    port: u16,
}

impl ServerApp {
    pub async fn run(self) -> Result<()> {
        let server =
            HttpServer::new(move || App::new().service(ResourceFiles::new("/", frontend())))
                .bind((self.host, self.port))?
                .run();
        let _handle = server.handle();
        server.await?;
        Ok(())
    }
}
