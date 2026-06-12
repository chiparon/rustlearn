use std::{fs, path::Path};

use chrono::Duration;
use rusqlite::{Connection, OptionalExtension, params};

use crate::models::{Admin, Book, BorrowView, ExceptionView, Reader, ReturnView};
use crate::utils::{hash_password, today};
pub(crate) fn open_conn(path: &Path) -> rusqlite::Result<Connection> {
    let conn = Connection::open(path)?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    Ok(conn)
}

pub(crate) fn init_database(path: &Path) -> rusqlite::Result<()> {
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

pub(crate) fn ensure_daily_backup(db_path: &Path) -> rusqlite::Result<()> {
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

pub(crate) fn vacuum_into(conn: &Connection, path: &Path) -> rusqlite::Result<()> {
    let sql_path = path
        .to_string_lossy()
        .replace('\\', "/")
        .replace('\'', "''");
    conn.execute_batch(&format!("VACUUM INTO '{sql_path}';"))
}

pub(crate) fn list_readers(path: &Path) -> rusqlite::Result<Vec<Reader>> {
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

pub(crate) fn get_reader(path: &Path, id: &str) -> rusqlite::Result<Option<Reader>> {
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

pub(crate) fn list_books(path: &Path) -> rusqlite::Result<Vec<Book>> {
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

pub(crate) fn list_admins(path: &Path) -> rusqlite::Result<Vec<Admin>> {
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

pub(crate) fn list_active_borrows(
    path: &Path,
    reader_id: Option<&str>,
) -> rusqlite::Result<Vec<BorrowView>> {
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

pub(crate) fn list_returns(
    path: &Path,
    reader_id: Option<&str>,
) -> rusqlite::Result<Vec<ReturnView>> {
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

pub(crate) fn list_exceptions(
    path: &Path,
    reader_id: Option<&str>,
) -> rusqlite::Result<Vec<ExceptionView>> {
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
