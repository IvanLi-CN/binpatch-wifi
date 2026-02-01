# 默认端口（Ports）

## 分配策略

- 端口使用高位段，降低与常见开发服务（3000/5173/8080 等）冲突概率。
- 统一绑定 `127.0.0.1`，并启用 strict/exact port，避免“自动换端口导致误连接”。

## 配置源位置（Source of truth）

- Vite dev/preview：`vite.config.ts`
  - `server.port = 65173`
  - `preview.port = 65175`
- Storybook：`package.json`
  - `storybook dev -p 65176 -h 127.0.0.1 --exact-port`

## 当前默认端口一览

| 服务 | 命令 | Host | 端口 | 说明 |
|------|------|------|------|------|
| Web dev（Vite） | `bun run dev` | `127.0.0.1` | `65173` | strict |
| Web preview（Vite） | `bun run preview` | `127.0.0.1` | `65175` | strict |
| Storybook | `bun run storybook` | `127.0.0.1` | `65176` | exact |

## 如何改端口（避免冲突时）

1. 选择新的端口（建议仍在高位段，并保持三者相邻，便于记忆）
2. 同步修改：
   - `vite.config.ts`（dev/preview）
   - `package.json`（storybook）
3. 运行 `bun run dev` 与 `bun run storybook` 确认端口生效（strict/exact 会在冲突时立即报错）

