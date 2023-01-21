mod routes;

use std::net::SocketAddr;

use anyhow::Result;
use axum::Server;
use core::AppState;

use self::routes::router;

pub async fn serve(addr: &SocketAddr, state: AppState) -> Result<()> {
    let app = router(state);
    Server::bind(addr).serve(app.into_make_service()).await?;
    Ok(())
}
