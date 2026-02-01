# 设计文档（Design）

这里放“项目级”的稳定文档：架构、选型、约定与默认值等。它们用于回答：

- 项目是什么（Web/CLI 交付形态是什么）
- 为什么这样做（关键决策与约束）
- 怎么扩展（新增 profile、新增能力应该改哪里）

注意：这里写的是**稳定口径**；执行计划与验收冻结在 `docs/plan/`。

## 目录

- `architecture.md`：核心架构与模块边界（Rust core / CLI / WASM / Web）
- `tech-stack.md`：技术选型与仓库工程约定（Bun/Vite/Storybook/Rust workspace 等）
- `ports.md`：默认端口与端口分配策略（避免冲突）
- `workflows.md`：实际使用/开发工作流（Web/CLI 如何 verify/patch、常见错误怎么排查）

## 快速结论（给新读者）

- 本项目同时交付 **Web** 与 **CLI** 两种固件替换方案。
- **唯一权威实现**在 Rust：`crates/binpatch-core`。
  - CLI：`crates/binpatch-cli`（批处理 / CI / 自动化）
  - Web：通过 `crates/binpatch-wasm` 暴露 WASM API，由前端调用（不允许 TS/Rust 双实现）
- 固件处理由外部 Profile（TOML）驱动，稳定入口在 `profiles/`（运行/交付不得依赖 `docs/plan/`）。

## 我应该从哪里开始读？

| 你想做什么 | 建议入口 |
|------------|----------|
| 我只想快点跑起来（Web） | 根 `README.md` + `workflows.md` |
| 我只想快点跑起来（CLI） | 根 `README.md`（CLI 部分）+ `workflows.md` |
| 我想理解“如何判定命中/如何校验/如何写入” | `architecture.md`（验证/替换算法） |
| 我想新增/修改 profile（TOML） | `profiles/README.md`（schema + 示例） |
| 我想改默认端口（避免冲突） | `ports.md`（含配置源位置） |

## 相关入口

- 项目根 README：`../../README.md`
- Profiles（稳定 schema + 示例）：`../../profiles/README.md`
- Plans（冻结验收与里程碑）：`../plan/README.md`
