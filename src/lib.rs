mod routers;

use crate::routers::get_router;
use axum::Router;
use tokio::net::TcpListener;

pub async fn run() {
    let app = Router::new();
    let listener = TcpListener::bind("127.0.0.1:9000")
        .await.unwrap();
    axum::serve(listener, get_router(app))
        .await.unwrap();
}