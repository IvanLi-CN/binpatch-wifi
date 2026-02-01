# 通用固件 Wi‑Fi 配置替换工具：Profile(TOML) v1 规范与契约（#zz4me）

## 状态

- Status: 已完成
- Created: 2026-01-31
- Last: 2026-02-01

## 背景 / 问题陈述

- 工具要“通用”，关键在于把固件内 Wi‑Fi 配置块的定位/校验/写入规则外置为 profile。
- profile 必须足够强表达（endianness/offset/size/checksum/skip），同时保持可验证与可向后兼容扩展。

## 目标 / 非目标

### Goals

- 定义 `profile_version = 1` 的 TOML schema（最小可行 + 明确可扩展点）。
- 定义“强校验命中”的判定流程与多命中处理规则。
- 提供 1 个参考 profile（对应“当前方案规则”），作为后续实现与测试的基准输入。

### Non-goals

- 不在本计划内实现具体算法或 UI。
- 不追求适配任意固件格式（例如 NVS 加密/运行时生成配置）。

## 范围（Scope）

### In scope

- TOML schema：必填字段、默认值、校验规则（profile 自身合法性校验）。
- checksum 算法枚举：首版至少支持 `crc16_modbus`（其余算法作为扩展点定义占位）。
- 可选扩展段：`[flash]` 与 `[github]` 的字段形状（先定“数据结构”，实现可分计划落地）。

### Out of scope

- `.elf` 转烧录镜像（如果需要，另开计划）。

## 需求（Requirements）

### MUST

- 支持 profile 自描述：`id`、`name`、`profile_version`。
- 扫描规则至少支持：`magic_u32`、`endianness`、`struct_size`、`max_matches`。
- layout 至少支持：`version`（值+offset+len）、`checksum`（offset+len）、ssid/psk offset + max + length offset。
- checksum 至少支持：跳过 checksum 字段本身（`skip[]`），以避免自包含导致不稳定。
- profile 解析失败必须给出明确错误（字段缺失/类型错误/越界配置等）。

### SHOULD

- 约束“字节数”口径：ssid/psk 以 UTF‑8 bytes 计，UI 必须展示 bytes 计数。
- 支持 `max_matches` 超限时的 UX 行为：不允许自动选中，必须用户选择或明确报错。

### COULD

- 允许定义“候选过滤”的额外规则（例如字段范围/保留字节必须为 0 等），进一步降低误命中风险。

## 接口契约（Interfaces & Contracts）

### 接口清单（Inventory）

| 接口（Name） | 类型（Kind） | 范围（Scope） | 变更（Change） | 契约文档（Contract Doc） | 负责人（Owner） | 使用方（Consumers） | 备注（Notes） |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Profile TOML v1 | file-formats | external | New | ./contracts/file-formats.md | Ivan | Web App / core lib | 可内置+可上传 |

### 契约文档（按 Kind 拆分）

- [contracts/README.md](./contracts/README.md)
- [contracts/file-formats.md](./contracts/file-formats.md)

## 验收标准（Acceptance Criteria）

- Given：一个 v1 profile（内置或上传）
  When：解析并进行 schema 校验
  Then：得到结构化对象；任何非法字段/缺失都能给出可读错误

- Given：固件 bytes 与 v1 profile
  When：按 profile 描述扫描
  Then：输出“候选列表 + 强校验结果”，且多命中不自动选

## 实现前置条件（Definition of Ready / Preconditions）

- 主人确认：首版要绑定的“当前方案规则”具体字段（magic/version/struct_size/checksum/offset 等）与样本固件。
- 主人确认：`[flash]` 的最小能力边界（单段 vs 多段，见 #9uucu/#f5pnu 依赖）。

## 非功能性验收 / 质量门槛（Quality Gates）

### Testing

- Unit tests: profile schema 校验的正/反例（缺字段、类型错、offset 越界、skip 越界）。

### Quality checks

- Typecheck: profile 类型定义必须可静态检查（避免 `any` 漏网）。

## 文档更新（Docs to Update）

- `profiles/README.md`: profile 字段说明 + 示例（实现阶段晋升为稳定文档）。

## 计划资产（Plan assets）

- Directory: `docs/plan/zz4me-profile-v1/assets/`
- In-plan references: `![...](./assets/<file>.png)`

## 资产晋升（Asset promotion）

| Asset | Plan source (path) | Used by (runtime/test/docs) | Promote method (copy/derive/export) | Target (project path) | References to update | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| Profile v1 文档 | docs/plan/zz4me-profile-v1/contracts/file-formats.md | docs | copy | `profiles/README.md` | `README.md` | 作为稳定文档发布 |

## 实现里程碑（Milestones）

- [x] M1: 定稿 profile schema（v1）与校验规则
- [x] M2: 提供参考 profile（对应当前方案）
- [x] M3: 文档落地（`profiles/README.md`）并从计划资产晋升

## 方案概述（Approach, high-level）

- profile v1 以“可验证、安全优先”为准绳：只要 profile 描述不完整或自相矛盾，必须拒绝执行 scan/patch。
- 扩展策略：v1 字段保持稳定；新增能力只加字段并提供默认值；重大破坏改动通过 `profile_version = 2` 另开。

## 风险 / 开放问题 / 假设（Risks, Open Questions, Assumptions）

- 风险：字段过多导致 profile 难写；字段过少导致误命中风险上升。
- 需要决策的问题：
  - “当前方案规则”是否完全等同于 Power Desk 既有块（用于参考 profile 与单测夹具）。
- 假设（需主人确认）：首版 checksum 算法先只实现 `crc16_modbus`，其余算法暂不纳入 v1 的 MUST。

## 变更记录（Change log）

- 2026-01-31: 新建计划。
- 2026-02-01: 完成 v1 schema（含示例 profile），并晋升稳定文档到 `profiles/README.md`。

## 参考（References）

- Power Desk 参考 profile 来源：`power-desk/web-config-tool/config-tool.js`（仅指路）
