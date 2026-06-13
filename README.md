# rustlearn

A repo that records my Rust language learning, CS110L practice, and small
application projects. It is both a study notebook and a capacity proof: notes,
experiments, working code, reviews, refactors, and verification history stay in
one place.

## Learning Path

- 2026-06-11: set up `rustc` and `cargo` in WSL, then started with small syntax
  experiments under `language/`.
- 2026-06-11: moved from basic grammar into ownership, memory, Cargo, and
  CS110L reading/practice.
- 2026-06-12: switched to project-driven learning and built the first local
  library-management demo from the database course report.
- 2026-06-12 to 2026-06-13: refactored the demo into a structured Rust web
  application with tests, CI, and documented review follow-up work.

## Repository Map

- `language/`: focused Rust language experiments and notes, including grammar,
  functions, enums, closures, memory, and ownership.
- `CS110L/assignment/`: CS110L assignment work and progress notes.
- `projects/trial1/` and `projects/trial2/`: early Cargo project experiments.
- `projects/libadmin/`: Rust + Axum + SQLite library management system.
- `review/`: review notes, definition-of-done material, and follow-up guidance
  used to drive the `libadmin` refactor.

## Main Project: libadmin

`projects/libadmin` is the main applied project in this repo. It started as a
single-file Rust implementation and was refactored into a maintainable web
application:

- thin binary entrypoint in `src/main.rs`;
- application startup and router construction in `src/app.rs`;
- domain/view models in `src/models.rs`;
- request/query forms in `src/forms.rs`;
- SQLite setup, seed data, backups, and query helpers in `src/db/`;
- business logic in `src/services/`;
- HTTP handlers split by auth, dashboard, reader, admin, and shared concerns in
  `src/web/handlers/`;
- HTML rendering helpers and styles in `src/web/views.rs`;
- utility code for password hashing, validation, dates, and database errors in
  `src/utils.rs`;
- typed service errors in `src/errors.rs`.

Implemented behavior includes reader/admin login, cookie sessions, reader
profile maintenance, book search, borrow/return/renew flows, admin CRUD for
readers/books/admins, exception reporting and compensation handling, SQLite seed
data, and backup support.

## Quality Gate

The libadmin worklog records the current quality gate as:

```bash
cd projects/libadmin
cargo fmt -- --check
cargo check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```

Recent verification passed with 16 tests total: 11 unit/service tests and 5 HTTP
integration tests. CI for the same Rust checks lives in
`.github/workflows/libadmin-rust.yml`.

## Notes

- Rust build output such as `target/` is ignored and should not be committed.
- `timeline.txt` is a local personal timeline file and is no longer tracked.
- Project-level implementation details and run commands live in
  `projects/libadmin/README.md`.
- Refactor history and rationale live in `projects/libadmin/WORKLOG.md`.



timeline:
1
6-11 12:12 
installed rustc&cargo in wsl, run `language/try1.rs`.
6-11 12:35
a peek of language basic grammar format, goto hospital for sport treatment.
6-11 18:00
finally back to dorm. Start reading CS110L.

6-11 19:22
learnt basic grammar of rust. turn to ownership hardcore chain?
ready to touch cargo && toolchain stuff.
// commit and push
gofor supper and exercise.

/*
611night & 612morining: hustle for machine learning experiment check.
612 noon: maintain my bicycles.
*/

6-12 15:46
back to project
decide to create a small project driven by vc.
Using :
`codex` agent
`rust` language
`wsl` env of rust compile
`gpt5.5 xh` llm and reasoning effort.

6-12 17:28
successfully built the very first demo. Codex all stirred function inside main.rs.
Search outstand rust repos, observe how they construct and design their files and codes.
Onehand: Use manus to search popular rust repos and summarize their ideas and experiences.
Otherhand: Use chatGPT to write a rust build guide skill.

6-12 18:07
run the gpt version refactor. Go for dinner.
6-12 21:44
Add Worklog and timeline. I always keep codex worklog, but it will limit its capability.
After it runs the refactor and we need a comparison, I think it is the time.

Branch reversed,Start Manus version refactor.

6-12 22:42
completed manus refactor.
dealt branch and work tree issue.

6-12 23:12
use claude code to review current branch code.

6-12 23:28
Adopt refactor version manus-guide for continuous dev.

6-12 23:30
Start CS110L exercises in another work tree. That is because I need more interpretation on rust\
,not just an aiuser.

6-13 00:08
continue VC in codex, realize advice in the review of Claude.llm:`Opus 4.8`

6-13 00:14
Codex finished iteration. Go for claude for further advice.

6-13 01:20
claude return a new version of review, which contains my perspective of order. Throw to codex for
dev. I need goto bed for tomorrow's lab attendance.
//gngl

6-13 09:48
wake up, add review commit, continue CS110L

6-13 10:29
claude reviewed if our small project could end up.

6-13 11:00
instruct codex do last work.
Scrolled pages of runoob.

