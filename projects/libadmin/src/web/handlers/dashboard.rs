use axum::{
    extract::{Query, State},
    http::HeaderMap,
    response::{Html, IntoResponse, Response},
};
use rusqlite::params;

use super::shared::*;
use crate::{
    app::AppState,
    db::{list_books, open_conn},
    forms::{BookQuery, NoticeQuery},
    utils::db_err,
    web::views::{
        esc, flash, html_table, layout, matches_status, matches_text, metric, selected,
        status_label,
    },
};
pub(crate) async fn reader_dashboard(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<NoticeQuery>,
) -> Response {
    let session = match require_reader(&state, &headers) {
        Ok(session) => session,
        Err(response) => return response,
    };
    let conn = match open_conn(&state.db_path) {
        Ok(conn) => conn,
        Err(err) => return redirect_msg("/login", &db_err(err)),
    };
    let active = conn
        .query_row(
            "SELECT COUNT(*) FROM borrows WHERE reader_id = ?1 AND returned = 0",
            params![session.user_id],
            |row| row.get::<_, i64>(0),
        )
        .unwrap_or(0);
    let history = conn
        .query_row(
            "SELECT COUNT(*) FROM returns WHERE reader_id = ?1",
            params![session.user_id],
            |row| row.get::<_, i64>(0),
        )
        .unwrap_or(0);
    let open_exceptions = conn
        .query_row(
            "SELECT COUNT(*) FROM exceptions WHERE reader_id = ?1 AND status != '已处理'",
            params![session.user_id],
            |row| row.get::<_, i64>(0),
        )
        .unwrap_or(0);
    let available = conn
        .query_row(
            "SELECT COUNT(*) FROM books WHERE status = 'available'",
            [],
            |row| row.get::<_, i64>(0),
        )
        .unwrap_or(0);
    let body = format!(
        r#"<h1>读者工作台</h1>
{}
<div class="grid">{}{}{}{}</div>
<div class="section">
  <div class="actions">
    <a class="button" href="/books">检索并借阅图书</a>
    <a class="button secondary" href="/reader/loans">归还 / 续借</a>
    <a class="button secondary" href="/reader/profile">修改个人信息</a>
    <a class="button secondary" href="/reader/exceptions">异常申报</a>
  </div>
</div>"#,
        flash(query.msg.as_deref()),
        metric("当前在借", active),
        metric("历史归还", history),
        metric("未处理异常", open_exceptions),
        metric("可借馆藏", available)
    );
    Html(layout("读者工作台", Some(&session), body)).into_response()
}

pub(crate) async fn admin_dashboard(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<NoticeQuery>,
) -> Response {
    let session = match require_admin(&state, &headers) {
        Ok(session) => session,
        Err(response) => return response,
    };
    let conn = match open_conn(&state.db_path) {
        Ok(conn) => conn,
        Err(err) => return redirect_msg("/login", &db_err(err)),
    };
    let count = |sql: &str| -> i64 {
        conn.query_row(sql, [], |row| row.get::<_, i64>(0))
            .unwrap_or(0)
    };
    let body = format!(
        r#"<h1>管理员工作台</h1>
{}
<div class="grid">{}{}{}{}</div>
<div class="section">
  <div class="actions">
    <a class="button" href="/admin/readers">读者管理</a>
    <a class="button" href="/admin/books">图书管理</a>
    <a class="button secondary" href="/admin/records">借还记录</a>
    <a class="button secondary" href="/admin/exceptions">异常处理</a>
    <form method="post" action="/admin/backup"><button class="secondary" type="submit">生成备份</button></form>
  </div>
</div>"#,
        flash(query.msg.as_deref()),
        metric("读者数量", count("SELECT COUNT(*) FROM readers")),
        metric("馆藏图书", count("SELECT COUNT(*) FROM books")),
        metric(
            "在借图书",
            count("SELECT COUNT(*) FROM borrows WHERE returned = 0")
        ),
        metric(
            "待处理异常",
            count("SELECT COUNT(*) FROM exceptions WHERE status != '已处理'")
        )
    );
    Html(layout("管理员工作台", Some(&session), body)).into_response()
}

pub(crate) async fn books_page(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<BookQuery>,
) -> Response {
    let session = match require_session(&state, &headers) {
        Ok(session) => session,
        Err(response) => return response,
    };
    let books = match list_books(&state.db_path) {
        Ok(books) => books,
        Err(err) => return redirect_msg("/", &db_err(err)),
    };
    let mut rows = Vec::new();
    for book in books
        .into_iter()
        .filter(|book| {
            matches_text(&book.id, &query.id)
                && matches_text(&book.title, &query.title)
                && matches_text(&book.category, &query.category)
                && (matches_text(&book.keywords, &query.keyword)
                    || matches_text(&book.remark, &query.keyword))
                && matches_status(&book.status, &query.status)
        })
        .take(300)
    {
        let action = if session.role == "reader" && book.status == "available" {
            format!(
                r#"<form method="post" action="/reader/borrow">
  <input type="hidden" name="book_id" value="{}">
  <button type="submit">借阅</button>
</form>"#,
                esc(&book.id)
            )
        } else if session.role == "admin" {
            "<span class=\"muted\">可在图书管理中维护</span>".to_string()
        } else {
            String::new()
        };
        rows.push(vec![
            esc(&book.id),
            esc(&book.title),
            esc(&book.category),
            esc(&book.keywords),
            format!("<span class=\"pill\">{}</span>", status_label(&book.status)),
            esc(&book.remark),
            action,
        ]);
    }
    let body = format!(
        r#"<h1>图书查询</h1>
{}
<div class="section">
  <form method="get" action="/books" class="form-grid">
    <label>书籍 ID<input name="id" value="{}"></label>
    <label>书名<input name="title" value="{}"></label>
    <label>类别<input name="category" value="{}"></label>
    <label>关键词<input name="keyword" value="{}"></label>
    <label>状态
      <select name="status">
        <option value="">全部</option>
        <option value="available" {}>在馆可借</option>
        <option value="borrowed" {}>已借出</option>
        <option value="discarded" {}>报废</option>
      </select>
    </label>
    <button type="submit">查询</button>
  </form>
</div>
{}"#,
        flash(query.msg.as_deref()),
        esc(query.id.as_deref().unwrap_or("")),
        esc(query.title.as_deref().unwrap_or("")),
        esc(query.category.as_deref().unwrap_or("")),
        esc(query.keyword.as_deref().unwrap_or("")),
        selected(query.status.as_deref().unwrap_or(""), "available"),
        selected(query.status.as_deref().unwrap_or(""), "borrowed"),
        selected(query.status.as_deref().unwrap_or(""), "discarded"),
        html_table(
            &["ID", "书名", "类别", "关键词", "状态", "备注", "操作"],
            rows
        )
    );
    Html(layout("图书查询", Some(&session), body)).into_response()
}
