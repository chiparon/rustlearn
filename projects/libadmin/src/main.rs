use axum::{
    Router,
    body::Body,
    extract::{Form, Query, State},
    http::{HeaderMap, StatusCode, header},
    response::{Html, IntoResponse, Redirect, Response},
    routing::get,
};
use chrono::{Duration, Local, NaiveDate};
use rusqlite::{Connection, OptionalExtension, params};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    fs,
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};
use tokio::net::TcpListener;
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    db_path: PathBuf,
    sessions: Arc<Mutex<HashMap<String, Session>>>,
}

#[derive(Clone)]
struct Session {
    role: String,
    user_id: String,
    display_name: String,
}

#[derive(Clone)]
struct Reader {
    id: String,
    name: String,
    gender: String,
    profession: String,
    max_borrow: i64,
    borrow_days: i64,
    remark: String,
}

#[derive(Clone)]
struct Book {
    id: String,
    title: String,
    category: String,
    keywords: String,
    status: String,
    remark: String,
}

#[derive(Clone)]
struct Admin {
    id: String,
    name: String,
    level: i64,
    remark: String,
}

struct BorrowView {
    id: i64,
    reader_id: String,
    reader_name: String,
    book_id: String,
    title: String,
    borrow_date: String,
    due_date: String,
    renew_count: i64,
}

struct ReturnView {
    id: i64,
    reader_id: String,
    reader_name: String,
    book_id: String,
    title: String,
    return_date: String,
    due_date: String,
    remark: String,
}

struct ExceptionView {
    id: i64,
    exception_type: String,
    amount: f64,
    status: String,
    process_date: String,
    reader_id: String,
    reader_name: String,
    book_id: String,
    title: String,
    remark: String,
}

#[derive(Deserialize)]
struct NoticeQuery {
    msg: Option<String>,
}

#[derive(Deserialize)]
struct LoginForm {
    role: String,
    user_id: String,
    password: String,
}

#[derive(Deserialize)]
struct RegisterForm {
    id: String,
    name: String,
    password: String,
    gender: String,
    profession: String,
    remark: Option<String>,
}

#[derive(Deserialize)]
struct BookQuery {
    id: Option<String>,
    title: Option<String>,
    category: Option<String>,
    keyword: Option<String>,
    status: Option<String>,
    msg: Option<String>,
}

#[derive(Deserialize)]
struct ReaderQuery {
    id: Option<String>,
    name: Option<String>,
    gender: Option<String>,
    profession: Option<String>,
    msg: Option<String>,
}

#[derive(Deserialize)]
struct AdminQuery {
    id: Option<String>,
    name: Option<String>,
    level: Option<String>,
    msg: Option<String>,
}

#[derive(Deserialize)]
struct RecordQuery {
    reader_id: Option<String>,
    book_id: Option<String>,
    msg: Option<String>,
}

#[derive(Deserialize)]
struct ExceptionQuery {
    reader_id: Option<String>,
    book_id: Option<String>,
    exception_type: Option<String>,
    status: Option<String>,
    msg: Option<String>,
}

#[derive(Deserialize)]
struct ProfileForm {
    name: String,
    gender: String,
    profession: String,
    remark: Option<String>,
}

#[derive(Deserialize)]
struct BorrowForm {
    reader_id: Option<String>,
    book_id: String,
    remark: Option<String>,
}

#[derive(Deserialize)]
struct BorrowIdForm {
    borrow_id: i64,
}

#[derive(Deserialize)]
struct ReaderUpsertForm {
    id: String,
    name: String,
    password: Option<String>,
    gender: String,
    profession: String,
    max_borrow: i64,
    borrow_days: i64,
    remark: Option<String>,
}

#[derive(Deserialize)]
struct BookUpsertForm {
    id: String,
    title: String,
    category: String,
    keywords: String,
    status: Option<String>,
    remark: Option<String>,
}

#[derive(Deserialize)]
struct AdminUpsertForm {
    id: String,
    name: String,
    password: Option<String>,
    level: i64,
    remark: Option<String>,
}

#[derive(Deserialize)]
struct IdForm {
    id: String,
}

#[derive(Deserialize)]
struct ReportExceptionForm {
    book_id: String,
    exception_type: String,
    remark: Option<String>,
}

#[derive(Deserialize)]
struct ExceptionAddForm {
    exception_type: String,
    reader_id: String,
    book_id: String,
    borrow_id: Option<String>,
    amount: f64,
    status: String,
    remark: Option<String>,
}

#[derive(Deserialize)]
struct ExceptionResolveForm {
    id: i64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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
        .route("/", get(index))
        .route("/login", get(login_page).post(login_submit))
        .route("/register", get(register_page).post(register_submit))
        .route("/logout", get(logout))
        .route("/help", get(help_page))
        .route("/books", get(books_page))
        .route("/reader", get(reader_dashboard))
        .route(
            "/reader/profile",
            get(reader_profile).post(reader_profile_submit),
        )
        .route("/reader/cancel", get(reader_cancel).post(reader_cancel))
        .route("/reader/borrow", axum::routing::post(reader_borrow))
        .route("/reader/loans", get(reader_loans))
        .route("/reader/return", axum::routing::post(reader_return))
        .route("/reader/renew", axum::routing::post(reader_renew))
        .route("/reader/exceptions", get(reader_exceptions))
        .route(
            "/reader/exceptions/report",
            axum::routing::post(reader_report_exception),
        )
        .route("/admin", get(admin_dashboard))
        .route("/admin/readers", get(admin_readers))
        .route("/admin/readers/add", axum::routing::post(admin_add_reader))
        .route(
            "/admin/readers/update",
            axum::routing::post(admin_update_reader),
        )
        .route(
            "/admin/readers/delete",
            axum::routing::post(admin_delete_reader),
        )
        .route("/admin/books", get(admin_books))
        .route("/admin/books/add", axum::routing::post(admin_add_book))
        .route(
            "/admin/books/update",
            axum::routing::post(admin_update_book),
        )
        .route(
            "/admin/books/delete",
            axum::routing::post(admin_delete_book),
        )
        .route("/admin/admins", get(admin_admins))
        .route("/admin/admins/add", axum::routing::post(admin_add_admin))
        .route(
            "/admin/admins/update",
            axum::routing::post(admin_update_admin),
        )
        .route(
            "/admin/admins/delete",
            axum::routing::post(admin_delete_admin),
        )
        .route("/admin/records", get(admin_records))
        .route("/admin/borrow", axum::routing::post(admin_borrow))
        .route("/admin/return", axum::routing::post(admin_return))
        .route("/admin/renew", axum::routing::post(admin_renew))
        .route("/admin/exceptions", get(admin_exceptions))
        .route(
            "/admin/exceptions/add",
            axum::routing::post(admin_add_exception),
        )
        .route(
            "/admin/exceptions/resolve",
            axum::routing::post(admin_resolve_exception),
        )
        .route("/admin/backup", axum::routing::post(admin_backup))
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

fn open_conn(path: &Path) -> rusqlite::Result<Connection> {
    let conn = Connection::open(path)?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    Ok(conn)
}

fn init_database(path: &Path) -> rusqlite::Result<()> {
    let mut conn = open_conn(path)?;
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS readers (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            gender TEXT NOT NULL DEFAULT '',
            profession TEXT NOT NULL DEFAULT '',
            max_borrow INTEGER NOT NULL DEFAULT 5,
            borrow_days INTEGER NOT NULL DEFAULT 30,
            password_hash TEXT NOT NULL,
            remark TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );
        CREATE TABLE IF NOT EXISTS books (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            category TEXT NOT NULL,
            keywords TEXT NOT NULL DEFAULT '',
            status TEXT NOT NULL DEFAULT 'available',
            remark TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );
        CREATE TABLE IF NOT EXISTS admins (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            password_hash TEXT NOT NULL,
            level INTEGER NOT NULL DEFAULT 1,
            remark TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );
        CREATE TABLE IF NOT EXISTS borrows (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            reader_id TEXT NOT NULL,
            book_id TEXT NOT NULL,
            borrow_date TEXT NOT NULL,
            due_date TEXT NOT NULL,
            returned INTEGER NOT NULL DEFAULT 0,
            renew_count INTEGER NOT NULL DEFAULT 0,
            remark TEXT NOT NULL DEFAULT ''
        );
        CREATE TABLE IF NOT EXISTS returns (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            reader_id TEXT NOT NULL,
            book_id TEXT NOT NULL,
            borrow_id INTEGER NOT NULL,
            return_date TEXT NOT NULL,
            due_date TEXT NOT NULL,
            remark TEXT NOT NULL DEFAULT ''
        );
        CREATE TABLE IF NOT EXISTS exceptions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            exception_type TEXT NOT NULL,
            amount REAL NOT NULL DEFAULT 0,
            status TEXT NOT NULL DEFAULT '待处理',
            process_date TEXT NOT NULL,
            reader_id TEXT NOT NULL,
            book_id TEXT NOT NULL,
            borrow_id INTEGER,
            remark TEXT NOT NULL DEFAULT ''
        );
        CREATE INDEX IF NOT EXISTS idx_books_status ON books(status);
        CREATE INDEX IF NOT EXISTS idx_borrows_reader ON borrows(reader_id, returned);
        CREATE INDEX IF NOT EXISTS idx_borrows_book ON borrows(book_id, returned);
        CREATE INDEX IF NOT EXISTS idx_exceptions_reader ON exceptions(reader_id, status);
        ",
    )?;
    seed_database(&mut conn)?;
    Ok(())
}

