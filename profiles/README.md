# Profiles（TOML）

本目录存放可随版本发布的内置 profile（TOML），供 Web/CLI 离线使用。

- 内置 profile 示例：`wifi_cfg_v1_crc16_modbus.toml`

## Profile TOML v1（`*.toml`）

- 范围（Scope）: external
- 变更（Change）: New
- 编码（Encoding）: utf-8

### Schema（结构）

#### Top-level

- `profile_version`（int，MUST）: `1`
- `id`（string，MUST）: profile 的稳定标识（用于日志/诊断输出）
- `name`（string，MUST）: UI 展示名
- `description`（string，optional）

#### `[scan]`（MUST）

- `magic_u32`（int，MUST）: 4 字节 magic（例：`0x57494649`）
- `endianness`（string，MUST）: `little` | `big`
- `struct_size`（int，MUST）: 配置块总字节数
- `max_matches`（int，optional）: 默认 `1`；大于 1 时 UI 必须要求用户选择命中项

#### `[layout]`（MUST）

- `version`（int，MUST）: 固件侧写入的版本值（用于强校验）
- `version_offset`（int，MUST）
- `version_len`（int，optional）: 默认 `2`（u16）

- `checksum_offset`（int，MUST）
- `checksum_len`（int，MUST）

- `ssid_len_offset`（int，MUST）
- `psk_len_offset`（int，MUST）

- `ssid_offset`（int，MUST）
- `ssid_max`（int，MUST）: MUST `<= 255`（因为 `ssid_len_offset` 存的是 `u8`）

- `psk_offset`（int，MUST）
- `psk_max`（int，MUST）: MUST `<= 255`（因为 `psk_len_offset` 存的是 `u8`）

#### `[checksum]`（MUST）

- `algo`（string，MUST）: `crc16_modbus`（v1 仅要求实现该值）
- `skip`（array，optional）: 形如 `[{ offset = 6, len = 2 }]`，用于计算校验时跳过某段（通常为 checksum 字段本身）

#### `[flash]`（optional, v1 可定义形状）

- `segments`（array，optional）:
  - 每项：`{ address = <int>, source = "firmware" }`
  - v1 约定：`source` 先只允许 `"firmware"`（表示“输出 patched 固件文件”）

#### `[github]`（optional, v1 可定义形状）

- `repo`（string，optional）: `owner/name`
- `asset_name_regex`（string，optional）: 用于筛选固件资产
- `prefer_ext`（string，optional）: `"bin"` 或 `"elf"`（只影响 UI 默认选择，不影响安全）

### Examples（示例）

```toml
profile_version = 1
id = "wifi_cfg_v1_crc16_modbus"
name = "Generic WiFi config v1 (CRC16/MODBUS)"

[scan]
magic_u32 = 0x57494649
endianness = "little"
struct_size = 108
max_matches = 4

[layout]
version = 1
version_offset = 4
version_len = 2
checksum_offset = 6
checksum_len = 2
ssid_len_offset = 8
psk_len_offset = 9
ssid_offset = 12
ssid_max = 32
psk_offset = 44
psk_max = 64

[checksum]
algo = "crc16_modbus"
skip = [{ offset = 6, len = 2 }]
```

### 兼容性与迁移（Compatibility / migration）

- v1 的字段若新增：必须有默认值或为 optional，且不改变既有字段语义。
- 若必须改变既有字段语义：通过 `profile_version = 2` 新开 schema。
