use std::{fs, path::Path};

use axum::{
    extract::{Form, Query, State},
    http::HeaderMap,
    response::{Html, IntoResponse, Response},
};
use chrono::Local;

use super::shared::*;
use crate::{
    app::AppState,
    db::{
        list_active_borrows, list_admins, list_books, list_exceptions, list_readers, list_returns,
        open_conn, vacuum_into,
    },
    forms::{
        AdminQuery, AdminUpsertForm, BookQuery, BookUpsertForm, BorrowForm, BorrowIdForm,
        ExceptionAddForm, ExceptionQuery, ExceptionResolveForm, IdForm, ReaderQuery,
        ReaderUpsertForm, RecordQuery,
    },
    services::{
        AdminInput, BookInput, ExceptionInput, ReaderInput, complete_return, create_admin,
        create_book, create_borrow, create_exception, create_reader, delete_admin,
        delete_book_if_available, delete_reader_if_clear, renew_borrow, resolve_exception,
        update_admin, update_book, update_reader,
    },
    utils::{db_err, parse_date, today},
    web::views::{
        esc, flash, html_table, layout, matches_status, matches_text, selected, status_label,
    },
};
fn reader_form_fields(require_password: bool) -> String {
    let required = if require_password { "required" } else { "" };
    let password_label = if require_password {
        "初始密码"
    } else {
        "新密码（留空不改）"
    };
    format!(
        r#"<label>读者 ID<input name="id" required></label>
<label>姓名<input name="name" required></label>
<label>{password_label}<input name="password" type="password" {required}></label>
<label>性别<select name="gender"><option>男</option><option>女</option><option>其他</option></select></label>
<label>职业<input name="profession" required></label>
<label>最大借书数<input name="max_borrow" type="number" min="1" value="5" required></label>
<label>借书期限<input name="borrow_days" type="number" min="1" value="30" required></label>
<label>备注<input name="remark"></label>
<button type="submit">保存</button>"#
    )
}

pub(crate) async fn admin_readers(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ReaderQuery>,
) -> Response {
    let session = match require_admin(&state, &headers) {
        Ok(session) => session,
        Err(response) => return *response,
    };
    let readers = match list_readers(&state.db_path) {
        Ok(readers) => readers,
        Err(err) => return redirect_msg("/admin", &db_err(err)),
    };
    let rows = readers
        .into_iter()
        .filter(|reader| {
            matches_text(&reader.id, &query.id)
                && matches_text(&reader.name, &query.name)
                && matches_text(&reader.gender, &query.gender)
                && matches_text(&reader.profession, &query.profession)
        })
        .take(300)
        .map(|reader| {
            vec![
                esc(&reader.id),
                esc(&reader.name),
                esc(&reader.gender),
                esc(&reader.profession),
                reader.max_borrow.to_string(),
                reader.borrow_days.to_string(),
                esc(&reader.remark),
            ]
        })
        .collect::<Vec<_>>();
    let body = format!(
        r#"<h1>读者管理</h1>
{}
<div class="section">
  <form method="get" action="/admin/readers" class="form-grid">
    <label>读者 ID<input name="id" value="{}"></label>
    <label>姓名<input name="name" value="{}"></label>
    <label>性别<input name="gender" value="{}"></label>
    <label>职业<input name="profession" value="{}"></label>
    <button type="submit">查询</button>
  </form>
</div>
<div class="section"><h2>新增读者</h2><form method="post" action="/admin/readers/add" class="form-grid">{}</form></div>
<div class="section"><h2>修改读者</h2><form method="post" action="/admin/readers/update" class="form-grid">{}</form></div>
<div class="section"><h2>删除读者</h2><form method="post" action="/admin/readers/delete" class="form-grid"><label>读者 ID<input name="id" required></label><button class="danger" type="submit">删除</button></form></div>
{}"#,
        flash(query.msg.as_deref()),
        esc(query.id.as_deref().unwrap_or("")),
        esc(query.name.as_deref().unwrap_or("")),
        esc(query.gender.as_deref().unwrap_or("")),
        esc(query.profession.as_deref().unwrap_or("")),
        reader_form_fields(true),
        reader_form_fields(false),
        html_table(
            &["ID", "姓名", "性别", "职业", "最大借书", "期限", "备注"],
            rows
        )
    );
    Html(layout("读者管理", Some(&session), body)).into_response()
}

