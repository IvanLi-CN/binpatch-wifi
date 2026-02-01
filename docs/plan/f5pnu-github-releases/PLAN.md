# 通用固件 Wi‑Fi 配置替换工具：GitHub Releases 拉取与下载（可选代理）（#f5pnu）

## 状态

- Status: 待实现
- Created: 2026-01-31
- Last: 2026-01-31

## 背景 / 问题陈述

- 期望用户“不用本地找文件”，可以直接从 GitHub Releases 选择固件版本下载后再 patch。
- 浏览器侧直接下载 Release 资产常会遇到 CORS / 跨域限制，可能需要一个可配置的下载代理。

## 目标 / 非目标

### Goals

- UI 支持配置一个 GitHub repo（来自 profile 或用户输入），列出 releases 与 assets，允许选择下载固件。
- 设计一个**默认关闭**的可选本地代理（或自托管代理）来解决下载与 CORS 问题。
- 确保任何网络请求都不携带 SSID/PSK（包括 query 参数、路径、header）。

### Non-goals

- 不做复杂的“固件资产命名约定自动推断”；只提供 regex/规则配置（来自 profile）。
- 不实现 GitHub 私有仓库的 OAuth 登录（首版默认公开仓库）。

## 范围（Scope）

### In scope

- GitHub Releases 列表与资产选择（bin/elf 资产都可）。
- 下载方式两条路径：
  1) 直连 GitHub（成功则无需代理）
  2) 通过可选代理转发下载（用于规避 CORS）
- 安全约束：网络层严禁发送 SSID/PSK。

### Out of scope

- 代理侧的缓存/鉴权/配额（首版只做最小可用）。

## 需求（Requirements）

### MUST

- Repo 选择必须可配置（不能写死 allowlist）。
- 下载前必须展示资产信息（文件名、大小、类型），并由用户显式确认/选择。
- 任何网络请求不得包含 SSID/PSK：即使开启 GitHub 功能，也必须把用户输入与网络请求解耦。
- 离线模式仍可用：用户可以跳过 GitHub 下载，直接上传本地固件。
- （若实现代理）必须避免 SSRF/open proxy：代理只允许 GitHub Releases 资产路径输入（不接受任意 URL），并限制 redirect 目标域名。

### SHOULD

- 支持 profile 提供默认 repo 与 asset 筛选规则（regex）。
- 下载失败能给出可读错误与建议（例如提示用户改用代理或本地上传）。
- UI 应引导用户优先选择 `.bin` 资产用于网页烧录；`.elf` 仅用于 patch/下载与高级用户场景。

### COULD

- 增加“拖拽固件 URL”下载（仍需遵循安全约束与 CORS 策略）。

## 接口契约（Interfaces & Contracts）

### 接口清单（Inventory）

| 接口（Name） | 类型（Kind） | 范围（Scope） | 变更（Change） | 契约文档（Contract Doc） | 负责人（Owner） | 使用方（Consumers） | 备注（Notes） |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Download proxy | http-apis | internal | New | ./contracts/http-apis.md | Ivan | Web App | 默认关闭，可选启用 |

### 契约文档（按 Kind 拆分）

- [contracts/README.md](./contracts/README.md)
- [contracts/http-apis.md](./contracts/http-apis.md)

## 验收标准（Acceptance Criteria）

- Given：一个公开 GitHub repo（由用户或 profile 配置）
  When：打开“从 Releases 选择固件”
  Then：能列出 releases 与 assets，并能选择一个资产下载为 bytes

- Given：浏览器直连下载因 CORS 失败
  When：用户启用代理并重试
  Then：下载成功且未泄露 SSID/PSK（网络请求不包含用户输入）

## 实现前置条件（Definition of Ready / Preconditions）

- 主人确认：首版是否必须支持“代理模式”（若必须，需决定部署形态：本地 server / 自托管）。
- 主人确认：repo 配置来源优先级（profile 默认值 vs 用户手动输入）。

## 非功能性验收 / 质量门槛（Quality Gates）

### Testing

- Unit tests: URL 构造与“禁止携带敏感字段”检查（可通过 request builder 单测验证）。

### Quality checks

- Security: 不向 console 输出 SSID/PSK；网络层日志仅允许长度或固定占位符。

## 文档更新（Docs to Update）

- `README.md`: GitHub 下载使用方式 + 代理启用方式 + 安全说明（实现后补齐）。

## 计划资产（Plan assets）

- Directory: `docs/plan/f5pnu-github-releases/assets/`
- In-plan references: `![...](./assets/<file>.png)`

## 资产晋升（Asset promotion）

None

## 实现里程碑（Milestones）

- [ ] M1: GitHub Releases 列表与资产选择（直连路径）
- [ ] M2: 可选代理（默认关闭）与 UI 开关
- [ ] M3: 安全回归：请求不携带 SSID/PSK（含单测）

## 方案概述（Approach, high-level）

- GitHub API 查询与 asset 下载采用“无状态请求构造”，与用户输入（ssid/psk）完全隔离。
- 代理仅做“转发下载”与必要的 header 处理，不做业务逻辑；并且 repo/资产选择仍由客户端控制（避免代理写死 allowlist）。

## 风险 / 开放问题 / 假设（Risks, Open Questions, Assumptions）

- 风险：不同资产下载域名/重定向策略会影响 CORS 成功率，导致“是否需要代理”在不同浏览器环境下不稳定。
- 需要决策的问题：代理的形态与部署策略。
- 假设（需主人确认）：首版只覆盖公开 repo；私有仓库后续另开计划。

## 变更记录（Change log）

- 2026-01-31: 新建计划。

## 参考（References）

- GitHub Releases：用户期望功能（需求简述见 requirements brief）
