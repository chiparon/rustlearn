use axum::{
    extract::{Form, Query, State},
    http::HeaderMap,
    response::{Html, IntoResponse, Response},
};
use rusqlite::{OptionalExtension, params};

use super::shared::*;
use crate::{
    app::AppState,
    db::{get_reader, list_active_borrows, list_exceptions, list_returns, open_conn},
    forms::{BorrowForm, BorrowIdForm, NoticeQuery, ProfileForm, ReportExceptionForm},
    services::{complete_return, create_borrow, delete_reader_if_clear, renew_borrow},
    utils::{db_err, parse_date, today},
    web::views::{esc, flash, html_table, layout, selected},
};
pub(crate) async fn reader_profile(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<NoticeQuery>,
) -> Response {
    let session = match require_reader(&state, &headers) {
        Ok(session) => session,
        Err(response) => return response,
    };
    let reader = match get_reader(&state.db_path, &session.user_id) {
        Ok(Some(reader)) => reader,
        Ok(None) => return redirect_msg("/logout", "账号不存在，请重新登录"),
        Err(err) => return redirect_msg("/reader", &db_err(err)),
    };
    let body = format!(
        r#"<h1>个人信息</h1>
{}
<div class="section">
  <form method="post" action="/reader/profile" class="form-grid">
    <label>读者 ID<input value="{}" disabled></label>
    <label>姓名<input name="name" value="{}" required></label>
    <label>性别<select name="gender"><option {}>男</option><option {}>女</option><option {}>其他</option></select></label>
    <label>职业<input name="profession" value="{}" required></label>
    <label>最大借书数<input value="{}" disabled></label>
    <label>借书期限<input value="{} 天" disabled></label>
    <label>备注<input name="remark" value="{}"></label>
    <button type="submit">保存</button>
  </form>
</div>
<div class="section">
  <form method="post" action="/reader/cancel">
    <button class="danger" type="submit">注销账号</button>
  </form>
</div>"#,
        flash(query.msg.as_deref()),
        esc(&reader.id),
        esc(&reader.name),
        selected(&reader.gender, "男"),
        selected(&reader.gender, "女"),
        selected(&reader.gender, "其他"),
        esc(&reader.profession),
        reader.max_borrow,
        reader.borrow_days,
        esc(&reader.remark)
    );
    Html(layout("个人信息", Some(&session), body)).into_response()
}

pub(crate) async fn reader_profile_submit(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<ProfileForm>,
) -> Response {
    let session = match require_reader(&state, &headers) {
        Ok(session) => session,
        Err(response) => return response,
    };
    let conn = match open_conn(&state.db_path) {
        Ok(conn) => conn,
        Err(err) => return redirect_msg("/reader/profile", &db_err(err)),
    };
    let result = conn.execute(
        "UPDATE readers SET name = ?1, gender = ?2, profession = ?3, remark = ?4 WHERE id = ?5",
        params![
            form.name.trim(),
            form.gender.trim(),
            form.profession.trim(),
            form.remark.unwrap_or_default(),
            session.user_id
        ],
    );
    match result {
        Ok(_) => redirect_msg("/reader/profile", "个人信息已更新"),
        Err(err) => redirect_msg("/reader/profile", &db_err(err)),
    }
}

pub(crate) async fn reader_cancel(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let session = match require_reader(&state, &headers) {
        Ok(session) => session,
        Err(response) => return response,
    };
    match delete_reader_if_clear(&state.db_path, &session.user_id) {
        Ok(()) => redirect_with_cookie(
            "/login?msg=%E8%B4%A6%E5%8F%B7%E5%B7%B2%E6%B3%A8%E9%94%80",
            "libadmin_session=deleted; Path=/; HttpOnly; SameSite=Lax; Max-Age=0".to_string(),
        ),
        Err(message) => redirect_msg("/reader/profile", &message),
    }
}

pub(crate) async fn reader_borrow(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<BorrowForm>,
) -> Response {
    let session = match require_reader(&state, &headers) {
        Ok(session) => session,
        Err(response) => return response,
    };
    match create_borrow(
        &state.db_path,
        &session.user_id,
        form.book_id.trim(),
        form.remark.unwrap_or_default().trim(),
    ) {
        Ok(()) => redirect_msg("/reader/loans", "借阅成功"),
        Err(message) => redirect_msg("/books", &message),
    }
}

