use std::{fs, net::SocketAddr};
use tokio::net::TcpListener;

mod auth;
mod db;
mod forms;
mod models;
mod routes;
mod util;
mod views;

use db::{ensure_daily_backup, init_database};
use models::AppState;
use routes::app_router;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let data_dir = std::env::current_dir()?.join("data");
    fs::create_dir_all(&data_dir)?;
    let db_path = data_dir.join("libadmin.db");
    init_database(&db_path)?;
    ensure_daily_backup(&db_path)?;

    let state = AppState::new(db_path);

    let app = app_router(state);

    let listener = bind_first_available().await?;
    let addr = listener.local_addr()?;
    println!("libadmin running at http://127.0.0.1:{}", addr.port());
    axum::serve(listener, app).await?;
    Ok(())
}

async fn bind_first_available() -> std::io::Result<TcpListener> {
    for port in 8088..=8098 {
        let addr = SocketAddr::from(([0, 0, 0, 0], port));
        match TcpListener::bind(addr).await {
            Ok(listener) => return Ok(listener),
            Err(err) if err.kind() == std::io::ErrorKind::AddrInUse => continue,
            Err(err) => return Err(err),
        }
    }
    TcpListener::bind(SocketAddr::from(([0, 0, 0, 0], 0))).await
}