fn seed_database(conn: &mut Connection) -> rusqlite::Result<()> {
    if conn.query_row("SELECT COUNT(*) FROM readers", [], |row| {
        row.get::<_, i64>(0)
    })? == 0
    {
        let jobs = ["学生", "教师", "工程师", "医生", "职员", "自由职业"];
        let tx = conn.transaction()?;
        for i in 1..=80 {
            tx.execute(
                "INSERT INTO readers (id, name, gender, profession, max_borrow, borrow_days, password_hash, remark)
                 VALUES (?1, ?2, ?3, ?4, 5, 30, ?5, ?6)",
                params![
                    format!("R{i:03}"),
                    format!("读者{i:03}"),
                    if i % 2 == 0 { "女" } else { "男" },
                    jobs[(i as usize - 1) % jobs.len()],
                    hash_password(&format!("reader{i:03}")),
                    "系统初始化读者"
                ],
            )?;
        }
        tx.commit()?;
    }

    if conn.query_row("SELECT COUNT(*) FROM books", [], |row| row.get::<_, i64>(0))? == 0 {
        let categories = [
            "文学",
            "计算机",
            "历史",
            "经济",
            "艺术",
            "教育",
            "医学",
            "工程",
        ];
        let tx = conn.transaction()?;
        for i in 1..=220 {
            let category = categories[(i as usize - 1) % categories.len()];
            tx.execute(
                "INSERT INTO books (id, title, category, keywords, status, remark)
                 VALUES (?1, ?2, ?3, ?4, 'available', ?5)",
                params![
                    format!("B{i:04}"),
                    format!("{category}馆藏精选{i:03}"),
                    category,
                    format!("{category};馆藏;实验数据;第{i:03}册"),
                    "系统初始化图书"
                ],
            )?;
        }
        tx.commit()?;
    }

    if conn.query_row("SELECT COUNT(*) FROM admins", [], |row| {
        row.get::<_, i64>(0)
    })? == 0
    {
        let admins = [
            ("A001", "总管理员", "admin123", 9, "默认最高权限账号"),
            ("A002", "借还管理员", "admin123", 5, "负责借还业务"),
            ("A003", "馆藏管理员", "admin123", 5, "负责图书维护"),
            ("A004", "异常管理员", "admin123", 5, "负责赔偿处理"),
        ];
        let tx = conn.transaction()?;
        for (id, name, password, level, remark) in admins {
            tx.execute(
                "INSERT INTO admins (id, name, password_hash, level, remark)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![id, name, hash_password(password), level, remark],
            )?;
        }
        tx.commit()?;
    }

    if conn.query_row("SELECT COUNT(*) FROM borrows", [], |row| {
        row.get::<_, i64>(0)
    })? == 0
    {
        let now = today();
        let tx = conn.transaction()?;
        for i in 1..=20 {
            let reader_id = format!("R{i:03}");
            let book_id = format!("B{i:04}");
            let borrow_date = now - Duration::days(50 - i as i64);
            let due_date = borrow_date + Duration::days(30);
            tx.execute(
                "INSERT INTO borrows (reader_id, book_id, borrow_date, due_date, returned, renew_count, remark)
                 VALUES (?1, ?2, ?3, ?4, 1, 0, ?5)",
                params![
                    reader_id,
                    book_id,
                    borrow_date.to_string(),
                    due_date.to_string(),
                    "初始化历史借阅"
                ],
            )?;
            let borrow_id = tx.last_insert_rowid();
            tx.execute(
                "INSERT INTO returns (reader_id, book_id, borrow_id, return_date, due_date, remark)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    format!("R{i:03}"),
                    format!("B{i:04}"),
                    borrow_id,
                    (due_date - Duration::days((i % 5) as i64)).to_string(),
                    due_date.to_string(),
                    "初始化归还记录"
                ],
            )?;
        }
        for i in 21..=32 {
            let reader_id = format!("R{i:03}");
            let book_id = format!("B{i:04}");
            let borrow_date = now - Duration::days((i % 18 + 3) as i64);
            let due_date = borrow_date + Duration::days(30);
            tx.execute(
                "INSERT INTO borrows (reader_id, book_id, borrow_date, due_date, returned, renew_count, remark)
                 VALUES (?1, ?2, ?3, ?4, 0, 0, ?5)",
                params![
                    reader_id,
                    book_id,
                    borrow_date.to_string(),
                    due_date.to_string(),
                    "初始化在借记录"
                ],
            )?;
            tx.execute(
                "UPDATE books SET status = 'borrowed' WHERE id = ?1",
                params![book_id],
            )?;
        }
        for i in 33..=37 {
            let reader_id = format!("R{i:03}");
            let book_id = format!("B{i:04}");
            let borrow_date = now - Duration::days(45 + i as i64 % 4);
            let due_date = borrow_date + Duration::days(30);
            tx.execute(
                "INSERT INTO borrows (reader_id, book_id, borrow_date, due_date, returned, renew_count, remark)
                 VALUES (?1, ?2, ?3, ?4, 0, 0, ?5)",
                params![
                    reader_id,
                    book_id,
                    borrow_date.to_string(),
                    due_date.to_string(),
                    "初始化超期记录"
                ],
            )?;
            let borrow_id = tx.last_insert_rowid();
            tx.execute(
                "UPDATE books SET status = 'borrowed' WHERE id = ?1",
                params![book_id],
            )?;
            tx.execute(
                "INSERT INTO exceptions (exception_type, amount, status, process_date, reader_id, book_id, borrow_id, remark)
                 VALUES ('超期', ?1, '待处理', ?2, ?3, ?4, ?5, ?6)",
                params![
                    ((now - due_date).num_days().max(1)) as f64,
                    now.to_string(),
                    format!("R{i:03}"),
                    format!("B{i:04}"),
                    borrow_id,
                    "初始化超期赔偿"
                ],
            )?;
        }
        tx.commit()?;
    }

    Ok(())
}

