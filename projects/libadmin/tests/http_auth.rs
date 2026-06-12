use std::{fs, path::PathBuf};

use axum::{
    body::{Body, to_bytes},
    http::{Method, Request, StatusCode, header},
    response::Response,
};
use rusqlite::params;
use tower::ServiceExt;
use uuid::Uuid;

struct TestApp {
    app: axum::Router,
    db_path: PathBuf,
}

impl TestApp {
    fn new() -> Self {
        let db_path = std::env::temp_dir().join(format!("libadmin-http-{}.db", Uuid::new_v4()));
        let app = libadmin::build_router(db_path.clone()).expect("router should build");
        Self { app, db_path }
    }

    async fn get(&self, uri: &str, cookie: Option<&str>) -> Response {
        let mut builder = Request::builder().method(Method::GET).uri(uri);
        if let Some(cookie) = cookie {
            builder = builder.header(header::COOKIE, cookie);
        }
        self.app
            .clone()
            .oneshot(builder.body(Body::empty()).expect("request should build"))
            .await
            .expect("request should complete")
    }

    async fn post_form(&self, uri: &str, body: &str, cookie: Option<&str>) -> Response {
        let mut builder = Request::builder()
            .method(Method::POST)
            .uri(uri)
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded");
        if let Some(cookie) = cookie {
            builder = builder.header(header::COOKIE, cookie);
        }
        self.app
            .clone()
            .oneshot(
                builder
                    .body(Body::from(body.to_string()))
                    .expect("request should build"),
            )
            .await
            .expect("request should complete")
    }

    async fn login(&self, role: &str, user_id: &str, password: &str) -> String {
        let response = self
            .post_form(
                "/login",
                &format!("role={role}&user_id={user_id}&password={password}"),
                None,
            )
            .await;

        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        response
            .headers()
            .get(header::SET_COOKIE)
            .expect("login should set a cookie")
            .to_str()
            .expect("cookie should be valid header text")
            .split(';')
            .next()
            .expect("cookie should include session pair")
            .to_string()
    }
}

impl Drop for TestApp {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.db_path);
    }
}

async fn body_text(response: Response) -> String {
    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body should be readable");
    String::from_utf8(bytes.to_vec()).expect("response body should be utf-8")
}

#[tokio::test]
async fn admin_route_requires_login() {
    let app = TestApp::new();

    let response = app.get("/admin", None).await;

    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    let location = response
        .headers()
        .get(header::LOCATION)
        .expect("redirect should include location")
        .to_str()
        .expect("location should be valid header text");
    assert!(location.starts_with("/login?msg="));
}

#[tokio::test]
async fn reader_is_forbidden_from_admin_dashboard() {
    let app = TestApp::new();
    let cookie = app.login("reader", "R001", "reader001").await;

    let response = app.get("/admin", Some(&cookie)).await;
    let status = response.status();
    let body = body_text(response).await;

    assert_eq!(status, StatusCode::FORBIDDEN);
    assert!(body.contains("权限不足"));
}

#[tokio::test]
async fn admin_login_reaches_dashboard() {
    let app = TestApp::new();
    let cookie = app.login("admin", "A001", "admin123").await;

    let response = app.get("/admin", Some(&cookie)).await;
    let status = response.status();
    let body = body_text(response).await;

    assert_eq!(status, StatusCode::OK);
    assert!(body.contains("管理员工作台"));
}

#[tokio::test]
async fn reader_can_borrow_renew_and_return_through_http() {
    let app = TestApp::new();
    let cookie = app.login("reader", "R001", "reader001").await;

    let response = app
        .post_form(
            "/reader/borrow",
            "book_id=B0038&remark=http-test",
            Some(&cookie),
        )
        .await;
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert_eq!(
        response.headers().get(header::LOCATION).unwrap(),
        "/reader/loans?msg=%E5%80%9F%E9%98%85%E6%88%90%E5%8A%9F"
    );

    let conn = rusqlite::Connection::open(&app.db_path).expect("database should open");
    let (borrow_id, book_status): (i64, String) = conn
        .query_row(
            "SELECT br.id, b.status
             FROM borrows br JOIN books b ON br.book_id = b.id
             WHERE br.reader_id = 'R001' AND br.book_id = 'B0038' AND br.returned = 0",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .expect("active borrow should exist");
    assert_eq!(book_status, "borrowed");
    drop(conn);

    let response = app
        .post_form(
            "/reader/renew",
            &format!("borrow_id={borrow_id}"),
            Some(&cookie),
        )
        .await;
    assert_eq!(response.status(), StatusCode::SEE_OTHER);

    let response = app
        .post_form(
            "/reader/return",
            &format!("borrow_id={borrow_id}"),
            Some(&cookie),
        )
        .await;
    assert_eq!(response.status(), StatusCode::SEE_OTHER);

    let conn = rusqlite::Connection::open(&app.db_path).expect("database should open");
    let (returned, renew_count, final_status): (i64, i64, String) = conn
        .query_row(
            "SELECT br.returned, br.renew_count, b.status
             FROM borrows br JOIN books b ON br.book_id = b.id
             WHERE br.id = ?1",
            params![borrow_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .expect("borrow row should remain");
    let return_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM returns WHERE borrow_id = ?1",
            params![borrow_id],
            |row| row.get(0),
        )
        .expect("return count should be readable");

    assert_eq!(returned, 1);
    assert_eq!(renew_count, 1);
    assert_eq!(final_status, "available");
    assert_eq!(return_count, 1);
}
