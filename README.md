# binpatch-wifi

Wi‑Fi config patcher for firmware binaries, driven by external TOML profiles.

Deliverables:

- Web UI (offline-first)
- Rust CLI (automation / batch / CI)

Core logic lives in Rust (`crates/binpatch-core`) and is reused by CLI and Web (via WASM).

Planning docs live under `docs/plan/`.

Project-level design docs live under `docs/design/`.

Key docs:

- Design index: `docs/design/README.md`
- Profiles schema: `profiles/README.md`

## Dev

- Install: `bun install`
- Start: `bun run dev` (default: `http://127.0.0.1:65173`)
- Prereq for Web (WASM): `wasm-pack` (install via `cargo install wasm-pack`)
- Format: `bun run format`
- Lint: `bun run lint`
- Check (lint+format rules): `bun run check`
- Typecheck: `bun run typecheck`
- Unit tests: `bun run test`
- Storybook: `bun run storybook` (default: `http://127.0.0.1:65176`)

## Repo hygiene

- One-time git hooks setup: `bunx lefthook install`

## Build

- `bun run build` (outputs `dist/`)

## CLI

- Build: `cargo build -p binpatch-cli`
- Verify: `cargo run -p binpatch-cli -- verify --profile profiles/wifi_cfg_v1_crc16_modbus.toml --firmware <file.bin>`
- Patch: `cargo run -p binpatch-cli -- patch --profile profiles/wifi_cfg_v1_crc16_modbus.toml --firmware <file.bin> --ssid <ssid> --psk <psk>`

Note: avoid pasting PSK in shared terminals/shell history.
