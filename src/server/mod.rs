mod routes;
mod state;

use self::{routes::frontend, state::State};
use crate::conf::{Conf, ConfFromPath};
use actix_web::{web, App, HttpServer};
use actix_web_static_files::ResourceFiles;
use anyhow::Result;
use clap::Parser;
use std::sync::Arc;

#[derive(Parser, Debug)]
pub struct ServerApp {
    #[clap(long = "host", default_value = "0.0.0.0")]
    host: String,

    #[clap(short = 'p', long = "port", default_value = "10092")]
    port: u16,

    #[clap(short = 'c', long = "conf", default_value = "mikage.toml")]
    conf: ConfFromPath,
}

impl ServerApp {
    pub async fn run(self) -> Result<()> {
        let conf: Arc<Conf> = Arc::new(self.conf.into());

        let server = HttpServer::new(move || {
            App::new()
                .app_data(web::Data::new(State::new(conf.clone())))
                .service(routes::login::login)
                .service(routes::login::callback)
                .service(ResourceFiles::new("/", frontend()))
        })
        .bind((self.host, self.port))?
        .run();
        let _handle = server.handle();
        server.await?;
        Ok(())
    }
}