pub(crate) async fn admin_add_reader(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<ReaderUpsertForm>,
) -> Response {
    if let Err(response) = require_admin(&state, &headers) {
        return *response;
    }
    let input = ReaderInput {
        id: &form.id,
        name: &form.name,
        password: form.password.as_deref(),
        gender: &form.gender,
        profession: &form.profession,
        max_borrow: form.max_borrow,
        borrow_days: form.borrow_days,
        remark: form.remark.as_deref().unwrap_or_default(),
    };
    match create_reader(&state.db_path, input) {
        Ok(()) => redirect_msg("/admin/readers", "读者已新增"),
        Err(err) => service_error("/admin/readers", &err),
    }
}

pub(crate) async fn admin_update_reader(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<ReaderUpsertForm>,
) -> Response {
    if let Err(response) = require_admin(&state, &headers) {
        return *response;
    }
    let input = ReaderInput {
        id: &form.id,
        name: &form.name,
        password: form.password.as_deref(),
        gender: &form.gender,
        profession: &form.profession,
        max_borrow: form.max_borrow,
        borrow_days: form.borrow_days,
        remark: form.remark.as_deref().unwrap_or_default(),
    };
    match update_reader(&state.db_path, input) {
        Ok(()) => redirect_msg("/admin/readers", "读者信息已更新"),
        Err(err) => service_error("/admin/readers", &err),
    }
}

pub(crate) async fn admin_delete_reader(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<IdForm>,
) -> Response {
    if let Err(response) = require_admin(&state, &headers) {
        return *response;
    }
    match delete_reader_if_clear(&state.db_path, &form.id.trim().to_uppercase()) {
        Ok(()) => redirect_msg("/admin/readers", "读者已删除"),
        Err(err) => service_error("/admin/readers", &err),
    }
}

fn book_form_fields(include_status: bool) -> String {
    let status = if include_status {
        r#"<label>状态<select name="status"><option value="available">在馆可借</option><option value="borrowed">已借出</option><option value="discarded">报废</option></select></label>"#
    } else {
        ""
    };
    format!(
        r#"<label>书籍 ID<input name="id" required></label>
<label>书名<input name="title" required></label>
<label>类别<input name="category" required></label>
<label>关键词<input name="keywords" required></label>
{status}
<label>备注<input name="remark"></label>
<button type="submit">保存</button>"#
    )
}

pub(crate) async fn admin_books(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<BookQuery>,
) -> Response {
    let session = match require_admin(&state, &headers) {
        Ok(session) => session,
        Err(response) => return *response,
    };
    let books = match list_books(&state.db_path) {
        Ok(books) => books,
        Err(err) => return redirect_msg("/admin", &db_err(err)),
    };
    let rows = books
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
        .map(|book| {
            vec![
                esc(&book.id),
                esc(&book.title),
                esc(&book.category),
                esc(&book.keywords),
                format!("<span class=\"pill\">{}</span>", status_label(&book.status)),
                esc(&book.remark),
            ]
        })
        .collect::<Vec<_>>();
    let body = format!(
        r#"<h1>图书管理</h1>
{}
<div class="section">
  <form method="get" action="/admin/books" class="form-grid">
    <label>书籍 ID<input name="id" value="{}"></label>
    <label>书名<input name="title" value="{}"></label>
    <label>类别<input name="category" value="{}"></label>
    <label>关键词<input name="keyword" value="{}"></label>
    <label>状态<select name="status"><option value="">全部</option><option value="available" {}>在馆可借</option><option value="borrowed" {}>已借出</option><option value="discarded" {}>报废</option></select></label>
    <button type="submit">查询</button>
  </form>
</div>
<div class="section"><h2>新增图书</h2><form method="post" action="/admin/books/add" class="form-grid">{}</form></div>
<div class="section"><h2>修改图书</h2><form method="post" action="/admin/books/update" class="form-grid">{}</form></div>
<div class="section"><h2>删除图书</h2><form method="post" action="/admin/books/delete" class="form-grid"><label>书籍 ID<input name="id" required></label><button class="danger" type="submit">删除</button></form></div>
{}"#,
        flash(query.msg.as_deref()),
        esc(query.id.as_deref().unwrap_or("")),
        esc(query.title.as_deref().unwrap_or("")),
        esc(query.category.as_deref().unwrap_or("")),
        esc(query.keyword.as_deref().unwrap_or("")),
        selected(query.status.as_deref().unwrap_or(""), "available"),
        selected(query.status.as_deref().unwrap_or(""), "borrowed"),
        selected(query.status.as_deref().unwrap_or(""), "discarded"),
        book_form_fields(false),
        book_form_fields(true),
        html_table(&["ID", "书名", "类别", "关键词", "状态", "备注"], rows)
    );
    Html(layout("图书管理", Some(&session), body)).into_response()
}

