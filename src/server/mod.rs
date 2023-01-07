mod routes;
mod state;

use self::{routes::frontend, state::State};
use crate::conf::{Conf, ConfFromPath};
use actix_web::{web, App, HttpServer, middleware::Logger};
use actix_web_static_files::ResourceFiles;
use anyhow::Result;
use clap::Parser;
use migration::{Migrator, MigratorTrait};
use sea_orm::Database;
use std::sync::Arc;

#[derive(Parser, Debug)]
pub struct ServerApp {
    #[clap(short = 'c', long = "conf", default_value = "mikage.toml")]
    conf: ConfFromPath,
}

impl ServerApp {
    pub async fn run(self) -> Result<()> {
        let conf: Arc<Conf> = Arc::new(self.conf.into());
        let host = conf.host.to_string();
        let port = conf.port;

        let connection = Database::connect(&conf.database_url).await?;
        Migrator::up(&connection, None).await?;
        let connection = Arc::new(connection);

        let server = HttpServer::new(move || {
            App::new()
                .wrap(Logger::default())
                .app_data(web::Data::new(State::new(conf.clone(), connection.clone())))
                .service(routes::login::login)
                .service(routes::login::callback)
                .service(ResourceFiles::new("/", frontend()))
        })
        .bind((host, port))?
        .run();
        let _handle = server.handle();
        server.await?;
        Ok(())
    }
}