pub(crate) async fn reader_loans(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<NoticeQuery>,
) -> Response {
    let session = match require_reader(&state, &headers) {
        Ok(session) => session,
        Err(response) => return response,
    };
    let active = match list_active_borrows(&state.db_path, Some(&session.user_id)) {
        Ok(items) => items,
        Err(err) => return redirect_msg("/reader", &db_err(err)),
    };
    let returns = match list_returns(&state.db_path, Some(&session.user_id)) {
        Ok(items) => items,
        Err(err) => return redirect_msg("/reader", &db_err(err)),
    };
    let active_rows = active
        .into_iter()
        .map(|item| {
            let overdue = parse_date(&item.due_date)
                .map(|due| today() > due)
                .unwrap_or(false);
            vec![
                item.id.to_string(),
                esc(&item.book_id),
                esc(&item.title),
                esc(&item.borrow_date),
                esc(&item.due_date),
                item.renew_count.to_string(),
                if overdue {
                    "<span class=\"danger-text\">已超期</span>".to_string()
                } else {
                    "<span class=\"pill\">借阅中</span>".to_string()
                },
                format!(
                    r#"<div class="actions">
  <form method="post" action="/reader/return"><input type="hidden" name="borrow_id" value="{}"><button type="submit">归还</button></form>
  <form method="post" action="/reader/renew"><input type="hidden" name="borrow_id" value="{}"><button class="secondary" type="submit">续借</button></form>
</div>"#,
                    item.id, item.id
                ),
            ]
        })
        .collect::<Vec<_>>();
    let return_rows = returns
        .into_iter()
        .take(100)
        .map(|item| {
            vec![
                item.id.to_string(),
                esc(&item.book_id),
                esc(&item.title),
                esc(&item.return_date),
                esc(&item.due_date),
                esc(&item.remark),
            ]
        })
        .collect::<Vec<_>>();
    let body = format!(
        r#"<h1>我的借阅</h1>
{}
<h2>未归还</h2>
{}
<h2>归还记录</h2>
{}"#,
        flash(query.msg.as_deref()),
        html_table(
            &[
                "借阅号",
                "书籍 ID",
                "书名",
                "借书日期",
                "应还日期",
                "续借次数",
                "状态",
                "操作"
            ],
            active_rows
        ),
        html_table(
            &["记录号", "书籍 ID", "书名", "还书日期", "应还日期", "备注"],
            return_rows
        )
    );
    Html(layout("我的借阅", Some(&session), body)).into_response()
}

pub(crate) async fn reader_return(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<BorrowIdForm>,
) -> Response {
    let session = match require_reader(&state, &headers) {
        Ok(session) => session,
        Err(response) => return response,
    };
    match complete_return(&state.db_path, Some(&session.user_id), form.borrow_id) {
        Ok(()) => redirect_msg("/reader/loans", "归还成功"),
        Err(message) => redirect_msg("/reader/loans", &message),
    }
}

pub(crate) async fn reader_renew(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<BorrowIdForm>,
) -> Response {
    let session = match require_reader(&state, &headers) {
        Ok(session) => session,
        Err(response) => return response,
    };
    match renew_borrow(&state.db_path, Some(&session.user_id), form.borrow_id) {
        Ok(()) => redirect_msg("/reader/loans", "续借成功"),
        Err(message) => redirect_msg("/reader/loans", &message),
    }
}

pub(crate) async fn reader_exceptions(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<NoticeQuery>,
) -> Response {
    let session = match require_reader(&state, &headers) {
        Ok(session) => session,
        Err(response) => return response,
    };
    let exceptions = match list_exceptions(&state.db_path, Some(&session.user_id)) {
        Ok(items) => items,
        Err(err) => return redirect_msg("/reader", &db_err(err)),
    };
    let rows = exceptions
        .into_iter()
        .map(|item| {
            vec![
                item.id.to_string(),
                esc(&item.book_id),
                esc(&item.title),
                format!("{:.2}", item.amount),
                esc(&item.exception_type),
                esc(&item.status),
                esc(&item.remark),
            ]
        })
        .collect::<Vec<_>>();
    let body = format!(
        r#"<h1>异常记录</h1>
{}
<div class="section">
  <h2>申报异常</h2>
  <form method="post" action="/reader/exceptions/report" class="form-grid">
    <label>书籍 ID<input name="book_id" required></label>
    <label>异常类型<select name="exception_type"><option>损坏</option><option>丢失</option></select></label>
    <label>备注<input name="remark"></label>
    <button type="submit">提交申报</button>
  </form>
</div>
{}"#,
        flash(query.msg.as_deref()),
        html_table(
            &["ID", "书籍 ID", "书名", "金额", "类型", "状态", "备注"],
            rows
        )
    );
    Html(layout("异常记录", Some(&session), body)).into_response()
}

pub(crate) async fn reader_report_exception(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<ReportExceptionForm>,
) -> Response {
    let session = match require_reader(&state, &headers) {
        Ok(session) => session,
        Err(response) => return response,
    };
    let conn = match open_conn(&state.db_path) {
        Ok(conn) => conn,
        Err(err) => return redirect_msg("/reader/exceptions", &db_err(err)),
    };
    let borrow_id: Option<i64> = match conn
        .query_row(
            "SELECT id FROM borrows WHERE reader_id = ?1 AND book_id = ?2 AND returned = 0",
            params![session.user_id, form.book_id.trim()],
            |row| row.get(0),
        )
        .optional()
    {
        Ok(value) => value,
        Err(err) => return redirect_msg("/reader/exceptions", &db_err(err)),
    };
    let Some(borrow_id) = borrow_id else {
        return redirect_msg("/reader/exceptions", "只能申报本人未归还图书的异常");
    };
    let result = conn.execute(
        "INSERT INTO exceptions (exception_type, amount, status, process_date, reader_id, book_id, borrow_id, remark)
         VALUES (?1, 0, '待管理员处理', ?2, ?3, ?4, ?5, ?6)",
        params![
            form.exception_type,
            today().to_string(),
            session.user_id,
            form.book_id.trim(),
            borrow_id,
            form.remark.unwrap_or_default()
        ],
    );
    match result {
        Ok(_) => redirect_msg("/reader/exceptions", "异常申报已提交，等待管理员处理"),
        Err(err) => redirect_msg("/reader/exceptions", &db_err(err)),
    }
}
