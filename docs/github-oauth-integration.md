# 技术方案：GitHub OAuth 认证集成

> 文档版本：v1.0
> 创建日期：2026-04-24
> 状态：待审核

---

## 1. 背景与目标

bend y-web-sential Admin 后台认证当前使用本地用户名密码 + JWT，存在以下问题：

- 密码存储安全依赖本地（SQLite + bcrypt）
- 用户需手动管理账号密码
- 缺乏身份来源管理

本方案引入 GitHub OAuth 作为**可选的第三方登录方式**，与本地登录并存（双轨制）。

---

## 2. 认证流程

### 2.1 整体流程

```
用户点击"GitHub登录"
        ↓
前端跳转 → GitHub OAuth 授权页
        ↓
用户授权 → GitHub 返回 Code
        ↓
后端用 Code 换 Access Token
        ↓
用 Token 请求 GitHub API 获取用户信息
        ↓
查询/创建本地用户记录（GitHub ID 关联）
        ↓
签发 JWT，返回登录成功
```

### 2.2 详细步骤

| 步骤 | 方向 | 说明 |
|---|---|---|
| 1 | 用户 → 前端 | 点击「用 GitHub 登录」 |
| 2 | 前端 → GitHub | 跳转 `https://github.com/login/oauth/authorize?client_id=...&scope=read:user` |
| 3 | GitHub → 用户 | 显示授权页面 |
| 4 | 用户 → GitHub | 授权 |
| 5 | GitHub → 前端 | 回调 `/api/v1/auth/github/callback?code=xxx` |
| 6 | 前端 → 后端 | 请求 `/api/v1/auth/github/callback` |
| 7 | 后端 → GitHub | 用 Code + Client Secret 换 Access Token |
| 8 | GitHub → 后端 | 返回 Access Token |
| 9 | 后端 → GitHub | 用 Access Token 获取用户信息 |
| 10 | 后端 → 本地 | 查询/创建用户，签发 JWT |
| 11 | 后端 → 前端 | 返回 JWT |
| 12 | 前端 | 存储 JWT，跳转后台 |

---

## 3. 数据模型变更

### 3.1 新增字段

`bws_admin_users` 表新增字段：

```sql
ALTER TABLE bws_admin_users ADD COLUMN github_id VARCHAR(255) UNIQUE;
ALTER TABLE bws_admin_users ADD COLUMN github_username VARCHAR(255);
ALTER TABLE bws_admin_users ADD COLUMN avatar_url TEXT;
```

### 3.2 用户创建逻辑

| 情况 | 处理 |
|---|---|
| GitHub ID 已存在 | 直接登录，更新 GitHub 用户信息 |
| GitHub ID 不存在 | 创建新用户，使用 GitHub 用户名 |

### 3.3 用户模型

```rust
pub struct AdminUser {
    pub id: i64,
    pub username: String,         // GitHub username 或本地账号
    pub password_hash: Option<String>,  // GitHub 用户无密码
    pub github_id: Option<String>,      // GitHub 用户唯一ID
    pub github_username: Option<String>,
    pub avatar_url: Option<String>,
    pub role: String,             // admin / viewer 等
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

---

## 4. API 设计

### 4.1 获取 GitHub OAuth 跳转地址

```
GET /api/v1/auth/github
```

返回 GitHub 授权页面地址，前端直接跳转。

### 4.2 GitHub 回调

```
GET /api/v1/auth/github/callback?code=xxx
```

后端逻辑：
1. 用 code + client_secret 换 access_token
2. 用 access_token 获取 GitHub 用户信息
3. 查询/创建本地用户
4. 签发 JWT
5. 前端存储 JWT

### 4.3 当前 API 保持不变（双轨兼容）

| 接口 | 方式 | 说明 |
|---|---|---|
| `POST /api/v1/auth/login` | 本地密码 | 原有，保留 |
| `POST /api/v1/auth/logout` | JWT | 原有，保留 |
| `GET /api/v1/auth/me` | JWT | 原有，保留 |
| `GET /api/v1/auth/github` | GitHub OAuth | **新增** |
| `GET /api/v1/auth/github/callback` | GitHub OAuth | **新增** |

---

## 5. GitHub OAuth 应用配置

### 5.1 创建 GitHub OAuth App

1. GitHub → Settings → Developer settings → OAuth Apps
2. 创建新 App
3. 填写：
   - **Application name**: bendy-web-sential
   - **Homepage URL**: `https://your-domain.com`
   - **Authorization callback URL**: `https://your-domain.com/api/v1/auth/github/callback`

### 5.2 环境变量

```env
# .env
BWS_GITHUB_CLIENT_ID=your_client_id
BWS_GITHUB_CLIENT_SECRET=your_client_secret
```

### 5.3 安全说明

- `Client Secret` 存储在 `.env`，不上传 Git
- 回调地址必须与 GitHub App 配置一致
- HTTPS 由 Cloudflare 提供（已确认）

---

## 6. 角色授权（RBAC）

### 6.1 角色定义

| 角色 | 权限 |
|---|---|
| `admin` | 全部权限：域名/路由 CRUD、用户管理、审计查看 |
| `operator` | 域名/路由 CRUD、审计查看（无用户管理）|
| `viewer` | 只读：查看域名/路由/审计 |

### 6.2 GitHub 用户默认角色

GitHub 用户首次登录时，默认分配 `viewer` 角色。

管理员可在后台将其升级为 `admin` 或 `operator`。

---

## 7. 前端对接（Phase 3 预留）

### 7.1 前端改动

- 登录页增加「用 GitHub 登录」按钮
- 登录成功跳转后台
- 用户信息展示 GitHub 头像和用户名

### 7.2 回调处理

GitHub 回调由前端处理：
1. 接收 URL 中的 token
2. 存储到 localStorage
3. 跳转到 `/admin/dashboard`

---

## 8. 实现计划

| 任务 | 说明 | 优先级 |
|---|---|---|
| 8.1 | GitHub OAuth App 创建文档 | P0 |
| 8.2 | 环境变量配置 | P0 |
| 8.3 | GitHub OAuth API (`/github`, `/github/callback`) | P0 |
| 8.4 | 用户表字段变更 + 迁移 | P0 |
| 8.5 | GitHub 用户信息获取逻辑 | P0 |
| 8.6 | JWT 签发（复用现有逻辑）| P0 |
| 8.7 | 角色授权 + RBAC 中间件 | P1 |
| 8.8 | 前端登录页按钮 + 回调处理 | P2 |
| 9.9 | 测试 + 文档更新 | P2 |

预计工时：**2-3 小时**

---

## 9. 风险与注意事项

1. **GitHub 组织成员限制**：如果需要限制为特定 GitHub 组织成员才能登录，需要额外申请 `read:org` scope
2. **回调地址匹配**：GitHub App 的 callback URL 必须完全一致
3. **用户数据迁移**：已有用户的 github_id 字段需要关联（可后台手动关联）
4. **注销逻辑**：GitHub 注销后，本地用户仍然存在，管理员可禁用

---

## 10. 相关文档

- [README.md](../README.md) — 项目概述
- [plan.md](../plan.md) — 开发计划
- [maintain.md](../maintain.md) — 版本维护记录
