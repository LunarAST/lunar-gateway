# lunar-gateway

**LunarAST 生态数据分发无状态边缘网关**

`lunar-gateway` 是基于 Cloudflare Workers + Wasm 构建的服务，可对 `lunar map` 生成的全局拓扑文件 `lunar-map.json` 提供安全、带缓存、支持参数筛选的访问能力。它是面向生产环境的 `lunar-serve` 替代方案，具备全球 CDN 加速、Ed25519-JWT 身份鉴权、边缘节点原生缓存等能力。

---

## 快速上手

### 1. 环境前置要求
- 拥有 Cloudflare 账号，并已开启 Workers 与 R2 对象存储功能
- 安装 [Wrangler](https://developers.cloudflare.com/workers/wrangler/) 命令行工具
- 提前通过 `lunar map` 生成好 `lunar-map.json` 拓扑数据文件

### 2. 配置 R2 对象存储桶
创建 R2 存储桶（示例名称：`lunar-ast-yourorg`），并上传拓扑数据文件：

```bash
wrangler r2 bucket create lunar-ast-yourorg
wrangler r2 object put lunar-ast-yourorg/lunar-map.json -f /path/to/lunar-map.json
```

### 3. 部署服务
编辑 `wrangler.toml` 配置文件，修改为你实际的存储桶名称，并填入公钥信息：

```toml
name = "lunar-gateway"
main = "src/lib.rs"
compatibility_date = "2026-06-08"

[[r2_buckets]]
binding = "LUNAR_BUCKET"
bucket_name = "lunar-ast-yourorg"

[vars]
LUNAR_PUBLIC_KEYS = "{ \"your-project\": \"<public-key-hex>\" }"
```

执行部署命令：
```bash
wrangler deploy
```

---

## 接口列表

| 接口地址 | 鉴权要求 | 功能说明 |
|:---|:---|:---|
| `GET /healthz` | 无需鉴权 | 健康状态检测 |
| `GET /lunar-map.json` | 无需鉴权 | 获取完整拓扑 JSON 数据 |
| `GET /lunar-map.md` | 无需鉴权 | 以 Markdown 格式渲染拓扑，支持筛选参数（`?summary`、`?scope`、`?status`、`?path`、`?style`） |
| `GET /private/lunar-map.md` | 需 JWT 鉴权 | 功能与上方一致，访问私有项目必须提供合法 Ed25519 格式 JWT 令牌 |

### 筛选参数说明

| 参数 | 示例 | 功能说明 |
|:---|:---|:---|
| `?summary=true` | `?summary=true` | 输出生态整体摘要（约 200 字符） |
| `?scope=<project>` | `?scope=auth-service` | 仅展示单个项目视图 |
| `?status=<status>` | `?status=orphaned` | 按契约状态筛选接口 |
| `?path=<path>` | `?path=/auth` | 按接口路径关键字筛选 |
| `?style=mermaid` | `?style=mermaid` | 输出 Mermaid 格式拓扑图表 |

---

## 身份鉴权
`lunar-gateway` 采用 **Ed25519（EdDSA）** 算法完成 JWT 签名校验，相关规范详见生态总规范第 8.3 章节。

### 1. 生成密钥对
```bash
lunar keygen my-project
```
执行后会在 `~/.lunar/keys/my-project.key` 生成私钥，并在终端输出对应的公钥。

### 2. 注册公钥
将公钥信息填写到 `wrangler.toml` 的 `LUNAR_PUBLIC_KEYS` 配置项中，格式如下：
```json
{ "my-project": "8a7b3c4d5e6f..." }
```

### 3. 签发 JWT 令牌
使用上述私钥生成标准 JWT，令牌载荷中需包含 `sub` 字段（对应项目名称）。
调用 `/private/lunar-map.md` 接口时，在请求头中携带 `Authorization: Bearer <令牌内容>` 完成鉴权。

---

## 配置项说明

### 环境变量

| 环境变量 | 说明 | 默认值 |
|:---|:---|:---|
| `LUNAR_PUBLIC_KEYS` | JSON 格式映射表，存储项目名与对应 Ed25519 十六进制公钥 | 无 |
| `LUNAR_CLOCK_SKEW_SECONDS` | JWT 校验允许的最大时钟偏移时长（秒） | `30` |
| `LUNAR_KEY_CACHE_TTL_SECONDS` | 公钥缓存有效期（秒） | `300` |
| `LUNAR_GATEWAY_BUFFER_LIMIT_BYTES` | 流式传输触发前的内存缓冲区上限（默认 2MB） | `2097152` |

---

## 与其他组件的关联关系

| 组件 | 关联说明 |
|:---|:---|
| `lunar` 命令行工具 | 生成 `lunar-map.json` 拓扑文件，为本网关提供数据源 |
| `lunar-serve` | 本地开发环境使用，是本网关的替代方案 |
| `lunar-scope` | 可从本网关拉取 `lunar-map.json` 数据用于可视化展示 |
| LunarAST 生态总规范 | 本网关整体架构定义在规范第 8 章节 |

`lunar-gateway` 属于纯只读数据分发服务，不会向磁盘写入任何文件，也不会修改配置。

---

## 开源许可证
Apache-2.0
