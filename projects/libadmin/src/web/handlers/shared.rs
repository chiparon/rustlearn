use axum::{
    body::Body,
    http::{HeaderMap, StatusCode, header},
    response::{Html, IntoResponse, Redirect, Response},
};

use crate::{
    app::AppState,
    models::Session,
    web::views::{esc, layout},
};
pub(super) fn session_from_headers(state: &AppState, headers: &HeaderMap) -> Option<Session> {
    let cookie = headers.get(header::COOKIE)?.to_str().ok()?;
    let token = cookie.split(';').find_map(|part| {
        let part = part.trim();
        part.strip_prefix("libadmin_session=").map(str::to_string)
    })?;
    state.sessions.lock().ok()?.get(&token).cloned()
}

pub(super) fn require_session(state: &AppState, headers: &HeaderMap) -> Result<Session, Response> {
    session_from_headers(state, headers).ok_or_else(|| redirect_msg("/login", "请先登录"))
}

pub(super) fn require_reader(state: &AppState, headers: &HeaderMap) -> Result<Session, Response> {
    let session = require_session(state, headers)?;
    if session.role == "reader" {
        Ok(session)
    } else {
        Err(forbidden("当前账号不是读者账号"))
    }
}

pub(super) fn require_admin(state: &AppState, headers: &HeaderMap) -> Result<Session, Response> {
    let session = require_session(state, headers)?;
    if session.role == "admin" {
        Ok(session)
    } else {
        Err(forbidden("权限不足，无法进入管理员功能"))
    }
}

fn forbidden(message: &str) -> Response {
    (
        StatusCode::FORBIDDEN,
        Html(layout(
            "权限不足",
            None,
            format!("<h1>权限不足</h1><p>{}</p>", esc(message)),
        )),
    )
        .into_response()
}

pub(super) fn redirect_msg(path: &str, message: &str) -> Response {
    let location = format!("{}?msg={}", path, urlencoding::encode(message));
    Redirect::to(&location).into_response()
}

pub(super) fn redirect_to(path: &str) -> Response {
    Redirect::to(path).into_response()
}

pub(super) fn redirect_with_cookie(path: &str, cookie: String) -> Response {
    Response::builder()
        .status(StatusCode::SEE_OTHER)
        .header(header::LOCATION, path)
        .header(header::SET_COOKIE, cookie)
        .body(Body::empty())
        .expect("valid redirect response")
}