pub(crate) async fn admin_add_book(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<BookUpsertForm>,
) -> Response {
    if let Err(response) = require_admin(&state, &headers) {
        return *response;
    }
    let input = BookInput {
        id: &form.id,
        title: &form.title,
        category: &form.category,
        keywords: &form.keywords,
        status: None,
        remark: form.remark.as_deref().unwrap_or_default(),
    };
    match create_book(&state.db_path, input) {
        Ok(()) => redirect_msg("/admin/books", "图书已新增"),
        Err(err) => service_error("/admin/books", &err),
    }
}

pub(crate) async fn admin_update_book(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<BookUpsertForm>,
) -> Response {
    if let Err(response) = require_admin(&state, &headers) {
        return *response;
    }
    let input = BookInput {
        id: &form.id,
        title: &form.title,
        category: &form.category,
        keywords: &form.keywords,
        status: form.status.as_deref(),
        remark: form.remark.as_deref().unwrap_or_default(),
    };
    match update_book(&state.db_path, input) {
        Ok(()) => redirect_msg("/admin/books", "图书信息已更新"),
        Err(err) => service_error("/admin/books", &err),
    }
}

pub(crate) async fn admin_delete_book(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<IdForm>,
) -> Response {
    if let Err(response) = require_admin(&state, &headers) {
        return *response;
    }
    match delete_book_if_available(&state.db_path, &form.id.trim().to_uppercase()) {
        Ok(()) => redirect_msg("/admin/books", "图书已删除"),
        Err(err) => service_error("/admin/books", &err),
    }
}

fn admin_form_fields(require_password: bool) -> String {
    let required = if require_password { "required" } else { "" };
    let password_label = if require_password {
        "初始密码"
    } else {
        "新密码（留空不改）"
    };
    format!(
        r#"<label>管理员 ID<input name="id" required></label>
<label>姓名<input name="name" required></label>
<label>{password_label}<input name="password" type="password" {required}></label>
<label>权限级别<input name="level" type="number" min="1" max="9" value="5" required></label>
<label>备注<input name="remark"></label>
<button type="submit">保存</button>"#
    )
}

pub(crate) async fn admin_admins(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<AdminQuery>,
) -> Response {
    let session = match require_admin(&state, &headers) {
        Ok(session) => session,
        Err(response) => return *response,
    };
    let admins = match list_admins(&state.db_path) {
        Ok(admins) => admins,
        Err(err) => return redirect_msg("/admin", &db_err(err)),
    };
    let rows = admins
        .into_iter()
        .filter(|admin| {
            matches_text(&admin.id, &query.id)
                && matches_text(&admin.name, &query.name)
                && query
                    .level
                    .as_deref()
                    .map(str::trim)
                    .filter(|v| !v.is_empty())
                    .map(|v| admin.level.to_string().contains(v))
                    .unwrap_or(true)
        })
        .map(|admin| {
            vec![
                esc(&admin.id),
                esc(&admin.name),
                admin.level.to_string(),
                esc(&admin.remark),
            ]
        })
        .collect::<Vec<_>>();
    let body = format!(
        r#"<h1>管理员账号</h1>
{}
<div class="section">
  <form method="get" action="/admin/admins" class="form-grid">
    <label>管理员 ID<input name="id" value="{}"></label>
    <label>姓名<input name="name" value="{}"></label>
    <label>权限级别<input name="level" value="{}"></label>
    <button type="submit">查询</button>
  </form>
</div>
<div class="section"><h2>新增管理员</h2><form method="post" action="/admin/admins/add" class="form-grid">{}</form></div>
<div class="section"><h2>修改管理员</h2><form method="post" action="/admin/admins/update" class="form-grid">{}</form></div>
<div class="section"><h2>删除管理员</h2><form method="post" action="/admin/admins/delete" class="form-grid"><label>管理员 ID<input name="id" required></label><button class="danger" type="submit">删除</button></form></div>
{}"#,
        flash(query.msg.as_deref()),
        esc(query.id.as_deref().unwrap_or("")),
        esc(query.name.as_deref().unwrap_or("")),
        esc(query.level.as_deref().unwrap_or("")),
        admin_form_fields(true),
        admin_form_fields(false),
        html_table(&["ID", "姓名", "权限级别", "备注"], rows)
    );
    Html(layout("管理员账号", Some(&session), body)).into_response()
}