fn ensure_daily_backup(db_path: &Path) -> rusqlite::Result<()> {
    let Some(data_dir) = db_path.parent() else {
        return Ok(());
    };
    let backup_dir = data_dir.join("backups");
    let _ = fs::create_dir_all(&backup_dir);
    let backup_path = backup_dir.join(format!("libadmin-{}.db", today()));
    if !backup_path.exists() {
        let conn = open_conn(db_path)?;
        vacuum_into(&conn, &backup_path)?;
    }
    Ok(())
}

fn vacuum_into(conn: &Connection, path: &Path) -> rusqlite::Result<()> {
    let sql_path = path
        .to_string_lossy()
        .replace('\\', "/")
        .replace('\'', "''");
    conn.execute_batch(&format!("VACUUM INTO '{sql_path}';"))
}

fn hash_password(password: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"libadmin:");
    hasher.update(password.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn today() -> NaiveDate {
    Local::now().date_naive()
}

fn parse_date(value: &str) -> Result<NaiveDate, String> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d").map_err(|_| format!("日期格式无效：{value}"))
}

fn db_err(err: rusqlite::Error) -> String {
    format!("数据库操作失败：{err}")
}

fn valid_id(id: &str) -> bool {
    let len = id.len();
    (2..=32).contains(&len)
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

fn esc(input: &str) -> String {
    input
        .chars()
        .map(|c| match c {
            '&' => "&amp;".to_string(),
            '<' => "&lt;".to_string(),
            '>' => "&gt;".to_string(),
            '"' => "&quot;".to_string(),
            '\'' => "&#39;".to_string(),
            _ => c.to_string(),
        })
        .collect()
}

fn status_label(status: &str) -> &'static str {
    match status {
        "available" => "在馆可借",
        "borrowed" => "已借出",
        "discarded" => "报废",
        _ => "未知",
    }
}

fn selected(value: &str, expected: &str) -> &'static str {
    if value == expected { "selected" } else { "" }
}

fn flash(message: Option<&str>) -> String {
    message
        .map(|text| format!("<div class=\"notice\">{}</div>", esc(text)))
        .unwrap_or_default()
}

fn metric(label: &str, value: impl std::fmt::Display) -> String {
    format!(
        "<div class=\"metric\"><span>{}</span><strong>{}</strong></div>",
        esc(label),
        value
    )
}

fn html_table(headers: &[&str], rows: Vec<Vec<String>>) -> String {
    let head = headers
        .iter()
        .map(|h| format!("<th>{}</th>", esc(h)))
        .collect::<String>();
    let body = if rows.is_empty() {
        format!(
            "<tr><td colspan=\"{}\" class=\"muted\">暂无数据</td></tr>",
            headers.len()
        )
    } else {
        rows.into_iter()
            .map(|row| {
                format!(
                    "<tr>{}</tr>",
                    row.into_iter()
                        .map(|cell| format!("<td>{cell}</td>"))
                        .collect::<String>()
                )
            })
            .collect::<String>()
    };
    format!(
        "<div class=\"table-wrap\"><table><thead><tr>{head}</tr></thead><tbody>{body}</tbody></table></div>"
    )
}

fn matches_text(value: &str, needle: &Option<String>) -> bool {
    let Some(needle) = needle.as_deref().map(str::trim).filter(|s| !s.is_empty()) else {
        return true;
    };
    value.to_lowercase().contains(&needle.to_lowercase())
}

fn matches_status(value: &str, status: &Option<String>) -> bool {
    let Some(status) = status.as_deref().map(str::trim).filter(|s| !s.is_empty()) else {
        return true;
    };
    value == status
}

