# 工作流（Workflows）

本页描述“现在就能用”的真实流程，避免只写口号。

## Web：verify / patch（当前最小可用 UI）

前提：

- `bun install`
- 安装 `wasm-pack`：`cargo install wasm-pack`

步骤：

1. 启动：`bun run dev`（会先构建 WASM，再启动 Vite）
2. 打开：`http://127.0.0.1:65173`
3. 上传固件文件（`.bin`/`.elf` 均可按 bytes 读取）
4. Profile：
   - 默认会加载内置 profile：`profiles/wifi_cfg_v1_crc16_modbus.toml`
   - 你可以直接在页面中编辑 TOML（错误会在 verify 时反馈）
5. 点击 `Verify`：
   - 结果会列出 `valid_offsets`（强校验通过的命中）
   - 若为 0：表示“没有强校验通过的命中”（不会盲目猜测）
6. 填写 SSID/PSK 并 `Patch`：
   - 如果存在多命中：必须选择 `select_index` 才能 patch（避免写错位置）
   - patch 完成后会再次做一次 verify（post-verify），确保仍能命中
7. 下载 patched 固件：页面会提供下载链接（不会展示 PSK 明文）

## CLI：verify / patch（可脚本化）

Verify：

- `cargo run -p binpatch-cli -- verify --profile profiles/wifi_cfg_v1_crc16_modbus.toml --firmware <file.bin>`

Patch（输出文件默认在输入旁边，文件名会追加 `.wifi`）：

- 单命中：`cargo run -p binpatch-cli -- patch --profile profiles/wifi_cfg_v1_crc16_modbus.toml --firmware <file.bin> --ssid <ssid> --psk <psk>`
- 多命中：必须显式选择，例如 `--select-index 0`

注意：

- SSID/PSK 长度以 UTF‑8 bytes 计；超出 profile 的 `*_max` 会拒绝写入
- CLI 输出不会打印 PSK 明文（但仍建议避免把 PSK 放在共享终端/历史记录里）

## 常见问题（Troubleshooting）

- `wasm-pack: command not found`
  - 安装：`cargo install wasm-pack`
- Rust 没有 wasm target
  - 安装：`rustup target add wasm32-unknown-unknown`
- verify 结果多命中
  - 这是预期行为：上层必须显式选择 `select_index` 才能写入（避免写错块）