pub(crate) async fn admin_add_admin(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<AdminUpsertForm>,
) -> Response {
    if let Err(response) = require_admin(&state, &headers) {
        return *response;
    }
    let input = AdminInput {
        id: &form.id,
        name: &form.name,
        password: form.password.as_deref(),
        level: form.level,
        remark: form.remark.as_deref().unwrap_or_default(),
    };
    match create_admin(&state.db_path, input) {
        Ok(()) => redirect_msg("/admin/admins", "管理员已新增"),
        Err(err) => service_error("/admin/admins", &err),
    }
}

pub(crate) async fn admin_update_admin(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<AdminUpsertForm>,
) -> Response {
    if let Err(response) = require_admin(&state, &headers) {
        return *response;
    }
    let input = AdminInput {
        id: &form.id,
        name: &form.name,
        password: form.password.as_deref(),
        level: form.level,
        remark: form.remark.as_deref().unwrap_or_default(),
    };
    match update_admin(&state.db_path, input) {
        Ok(()) => redirect_msg("/admin/admins", "管理员信息已更新"),
        Err(err) => service_error("/admin/admins", &err),
    }
}

pub(crate) async fn admin_delete_admin(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<IdForm>,
) -> Response {
    let session = match require_admin(&state, &headers) {
        Ok(session) => session,
        Err(response) => return *response,
    };
    match delete_admin(&state.db_path, &form.id, &session.user_id) {
        Ok(()) => redirect_msg("/admin/admins", "管理员已删除"),
        Err(err) => service_error("/admin/admins", &err),
    }
}

