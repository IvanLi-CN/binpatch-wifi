# HTTP API

每个 endpoint 一个小节；保持短小但可实现、可测试。

## 下载代理：转发固件资产（GET `/proxy/github/release-asset`）

- 范围（Scope）: internal
- 变更（Change）: New
- 鉴权（Auth）: none（仅允许本地/受信网络启用；若暴露到不受信客户端，必须加鉴权与限流）

### 请求（Request）

- Headers:
  - `Accept: application/octet-stream`
- Query:
  - `path`（string, MUST）：GitHub Release asset 的下载路径（仅 path，不允许完整 URL）
    - 例：`/OWNER/REPO/releases/download/<tag>/<asset-filename>`
    - 必须满足：
      - 以 `/` 开头
      - 不包含 `..`、反斜杠、空字节
      - 正则约束（示例，raw regex）：`^/[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+/releases/download/\\S+$`
        - 注：`\\S` 表示“非空白字符”。避免把 `\\s` 误当成“空白”而在 raw regex 里写成 `[^\\s]` 这种歧义形式。
    - 代理端将其拼接为 `https://github.com{path}` 发起下载
- Body: none

### 响应（Response）

- Success:
  - Status: `200`
  - Body: binary（直接透传 bytes）
- Error:
  - Status: `4xx/5xx`
  - Body: `{ error: string }`（json）

### 错误（Errors）

- `400`: `invalid_path`（retryable: no）
- `400`: `invalid_redirect`（retryable: no）
- `413`: `payload_too_large`（retryable: no）
- `502`: `upstream_failed`（retryable: yes）

### 示例（Examples）

- Request（请求）:
  - `GET /proxy/github/release-asset?path=%2FOWNER%2FREPO%2Freleases%2Fdownload%2Fv1.2.3%2Ffirmware.bin`
- Response（响应）:
  - `200` + binary bytes

### 兼容性与迁移（Compatibility / migration）

- 安全约束（必须实现）：
  - 只允许下载 GitHub Releases 资产：请求只接受 `path`，服务端固定目标域名为 `github.com`
  - 重定向策略：最多跟随 5 次 redirect；仅允许跳转到 `github.com` 或 `objects.githubusercontent.com`，否则返回 `invalid_redirect`
  - 资源限制：必须设置总下载大小上限（建议 64MiB）与超时（建议 30s）
  - 部署限制：默认只绑定 `127.0.0.1`；若要对外暴露，必须增加鉴权（例如静态 token）与限流/滥用防护

- v1 的 contract 已避免“任意 URL 转发”，以降低 SSRF/open-proxy 风险；如需支持更多下载源，必须另开计划扩展并在 contract 中明确 allowlist。
