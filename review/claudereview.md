# libadmin 两个 refactor 分支结构评审

> 评审对象：`projects/libadmin`（Rust + Axum + rusqlite 图书馆管理系统）
> 对比分支：`refactor-gptskilled`、`refactor-manusskilled`，基线为 `main`
> 评审日期：2026-06-12

## 0. 背景：重构前的状态（main）

`main` 分支上 `libadmin` 是一个**单文件巨石**：`src/main.rs` 一个文件 2918 行，承担了入口启动、路由注册、所有 HTTP handler、全部 SQL、业务规则、HTML 渲染、密码哈希等所有职责。两个 refactor 分支都是在拆这一个文件，参考各自的指南文档完成。

| 维度 | main | refactor-gptskilled | refactor-manusskilled |
|------|------|--------------------|----------------------|
| src 文件数 | 1 | 8 | 16 |
| src 总行数 | 2918 | 2985 | 3224 |
| 指南文档 | 无 | `projects/g-rust-skill.md`（Cargo 工作流） | `projects/Rust 项目架构设计指南.md`（架构原则） |
| 单元/集成测试 | 0 | 0 | 6（utils 3 + services 2 + 自带断言） |
| 业务逻辑是否独立分层 | 否 | 部分（混在 db.rs + routes.rs） | 是（独立 services 层） |

---

## 1. refactor-gptskilled 的结构

```
src/
├── main.rs      (45)   入口：建库、备份、绑端口、起服务
├── auth.rs      (71)   会话提取、权限校验、重定向/403 响应
├── db.rs        (787)  建表、种子数据、备份 + 查询 + 借还/续借/异常事务
├── forms.rs     (145)  Axum Form / Query DTO
├── models.rs    (94)   AppState、Session、领域结构、视图行结构
├── routes.rs    (1650) 路由表 + 39 个 handler（含部分内联 SQL）
├── util.rs      (43)   密码哈希、日期、ID 校验、过滤辅助
└── views.rs     (150)  HTML 转义、layout、表格、CSS
```

**特点**：按"技术职责"做了一次水平切分，是非常标准、低风险的"提取模块"式重构。`main.rs` 瘦身到 45 行很干净，`auth/util/views/forms/models` 边界清晰，可见性统一收敛为 `pub(crate)`，并修了 Clippy（boxed 大 error 类型、合并嵌套 if）。

**问题**：
- `routes.rs` 仍有 1650 行、39 个 handler 挤在一个文件里，只是把"2918 行的巨石"换成了"1650 行的次巨石"。
- 业务规则没有独立分层：借阅/归还/续借的事务逻辑落在 `db.rs`，而新增 reader/book/admin/exception 的写操作 SQL 又直接内联在 `routes.rs` 的 handler 里（`conn.execute("INSERT INTO ...")`）。数据访问点分散在两个文件，职责边界模糊。
- **0 测试**。WORKLOG 自己也承认 `cargo test` reports 0 tests，把补测试列为 follow-up。

---

## 2. refactor-manusskilled 的结构

```
src/
├── main.rs            (4)    极薄入口，只调用 libadmin::run()
├── lib.rs             (9)    模块声明 + pub use app::run
├── app.rs            (144)   AppState、run()、路由表、绑端口
├── models.rs          (70)   会话与领域/视图结构
├── forms.rs          (145)   Axum DTO
├── utils.rs           (64)   密码、日期、ID 校验（含 3 个单元测试）
├── db/mod.rs         (442)   建表、种子、备份、查询辅助（纯数据访问）
├── services/mod.rs   (451)   借/还/续借/异常/删除守卫 业务逻辑（含 2 个集成测试）
└── web/
    ├── mod.rs          (2)
    ├── views.rs      (164)   HTML 渲染
    └── handlers/
        ├── handlers.rs(10)   子模块聚合 + re-export
        ├── shared.rs  (71)   会话/权限/重定向辅助（pub(super)）
        ├── auth.rs   (202)
        ├── dashboard.rs(210)
        ├── reader.rs (338)
        └── admin.rs  (898)
```