pub(crate) async fn admin_records(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<RecordQuery>,
) -> Response {
    let session = match require_admin(&state, &headers) {
        Ok(session) => session,
        Err(response) => return *response,
    };
    let active = match list_active_borrows(&state.db_path, None) {
        Ok(items) => items,
        Err(err) => return redirect_msg("/admin", &db_err(err)),
    };
    let returns = match list_returns(&state.db_path, None) {
        Ok(items) => items,
        Err(err) => return redirect_msg("/admin", &db_err(err)),
    };
    let active_rows = active
        .into_iter()
        .filter(|item| {
            matches_text(&item.reader_id, &query.reader_id)
                && matches_text(&item.book_id, &query.book_id)
        })
        .take(300)
        .map(|item| {
            let overdue = parse_date(&item.due_date)
                .map(|due| today() > due)
                .unwrap_or(false);
            vec![
                item.id.to_string(),
                format!("{} / {}", esc(&item.reader_id), esc(&item.reader_name)),
                format!("{} / {}", esc(&item.book_id), esc(&item.title)),
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
  <form method="post" action="/admin/return"><input type="hidden" name="borrow_id" value="{}"><button type="submit">归还</button></form>
  <form method="post" action="/admin/renew"><input type="hidden" name="borrow_id" value="{}"><button class="secondary" type="submit">续借</button></form>
</div>"#,
                    item.id, item.id
                ),
            ]
        })
        .collect::<Vec<_>>();
    let return_rows = returns
        .into_iter()
        .filter(|item| {
            matches_text(&item.reader_id, &query.reader_id)
                && matches_text(&item.book_id, &query.book_id)
        })
        .take(300)
        .map(|item| {
            vec![
                item.id.to_string(),
                format!("{} / {}", esc(&item.reader_id), esc(&item.reader_name)),
                format!("{} / {}", esc(&item.book_id), esc(&item.title)),
                esc(&item.return_date),
                esc(&item.due_date),
                esc(&item.remark),
            ]
        })
        .collect::<Vec<_>>();
    let body = format!(
        r#"<h1>借还记录</h1>
{}
<div class="section">
  <form method="get" action="/admin/records" class="form-grid">
    <label>读者 ID<input name="reader_id" value="{}"></label>
    <label>书籍 ID<input name="book_id" value="{}"></label>
    <button type="submit">查询</button>
  </form>
</div>
<div class="section">
  <h2>管理员办理借书</h2>
  <form method="post" action="/admin/borrow" class="form-grid">
    <label>读者 ID<input name="reader_id" required></label>
    <label>书籍 ID<input name="book_id" required></label>
    <label>备注<input name="remark"></label>
    <button type="submit">办理借书</button>
  </form>
</div>
<h2>未归还记录</h2>
{}
<h2>归还记录</h2>
{}"#,
        flash(query.msg.as_deref()),
        esc(query.reader_id.as_deref().unwrap_or("")),
        esc(query.book_id.as_deref().unwrap_or("")),
        html_table(
            &[
                "借阅号",
                "读者",
                "图书",
                "借书日期",
                "应还日期",
                "续借次数",
                "状态",
                "操作"
            ],
            active_rows
        ),
        html_table(
            &["记录号", "读者", "图书", "还书日期", "应还日期", "备注"],
            return_rows
        )
    );
    Html(layout("借还记录", Some(&session), body)).into_response()
}

pub(crate) async fn admin_borrow(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<BorrowForm>,
) -> Response {
    if let Err(response) = require_admin(&state, &headers) {
        return *response;
    }
    let Some(reader_id) = form.reader_id.as_deref() else {
        return redirect_msg("/admin/records", "管理员办理借书必须填写读者 ID");
    };
    match create_borrow(
        &state.db_path,
        reader_id.trim(),
        form.book_id.trim(),
        form.remark.unwrap_or_default().trim(),
    ) {
        Ok(()) => redirect_msg("/admin/records", "借书办理成功"),
        Err(err) => service_error("/admin/records", &err),
    }
}

pub(crate) async fn admin_return(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<BorrowIdForm>,
) -> Response {
    if let Err(response) = require_admin(&state, &headers) {
        return *response;
    }
    match complete_return(&state.db_path, None, form.borrow_id) {
        Ok(()) => redirect_msg("/admin/records", "归还成功"),
        Err(err) => service_error("/admin/records", &err),
    }
}

pub(crate) async fn admin_renew(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<BorrowIdForm>,
) -> Response {
    if let Err(response) = require_admin(&state, &headers) {
        return *response;
    }
    match renew_borrow(&state.db_path, None, form.borrow_id) {
        Ok(()) => redirect_msg("/admin/records", "续借成功"),
        Err(err) => service_error("/admin/records", &err),
    }
}