const STYLE: &str = r#"
:root {
  --bg:#f5f7f4; --panel:#fff; --line:#d8dfd2; --text:#1f2a24;
  --muted:#657267; --primary:#2e6f57; --primary-dark:#20523f;
  --accent:#b46b2a; --danger:#b33a3a; --soft:#edf3ee;
}
* { box-sizing:border-box; }
body { margin:0; font-family:"Microsoft YaHei","PingFang SC",Arial,sans-serif; background:var(--bg); color:var(--text); }
header { position:sticky; top:0; z-index:10; background:var(--panel); border-bottom:1px solid var(--line); }
.topbar { max-width:1180px; margin:0 auto; padding:12px 18px; display:flex; justify-content:space-between; align-items:center; gap:16px; }
.brand { font-weight:700; white-space:nowrap; }
nav { display:flex; flex-wrap:wrap; gap:8px; justify-content:flex-end; align-items:center; }
nav a, nav span { color:var(--text); text-decoration:none; padding:7px 10px; border-radius:8px; font-size:14px; }
nav a:hover { background:var(--soft); }
nav span { color:var(--muted); }
main { max-width:1180px; margin:0 auto; padding:22px 18px 48px; }
h1 { font-size:28px; margin:0 0 8px; letter-spacing:0; }
h2 { font-size:18px; margin:22px 0 12px; letter-spacing:0; }
p { color:var(--muted); line-height:1.7; }
.section { background:var(--panel); border:1px solid var(--line); border-radius:8px; padding:18px; margin:14px 0; }
.grid { display:grid; grid-template-columns:repeat(auto-fit,minmax(210px,1fr)); gap:12px; }
.metric { border:1px solid var(--line); border-radius:8px; padding:14px; background:#fbfcfa; min-height:84px; }
.metric span { display:block; color:var(--muted); font-size:13px; }
.metric strong { display:block; margin-top:8px; font-size:26px; }
.form-grid { display:grid; grid-template-columns:repeat(auto-fit,minmax(170px,1fr)); gap:10px; align-items:end; }
label { display:grid; gap:5px; font-size:13px; color:var(--muted); }
input, select, textarea { width:100%; border:1px solid var(--line); border-radius:8px; padding:9px 10px; font:inherit; color:var(--text); background:#fff; }
textarea { min-height:70px; resize:vertical; }
button, .button { border:0; border-radius:8px; background:var(--primary); color:#fff; padding:9px 12px; font:inherit; cursor:pointer; text-decoration:none; display:inline-flex; align-items:center; justify-content:center; gap:6px; min-height:38px; white-space:nowrap; }
button:hover, .button:hover { background:var(--primary-dark); }
.secondary { background:#53635b; }
.danger { background:var(--danger); }
.actions { display:flex; flex-wrap:wrap; gap:8px; align-items:center; }
.notice { border-left:4px solid var(--accent); background:#fff8ee; padding:10px 12px; border-radius:8px; margin:12px 0; color:#5c3d1f; }
.table-wrap { overflow-x:auto; border:1px solid var(--line); border-radius:8px; background:#fff; }
table { width:100%; border-collapse:collapse; }
th, td { border-bottom:1px solid var(--line); padding:10px 8px; text-align:left; vertical-align:top; }
th { color:var(--muted); font-size:13px; font-weight:600; background:#f8faf7; }
.pill { display:inline-flex; border-radius:999px; padding:4px 8px; background:var(--soft); color:var(--primary-dark); font-size:12px; white-space:nowrap; }
.danger-text { color:var(--danger); }
.muted { color:var(--muted); }
@media (max-width:680px) {
  .topbar { align-items:flex-start; flex-direction:column; }
  nav { justify-content:flex-start; }
  main { padding:18px 12px 36px; }
  h1 { font-size:23px; }
  .section { padding:14px; }
}
"#;

fn layout(title: &str, session: Option<&Session>, body: String) -> String {
    let nav = match session {
        Some(s) if s.role == "reader" => format!(
            "<a href=\"/reader\">读者台</a><a href=\"/books\">图书查询</a><a href=\"/reader/loans\">我的借阅</a><a href=\"/reader/exceptions\">异常记录</a><a href=\"/help\">帮助</a><a href=\"/logout\">退出</a><span>{}</span>",
            esc(&s.display_name)
        ),
        Some(s) if s.role == "admin" => format!(
            "<a href=\"/admin\">管理台</a><a href=\"/admin/readers\">读者</a><a href=\"/admin/books\">图书</a><a href=\"/admin/records\">借还</a><a href=\"/admin/exceptions\">异常</a><a href=\"/admin/admins\">管理员</a><a href=\"/help\">帮助</a><a href=\"/logout\">退出</a><span>{}</span>",
            esc(&s.display_name)
        ),
        _ => "<a href=\"/login\">登录</a><a href=\"/register\">读者注册</a><a href=\"/help\">帮助</a>".to_string(),
    };

    format!(
        r#"<!doctype html>
<html lang="zh-CN">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>{}</title>
  <style>{STYLE}</style>
</head>
<body>
  <header><div class="topbar"><div class="brand">图书馆管理系统</div><nav>{nav}</nav></div></header>
  <main>{body}</main>
</body>
</html>"#,
        esc(title)
    )
}

fn session_from_headers(state: &AppState, headers: &HeaderMap) -> Option<Session> {
    let cookie = headers.get(header::COOKIE)?.to_str().ok()?;
    let token = cookie.split(';').find_map(|part| {
        let part = part.trim();
        part.strip_prefix("libadmin_session=").map(str::to_string)
    })?;
    state.sessions.lock().ok()?.get(&token).cloned()
}

fn require_session(state: &AppState, headers: &HeaderMap) -> Result<Session, Response> {
    session_from_headers(state, headers).ok_or_else(|| redirect_msg("/login", "请先登录"))
}

fn require_reader(state: &AppState, headers: &HeaderMap) -> Result<Session, Response> {
    let session = require_session(state, headers)?;
    if session.role == "reader" {
        Ok(session)
    } else {
        Err(forbidden("当前账号不是读者账号"))
    }
}

fn require_admin(state: &AppState, headers: &HeaderMap) -> Result<Session, Response> {
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

fn redirect_msg(path: &str, message: &str) -> Response {
    let location = format!("{}?msg={}", path, urlencoding::encode(message));
    Redirect::to(&location).into_response()
}

fn redirect_to(path: &str) -> Response {
    Redirect::to(path).into_response()
}

fn redirect_with_cookie(path: &str, cookie: String) -> Response {
    Response::builder()
        .status(StatusCode::SEE_OTHER)
        .header(header::LOCATION, path)
        .header(header::SET_COOKIE, cookie)
        .body(Body::empty())
        .expect("valid redirect response")
}

async fn index(State(state): State<AppState>, headers: HeaderMap) -> Response {
    match session_from_headers(&state, &headers) {
        Some(session) if session.role == "admin" => redirect_to("/admin"),
        Some(_) => redirect_to("/reader"),
        None => redirect_to("/login"),
    }
}

async fn login_page(
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

async fn login_submit(State(state): State<AppState>, Form(form): Form<LoginForm>) -> Response {
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

async fn logout(State(state): State<AppState>, headers: HeaderMap) -> Response {
    if let Some(cookie) = headers.get(header::COOKIE).and_then(|v| v.to_str().ok()) {
        if let Some(token) = cookie.split(';').find_map(|part| {
            let part = part.trim();
            part.strip_prefix("libadmin_session=").map(str::to_string)
        }) {
            let _ = state.sessions.lock().expect("session lock").remove(&token);
        }
    }
    redirect_with_cookie(
        "/login",
        "libadmin_session=deleted; Path=/; HttpOnly; SameSite=Lax; Max-Age=0".to_string(),
    )
}

async fn register_page(Query(query): Query<NoticeQuery>) -> Response {
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

async fn register_submit(
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

async fn help_page(State(state): State<AppState>, headers: HeaderMap) -> Response {
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

async fn reader_dashboard(
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

async fn admin_dashboard(
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

async fn books_page(
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

async fn reader_profile(
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

async fn reader_profile_submit(
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

async fn reader_cancel(State(state): State<AppState>, headers: HeaderMap) -> Response {
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

async fn reader_borrow(
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

async fn reader_loans(
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

async fn reader_return(
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

async fn reader_renew(
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

async fn reader_exceptions(
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

async fn reader_report_exception(
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

async fn admin_readers(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ReaderQuery>,
) -> Response {
    let session = match require_admin(&state, &headers) {
        Ok(session) => session,
        Err(response) => return response,
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

async fn admin_add_reader(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<ReaderUpsertForm>,
) -> Response {
    if let Err(response) = require_admin(&state, &headers) {
        return response;
    }
    let id = form.id.trim().to_uppercase();
    if !valid_id(&id) {
        return redirect_msg("/admin/readers", "读者 ID 格式错误");
    }
    let Some(password) = form.password.filter(|p| !p.is_empty()) else {
        return redirect_msg("/admin/readers", "新增读者必须填写初始密码");
    };
    let conn = match open_conn(&state.db_path) {
        Ok(conn) => conn,
        Err(err) => return redirect_msg("/admin/readers", &db_err(err)),
    };
    let result = conn.execute(
        "INSERT INTO readers (id, name, gender, profession, max_borrow, borrow_days, password_hash, remark)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            id,
            form.name.trim(),
            form.gender.trim(),
            form.profession.trim(),
            form.max_borrow,
            form.borrow_days,
            hash_password(&password),
            form.remark.unwrap_or_default()
        ],
    );
    match result {
        Ok(_) => redirect_msg("/admin/readers", "读者已新增"),
        Err(err) => redirect_msg("/admin/readers", &db_err(err)),
    }
}

async fn admin_update_reader(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<ReaderUpsertForm>,
) -> Response {
    if let Err(response) = require_admin(&state, &headers) {
        return response;
    }
    let id = form.id.trim().to_uppercase();
    let conn = match open_conn(&state.db_path) {
        Ok(conn) => conn,
        Err(err) => return redirect_msg("/admin/readers", &db_err(err)),
    };
    let result = if let Some(password) = form.password.filter(|p| !p.is_empty()) {
        conn.execute(
            "UPDATE readers SET name = ?1, gender = ?2, profession = ?3, max_borrow = ?4,
             borrow_days = ?5, password_hash = ?6, remark = ?7 WHERE id = ?8",
            params![
                form.name.trim(),
                form.gender.trim(),
                form.profession.trim(),
                form.max_borrow,
                form.borrow_days,
                hash_password(&password),
                form.remark.unwrap_or_default(),
                id
            ],
        )
    } else {
        conn.execute(
            "UPDATE readers SET name = ?1, gender = ?2, profession = ?3, max_borrow = ?4,
             borrow_days = ?5, remark = ?6 WHERE id = ?7",
            params![
                form.name.trim(),
                form.gender.trim(),
                form.profession.trim(),
                form.max_borrow,
                form.borrow_days,
                form.remark.unwrap_or_default(),
                id
            ],
        )
    };
    match result {
        Ok(0) => redirect_msg("/admin/readers", "未找到该读者"),
        Ok(_) => redirect_msg("/admin/readers", "读者信息已更新"),
        Err(err) => redirect_msg("/admin/readers", &db_err(err)),
    }
}

async fn admin_delete_reader(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<IdForm>,
) -> Response {
    if let Err(response) = require_admin(&state, &headers) {
        return response;
    }
    match delete_reader_if_clear(&state.db_path, &form.id.trim().to_uppercase()) {
        Ok(()) => redirect_msg("/admin/readers", "读者已删除"),
        Err(message) => redirect_msg("/admin/readers", &message),
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

async fn admin_books(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<BookQuery>,
) -> Response {
    let session = match require_admin(&state, &headers) {
        Ok(session) => session,
        Err(response) => return response,
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

async fn admin_add_book(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<BookUpsertForm>,
) -> Response {
    if let Err(response) = require_admin(&state, &headers) {
        return response;
    }
    let id = form.id.trim().to_uppercase();
    if !valid_id(&id) {
        return redirect_msg("/admin/books", "书籍 ID 格式错误");
    }
    let conn = match open_conn(&state.db_path) {
        Ok(conn) => conn,
        Err(err) => return redirect_msg("/admin/books", &db_err(err)),
    };
    let result = conn.execute(
        "INSERT INTO books (id, title, category, keywords, status, remark)
         VALUES (?1, ?2, ?3, ?4, 'available', ?5)",
        params![
            id,
            form.title.trim(),
            form.category.trim(),
            form.keywords.trim(),
            form.remark.unwrap_or_default()
        ],
    );
    match result {
        Ok(_) => redirect_msg("/admin/books", "图书已新增"),
        Err(err) => redirect_msg("/admin/books", &db_err(err)),
    }
}

async fn admin_update_book(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<BookUpsertForm>,
) -> Response {
    if let Err(response) = require_admin(&state, &headers) {
        return response;
    }
    let conn = match open_conn(&state.db_path) {
        Ok(conn) => conn,
        Err(err) => return redirect_msg("/admin/books", &db_err(err)),
    };
    let result = conn.execute(
        "UPDATE books SET title = ?1, category = ?2, keywords = ?3, status = ?4, remark = ?5 WHERE id = ?6",
        params![
            form.title.trim(),
            form.category.trim(),
            form.keywords.trim(),
            form.status.unwrap_or_else(|| "available".to_string()),
            form.remark.unwrap_or_default(),
            form.id.trim().to_uppercase()
        ],
    );
    match result {
        Ok(0) => redirect_msg("/admin/books", "未找到该图书"),
        Ok(_) => redirect_msg("/admin/books", "图书信息已更新"),
        Err(err) => redirect_msg("/admin/books", &db_err(err)),
    }
}

async fn admin_delete_book(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<IdForm>,
) -> Response {
    if let Err(response) = require_admin(&state, &headers) {
        return response;
    }
    match delete_book_if_available(&state.db_path, &form.id.trim().to_uppercase()) {
        Ok(()) => redirect_msg("/admin/books", "图书已删除"),
        Err(message) => redirect_msg("/admin/books", &message),
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

async fn admin_admins(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<AdminQuery>,
) -> Response {
    let session = match require_admin(&state, &headers) {
        Ok(session) => session,
        Err(response) => return response,
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

async fn admin_add_admin(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<AdminUpsertForm>,
) -> Response {
    if let Err(response) = require_admin(&state, &headers) {
        return response;
    }
    let id = form.id.trim().to_uppercase();
    if !valid_id(&id) {
        return redirect_msg("/admin/admins", "管理员 ID 格式错误");
    }
    let Some(password) = form.password.filter(|p| !p.is_empty()) else {
        return redirect_msg("/admin/admins", "新增管理员必须填写初始密码");
    };
    let conn = match open_conn(&state.db_path) {
        Ok(conn) => conn,
        Err(err) => return redirect_msg("/admin/admins", &db_err(err)),
    };
    let result = conn.execute(
        "INSERT INTO admins (id, name, password_hash, level, remark) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            id,
            form.name.trim(),
            hash_password(&password),
            form.level,
            form.remark.unwrap_or_default()
        ],
    );
    match result {
        Ok(_) => redirect_msg("/admin/admins", "管理员已新增"),
        Err(err) => redirect_msg("/admin/admins", &db_err(err)),
    }
}

async fn admin_update_admin(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<AdminUpsertForm>,
) -> Response {
    if let Err(response) = require_admin(&state, &headers) {
        return response;
    }
    let id = form.id.trim().to_uppercase();
    let conn = match open_conn(&state.db_path) {
        Ok(conn) => conn,
        Err(err) => return redirect_msg("/admin/admins", &db_err(err)),
    };
    let result = if let Some(password) = form.password.filter(|p| !p.is_empty()) {
        conn.execute(
            "UPDATE admins SET name = ?1, password_hash = ?2, level = ?3, remark = ?4 WHERE id = ?5",
            params![
                form.name.trim(),
                hash_password(&password),
                form.level,
                form.remark.unwrap_or_default(),
                id
            ],
        )
    } else {
        conn.execute(
            "UPDATE admins SET name = ?1, level = ?2, remark = ?3 WHERE id = ?4",
            params![
                form.name.trim(),
                form.level,
                form.remark.unwrap_or_default(),
                id
            ],
        )
    };
    match result {
        Ok(0) => redirect_msg("/admin/admins", "未找到该管理员"),
        Ok(_) => redirect_msg("/admin/admins", "管理员信息已更新"),
        Err(err) => redirect_msg("/admin/admins", &db_err(err)),
    }
}

async fn admin_delete_admin(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<IdForm>,
) -> Response {
    let session = match require_admin(&state, &headers) {
        Ok(session) => session,
        Err(response) => return response,
    };
    let id = form.id.trim().to_uppercase();
    if id == "A001" {
        return redirect_msg("/admin/admins", "默认最高权限管理员不可删除");
    }
    if id == session.user_id {
        return redirect_msg("/admin/admins", "不能删除当前登录账号");
    }
    let conn = match open_conn(&state.db_path) {
        Ok(conn) => conn,
        Err(err) => return redirect_msg("/admin/admins", &db_err(err)),
    };
    match conn.execute("DELETE FROM admins WHERE id = ?1", params![id]) {
        Ok(0) => redirect_msg("/admin/admins", "未找到该管理员"),
        Ok(_) => redirect_msg("/admin/admins", "管理员已删除"),
        Err(err) => redirect_msg("/admin/admins", &db_err(err)),
    }
}

async fn admin_records(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<RecordQuery>,
) -> Response {
    let session = match require_admin(&state, &headers) {
        Ok(session) => session,
        Err(response) => return response,
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

async fn admin_borrow(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<BorrowForm>,
) -> Response {
    if let Err(response) = require_admin(&state, &headers) {
        return response;
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
        Err(message) => redirect_msg("/admin/records", &message),
    }
}

async fn admin_return(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<BorrowIdForm>,
) -> Response {
    if let Err(response) = require_admin(&state, &headers) {
        return response;
    }
    match complete_return(&state.db_path, None, form.borrow_id) {
        Ok(()) => redirect_msg("/admin/records", "归还成功"),
        Err(message) => redirect_msg("/admin/records", &message),
    }
}

async fn admin_renew(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<BorrowIdForm>,
) -> Response {
    if let Err(response) = require_admin(&state, &headers) {
        return response;
    }
    match renew_borrow(&state.db_path, None, form.borrow_id) {
        Ok(()) => redirect_msg("/admin/records", "续借成功"),
        Err(message) => redirect_msg("/admin/records", &message),
    }
}

async fn admin_exceptions(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ExceptionQuery>,
) -> Response {
    let session = match require_admin(&state, &headers) {
        Ok(session) => session,
        Err(response) => return response,
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

async fn admin_add_exception(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<ExceptionAddForm>,
) -> Response {
    if let Err(response) = require_admin(&state, &headers) {
        return response;
    }
    let borrow_id = form
        .borrow_id
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .and_then(|s| s.parse::<i64>().ok());
    let conn = match open_conn(&state.db_path) {
        Ok(conn) => conn,
        Err(err) => return redirect_msg("/admin/exceptions", &db_err(err)),
    };
    let result = conn.execute(
        "INSERT INTO exceptions (exception_type, amount, status, process_date, reader_id, book_id, borrow_id, remark)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            form.exception_type,
            form.amount,
            form.status,
            today().to_string(),
            form.reader_id.trim().to_uppercase(),
            form.book_id.trim().to_uppercase(),
            borrow_id,
            form.remark.unwrap_or_default()
        ],
    );
    match result {
        Ok(_) => redirect_msg("/admin/exceptions", "异常记录已保存"),
        Err(err) => redirect_msg("/admin/exceptions", &db_err(err)),
    }
}

async fn admin_resolve_exception(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<ExceptionResolveForm>,
) -> Response {
    if let Err(response) = require_admin(&state, &headers) {
        return response;
    }
    match resolve_exception(&state.db_path, form.id) {
        Ok(()) => redirect_msg("/admin/exceptions", "异常已标记处理完成"),
        Err(message) => redirect_msg("/admin/exceptions", &message),
    }
}

async fn admin_backup(State(state): State<AppState>, headers: HeaderMap) -> Response {
    if let Err(response) = require_admin(&state, &headers) {
        return response;
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

fn list_readers(path: &Path) -> rusqlite::Result<Vec<Reader>> {
    let conn = open_conn(path)?;
    let mut stmt = conn.prepare(
        "SELECT id, name, gender, profession, max_borrow, borrow_days, remark
         FROM readers ORDER BY id",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(Reader {
            id: row.get(0)?,
            name: row.get(1)?,
            gender: row.get(2)?,
            profession: row.get(3)?,
            max_borrow: row.get(4)?,
            borrow_days: row.get(5)?,
            remark: row.get(6)?,
        })
    })?;
    rows.collect()
}

fn get_reader(path: &Path, id: &str) -> rusqlite::Result<Option<Reader>> {
    let conn = open_conn(path)?;
    conn.query_row(
        "SELECT id, name, gender, profession, max_borrow, borrow_days, remark FROM readers WHERE id = ?1",
        params![id],
        |row| {
            Ok(Reader {
                id: row.get(0)?,
                name: row.get(1)?,
                gender: row.get(2)?,
                profession: row.get(3)?,
                max_borrow: row.get(4)?,
                borrow_days: row.get(5)?,
                remark: row.get(6)?,
            })
        },
    )
    .optional()
}

fn list_books(path: &Path) -> rusqlite::Result<Vec<Book>> {
    let conn = open_conn(path)?;
    let mut stmt = conn
        .prepare("SELECT id, title, category, keywords, status, remark FROM books ORDER BY id")?;
    let rows = stmt.query_map([], |row| {
        Ok(Book {
            id: row.get(0)?,
            title: row.get(1)?,
            category: row.get(2)?,
            keywords: row.get(3)?,
            status: row.get(4)?,
            remark: row.get(5)?,
        })
    })?;
    rows.collect()
}

fn list_admins(path: &Path) -> rusqlite::Result<Vec<Admin>> {
    let conn = open_conn(path)?;
    let mut stmt = conn.prepare("SELECT id, name, level, remark FROM admins ORDER BY id")?;
    let rows = stmt.query_map([], |row| {
        Ok(Admin {
            id: row.get(0)?,
            name: row.get(1)?,
            level: row.get(2)?,
            remark: row.get(3)?,
        })
    })?;
    rows.collect()
}

fn list_active_borrows(path: &Path, reader_id: Option<&str>) -> rusqlite::Result<Vec<BorrowView>> {
    let conn = open_conn(path)?;
    let mut stmt = conn.prepare(
        "
        SELECT br.id, br.reader_id, COALESCE(r.name, ''), br.book_id, COALESCE(b.title, ''),
               br.borrow_date, br.due_date, br.renew_count
        FROM borrows br
        LEFT JOIN readers r ON r.id = br.reader_id
        LEFT JOIN books b ON b.id = br.book_id
        WHERE br.returned = 0
          AND (?1 IS NULL OR br.reader_id = ?1)
        ORDER BY br.due_date, br.id DESC",
    )?;
    let rows = stmt.query_map(params![reader_id], |row| {
        Ok(BorrowView {
            id: row.get(0)?,
            reader_id: row.get(1)?,
            reader_name: row.get(2)?,
            book_id: row.get(3)?,
            title: row.get(4)?,
            borrow_date: row.get(5)?,
            due_date: row.get(6)?,
            renew_count: row.get(7)?,
        })
    })?;
    rows.collect()
}

fn list_returns(path: &Path, reader_id: Option<&str>) -> rusqlite::Result<Vec<ReturnView>> {
    let conn = open_conn(path)?;
    let mut stmt = conn.prepare(
        "
        SELECT ret.id, ret.reader_id, COALESCE(r.name, ''), ret.book_id, COALESCE(b.title, ''),
               ret.return_date, ret.due_date, ret.remark
        FROM returns ret
        LEFT JOIN readers r ON r.id = ret.reader_id
        LEFT JOIN books b ON b.id = ret.book_id
        WHERE (?1 IS NULL OR ret.reader_id = ?1)
        ORDER BY ret.return_date DESC, ret.id DESC",
    )?;
    let rows = stmt.query_map(params![reader_id], |row| {
        Ok(ReturnView {
            id: row.get(0)?,
            reader_id: row.get(1)?,
            reader_name: row.get(2)?,
            book_id: row.get(3)?,
            title: row.get(4)?,
            return_date: row.get(5)?,
            due_date: row.get(6)?,
            remark: row.get(7)?,
        })
    })?;
    rows.collect()
}

fn list_exceptions(path: &Path, reader_id: Option<&str>) -> rusqlite::Result<Vec<ExceptionView>> {
    let conn = open_conn(path)?;
    let mut stmt = conn.prepare(
        "
        SELECT ex.id, ex.exception_type, ex.amount, ex.status, ex.process_date,
               ex.reader_id, COALESCE(r.name, ''), ex.book_id, COALESCE(b.title, ''),
               ex.remark
        FROM exceptions ex
        LEFT JOIN readers r ON r.id = ex.reader_id
        LEFT JOIN books b ON b.id = ex.book_id
        WHERE (?1 IS NULL OR ex.reader_id = ?1)
        ORDER BY CASE WHEN ex.status = '已处理' THEN 1 ELSE 0 END, ex.id DESC",
    )?;
    let rows = stmt.query_map(params![reader_id], |row| {
        Ok(ExceptionView {
            id: row.get(0)?,
            exception_type: row.get(1)?,
            amount: row.get(2)?,
            status: row.get(3)?,
            process_date: row.get(4)?,
            reader_id: row.get(5)?,
            reader_name: row.get(6)?,
            book_id: row.get(7)?,
            title: row.get(8)?,
            remark: row.get(9)?,
        })
    })?;
    rows.collect()
}

fn create_borrow(path: &Path, reader_id: &str, book_id: &str, remark: &str) -> Result<(), String> {
    if !valid_id(reader_id) || !valid_id(book_id) {
        return Err("读者 ID 或书籍 ID 格式错误".to_string());
    }
    let mut conn = open_conn(path).map_err(db_err)?;
    let tx = conn.transaction().map_err(db_err)?;
    let (max_borrow, borrow_days): (i64, i64) = tx
        .query_row(
            "SELECT max_borrow, borrow_days FROM readers WHERE id = ?1",
            params![reader_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .map_err(db_err)?
        .ok_or_else(|| "读者不存在".to_string())?;
    let active: i64 = tx
        .query_row(
            "SELECT COUNT(*) FROM borrows WHERE reader_id = ?1 AND returned = 0",
            params![reader_id],
            |row| row.get(0),
        )
        .map_err(db_err)?;
    if active >= max_borrow {
        return Err("借书数量已达上限，无法继续借阅".to_string());
    }
    let status: Option<String> = tx
        .query_row(
            "SELECT status FROM books WHERE id = ?1",
            params![book_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(db_err)?;
    match status.as_deref() {
        Some("available") => {}
        Some("borrowed") => return Err("书籍已借出".to_string()),
        Some("discarded") => return Err("书籍已报废，无法借阅".to_string()),
        Some(_) => return Err("书籍状态异常，无法借阅".to_string()),
        None => return Err("书籍不存在".to_string()),
    }
    let borrow_date = today();
    let due_date = borrow_date + Duration::days(borrow_days);
    tx.execute(
        "INSERT INTO borrows (reader_id, book_id, borrow_date, due_date, returned, renew_count, remark)
         VALUES (?1, ?2, ?3, ?4, 0, 0, ?5)",
        params![
            reader_id,
            book_id,
            borrow_date.to_string(),
            due_date.to_string(),
            remark
        ],
    )
    .map_err(db_err)?;
    tx.execute(
        "UPDATE books SET status = 'borrowed' WHERE id = ?1",
        params![book_id],
    )
    .map_err(db_err)?;
    tx.commit().map_err(db_err)?;
    Ok(())
}

fn complete_return(path: &Path, actor_reader: Option<&str>, borrow_id: i64) -> Result<(), String> {
    let mut conn = open_conn(path).map_err(db_err)?;
    let tx = conn.transaction().map_err(db_err)?;
    let borrow: Option<(String, String, String, i64)> = tx
        .query_row(
            "SELECT reader_id, book_id, due_date, returned FROM borrows WHERE id = ?1",
            params![borrow_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .optional()
        .map_err(db_err)?;
    let Some((reader_id, book_id, due_date_text, returned)) = borrow else {
        return Err("借阅记录不存在".to_string());
    };
    if let Some(actor_reader) = actor_reader {
        if actor_reader != reader_id {
            return Err("只能归还本人借阅的图书".to_string());
        }
    }
    if returned != 0 {
        return Err("该图书已经归还".to_string());
    }
    let due_date = parse_date(&due_date_text)?;
    let now = today();
    if now > due_date {
        let existing: i64 = tx
            .query_row(
                "SELECT COUNT(*) FROM exceptions WHERE borrow_id = ?1 AND status != '已处理'",
                params![borrow_id],
                |row| row.get(0),
            )
            .map_err(db_err)?;
        if existing == 0 {
            let overdue_days = (now - due_date).num_days().max(1);
            tx.execute(
                "INSERT INTO exceptions (exception_type, amount, status, process_date, reader_id, book_id, borrow_id, remark)
                 VALUES ('超期', ?1, '待处理', ?2, ?3, ?4, ?5, ?6)",
                params![
                    overdue_days as f64,
                    now.to_string(),
                    reader_id,
                    book_id,
                    borrow_id,
                    format!("逾期 {overdue_days} 天，按 1 元/天生成赔偿")
                ],
            )
            .map_err(db_err)?;
        }
        tx.commit().map_err(db_err)?;
        return Err("图书已超期，已生成待处理异常记录，需管理员处理后完成归还".to_string());
    }
    finish_return_in_transaction(
        &tx,
        borrow_id,
        &reader_id,
        &book_id,
        &due_date_text,
        "正常归还",
    )?;
    tx.commit().map_err(db_err)?;
    Ok(())
}

fn finish_return_in_transaction(
    tx: &rusqlite::Transaction<'_>,
    borrow_id: i64,
    reader_id: &str,
    book_id: &str,
    due_date: &str,
    remark: &str,
) -> Result<(), String> {
    tx.execute(
        "INSERT INTO returns (reader_id, book_id, borrow_id, return_date, due_date, remark)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            reader_id,
            book_id,
            borrow_id,
            today().to_string(),
            due_date,
            remark
        ],
    )
    .map_err(db_err)?;
    tx.execute(
        "UPDATE borrows SET returned = 1 WHERE id = ?1",
        params![borrow_id],
    )
    .map_err(db_err)?;
    tx.execute(
        "UPDATE books SET status = 'available' WHERE id = ?1 AND status != 'discarded'",
        params![book_id],
    )
    .map_err(db_err)?;
    Ok(())
}

fn renew_borrow(path: &Path, actor_reader: Option<&str>, borrow_id: i64) -> Result<(), String> {
    let mut conn = open_conn(path).map_err(db_err)?;
    let tx = conn.transaction().map_err(db_err)?;
    let borrow: Option<(String, String, i64, i64)> = tx
        .query_row(
            "SELECT br.reader_id, br.due_date, br.returned, br.renew_count
             FROM borrows br WHERE br.id = ?1",
            params![borrow_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .optional()
        .map_err(db_err)?;
    let Some((reader_id, due_date_text, returned, renew_count)) = borrow else {
        return Err("借阅记录不存在".to_string());
    };
    if let Some(actor_reader) = actor_reader {
        if actor_reader != reader_id {
            return Err("只能续借本人借阅的图书".to_string());
        }
    }
    if returned != 0 {
        return Err("已归还图书不能续借".to_string());
    }
    if renew_count >= 2 {
        return Err("单本图书续借次数已达上限".to_string());
    }
    let due_date = parse_date(&due_date_text)?;
    if today() > due_date {
        return Err("书籍已超期，请先办理赔偿手续".to_string());
    }
    let borrow_days: i64 = tx
        .query_row(
            "SELECT borrow_days FROM readers WHERE id = ?1",
            params![reader_id],
            |row| row.get(0),
        )
        .map_err(db_err)?;
    let new_due = due_date + Duration::days(borrow_days);
    tx.execute(
        "UPDATE borrows SET due_date = ?1, renew_count = renew_count + 1,
         remark = TRIM(remark || ' 续借至' || ?1) WHERE id = ?2",
        params![new_due.to_string(), borrow_id],
    )
    .map_err(db_err)?;
    tx.commit().map_err(db_err)?;
    Ok(())
}

fn resolve_exception(path: &Path, exception_id: i64) -> Result<(), String> {
    let mut conn = open_conn(path).map_err(db_err)?;
    let tx = conn.transaction().map_err(db_err)?;
    let item: Option<(String, String, Option<i64>, String)> = tx
        .query_row(
            "SELECT exception_type, book_id, borrow_id, status FROM exceptions WHERE id = ?1",
            params![exception_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .optional()
        .map_err(db_err)?;
    let Some((exception_type, book_id, borrow_id, status)) = item else {
        return Err("异常记录不存在".to_string());
    };
    if status == "已处理" {
        return Err("异常记录已经处理".to_string());
    }
    tx.execute(
        "UPDATE exceptions SET status = '已处理', process_date = ?1 WHERE id = ?2",
        params![today().to_string(), exception_id],
    )
    .map_err(db_err)?;

    if exception_type.contains("丢失") {
        tx.execute(
            "UPDATE books SET status = 'discarded' WHERE id = ?1",
            params![book_id],
        )
        .map_err(db_err)?;
        if let Some(borrow_id) = borrow_id {
            tx.execute(
                "UPDATE borrows SET returned = 1 WHERE id = ?1 AND returned = 0",
                params![borrow_id],
            )
            .map_err(db_err)?;
        }
    } else if let Some(borrow_id) = borrow_id {
        let active: Option<(String, String, String)> = tx
            .query_row(
                "SELECT reader_id, book_id, due_date FROM borrows WHERE id = ?1 AND returned = 0",
                params![borrow_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .optional()
            .map_err(db_err)?;
        if let Some((reader_id, book_id, due_date)) = active {
            finish_return_in_transaction(
                &tx,
                borrow_id,
                &reader_id,
                &book_id,
                &due_date,
                "异常处理后归还",
            )?;
        }
    } else if exception_type.contains("损坏") {
        tx.execute(
            "UPDATE books SET status = 'available' WHERE id = ?1 AND status = 'borrowed'",
            params![book_id],
        )
        .map_err(db_err)?;
    }
    tx.commit().map_err(db_err)?;
    Ok(())
}

fn delete_reader_if_clear(path: &Path, reader_id: &str) -> Result<(), String> {
    let mut conn = open_conn(path).map_err(db_err)?;
    let tx = conn.transaction().map_err(db_err)?;
    let active: i64 = tx
        .query_row(
            "SELECT COUNT(*) FROM borrows WHERE reader_id = ?1 AND returned = 0",
            params![reader_id],
            |row| row.get(0),
        )
        .map_err(db_err)?;
    if active > 0 {
        return Err("尚有未归还图书，无法注销或删除读者".to_string());
    }
    let unpaid: i64 = tx
        .query_row(
            "SELECT COUNT(*) FROM exceptions WHERE reader_id = ?1 AND status != '已处理'",
            params![reader_id],
            |row| row.get(0),
        )
        .map_err(db_err)?;
    if unpaid > 0 {
        return Err("尚有未处理赔偿记录，无法注销或删除读者".to_string());
    }
    let changed = tx
        .execute("DELETE FROM readers WHERE id = ?1", params![reader_id])
        .map_err(db_err)?;
    if changed == 0 {
        return Err("未找到该读者".to_string());
    }
    tx.commit().map_err(db_err)?;
    Ok(())
}

fn delete_book_if_available(path: &Path, book_id: &str) -> Result<(), String> {
    let mut conn = open_conn(path).map_err(db_err)?;
    let tx = conn.transaction().map_err(db_err)?;
    let active: i64 = tx
        .query_row(
            "SELECT COUNT(*) FROM borrows WHERE book_id = ?1 AND returned = 0",
            params![book_id],
            |row| row.get(0),
        )
        .map_err(db_err)?;
    if active > 0 {
        return Err("书籍当前被借阅，无法删除".to_string());
    }
    let changed = tx
        .execute("DELETE FROM books WHERE id = ?1", params![book_id])
        .map_err(db_err)?;
    if changed == 0 {
        return Err("未找到该图书".to_string());
    }
    tx.commit().map_err(db_err)?;
    Ok(())
}
