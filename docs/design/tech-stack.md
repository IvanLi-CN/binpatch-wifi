# 技术选型（Tech stack）

## 总原则

- **一份权威核心**：固件 bytes 的 scan/verify/patch 只在 Rust core 实现一次。
- **多形态交付**：CLI 与 Web 都复用同一核心（Web 通过 WASM）。
- **可复现**：工具链尽量固定版本、脚本化构建与验证。

## 版本与“真源”（Source of truth）

- Rust 版本：`rust-toolchain.toml`（当前固定 `1.91.0`，并声明 `wasm32-unknown-unknown` target）
- Rust 依赖锁：`Cargo.lock`
- Bun 版本：`.bun-version` + CI 固定（见 `.github/workflows/ci.yml`）
- Web 依赖锁：`bun.lock`

## Rust

- Workspace：根 `Cargo.toml`（crate 见 `crates/`）
  - `binpatch-core`：纯逻辑核心（profile 解析、scan/verify/patch）
  - `binpatch-cli`：命令行工具（文件 IO + 参数解析）
  - `binpatch-wasm`：WASM 导出层（仅 JS/Rust 边界转换）
- Toolchain：`rust-toolchain.toml` 固定 Rust 版本，并包含 `wasm32-unknown-unknown` target。

## Web

- 前端框架：Vite + React + TypeScript（静态站点，可离线优先）
- WASM 构建：`wasm-pack build crates/binpatch-wasm --target web`（输出 `crates/binpatch-wasm/pkg/`，已被 gitignore）
- Web 侧调用 Rust core 的位置：
  - `src/core/binpatchWasm.ts`：lazy-load + `await init()` 后调用 `verify/patch`
  - `src/core/defaultProfile.ts`：默认内置 profile（从 `profiles/*.toml` 以 `?raw` 方式读取）

## 包管理与工程化

- 包管理器：Bun（`bun.lock`）
- UI 组件/页面预览：Storybook（`bun run storybook`）
- 代码格式化/检查：Biome（`bun run check` / `bun run format`）
- Git hooks：Lefthook（`bunx lefthook install`）
- Commit 规范：commitlint（conventional commits）
- CI：GitHub Actions（Web + Rust 两条流水线）

## 为什么这样选（简短理由）

- Rust core：避免 TS/Rust 双实现导致行为漂移；对安全口径（强校验、多命中选择）更可控。
- WASM：让 Web 与 CLI 复用同一核心，而不是复制算法到前端。
- Bun + Vite：本地开发启动快、锁文件明确；静态产物易发布（离线优先）。
- Storybook：默认提供 UI 变更的可视化回归入口（不把 UI 逻辑藏在页面里）。
