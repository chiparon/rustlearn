use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct NoticeQuery {
    pub(crate) msg: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct LoginForm {
    pub(crate) role: String,
    pub(crate) user_id: String,
    pub(crate) password: String,
}

#[derive(Deserialize)]
pub(crate) struct RegisterForm {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) password: String,
    pub(crate) gender: String,
    pub(crate) profession: String,
    pub(crate) remark: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct BookQuery {
    pub(crate) id: Option<String>,
    pub(crate) title: Option<String>,
    pub(crate) category: Option<String>,
    pub(crate) keyword: Option<String>,
    pub(crate) status: Option<String>,
    pub(crate) msg: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct ReaderQuery {
    pub(crate) id: Option<String>,
    pub(crate) name: Option<String>,
    pub(crate) gender: Option<String>,
    pub(crate) profession: Option<String>,
    pub(crate) msg: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct AdminQuery {
    pub(crate) id: Option<String>,
    pub(crate) name: Option<String>,
    pub(crate) level: Option<String>,
    pub(crate) msg: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct RecordQuery {
    pub(crate) reader_id: Option<String>,
    pub(crate) book_id: Option<String>,
    pub(crate) msg: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct ExceptionQuery {
    pub(crate) reader_id: Option<String>,
    pub(crate) book_id: Option<String>,
    pub(crate) exception_type: Option<String>,
    pub(crate) status: Option<String>,
    pub(crate) msg: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct ProfileForm {
    pub(crate) name: String,
    pub(crate) gender: String,
    pub(crate) profession: String,
    pub(crate) remark: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct BorrowForm {
    pub(crate) reader_id: Option<String>,
    pub(crate) book_id: String,
    pub(crate) remark: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct BorrowIdForm {
    pub(crate) borrow_id: i64,
}

#[derive(Deserialize)]
pub(crate) struct ReaderUpsertForm {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) password: Option<String>,
    pub(crate) gender: String,
    pub(crate) profession: String,
    pub(crate) max_borrow: i64,
    pub(crate) borrow_days: i64,
    pub(crate) remark: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct BookUpsertForm {
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) category: String,
    pub(crate) keywords: String,
    pub(crate) status: Option<String>,
    pub(crate) remark: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct AdminUpsertForm {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) password: Option<String>,
    pub(crate) level: i64,
    pub(crate) remark: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct IdForm {
    pub(crate) id: String,
}

#[derive(Deserialize)]
pub(crate) struct ReportExceptionForm {
    pub(crate) book_id: String,
    pub(crate) exception_type: String,
    pub(crate) remark: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct ExceptionAddForm {
    pub(crate) exception_type: String,
    pub(crate) reader_id: String,
    pub(crate) book_id: String,
    pub(crate) borrow_id: Option<String>,
    pub(crate) amount: f64,
    pub(crate) status: String,
    pub(crate) remark: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct ExceptionResolveForm {
    pub(crate) id: i64,
}