pub(crate) async fn admin_exceptions(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ExceptionQuery>,
) -> Response {
    let session = match require_admin(&state, &headers) {
        Ok(session) => session,
        Err(response) => return *response,
    };
    let exceptions = match list_exceptions(&state.db_path, None) {
        Ok(items) => items,
        Err(err) => return redirect_msg("/admin", &db_err(err)),
    };
    let rows = exceptions
        .into_iter()
        .filter(|item| {
            matches_text(&item.reader_id, &query.reader_id)
                && matches_text(&item.book_id, &query.book_id)
                && matches_text(&item.exception_type, &query.exception_type)
                && matches_text(&item.status, &query.status)
        })
        .take(300)
        .map(|item| {
            let action = if item.status == "已处理" {
                "<span class=\"muted\">已办结</span>".to_string()
            } else {
                format!(
                    r#"<form method="post" action="/admin/exceptions/resolve">
  <input type="hidden" name="id" value="{}">
  <button type="submit">标记处理完成</button>
</form>"#,
                    item.id
                )
            };
            vec![
                item.id.to_string(),
                esc(&item.exception_type),
                format!("{} / {}", esc(&item.reader_id), esc(&item.reader_name)),
                format!("{} / {}", esc(&item.book_id), esc(&item.title)),
                esc(&item.process_date),
                format!("{:.2}", item.amount),
                esc(&item.status),
                esc(&item.remark),
                action,
            ]
        })
        .collect::<Vec<_>>();
    let body = format!(
        r#"<h1>异常处理</h1>
{}
<div class="section">
  <form method="get" action="/admin/exceptions" class="form-grid">
    <label>读者 ID<input name="reader_id" value="{}"></label>
    <label>书籍 ID<input name="book_id" value="{}"></label>
    <label>异常类型<input name="exception_type" value="{}"></label>
    <label>状态<input name="status" value="{}"></label>
    <button type="submit">查询</button>
  </form>
</div>
<div class="section">
  <h2>登记异常</h2>
  <form method="post" action="/admin/exceptions/add" class="form-grid">
    <label>异常类型<select name="exception_type"><option>超期</option><option>损坏</option><option>丢失</option></select></label>
    <label>读者 ID<input name="reader_id" required></label>
    <label>书籍 ID<input name="book_id" required></label>
    <label>借阅号<input name="borrow_id"></label>
    <label>赔偿金额<input type="number" name="amount" min="0" step="0.01" value="0" required></label>
    <label>状态<select name="status"><option>待处理</option><option>待管理员处理</option><option>已处理</option></select></label>
    <label>备注<input name="remark"></label>
    <button type="submit">保存异常</button>
  </form>
</div>
{}"#,
        flash(query.msg.as_deref()),
        esc(query.reader_id.as_deref().unwrap_or("")),
        esc(query.book_id.as_deref().unwrap_or("")),
        esc(query.exception_type.as_deref().unwrap_or("")),
        esc(query.status.as_deref().unwrap_or("")),
        html_table(
            &[
                "ID",
                "类型",
                "读者",
                "图书",
                "处理日期",
                "金额",
                "状态",
                "备注",
                "操作"
            ],
            rows
        )
    );
    Html(layout("异常处理", Some(&session), body)).into_response()
}

pub(crate) async fn admin_add_exception(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<ExceptionAddForm>,
) -> Response {
    if let Err(response) = require_admin(&state, &headers) {
        return *response;
    }
    let input = ExceptionInput {
        exception_type: &form.exception_type,
        amount: form.amount,
        status: &form.status,
        reader_id: &form.reader_id,
        book_id: &form.book_id,
        borrow_id: form.borrow_id.as_deref(),
        remark: form.remark.as_deref().unwrap_or_default(),
    };
    match create_exception(&state.db_path, input) {
        Ok(()) => redirect_msg("/admin/exceptions", "异常记录已保存"),
        Err(err) => service_error("/admin/exceptions", &err),
    }
}

pub(crate) async fn admin_resolve_exception(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<ExceptionResolveForm>,
) -> Response {
    if let Err(response) = require_admin(&state, &headers) {
        return *response;
    }
    match resolve_exception(&state.db_path, form.id) {
        Ok(()) => redirect_msg("/admin/exceptions", "异常已标记处理完成"),
        Err(err) => service_error("/admin/exceptions", &err),
    }
}

pub(crate) async fn admin_backup(State(state): State<AppState>, headers: HeaderMap) -> Response {
    if let Err(response) = require_admin(&state, &headers) {
        return *response;
    }
    let backup_dir = state
        .db_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("backups");
    let _ = fs::create_dir_all(&backup_dir);
    let backup_path = backup_dir.join(format!(
        "libadmin-manual-{}.db",
        Local::now().format("%Y%m%d-%H%M%S")
    ));
    let conn = match open_conn(&state.db_path) {
        Ok(conn) => conn,
        Err(err) => return redirect_msg("/admin", &db_err(err)),
    };
    match vacuum_into(&conn, &backup_path) {
        Ok(_) => redirect_msg("/admin", &format!("备份已生成：{}", backup_path.display())),
        Err(err) => redirect_msg("/admin", &db_err(err)),
    }
}
