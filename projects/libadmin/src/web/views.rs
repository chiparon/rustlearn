use crate::models::Session;

pub(crate) fn esc(input: &str) -> String {
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

pub(crate) fn status_label(status: &str) -> &'static str {
    match status {
        "available" => "在馆可借",
        "borrowed" => "已借出",
        "discarded" => "报废",
        _ => "未知",
    }
}

pub(crate) fn selected(value: &str, expected: &str) -> &'static str {
    if value == expected { "selected" } else { "" }
}

pub(crate) fn flash(message: Option<&str>) -> String {
    message
        .map(|text| format!("<div class=\"notice\">{}</div>", esc(text)))
        .unwrap_or_default()
}

pub(crate) fn metric(label: &str, value: impl std::fmt::Display) -> String {
    format!(
        "<div class=\"metric\"><span>{}</span><strong>{}</strong></div>",
        esc(label),
        value
    )
}

pub(crate) fn html_table(headers: &[&str], rows: Vec<Vec<String>>) -> String {
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

pub(crate) fn matches_text(value: &str, needle: &Option<String>) -> bool {
    let Some(needle) = needle.as_deref().map(str::trim).filter(|s| !s.is_empty()) else {
        return true;
    };
    value.to_lowercase().contains(&needle.to_lowercase())
}

pub(crate) fn matches_status(value: &str, status: &Option<String>) -> bool {
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

pub(crate) fn layout(title: &str, session: Option<&Session>, body: String) -> String {
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
