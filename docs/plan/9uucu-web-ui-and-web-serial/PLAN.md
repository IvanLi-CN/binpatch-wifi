# 通用固件 Wi‑Fi 配置替换工具：Web UI 工作流 + Web Serial 烧录（#9uucu）

## 状态

- Status: 待实现
- Created: 2026-01-31
- Last: 2026-01-31

## 背景 / 问题陈述

- 目标用户在浏览器内完成：选择固件 + 选择/上传 profile + 填 SSID/PSK + verify + patch + 下载。
- 可选烧录能力可减少“下载后再找工具刷写”的门槛，但必须强约束用户显式选择设备，且默认不自动连接。

## 目标 / 非目标

### Goals

- 实现完整 UI 流程（离线优先）并接入 core 库。
- UI 必须展示 bytes 计数，避免“字符数≠字节数”导致写入失败。
- 集成可选 Web Serial 烧录：用户显式选择设备后，按 profile 描述写入。

### Non-goals

- 不做设备在线配网（NVS 写入/HTTP provisioning）。
- 不做“自动选择串口/自动连接/自动刷写”。

## 范围（Scope）

### In scope

- 固件输入：上传本地文件（bin/elf）；（从 Releases 下载的输入由 #f5pnu 提供）
- profile 输入：内置 profiles + 上传自定义 TOML（由 #zz4me 定义 schema）
- 展示与操作：
  - verify-only
  - 读取当前 SSID（明文）与 PSK（masked 或仅长度）
  - patch + 下载
- Web Serial（可选）：
  - 仅在浏览器支持时显示入口
  - 用户选择设备后才开始刷写
  - 输出日志与进度，可重试

### Out of scope

- 失败自动恢复/断电保护策略（属于刷写工具层面的增强）。

## 需求（Requirements）

### MUST

- 多命中时：UI 必须让用户选择命中项或明确报错；禁止自动选择。
- SSID/PSK 超过 bytes 上限时：阻止写入并提示“按 bytes 计”的原因。
- 不记录明文 PSK 到 console；错误信息也不得包含明文 PSK。
- 默认下载命名：`<origName>.wifi.<ext>`（或等价规则），并保留原扩展名倾向。
- 烧录：必须用户显式选择设备端口；默认不自动连接。
- 参考实现一致性：
  - 连接成功后自动开始烧录（用户已确认保留该行为）。
  - 本地保存 SSID/PSK（localStorage，带过期策略；用户已确认保留该行为）。

### SHOULD

- 提供“诊断面板”：显示命中 offset（hex）、checksum 值、profile id、校验失败原因（不含敏感信息）。
- 扫描耗时 >3s 时显示进度/提示（UI 层处理）。

### COULD

- 支持一次会话保存“最近使用 profile”（localStorage，不保存 PSK）。

## 接口契约（Interfaces & Contracts）

None

## 验收标准（Acceptance Criteria）

- Given：固件含 1 个合法配置块 + 正确 profile
  When：加载固件并 verify
  Then：显示当前 SSID/PSK（PSK masked）且 valid=true

- Given：同上
  When：输入新 SSID/PSK 并应用后下载 patched 固件，再次加载该固件
  Then：显示新值且 valid=true

- Given：固件不含配置块
  When：verify
  Then：明确报错（不可 silent fail）

- Given：启用烧录且浏览器支持 Web Serial
  When：用户显式选择设备并开始刷写
  Then：刷写日志可见、失败可重试、默认不自动连接设备

## 实现前置条件（Definition of Ready / Preconditions）

- core 库与 profile v1 已可用（#fab9f、#zz4me）。
- 已冻结：烧录 flash map 首版按参考实现固定单段 `0x0000`（后续可通过 profile 扩展多段）。
- 已冻结：`.elf` 仅支持 patch/下载，不直接烧录（网页烧录仅对 `.bin` 启用）。

## 非功能性验收 / 质量门槛（Quality Gates）

### Testing

- Unit tests: UI 状态机的关键路径（无固件/无 profile/多命中/超长/成功下载）。
- E2E tests (if applicable): 如仓库已有惯例则补齐；否则本计划不引入新工具。

### Quality checks

- Accessibility: 文件输入、按钮状态、错误提示可见且可理解。

## 文档更新（Docs to Update）

- `README.md`: 使用说明（上传/选择 profile/verify/patch/下载/烧录）。

## 计划资产（Plan assets）

- Directory: `docs/plan/9uucu-web-ui-and-web-serial/assets/`
- In-plan references: `![...](./assets/<file>.png)`

## 资产晋升（Asset promotion）

None

## 实现里程碑（Milestones）

- [ ] M1: UI 核心流程（上传→verify→patch→下载）
- [ ] M2: 多命中/边界错误处理与诊断面板
- [ ] M3: Web Serial 烧录入口（feature flag）与日志/进度

## 方案概述（Approach, high-level）

- UI 以状态机方式组织：输入（firmware/profile/ssid/psk）→验证→输出（报告/下载/烧录）。
- 任何“联网能力”（Releases 拉取/代理）不与 SSID/PSK 输入绑定；即便联网也只处理固件与公开元数据。

## 风险 / 开放问题 / 假设（Risks, Open Questions, Assumptions）

- 风险：Web Serial 在不同浏览器/权限模型下行为差异大，需日志与可恢复交互。
- 需要决策的问题：烧录 flash map 的首版策略、`.elf` 是否允许烧录。
- 假设（需主人确认）：首版以 `.bin` 烧录为主；`.elf` 提供 patch/下载但不直接刷写。

## 变更记录（Change log）

- 2026-01-31: 新建计划。

## 参考（References）

- Power Desk UI 参考：`power-desk/web-config-tool/README.md`（仅指路）