**特点**：这是一次**架构级**重构，落实了指南里的多条原则：
- **二进制/库分离**：`main.rs` 4 行，业务全部进 `lib.rs` 暴露的库，使集成测试可直接依赖 crate。
- **三层分明**：`db`（数据访问）→ `services`（业务规则/事务）→ `web/handlers`（HTTP）。handler 通过 `create_borrow / complete_return / renew_borrow / resolve_exception` 调用 service，借还核心逻辑只有一份、可被测试直接调用。
- **目录模块 + 可见性分级**：`web/handlers/*` 按角色（auth/dashboard/reader/admin）拆分，`shared.rs` 用 `pub(super)` 把辅助函数限制在 handler 子树内，内聚更高。
- **带测试**：services 层有 SQLite 落库的借→续→还全流程回归测试，以及"在借图书不可删除"的守卫测试；utils 有纯函数单元测试。

**问题**：
- `web/handlers/admin.rs` 仍有 898 行，是最大的遗留热点。它对 reader/book/admin 的增改删仍内联 SQL（24 处 `conn.execute`/`open_conn`），只有借还/异常这类带事务的复杂逻辑进了 service。CRUD 与业务分层不一致。
- 存在 `web/handlers.rs` 和 `web/handlers/` 目录并存的写法，可读，但 `mod.rs` 与同名 `.rs` 混用对初学者略绕。
- 总行数最高（3224），引入了一定的样板成本（`lib.rs`、`mod.rs`、re-export）。

---

## 3. 三个优化方向

### 方向一：把剩余的内联 SQL 全部下沉到数据/业务层，消灭"次巨石"

两个分支都有同一个病灶——CRUD 的 SQL 还散在 handler 里（gpt 在 `routes.rs`，manus 在 `admin.rs`）。建议把 reader/book/admin/exception 的增改删也做成 `db::*` 或 `services::*` 函数（如 `create_reader / update_book / delete_admin`），handler 只负责"解析表单 → 调函数 → 渲染结果"。这样 `routes.rs`/`admin.rs` 能从 ~900–1650 行降到几百行，单点 SQL 也便于后续加索引、改 schema。对 manus 而言这是"补齐分层一致性"，对 gpt 而言这是"完成它列在 follow-up 里的拆分"。

### 方向二：引入统一错误类型（thiserror / anyhow），替换 `Result<_, String>`

当前两分支的业务函数都返回 `Result<(), String>`，用字符串拼接传错误（`db_err` 把 `rusqlite::Error` format 成中文串）。这丢失了错误类型信息，也无法区分"用户可见的业务错误"和"内部数据库错误"。建议在 `services` 层定义 `enum LibError { NotFound, RuleViolation(String), Db(#[from] rusqlite::Error) }`（`thiserror`），应用层用 `anyhow` 聚合。好处：handler 可按错误类型决定 HTTP 状态码（404 vs 400 vs 500），错误链可追溯，且这正是 manus 指南第 3 条明确推荐、却尚未落地的点。

### 方向三：用连接池 + 共享连接替换"每次操作 open 一次库"

两分支的 `db`/`services` 里几乎每个函数都 `open_conn(path)` 重新打开 SQLite 连接，再加 `PRAGMA foreign_keys=ON`。在并发请求下这是反复的开销与潜在的文件锁竞争。建议把连接管理收敛：用 `r2d2 + r2d2_sqlite`（或 `deadpool`）建池放进 `AppState`，handler/service 从池里取连接。既减少开销，也让 `AppState` 从"持有路径字符串"升级为"持有真正的资源句柄"，更符合 Rust 的资源所有权习惯。

---

## 4. 两个提升应用能力的建议

### 建议一：补齐自动化测试与 CI，把"能编译"升级为"有保障"

