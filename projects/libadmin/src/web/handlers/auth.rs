use axum::{
    extract::{Form, Query, State},
    http::{HeaderMap, header},
    response::{Html, IntoResponse, Response},
};
use rusqlite::{OptionalExtension, params};
use uuid::Uuid;

use super::shared::*;
use crate::{
    app::AppState,
    db::open_conn,
    forms::{LoginForm, NoticeQuery, RegisterForm},
    models::Session,
    utils::{db_err, hash_password, valid_id},
    web::views::{flash, layout},
};
pub(crate) async fn index(State(state): State<AppState>, headers: HeaderMap) -> Response {
    match session_from_headers(&state, &headers) {
        Some(session) if session.role == "admin" => redirect_to("/admin"),
        Some(_) => redirect_to("/reader"),
        None => redirect_to("/login"),
    }
}

pub(crate) async fn login_page(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<NoticeQuery>,
) -> Response {
    if let Some(session) = session_from_headers(&state, &headers) {
        return if session.role == "admin" {
            redirect_to("/admin")
        } else {
            redirect_to("/reader")
        };
    }
    let body = format!(
        r#"<h1>登录</h1>
{}
<div class="section">
  <form method="post" action="/login" class="form-grid">
    <label>身份
      <select name="role"><option value="reader">普通读者</option><option value="admin">管理员</option></select>
    </label>
    <label>账号 ID<input name="user_id" required autocomplete="username"></label>
    <label>密码<input name="password" type="password" required autocomplete="current-password"></label>
    <button type="submit">登录</button>
  </form>
</div>
<div class="section">
  <h2>测试账号</h2>
  <p>管理员：A001 / admin123；读者：R001 / reader001。新读者可直接注册。</p>
  <a class="button secondary" href="/register">注册读者账号</a>
</div>"#,
        flash(query.msg.as_deref())
    );
    Html(layout("登录", None, body)).into_response()
}

pub(crate) async fn login_submit(
    State(state): State<AppState>,
    Form(form): Form<LoginForm>,
) -> Response {
    let user_id = form.user_id.trim().to_string();
    if !valid_id(&user_id) {
        return redirect_msg("/login", "账号 ID 只能包含字母、数字、下划线或连字符");
    }
    let password_hash = hash_password(&form.password);
    let conn = match open_conn(&state.db_path) {
        Ok(conn) => conn,
        Err(err) => return redirect_msg("/login", &db_err(err)),
    };
    let result: rusqlite::Result<Option<String>> = if form.role == "admin" {
        conn.query_row(
            "SELECT name FROM admins WHERE id = ?1 AND password_hash = ?2",
            params![user_id, password_hash],
            |row| row.get(0),
        )
        .optional()
    } else {
        conn.query_row(
            "SELECT name FROM readers WHERE id = ?1 AND password_hash = ?2",
            params![user_id, password_hash],
            |row| row.get(0),
        )
        .optional()
    };

    match result {
        Ok(Some(name)) => {
            let token = Uuid::new_v4().to_string();
            let role = if form.role == "admin" {
                "admin"
            } else {
                "reader"
            }
            .to_string();
            state.sessions.lock().expect("session lock").insert(
                token.clone(),
                Session {
                    role: role.clone(),
                    user_id,
                    display_name: name,
                },
            );
            let location = if role == "admin" { "/admin" } else { "/reader" };
            redirect_with_cookie(
                location,
                format!("libadmin_session={token}; Path=/; HttpOnly; SameSite=Lax"),
            )
        }
        Ok(None) => redirect_msg("/login", "账号或密码错误"),
        Err(err) => redirect_msg("/login", &db_err(err)),
    }
}

pub(crate) async fn logout(State(state): State<AppState>, headers: HeaderMap) -> Response {
    if let Some(cookie) = headers.get(header::COOKIE).and_then(|v| v.to_str().ok())
        && let Some(token) = cookie.split(';').find_map(|part| {
            let part = part.trim();
            part.strip_prefix("libadmin_session=").map(str::to_string)
        })
    {
        let _ = state.sessions.lock().expect("session lock").remove(&token);
    }
    redirect_with_cookie(
        "/login",
        "libadmin_session=deleted; Path=/; HttpOnly; SameSite=Lax; Max-Age=0".to_string(),
    )
}

pub(crate) async fn register_page(Query(query): Query<NoticeQuery>) -> Response {
    let body = format!(
        r#"<h1>读者注册</h1>
{}
<div class="section">
  <form method="post" action="/register" class="form-grid">
    <label>读者 ID<input name="id" required placeholder="例如 R900"></label>
    <label>姓名<input name="name" required></label>
    <label>密码<input type="password" name="password" required minlength="6"></label>
    <label>性别<select name="gender"><option>男</option><option>女</option><option>其他</option></select></label>
    <label>职业<input name="profession" required></label>
    <label>备注<input name="remark"></label>
    <button type="submit">注册</button>
  </form>
</div>"#,
        flash(query.msg.as_deref())
    );
    Html(layout("读者注册", None, body)).into_response()
}

pub(crate) async fn register_submit(
    State(state): State<AppState>,
    Form(form): Form<RegisterForm>,
) -> Response {
    let id = form.id.trim().to_uppercase();
    if !valid_id(&id) {
        return redirect_msg("/register", "读者 ID 只能包含字母、数字、下划线或连字符");
    }
    if form.password.len() < 6 {
        return redirect_msg("/register", "密码长度至少 6 位");
    }
    let conn = match open_conn(&state.db_path) {
        Ok(conn) => conn,
        Err(err) => return redirect_msg("/register", &db_err(err)),
    };
    let result = conn.execute(
        "INSERT INTO readers (id, name, gender, profession, max_borrow, borrow_days, password_hash, remark)
         VALUES (?1, ?2, ?3, ?4, 5, 30, ?5, ?6)",
        params![
            id,
            form.name.trim(),
            form.gender.trim(),
            form.profession.trim(),
            hash_password(&form.password),
            form.remark.unwrap_or_default()
        ],
    );
    match result {
        Ok(_) => redirect_msg("/login", "注册成功，请登录"),
        Err(rusqlite::Error::SqliteFailure(_, _)) => {
            redirect_msg("/register", "用户已存在，请勿重复注册")
        }
        Err(err) => redirect_msg("/register", &db_err(err)),
    }
}

pub(crate) async fn help_page(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let session = session_from_headers(&state, &headers);
    let body = r#"<h1>联机帮助</h1>
<div class="section">
  <h2>读者操作</h2>
  <p>读者可修改本人资料、检索馆藏、借阅在馆图书、归还或续借未超期图书，并查询本人借还与异常赔偿记录。</p>
  <h2>管理员操作</h2>
  <p>管理员可维护读者、图书和管理员账号，办理借还续借，登记和处理超期、损坏、丢失等异常记录，并生成数据库备份。</p>
  <h2>业务限制</h2>
  <p>借书前校验图书状态和读者限额；归还时自动判断超期；注销读者和删除图书前会检查未办结业务。</p>
</div>"#
        .to_string();
    Html(layout("帮助", session.as_ref(), body)).into_response()
}
