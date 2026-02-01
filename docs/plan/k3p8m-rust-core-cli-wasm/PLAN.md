# 通用固件 Wi‑Fi 配置替换工具：Rust core + Rust CLI + WASM(Web) 统一核心（#k3p8m）

## 状态

- Status: 已完成
- Created: 2026-02-01
- Last: 2026-02-01

## 背景 / 问题陈述

- 需要同时交付：
  - 面向最终用户的 Web（离线优先）
  - 面向自动化/批处理/CI 的 CLI
- 如果 core 逻辑存在两份实现（例如 TS core + Rust CLI），将不可避免地产生行为漂移与安全口径不一致。
- 因此必须收敛为“一份核心”：Rust core 作为唯一权威实现，CLI 与 Web(WASM) 都复用它。

## 目标 / 非目标

### Goals

- 定义并落地仓库结构：Rust workspace（core/cli/wasm）+ Web UI（调用 wasm）。
- Rust core 覆盖完整功能：profile 解析校验 + scan/verify/patch（不依赖 OS/DOM）。
- Rust CLI：提供 verify/patch 工作流（适配批处理），且不泄露 PSK 明文。
- Web：通过 WASM 调用同一核心逻辑实现 verify/patch（UI 负责交互与状态编排）。

### Non-goals

- 不在本计划内实现完整 UI/烧录/联网下载等复杂交互（由 #9uucu/#f5pnu 覆盖）。
- 不在本计划内实现“ELF → 可烧录镜像”的转换（如需要另开计划）。

## 范围（Scope）

### In scope

- Repo layout：
  - `crates/binpatch-core`：Rust core（唯一权威实现）
  - `crates/binpatch-cli`：Rust CLI（文件 IO + 参数解析）
  - `crates/binpatch-wasm`：WASM 导出层（只做 JS/Rust 边界转换）
  - 现有 Web（Vite/Storybook）继续作为 UI 宿主
- Core 能力（与 profile v1 契约一致）：
  - parse/validate profile（TOML v1）
  - scan candidates（magic + struct_size）
  - verify candidate（version + len range + checksum）
  - patch（写入 ssid/psk bytes + len + checksum）
- CLI 工作流：
  - `verify`：输出命中列表/失败原因分类（不含 PSK 明文）
  - `patch`：多命中必须显式选择，否则拒绝写入
- WASM：
  - 暴露 `verify`/`patch` 等最小 API，输入输出以 bytes / JSON 为主

### Out of scope

- Profile 扩展段（`[flash]`/`[github]`）的实际功能落地（可先保留 schema，不在本计划实现）。

## 需求（Requirements）

### MUST

- 单一核心：CLI 与 Web 使用同一套 Rust core 逻辑（不得出现 TS/Rust 双实现）。
- 强校验命中：`magic + version + checksum` 且必须包含 `len range` 校验。
- 多命中处理：返回全部命中项；不得默认选中某一个；需要上层显式选择。
- SSID/PSK 以 UTF‑8 bytes 计；超出 `*_max` 必须拒绝写入并返回“按 bytes 计”的错误。
- 不在任何日志/错误信息中输出明文 PSK（允许输出长度）。

### SHOULD

- Rust core 的错误类型可序列化（WASM/CLI 统一错误分类与展示口径）。
- 提供最少的回归夹具与单测（无需真实固件也能覆盖误命中与边界）。

### COULD

- 未来支持 wasm 多线程/worker（UI 侧分离计算），但本计划不强制。

## 接口契约（Interfaces & Contracts）

- Profile TOML v1（依赖 #zz4me 的契约文档）：
  - `docs/plan/zz4me-profile-v1/contracts/file-formats.md`

## 验收标准（Acceptance Criteria）

- Given：任意 v1 profile（内置或文件）
  When：core 解析并校验
  Then：非法 profile 给出可读错误；合法 profile 可用于 scan/verify/patch

- Given：构造的固件 bytes（含 0/1/多处 magic 候选）
  When：verify
  Then：只保留强校验通过的命中；多命中时不自动选择

- Given：同上
  When：patch 写入新 SSID/PSK
  Then：输出固件再次 verify 为 valid=true，且写入 bytes 与输入一致（补零/长度字段/校验正确）

- Given：CLI
  When：执行 verify/patch 命令
  Then：可在本地离线完成固件处理；输出不包含明文 PSK

## 实现前置条件（Definition of Ready / Preconditions）

- Rust toolchain 可用（stable）。
- 已冻结：Profile TOML v1 schema（以契约文档为准）。

## 非功能性验收 / 质量门槛（Quality Gates）

- Rust：`cargo test` 通过（core 单测覆盖关键边界）。
- Web：仍保持离线可用（本计划不引入强制服务端依赖）。

## 文档更新（Docs to Update）

- `README.md`: 明确两种交付形态（Web/CLI）与共同核心（Rust core）。

## 计划资产（Plan assets）

- Directory: `docs/plan/k3p8m-rust-core-cli-wasm/assets/`
- In-plan references: `![...](./assets/<file>.png)`

## 资产晋升（Asset promotion）

None

## 实现里程碑（Milestones）

- [x] M1: Rust workspace + crates 边界（core/cli/wasm）落地
- [x] M2: Rust core 完成 profile v1 解析校验 + scan/verify/patch
- [x] M3: Rust CLI 提供 verify/patch（多命中需显式选择）
- [x] M4: WASM 导出层可被 Web 引用（API 形状冻结）
- [x] M5: 最小回归单测与文档补齐

## 方案概述（Approach, high-level）

- Rust core 只处理 bytes 与 profile（纯逻辑），不做 IO/DOM。
- CLI 做文件读写与参数解析；Web 做交互与状态机；两者都通过 core 保证一致性。
- WASM 层只负责边界转换（`Uint8Array`/JSON ↔ Rust 类型），不引入额外业务逻辑。

## 风险 / 开放问题 / 假设（Risks, Open Questions, Assumptions）

- 风险：WASM 构建链（wasm-pack/wasm-bindgen）会增加工程复杂度，需要把流程固化为脚本/CI。
- 开放问题：Web 端大文件处理是否需要 worker（避免 UI 卡顿）。
- 假设：首版以离线 patch 为主；联网能力（releases/代理）属于可选增强。

## 变更记录（Change log）

- 2026-02-01: 新建计划（替代 #fab9f 的 TS core 方向）。
- 2026-02-01: 完成 M1/M2/M3/M5（Rust workspace + core/cli/wasm crates、core 单测、CLI 骨架、docs/promote profile v1）。
- 2026-02-01: 完成 M4（wasm-pack 构建 + Web 侧调用 WASM verify/patch；补齐 `docs/design/` 项目级文档）。