manus 已经起了个好头（6 个测试），gpt 则是 0 测试。下一步应：(1) 在 `tests/` 加**集成测试**，用 `axum::Router` + `tower::ServiceExt::oneshot` 直接对路由发请求，断言登录、借书、越权访问的响应，这能覆盖 handler 层而不只是 service；(2) 把 `g-rust-skill.md` 里已经写好的 GitHub Actions 工作流真正落地为 `.github/workflows/rust.yml`，让 `fmt / check / test / clippy` 在每次 push 自动跑。先有测试和绿色 CI，后续重构（如方向一/二/三）才敢放心大改。这是从"写完能跑"到"工程化交付"的关键一跃。

### 建议二：补安全与健壮性短板，理解 Web 应用的真实约束

这套系统作为能力证明已经不错，但有几处典型的生产级缺口值得作为学习项补上：(1) **会话存在内存 HashMap**，进程重启即丢失，且无过期机制——可学习 cookie 签名/JWT 或持久化 session；(2) 密码哈希用的是 `SHA-256 + 固定前缀盐`，应换成 `argon2`/`bcrypt` 这类慢哈希、每用户独立盐；(3) HTML 输出虽有 `esc()` 转义，但建议系统性梳理所有用户输入的拼接点，确认无 XSS/SQL 注入遗漏（SQL 目前用了参数化 `params!`，是对的，保持）。这些点把练习项目和真实 Web 服务之间的差距讲清楚，比单纯拆模块更能提升工程判断力。

---

## 5. 两个分支版本对比与结论

| 对比项 | refactor-gptskilled | refactor-manusskilled |
|--------|--------------------|----------------------|
| 重构性质 | 提取模块（水平切分技术职责） | 分层架构（垂直 + 二进制/库分离） |
| 入口 `main.rs` | 45 行 | 4 行（+ `lib.rs` 9 行） |
| 业务逻辑分层 | 无独立层，散在 db.rs + routes.rs | 独立 `services` 层，handler 调用 |
| 最大文件 | `routes.rs` 1650 行 | `admin.rs` 898 行 |
| handler 组织 | 全挤在 routes.rs（39 个） | 按角色拆 4 个文件 + shared |
| 可测试性 | 低（0 测试，逻辑与 IO 耦合） | 较高（6 测试，库可被 tests 依赖） |
| 文件/样板数量 | 少（8 文件） | 多（16 文件，略多样板） |
| 改动激进程度 | 保守、低风险 | 较激进、改动面大 |
| 与各自指南契合度 | 契合（指南偏 Cargo 工作流，对结构要求低） | 高度契合（指南明确要求库分离/分层/测试） |

**结论**：

`refactor-gptskilled` 是一次**安全、克制的模块提取**。它把巨石按技术职责切成 8 块，`main.rs` 干净、可见性收敛、Clippy 通过，作为"第一步重构"完全合格、风险极低。但它没有触及更深的职责分层——`routes.rs` 仍是 1650 行的次巨石，业务逻辑没有独立、也没有任何测试。它的上限受限于参考文档：`g-rust-skill.md` 本质是一份 **Cargo 操作工作流**，强在"怎么 build/test/lint"，弱在"怎么设计模块边界"。

`refactor-manusskilled` 是一次**更彻底的架构重构**。它落实了二进制/库分离、`db → services → web` 三层、目录模块与 `pub(super)` 可见性分级，并真正写了可运行的回归测试。它更接近一个可持续演进的工程骨架，代价是文件和样板更多、`admin.rs` 仍偏大、改动面更大。它的优势同样来自文档：`Rust 项目架构设计指南.md` 系统讲了分层、错误处理、测试策略，给了它明确的架构靶子。

**总体推荐 `refactor-manusskilled` 作为继续演进的基线**：它在可测试性、职责分离、可维护性上明显更优，且与"能力证明"这一仓库目标（展示对 Rust 工程实践的掌握）更契合。理想做法是在 manus 的分层骨架上，吸收 gpt 的克制（避免过度样板），再按第 3 节三个方向补齐——把 `admin.rs` 的 CRUD 下沉、引入 `thiserror` 错误类型、加连接池——即可得到一个兼顾整洁与健壮的版本。若评判标准只是"低风险地让 main.rs 变小"，则 gpt 版足够；若标准是"展示架构设计能力并支撑后续迭代"，manus 版胜出。
