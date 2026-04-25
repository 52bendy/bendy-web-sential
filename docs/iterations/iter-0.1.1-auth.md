# Iteration 0.1.1: Authentication & Authorization

**Date**: 2026-04-24
**Status**: Completed

## 需求

- JWT 校验：路由配置 `auth_strategy=jwt` 时验证请求中的 Bearer Token
- API Key 校验：路由配置 `auth_strategy=api_key` 时验证请求中的 `X-API-Key` 头
- 角色鉴权：根据 `min_role` 字段检查用户角色，角色层级：`superadmin > admin > user`
- API Key 管理：提供 CRUD 接口管理 API Key

## 数据库变更

新增 `migrations/006_gateway_auth.sql`：

```sql
-- bws_api_keys 表
CREATE TABLE bws_api_keys (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    key_hash TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    role TEXT NOT NULL DEFAULT 'user',
    active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    expires_at TEXT,
    last_used_at TEXT
);
CREATE INDEX idx_apikeys_hash ON bws_api_keys(key_hash);

-- bws_routes 新增列
ALTER TABLE bws_routes ADD COLUMN auth_strategy TEXT DEFAULT 'none';
ALTER TABLE bws_routes ADD COLUMN min_role TEXT DEFAULT NULL;
ALTER TABLE bws_routes ADD COLUMN ratelimit_window INTEGER DEFAULT NULL;
ALTER TABLE bws_routes ADD COLUMN ratelimit_limit INTEGER DEFAULT NULL;
ALTER TABLE bws_routes ADD COLUMN ratelimit_dimension TEXT DEFAULT 'ip';
ALTER TABLE bws_routes ADD COLUMN health_check_path TEXT DEFAULT NULL;
ALTER TABLE bws_routes ADD COLUMN health_check_interval_secs INTEGER DEFAULT 30;
ALTER TABLE bws_routes ADD COLUMN transform_rules TEXT DEFAULT NULL;
```

## 实现详情

### 1. 类型定义 (`src/types.rs`)

新增 `AuthStrategy` 枚举：
```rust
pub enum AuthStrategy { None, Jwt, ApiKey }
```

`RouteWithAuth` 结构体用于网关路由匹配，携带鉴权所需的完整信息。

### 2. 鉴权中间件 (`src/middleware/auth.rs`)

核心函数 `check_route_auth` 接收预提取的凭证（避免跨 await 借用 `Request`）：

```rust
pub async fn check_route_auth(
    strategy: &AuthStrategy,
    min_role: &Option<String>,
    jwt_service: &JwtServiceClone,
    db: &DbPool,
    bearer_token: Option<String>,
    api_key: Option<String>,
) -> Result<AuthResult, AppError>
```

- JWT 验证使用 `JwtServiceClone::verify`，从 token 的 `sub` 字段提取用户名
- API Key 验证对 key 做 SHA256 hash 后在 `bws_api_keys` 表查找
- 角色检查函数 `role_meets` 实现层级比较：`superadmin(3) >= admin(2) >= user(1)`

API Key 管理函数：
- `create_api_key` - 生成随机 32 字节 hex 编码的 key
- `revoke_api_key` - 将 active 设为 0
- `list_api_keys` - 列表（不返回 hash）
- API Key 创建时返回原始 key 一次，之后不可查询

### 3. 网关集成 (`src/gateway/proxy.rs`)

`proxy_handler` 中在路由匹配后进行鉴权：

```rust
let bearer_token = extract_bearer_token(req.headers());
let api_key = extract_api_key(req.headers());
let auth_result = check_route_auth(
    &route.auth_strategy, &route.min_role,
    &state.jwt, &state.db,
    bearer_token, api_key,
).await;
```

鉴权失败返回 401/403 不穿透到后端。

### 4. API 接口 (`src/api/keys.rs`)

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/api/v1/keys` | 列出所有 API Key |
| POST | `/api/v1/keys` | 创建新 API Key |
| DELETE | `/api/v1/keys/{id}` | 吊销指定 API Key |

## Bug Fix: Handler Trait Error

### 问题现象

```
error[E0277]: the trait bound `fn(State<AppState>, ...) -> ... {proxy_handler}: Handler<_, _>` is not satisfied
    --> src/gateway/proxy.rs:54:19
     54 |         .fallback(proxy_handler)
```

### 根因分析

使用 `#[axum::debug_handler]` 宏后，编译器揭示真正的错误：

```
error: future cannot be sent between threads safely
  --> src/gateway/proxy.rs:80:1
   = help: future returned by `proxy_handler` is not `Send`
   = note: captured value is not `Send` because `&` references cannot be sent unless their referent is `Sync`
   --> src/middleware/auth.rs:32:5
   32 |     req: &Request<Body>,
   |         ^^^ has type `&http::Request<axum::body::Body>` which is not `Send`, because `http::Request<axum::body::Body>` is not `Sync`
```

`check_route_auth` 接收 `&Request<Body>` 引用跨 async 边界，导致整个 future 不可 `Send`，Axum 的 Handler trait 要求 future 必须是 `Send` 的。

### 修复方案

提取 auth 凭证同步执行，避免跨 await 借用 Request：

```rust
// 修复前：跨 async 借用 Request
pub async fn check_route_auth(..., req: &Request<Body>, ...) -> Result<...> {
    let token = extract_bearer_token(req)...;
    // ... await ...
}

// 修复后：同步提取凭证后再 await
let bearer_token = extract_bearer_token(req.headers());
let api_key = extract_api_key(req.headers());
let auth_result = check_route_auth(..., bearer_token, api_key).await;
```

## 验收测试用例

1. 创建路由配置 `auth_strategy=jwt`，不带 Token 请求 → 401 Unauthorized
2. 创建路由配置 `auth_strategy=api_key`，不带 `X-API-Key` 头 → 401 Unauthorized
3. 创建路由配置 `auth_strategy=jwt, min_role=admin`，带普通用户 Token → 403 Forbidden
4. 创建路由配置 `auth_strategy=jwt, min_role=superadmin`，带 admin Token → 403 Forbidden
5. 创建 API Key 后首次返回完整 key → 验证 key 可以正常鉴权
6. 吊销 API Key 后使用该 key 鉴权 → 401 Unauthorized
7. 带过期时间的 API Key 过期后使用 → 401 Unauthorized

## 涉及文件

| 文件 | 操作 | 说明 |
|------|------|------|
| `migrations/006_gateway_auth.sql` | 新增 | 数据库表和字段变更 |
| `src/types.rs` | 修改 | AuthStrategy、RouteWithAuth |
| `src/middleware/auth.rs` | 新增 | 鉴权逻辑、API Key 管理 |
| `src/gateway/proxy.rs` | 修改 | 集成鉴权入口 |
| `src/api/keys.rs` | 新增 | API Key CRUD 接口 |
