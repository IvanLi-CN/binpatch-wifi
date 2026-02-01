# 通用固件 Wi‑Fi 配置替换工具：核心扫描/校验/替换库 + 单测（#fab9f）

## 状态

- Status: 待实现
- Created: 2026-01-31
- Last: 2026-01-31

## 背景 / 问题陈述

- 需要一套“强校验、防误命中”的 core 库，供 UI 与未来项目复用。
- 工具必须离线工作，且不泄露 SSID/PSK。

## 目标 / 非目标

### Goals

- 实现 `parseProfile/scanCandidates/verify/patch` 的纯 TS 核心库。
- 支持 `.bin` 与 `.elf` 输入（两者都按 bytes 扫描定位与替换；输出文件扩展名随输入）。
- 提供可复现的单元测试覆盖核心风险点：误命中、边界长度、多命中、checksum 计算与 skip。

### Non-goals

- 不实现“ELF → 可烧录镜像”的转换（如需要另开计划）。
- 不实现 UI 与 GitHub 下载（由后续计划覆盖）。

## 范围（Scope）

### In scope

- 固件 bytes 处理：
  - scan：按 magic 扫描候选 offset
  - validate_candidate：version/len range/checksum 必须全部通过才算命中
  - patch：写入 ssid/psk len + bytes（剩余补 0）+ 重算 checksum
  - verify_only：只报告命中情况，不改动
- 性能：16MB 文件内扫描+校验应在可接受时间；超时需可中断/进度（由 UI 计划落地，core 提供可分段/可迭代接口）。

### Out of scope

- Web Serial 烧录与设备交互。

## 需求（Requirements）

### MUST

- 命中必须强校验：`magic + version + checksum`（且 checksum 计算需支持 skip）。
- 多命中处理：返回全部命中项；不得默认选中某一个；由 UI 要求用户选择或报错。
- SSID/PSK 以 UTF‑8 bytes 计；若 bytes 超出 `*_max`，必须拒绝写入并返回“超出 bytes”的错误。
- 输出必须能再次 verify 通过（自洽性）。
- 不在任何日志/异常消息中输出明文 PSK（可输出长度/是否为空）。

### SHOULD

- 支持读取并展示固件当前配置（PSK masked），并能从命中项解析出 ssid/psk bytes。
- 校验失败时输出“失败原因分类”（version mismatch / len invalid / checksum mismatch / out of bounds）。

### COULD

- 增加更多“候选过滤”策略（例如保留区必须为 0），进一步降低误命中。

## 接口契约（Interfaces & Contracts）

None（纯内部库 API；对外只通过 Web App 使用）

## 验收标准（Acceptance Criteria）

- Given：一个包含 1 个合法配置块的固件（bin 或 elf）+ 正确 profile
  When：执行 verify
  Then：输出命中数量=1，valid=true，并能解析出当前 ssid/psk_len（PSK 不在日志中明文出现）

- Given：同上
  When：写入新 SSID/PSK 并 patch
  Then：输出固件再次 verify 为 valid=true，且 ssid/psk bytes 与输入一致（填充/清零规则正确）

- Given：固件不含配置块
  When：verify
  Then：明确报错（不可 silent fail）

- Given：固件含多个 magic 候选
  When：verify
  Then：只保留 checksum 通过的候选；若仍 >1，返回多命中并要求上层处理（不随机选）

## 实现前置条件（Definition of Ready / Preconditions）

- `Profile(TOML) v1` 已定稿并有参考 profile（#zz4me）。
- 提供至少 1 个样本固件（bin/elf 各 1 个为佳）用于回归。

## 非功能性验收 / 质量门槛（Quality Gates）

### Testing

- Unit tests:
  - scan 命中/不命中
  - checksum mismatch 不通过
  - skip 区间正确生效
  - ssid/psk 超长（按 bytes）拒绝写入
  - 多命中返回列表且不默认选择

### Quality checks

- Typecheck: core API 类型完整（不允许 `any` 泄漏到外层）。

## 文档更新（Docs to Update）

- `README.md`: 描述离线 patch 的安全承诺与误命中防护逻辑（实现后补齐）。

## 计划资产（Plan assets）

- Directory: `docs/plan/fab9f-core-bytes-patching/assets/`
- In-plan references: `![...](./assets/<file>.png)`

## 资产晋升（Asset promotion）

None

## 实现里程碑（Milestones）

- [ ] M1: core API（parse/scan/verify/patch）实现
- [ ] M2: 单元测试覆盖关键边界与误命中防护
- [ ] M3: 样本固件回归用例（bin/elf）可稳定复现

## 方案概述（Approach, high-level）

- 所有操作均基于 `Uint8Array`；patch 在内存中产生新 bytes（或 copy-on-write），避免污染原始输入。
- 扫描流程先粗筛（magic），再强校验（version/len/checksum），把“误命中概率”压到最低。

## 风险 / 开放问题 / 假设（Risks, Open Questions, Assumptions）

- 风险：`.elf` 作为容器可能包含多处相似字节序列，需依赖强校验与额外过滤降低误命中。
- 需要决策的问题：是否增加额外候选过滤（保留字节/字段约束）作为 v1 的 MUST。
- 已冻结：`.elf` 的“支持”定义为“可 scan/verify/patch/下载”，网页烧录仅对 `.bin` 启用（与参考实现一致）。

## 变更记录（Change log）

- 2026-01-31: 新建计划。

## 参考（References）

- Power Desk core 逻辑参考：`power-desk/web-config-tool/config-tool.js`（仅指路）
