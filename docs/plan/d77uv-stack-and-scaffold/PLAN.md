# 通用固件 Wi‑Fi 配置替换工具：项目选型与仓库骨架（#d77uv）

## 状态

- Status: 已完成
- Created: 2026-01-31
- Last: 2026-02-01

## 背景 / 问题陈述

- 需要交付给他人使用的“无需重新编译”Wi‑Fi 配置注入工具。
- 现有 Power Desk 网页工具可用但为项目专用，无法复用到未来项目。
- 本仓库将承载“通用版”，以 profile（TOML）外置方式适配不同固件。

## 目标 / 非目标

### Goals

- 建立可持续迭代的仓库结构与工程化骨架（构建、lint/typecheck、测试、静态部署）。
- Web App 以“离线优先”为默认路径（不依赖服务端也能完成上传→替换→下载）。
- 为后续计划预留清晰的模块边界（core/UI/optional features）。

### Non-goals

- 本计划不实现 Wi‑Fi 配置块的扫描/校验/替换算法（由后续核心库计划覆盖）。
- 本计划不实现 GitHub Releases 拉取与 Web Serial 烧录（分别由后续计划覆盖）。

## 范围（Scope）

### In scope

- 技术选型（Build + UI 框架 + 语言 + 测试框架 + 部署方式）。
- Repo layout（`src/` 分层、`profiles/`、`docs/`）。
- 基础脚手架：本地开发、构建产物、静态部署入口。

### Out of scope

- 任何固件二进制处理逻辑（包括 CRC/scan/patch）。

## 需求（Requirements）

### MUST

- 默认离线：不需要启动服务端也能跑（`bun run dev` + 静态 `dist`）。
- 支持大文件（至少 16MB）在浏览器内处理的基础设施：使用 `ArrayBuffer/Uint8Array` 工作流。
- 明确“核心逻辑不依赖 DOM”，可被单元测试覆盖。

### SHOULD

- 支持 GitHub Pages（或等价静态托管）部署。
- CI：最少跑 typecheck + unit tests。

### COULD

- 增加 `pnpm`/`bun` 等工具链（按仓库惯例决定）。

## 接口契约（Interfaces & Contracts）

None

## 验收标准（Acceptance Criteria）

- Given：全新 clone 仓库
  When：按 README 指令安装依赖并启动 dev server
  Then：能打开一个空壳 UI（不含固件处理能力）且无 runtime error

- Given：CI 环境
  When：执行 lint/typecheck/tests
  Then：全部通过（不依赖网络/外部服务）

## 实现前置条件（Definition of Ready / Preconditions）

- 主人确认：首版包管理器（默认 Bun）与部署目标（GitHub Pages 或其它静态托管）。
- 主人确认：是否允许引入一个可选的本地代理 server（用于 GitHub 下载/CORS；默认关闭）。

## 非功能性验收 / 质量门槛（Quality Gates）

### Testing

- Unit tests: 具备测试框架与最小样例测试（例如 core 模块的 placeholder）。

### Quality checks

- Typecheck: 必须开启并在 CI 中执行。
- Formatting: 服从仓库既有约定（若暂无约定，本计划只提出“需确定”，不在 plan 阶段引入新工具）。

## 文档更新（Docs to Update）

- `README.md`: 项目定位、开发/构建/部署方式（骨架落地后再补齐）。

## 计划资产（Plan assets）

- Directory: `docs/plan/d77uv-stack-and-scaffold/assets/`
- In-plan references: `![...](./assets/<file>.png)`

## 资产晋升（Asset promotion）

None

## 实现里程碑（Milestones）

- [x] M1: 初始化前端工程骨架（build/dev）
- [x] M2: 建立模块分层（core/ui/optional）与最小页面
- [x] M3: 建立 CI（typecheck + unit tests）

## 方案概述（Approach, high-level）

- 采用 Vite + TypeScript 的静态站形态，核心逻辑封装为纯函数模块，UI 仅负责输入输出与状态编排。
- 所有“可选联网能力”（GitHub 拉取/代理）与“可选硬件能力”（Web Serial）均做 feature boundary，避免污染离线核心路径。

## 风险 / 开放问题 / 假设（Risks, Open Questions, Assumptions）

- 风险：工程化工具链选择过早固定可能与后续仓库惯例冲突。
- 需要决策的问题：包管理器与部署目标。
- 假设（需主人确认）：首版允许以单仓库同时包含静态站与可选的本地代理（但代理默认不启动）。

## 变更记录（Change log）

- 2026-01-31: 新建计划。
- 2026-02-01: 完成 M1–M3（Vite+TS 骨架、core/ui/optional 分层、CI typecheck+unit tests）。
- 2026-02-01: 工具链默认切到 Bun；Vite dev/preview 默认端口固定为 65173/65175（bind `127.0.0.1`，避免冲突与意外暴露）。
- 2026-02-01: Repo hygiene：引入 Biome + Lefthook + Commitlint（与常用仓库习惯对齐）。
- 2026-02-01: 增加 Storybook（默认端口 65176）作为 UI 基础设施的一部分。

## 参考（References）

- Power Desk 参考：`power-desk/web-config-tool/`（仅指路，不复制内容）
