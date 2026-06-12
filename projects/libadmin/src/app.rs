use std::{
    collections::HashMap,
    fs,
    net::SocketAddr,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use axum::{Router, routing::get};
use tokio::net::TcpListener;

use crate::db::{ensure_daily_backup, init_database};
use crate::models::Session;
use crate::web::handlers;

#[derive(Clone)]
pub(crate) struct AppState {
    pub(crate) db_path: PathBuf,
    pub(crate) sessions: Arc<Mutex<HashMap<String, Session>>>,
}

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let data_dir = std::env::current_dir()?.join("data");
    fs::create_dir_all(&data_dir)?;
    let db_path = data_dir.join("libadmin.db");
    init_database(&db_path)?;
    ensure_daily_backup(&db_path)?;

    let state = AppState {
        db_path,
        sessions: Arc::new(Mutex::new(HashMap::new())),
    };

    let app = Router::new()
        .route("/", get(handlers::index))
        .route(
            "/login",
            get(handlers::login_page).post(handlers::login_submit),
        )
        .route(
            "/register",
            get(handlers::register_page).post(handlers::register_submit),
        )
        .route("/logout", get(handlers::logout))
        .route("/help", get(handlers::help_page))
        .route("/books", get(handlers::books_page))
        .route("/reader", get(handlers::reader_dashboard))
        .route(
            "/reader/profile",
            get(handlers::reader_profile).post(handlers::reader_profile_submit),
        )
        .route(
            "/reader/cancel",
            get(handlers::reader_cancel).post(handlers::reader_cancel),
        )
        .route(
            "/reader/borrow",
            axum::routing::post(handlers::reader_borrow),
        )
        .route("/reader/loans", get(handlers::reader_loans))
        .route(
            "/reader/return",
            axum::routing::post(handlers::reader_return),
        )
        .route("/reader/renew", axum::routing::post(handlers::reader_renew))
        .route("/reader/exceptions", get(handlers::reader_exceptions))
        .route(
            "/reader/exceptions/report",
            axum::routing::post(handlers::reader_report_exception),
        )
        .route("/admin", get(handlers::admin_dashboard))
        .route("/admin/readers", get(handlers::admin_readers))
        .route(
            "/admin/readers/add",
            axum::routing::post(handlers::admin_add_reader),
        )
        .route(
            "/admin/readers/update",
            axum::routing::post(handlers::admin_update_reader),
        )
        .route(
            "/admin/readers/delete",
            axum::routing::post(handlers::admin_delete_reader),
        )
        .route("/admin/books", get(handlers::admin_books))
        .route(
            "/admin/books/add",
            axum::routing::post(handlers::admin_add_book),
        )
        .route(
            "/admin/books/update",
            axum::routing::post(handlers::admin_update_book),
        )
        .route(
            "/admin/books/delete",
            axum::routing::post(handlers::admin_delete_book),
        )
        .route("/admin/admins", get(handlers::admin_admins))
        .route(
            "/admin/admins/add",
            axum::routing::post(handlers::admin_add_admin),
        )
        .route(
            "/admin/admins/update",
            axum::routing::post(handlers::admin_update_admin),
        )
        .route(
            "/admin/admins/delete",
            axum::routing::post(handlers::admin_delete_admin),
        )
        .route("/admin/records", get(handlers::admin_records))
        .route("/admin/borrow", axum::routing::post(handlers::admin_borrow))
        .route("/admin/return", axum::routing::post(handlers::admin_return))
        .route("/admin/renew", axum::routing::post(handlers::admin_renew))
        .route("/admin/exceptions", get(handlers::admin_exceptions))
        .route(
            "/admin/exceptions/add",
            axum::routing::post(handlers::admin_add_exception),
        )
        .route(
            "/admin/exceptions/resolve",
            axum::routing::post(handlers::admin_resolve_exception),
        )
        .route("/admin/backup", axum::routing::post(handlers::admin_backup))
        .with_state(state);

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
