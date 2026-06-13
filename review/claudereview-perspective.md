# libadmin 新一轮评审：从「实验完备」与「Agent 快速构建 Rust 能力证明」两个视角

> 评审对象：`projects/libadmin`，分支 `refactor-manusskilled`
> 本轮基准提交：`95e7b69 Move admin CRUD into services`
> 上一轮评审：`review/claudereview.md`
> 评审日期：2026-06-13

## 0. 这一轮做了什么（对照上一轮 review）

上一轮我给了三个优化方向，第一条是「把散落在 handler 里的 CRUD SQL 下沉到数据/业务层」。本次提交 `95e7b69` 正是冲着这条来的，而且完成度相当高：

- `web/handlers/admin.rs` 从 898 行降到 777 行，其中**内联 SQL 基本清零**——只剩 `admin_backup` 里一处 `open_conn`（那是手动备份、调用 `vacuum_into`，属于合理的基础设施操作，不算业务 SQL 泄漏）。
- `services/mod.rs` 从 451 行增长到 963 行，新增了 `create_reader/update_reader`、`create_book/update_book`、`create_admin/update_admin/delete_admin`、`create_exception` 等业务函数。
- 引入了 4 个入参结构体 `ReaderInput / BookInput / AdminInput / ExceptionInput`，handler 现在只做「鉴权 → 表单映射成 Input → 调 service → 渲染」。
- 服务层测试从 2 个增加到 6 个（含 reader/book/admin CRUD 与守卫），全仓 `#[test]` 共 10 个（utils 4 + services 6）。
- WORKLOG 记录了 `cargo fmt/check/test/clippy -D warnings` 全绿。

结论先行：**这是一次方向正确、执行干净的迭代**。它说明「review → 选最高优先级方向 → 落地 → 记录验证」这个闭环你已经能跑通。下面分两个视角展开。

---

## 视角一：作为「课程实验功能与逻辑完备」的项目

这个视角关心的是：**功能是否齐、业务规则是否对、数据是否一致**。

### 做得好的地方

路由面已经覆盖了实验所需的完整业务：读者/图书/管理员三类主体的增删改查、借阅、归还、续借、异常（超期/丢失/损坏）上报与处理、记录查询、手动与每日备份、登录注册会话。借还核心逻辑是带事务的（`conn.transaction()`），并且把关键业务规则写进了 service：借书校验上限与书籍状态、归还时超期自动生成待处理异常、续借次数上限 2 次与超期拦截、删除读者/图书前检查未还与未结清异常。这些都是实验报告里最能体现「逻辑完备」的点，而且现在它们集中在 `services/mod.rs` 一处，**这恰好是逻辑完备性最该被集中保护的位置**——改规则只改一处，不会出现 handler A 和 handler B 规则不一致。

测试也正好压在业务规则上：借→续→还全流程会断言数据库状态流转，删除守卫会断言「在借图书不可删」。对实验项目而言，这种「规则有测试兜底」比单纯能跑更有说服力。

### 从实验完备角度仍要补的点

1. **错误信息仍是 `Result<(), String>` 字符串**（service 层 15 处）。实验演示时这够用，但它把「业务规则违反」（如"借书已达上限"）和「数据库故障」混成同一种字符串错误，handler 无法区分，最终给用户/批改老师的都是 200 + 一段文字，HTTP 语义上分不出 400/404/500。对「逻辑完备」是个隐性扣分项。

2. **并发一致性的边界**。`create_borrow` 用事务保护了「查上限→插借阅→改书状态」，是对的；但很多读路径仍是「每次 `open_conn` 新开连接」。SQLite 默认串行写问题不大，可一旦演示时并发点两次借同一本书，建议确认书状态判断是否在同一事务内（目前看是的，没问题），这点值得在报告里主动说明，是加分项。

3. **缺少 HTTP 层的端到端验证**。现在 10 个测试都在 service/utils 层，没有一个测试真正打过路由。实验里最容易出错、也最该证明的是「未登录访问 /admin 会被挡、读者越权访问管理员接口会 403」这类**鉴权链路**——而这条链路目前零测试覆盖。

---

## 视角二：作为「Agent 快速上手构建 Rust 项目」的能力证明

这个视角关心的不是图书馆系统本身，而是：**你（借助 agent）能不能稳、快、专业地把一个 Rust 项目从巨石推进到工程化骨架，并让别人看懂这个过程**。

