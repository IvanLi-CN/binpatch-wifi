# 架构（Architecture）

## 目标

- 支持两种使用形态：
  - **Web UI**：面向最终用户，离线优先
  - **CLI**：面向自动化/批处理/CI
- 保证“行为一致性”：固件扫描/校验/替换逻辑 **只能存在一份**。

## Repo 结构（与模块边界对应）

```text
.
├─ crates/
│  ├─ binpatch-core/   # 唯一权威：scan/verify/patch + Profile(TOML) 校验
│  ├─ binpatch-cli/    # CLI：文件 IO + 参数解析（调用 core）
│  └─ binpatch-wasm/   # WASM：JS/Rust 边界转换（调用 core）
├─ profiles/           # 稳定的 profile（可随版本发布，Web/CLI 直接引用）
└─ src/                # Web UI（Vite/React，调用 WASM）
```

## 高层模块

### `crates/binpatch-core`（唯一权威实现）

- 输入：Profile（TOML v1 文本） + 固件 bytes（`&[u8]`）
- 输出：verify 命中列表（offset + 分类）、patch 后 bytes
- 约束：
  - 只做纯逻辑（不做 IO、不依赖 OS/DOM）
  - 多命中 **不得默认选择**，必须由上层显式选择
  - 任何日志/错误信息 **不得输出明文 PSK**（允许输出长度）

#### Core 的稳定入口（当前实现）

- Profile 解析/校验：`Profile::parse_toml`（`crates/binpatch-core/src/profile.rs`）
- 扫描候选：`Profile::scan_candidates`（`crates/binpatch-core/src/firmware.rs`）
- 强校验：`Profile::verify`（返回 `VerifyReport { valid: Vec<Candidate> }`）
- 替换写入：`Profile::patch`（需要 `select_index` 才允许多命中写入）

### `crates/binpatch-cli`（Rust CLI）

- 负责：文件 IO、参数解析、将用户输入映射到 core
- 命令：`verify` / `patch`
- 目标：可脚本化、可在 CI/批处理场景可靠运行

### `crates/binpatch-wasm`（WASM 导出层）

- 只负责 JS/Rust 边界转换（`Uint8Array`/JSON ↔ Rust 类型）
- 不新增业务逻辑，不允许出现与 core 不一致的“第二份实现”

#### WASM API（当前冻结形状）

- `verify(profile_toml: string, firmware: Uint8Array) -> { valid_offsets: number[] }`
- `patch(profile_toml: string, firmware: Uint8Array, ssid: string, psk: string, select_index?: number) -> Uint8Array`

### `Web（Vite + React + TS）`

- 负责：文件选择、输入校验、展示 verify 结果、驱动 patch 流程
- 通过 WASM 调用 Rust core，作为 UI 宿主（不直接实现固件处理算法）

## 数据流（简化）

1. 用户在 Web/CLI 选择 Profile 与固件文件
2. 上层读取 bytes（Web: `File` → `Uint8Array`；CLI: `fs::read`）
3. 走统一核心：
   - verify：返回命中 offsets（及失败原因分类）
   - patch：写入 SSID/PSK bytes + len 字段 + checksum，再次 verify 必须有效

## 验证算法（精确口径，按当前实现）

### 1) scan_candidates（候选扫描）

候选成立的必要条件：

- 在固件上按字节滑窗匹配 `magic_u32`（按 `scan.endianness` 转为 4 bytes 比较）
- 且候选位置后必须能完整容纳 `scan.struct_size`（越界的 magic 直接忽略）
- `scan.max_matches` 用于上限保护（默认 1，超过会报错）

### 2) verify_candidate（强校验）

对每个候选块（`struct_size` bytes），按以下顺序强校验：

1. version：读取 `layout.version_offset + layout.version_len(默认 2)`，按 endianness 解析为整型，必须等于 `layout.version`
2. length range：读取 `ssid_len_offset` / `psk_len_offset`（u8），必须 `<= ssid_max/psk_max`
   - 因为 length 字段是 `u8`，所以 `ssid_max/psk_max` 必须 `<= 255`
3. checksum：读取存储的 checksum（按 endianness 解析为 u16），并对整块计算 CRC16/MODBUS
   - 若 profile 配置了 `checksum.skip`，计算时跳过这些片段（典型用法：跳过 checksum 字段本身）
   - 计算值必须等于存储值

verify 的输出：只返回强校验通过的 offsets；若 0 个通过会给出错误（并不会“猜一个”）。

## 替换算法（精确口径，按当前实现）

1. 先 `verify`，得到 valid candidates
2. 命中选择：
   - 只有 1 个命中：允许 `select_index` 为空
   - 多命中：必须显式提供 `select_index`（否则拒绝写入）
3. 写入：
   - `ssid/psk` 以 **UTF‑8 bytes** 计长度；超出 `*_max` 直接拒绝
   - 写 `ssid_len_offset` / `psk_len_offset`
   - 写 `ssid_offset..ssid_offset+ssid_max` / `psk_offset..`：先清零再写入（padded with zeros）
   - 重新计算 checksum 并写回 `layout.checksum_offset..`
4. 输出 patched firmware bytes（Web/CLI 上层负责下载/写文件）

## 构建与工具链（WASM）

- Web 侧会复用 Rust core 的 WASM 产物，因此需要：
  - Rust target：`wasm32-unknown-unknown`
  - 构建工具：`wasm-pack`（用于生成 `crates/binpatch-wasm/pkg/`）

## 稳定资产（Stable assets）

- `profiles/`：Profile TOML v1 的稳定样例与说明（供 CLI/Web 直接引用）
- `docs/design/`：项目级设计与约定
- `docs/plan/`：冻结验收与执行计划（不作为运行时依赖）
