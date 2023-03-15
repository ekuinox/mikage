mod routes;

use std::net::SocketAddr;

use anyhow::Result;
use axum::Server;
use axum_sessions::{async_session::MemoryStore, SameSite, SessionLayer};
use core::AppState;

use self::routes::router;

pub async fn serve(addr: &SocketAddr, state: AppState, secret: &[u8]) -> Result<()> {
    let store = MemoryStore::new();
    let session_layer = SessionLayer::new(store, secret).with_same_site_policy(SameSite::Lax);
    let app = router(state, session_layer);
    Server::bind(addr).serve(app.into_make_service()).await?;
    Ok(())
}