### 这一轮最有价值的能力信号

- **能读评审、能排优先级、能闭环**。上一轮三个方向，你没有一把全抓，而是挑了「下沉 CRUD」这个收益最大、风险可控的先做，并在 WORKLOG 里写清 Goal / Completed / Verification。这种「按 review 驱动、单步可验证」的节奏，本身就是 agent 协作能力证明里最值钱的部分——比一次性堆很多改动更可信。
- **抽象选择是对的**。新增 `*Input` 结构体而不是给 service 函数塞 8 个裸参数，说明在「让 handler 与 service 解耦」时做了正确的 API 设计判断。`pub(crate)` 可见性、目录模块、binary/library 分离也都延续得很稳。
- **质量门是真的**。`clippy -D warnings` 全绿、`fmt --check` 通过，不是「能编译就交」。这正是 `g-rust-skill.md` 里强调的标准命令序列，落到了实处。

### 作为能力证明，下一步最该展示的能力

这里的建议刻意和「实验完备」区分开——能力证明要展示的是**工程判断的广度**，而不只是把一个功能做透：

1. **把错误处理升级成 `thiserror` 类型化错误**（上一轮方向二，仍未动）。对能力证明来说，这一步的意义不在于功能，而在于它能展示你理解 Rust 错误处理的惯用法：库层用 `thiserror` 定义 `enum LibError { NotFound, RuleViolation(String), Db(#[from] rusqlite::Error) }`，handler 据此映射 HTTP 状态码。这是面试/评审里区分「会写 Rust」和「懂 Rust 工程」的典型分水岭，性价比极高。

2. **加 `tests/` 集成测试 + GitHub Actions CI**（上一轮方向，仍未落地）。`g-rust-skill.md` 里已经写好了 CI workflow 模板，但仓库里至今没有 `.github/workflows/`。用 `tower::ServiceExt::oneshot` 对 `Router` 发请求断言鉴权与借还流程，再让 CI 每次 push 自动跑 fmt/check/test/clippy——**「绿色 CI 徽章 + 端到端测试」是能力证明里最直观的一张名片**，它把「我本地验证过」变成「任何人都能复现验证」。这一步建议优先级提到最高，因为它的展示价值对「能力证明」这个目标的杠杆最大。

---

## 总结：两个出发点下，下一步分别该做什么

| 出发点 | 当前状态 | 下一步最该做的一件事 |
|--------|---------|--------------------|
| 课程实验功能/逻辑完备 | 功能已齐、核心规则有事务+测试保护，完备度高 | 补 HTTP 层鉴权与借还的端到端测试，证明链路正确（而不只是单元逻辑正确） |
| Agent 快速构建 Rust 能力证明 | review→落地→验证闭环已跑通，抽象与质量门到位 | 落地 CI + 集成测试，并把错误处理升级为 `thiserror`，展示工程判断广度 |

**两个目标的交汇点是同一件事：加集成测试。** 它既补上实验视角缺失的鉴权链路验证，又是能力证明视角里杠杆最大的一张名片。建议把它作为下一轮的第一优先级；`thiserror` 错误类型化排第二（能力证明权重高）；连接池（上一轮方向三）可以最后做，因为在 SQLite + 当前并发量下它收益最小，更多是「锦上添花」。

整体判断：这一轮迭代干净、方向准、有验证，**作为「能按评审驱动、稳步推进 Rust 重构」的能力证明，它本身就是一份有效证据**。继续保持「一次一个方向、每步都过质量门并记录」的节奏即可。

---

## 附：本轮关键数据

- 基准提交：`95e7b69 Move admin CRUD into services`
- `admin.rs`：898 → 777 行，内联业务 SQL 清零（仅余 `admin_backup` 的基础设施 `open_conn`）
- `services/mod.rs`：451 → 963 行，新增 8 个 CRUD service 函数 + 4 个 `*Input` 结构体
- 测试：service 层 2 → 6，全仓 `#[test]` 共 10（utils 4 + services 6）
- 质量门：`cargo fmt --check` / `check` / `test`（10 passed）/ `clippy -D warnings` 全绿
- 仍未落地：`thiserror`/`anyhow` 错误类型（方向二）、连接池（方向三）、`tests/` 集成测试、`.github/workflows` CI
