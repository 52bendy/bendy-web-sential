# bendy-web-sential 项目迭代任务书

> 记录当前项目的已经规划好的所有迭代任务，根据迭代任务进行操作，做完的任务标记为已做完。

---

## 📋 规范速查（必须严格遵守）

| 规范 | 说明 |
|---|---|
| **业务前缀** | `bws_`（配置在 README 中，所有 DB/Redis/接口前缀必须带） |
| **版本号** | 大.中.小（如 `0.2.0`），大版本开发者告知，中版本发布时更新，小版本每次提交维护 |
| **分支策略** | `main` 生产 / `dev` 开发 / `feat/任务名` 功能分支；中版本发布时 dev → main + 打 tag |
| **Commit 规范** | Conventional Commits：`feat:` / `fix:` / `chore:` / `test:` / `style:` |
| **自动化** | 每次迭代完：lint → build → 自动 `chore(release): vX.Y.Z` → push origin |
| **测试规范** | 核心安全模块覆盖率 ≥80%；Vitest (前端) + Rust 标准框架 (后端) |
| **安全规范** | 密钥走 `.env`（必须在 `.gitignore`）；TOTP 密钥加密存储；输入校验防 XSS/SQL 注入 |
| **环境管理** | `.env.development` / `.env.staging` / `.env.production`；构建用 `--mode` 切换（Phase 5 实现） |
| **DB 迁移** | 迁移文件管理，不手动改表 |
| **容器化** | `Dockerfile` + `docker-compose.yml` 一键启动（Phase 4） |
| **国际化** | i18next，从第一天抽离所有文案（Phase 3 前端阶段） |
| **依赖安全** | 构建前跑 `npm audit` / `pnpm audit`，高危漏洞不允许发布（Phase 5） |
| **备份灾备** | 按配置文件开启灾难备份；TOTP 密钥独立加密备份方案（默认关闭）（Phase 5） |
| **错误码** | 4 位数字：`1001 Token过期`、`1002 权限不足`、`1003 参数错误` 等 |
| **错误返回** | 统一 `{code, message, data}` 格式 |
| **日志** | 结构化格式；关键操作（登录、踢人、Token操作）必须留审计日志；后台有审计日志入口 |
| **二次验证** | 敏感操作（踢人、吊销Token）需 TOTP 二次验证（Phase 4） |

---

## 📅 Phase 1: 核心基础构建 (v0.2.x) — 开发中

> **策略：先做有用部分，延后的部分≠不做，只是侧重点不同，记录清晰不遗漏。**

### ✅ Phase 1 必做（本次迭代范围）

- [ ] **1.1.1** Rust 项目初始化（cargo init）
  - [ ] 设置 Cargo.toml 核心依赖（axum, tokio, tower, reqwest, rusqlite, serde, tracing, jsonwebtoken, totp 等）
  - [ ] 建立 src/ 目录结构（gateway/, middleware/, api/, config/, db/, types.rs, error.rs, main.rs）

- [ ] **1.1.2** 环境配置
  - [ ] 创建 `.env.example`（所有环境变量注释模板）
  - [ ] 创建 `.env`（本地密钥，**必须加入 .gitignore**）
  - [ ] 创建 `.gitignore`（.env, target/, .DS_Store 等）
  - [ ] Git 初始化，建立 `main` / `dev` 分支

- [ ] **1.2.1** 数据库迁移系统
  - [ ] 配置 SQLite 连接（数据库路径从 `.env` 读取，业务前缀 `bws_`）
  - [ ] 设计表结构：`bws_domains`（域名）、`bws_routes`（路由规则）、`bws_admin_users`（管理员）、`bws_audit_log`（审计日志表）
  - [ ] 编写 `migrations/001_init.sql` 迁移文件
  - [ ] DB Migration 自动执行脚本（启动时自动执行未执行的迁移）

- [ ] **1.3.1** Gateway 核心逻辑
  - [ ] Gateway 监听 **8080** 端口（HTTP）
  - [ ] Admin API 监听 **3000** 端口
  - [ ] 域名 + 路径路由匹配逻辑
  - [ ] action: proxy（转发到上游）/ redirect（重定向）/ static（静态文件）三种动作
  - [ ] 错误码规范（ErrorCode enum: 1001-1999 认证类，2001-2999 流控类）
  - [ ] 统一返回格式 `{code, message, data}`

- [ ] **1.3.2** JWT 简单认证（Phase 1 先做，Phase 4 再加 TOTP 加固）
  - [ ] 用户名密码登录，签发 JWT Token
  - [ ] Token 认证中间件
  - [ ] 登录接口 `/api/v1/auth/login`
  - [ ] 登出接口 `/api/v1/auth/logout`

- [ ] **1.4.1** 结构化日志
  - [ ] tracing + JSON 格式输出
  - [ ] 请求日志中间件（记录请求路径、域名、耗时、状态码）
  - [ ] 关键操作日志（登录、配置变更）

- [ ] **1.5.1** README 文档
  - [ ] 业务前缀 `bws_` 说明
  - [ ] 项目说明、技术栈介绍
  - [ ] 本地开发启动说明
  - [ ] 环境变量说明
  - [ ] 目录结构说明

### ⏸ Phase 1 延后项（后续 Phase 按计划做，不遗漏）

#### 延后至 Phase 3（前端的阶段）
- [ ] 初始化前端项目（Vite + TypeScript + Tailwind CSS + shadcn/ui）
- [ ] i18next 配置（中/英翻译文件初始化）
- [ ] 三环境 `.env` 分离（`.env.development` / `.env.staging` / `.env.production`）

#### 延后至 Phase 4（安全加固阶段）
- [ ] TOTP 生成与验证（totp-rs crate）
- [ ] TOTP 密钥加密存储（AES 加密，密钥在 .env）
- [ ] TOTP 密钥加密备份方案（默认关闭）
- [ ] Token 吊销机制（内存黑名单）
- [ ] 敏感操作二次验证接口
- [ ] 所有用户输入校验防 XSS / SQL 注入
- [ ] 核心安全模块单元测试（覆盖率 ≥80%）
- [ ] Prometheus 指标导出（`/metrics` 端点）
- [ ] 流量 / 并发 / 错误率实时统计
- [ ] 完整审计日志接入业务（表已建好，Phase 1 不接入）
- [ ] Admin 后台审计日志入口页面
- [ ] `Dockerfile` 编写
- [ ] `docker-compose.yml` 编写

#### 延后至 Phase 5（自动化阶段）
- [ ] 三环境 `.env` 分离
- [ ] 迭代完成自动化脚本（lint → build → auto-commit → push）
- [ ] 依赖安全扫描集成（`npm audit` / `cargo audit`）
- [ ] 高危漏洞禁止发布拦截
- [ ] 数据库定时备份策略
- [ ] 备份恢复流程文档化
- [ ] TOTP 密钥备份恢复流程

#### 延后至 Phase 2（流量控制阶段）
- [ ] 限流中间件（基于 IP/全局/路径的速率限制）
- [ ] 熔断中间件
- [ ] 重试策略

---

## 📅 Phase 2: 流量控制模块 (v0.3.x)

- [ ] **2.1 限流中间件 (Rate Limiting)**
  - [ ] 基于 IP 的速率限制（governor crate）
  - [ ] 基于全局的速率限制
  - [ ] 基于路径的速率限制
  - [ ] 限流触发时返回统一错误码 `2001`

- [ ] **2.2 熔断中间件 (Circuit Breaker)**
  - [ ] 基于请求成功率的熔断逻辑
  - [ ] 熔断开启时返回统一错误码 `2002`

- [ ] **2.3 重试策略 (Retries)**
  - [ ] 配置上游请求失败时的重试频率（指数退避）

---

## 📅 Phase 3: 管理后台与 UI (v0.4.x)

- [ ] **3.1 前端基础框架搭建**
  - [ ] Tailwind CSS + shadcn/ui 配置
  - [ ] 黑白灰三色主题 CSS 变量定义
  - [ ] 白天 / 夜间模式切换（切换按钮在顶部导航栏右侧）
  - [ ] i18next 中/英双语切换
  - [ ] API 请求层统一封装（统一错误处理 + toast 提示）

- [ ] **3.2 Admin API RESTful 接口**
  - [ ] 路由 / 域名规则 CRUD 接口
  - [ ] 指标数据聚合查询接口
  - [ ] 所有接口要求认证（JWT Bearer Token）
  - [ ] 统一返回格式 `{code, message, data}`

- [ ] **3.3 实时监控 WebSocket**
  - [ ] 关键请求指标实时推送

- [ ] **3.4 黑白灰风格 UI 实现**
  - [ ] 管理控制台主界面（Dashboard）
  - [ ] 路由规则可视化配置界面
  - [ ] 域名管理界面
  - [ ] 日志审计界面（**接入 Phase 1 建好的审计日志表**）

---

## 📅 Phase 4: 安全与生产加固 (v0.5.x)

- [ ] **4.1 安全特性增强**（**接入 Phase 1 延后的安全项**）
  - [ ] TOTP 生成与验证（totp-rs crate）
  - [ ] TOTP 密钥加密存储（AES 加密，密钥在 .env）
  - [ ] TOTP 密钥加密备份方案（默认关���，配置在 .env）
  - [ ] Token 吊销机制（内存黑名单）
  - [ ] 敏感操作二次验证接口
  - [ ] 所有用户输入校验防 XSS / SQL 注入
  - [ ] 核心安全模块单元测试（覆盖率 ≥80%）
  - [ ] JWT 认证加固（当前 Phase 1 只做简单认证）

- [ ] **4.2 监控与告警**
  - [ ] Prometheus 指标导出（`/metrics` 端点）
  - [ ] 流量 / 并发 / 错误率实时统计

- [ ] **4.3 Docker 容器化**
  - [ ] `Dockerfile` 编写
  - [ ] `docker-compose.yml` 编写

---

## 📅 Phase 5: 自动化与灾备 (v0.6.x)

- [ ] **5.1 三环境管理**（**接入 Phase 1 延后的环境分离**）
  - [ ] `.env.development` / `.env.staging` / `.env.production` 分离
  - [ ] 构建命令 `--mode` 参数切换环境

- [ ] **5.2 自动化流程**
  - [ ] 迭代完成自动化脚本：lint → build → auto-commit → push
  - [ ] 依赖安全扫描集成（`npm audit` / `cargo audit`）
  - [ ] 高危漏洞禁止发布拦截

- [ ] **5.3 灾备备份方案**
  - [ ] 数据库定时备份策略（按配置开启）
  - [ ] 备份恢复流程文档化（在 README 和 maintain.md 中记录）
  - [ ] TOTP 密钥备份恢复流程（在 README 和 maintain.md 中记录）

---

## 📋 开发流程规范

### 分支开发流程
```
feat/任务名 开发完成 → 合并到 dev
↓
中版本发布时：
dev 合并到 main
↓
打 tag：v{版本号}
↓
推送远程 origin
↓
切回 dev 分支
```

### Commit 规范
| 类型 | 使用场景 |
|---|---|
| `feat:` | 新增功能 |
| `fix:` | 修复问题 |
| `chore:` | 杂务/升级依赖/发布版本 |
| `test:` | 新增测试代码 |
| `style:` | 样式调整（不影响逻辑） |
| `docs:` | 文档更新 |

自动化 commit：`chore(release): v0.2.0 迭代完成`

### 错误码规范
| 范围 | 用途 |
|---|---|
| 1000-1999 | 认证授权类（1001 Token过期，1002 权限不足，1003 参数错误） |
| 2000-2999 | 流量控制类（2001 触发限流，2002 熔断开启） |
| 3000-3999 | 业务逻辑类 |
| 4000-4999 | 系统错误类 |

### 端口规范
| 端口 | 用途 |
|---|---|
| Gateway: 8080 | 接收外部 HTTP 请求 |
| Admin API: 3000 | 管理后台 API |
| 注：80/443 由 Cloudflare 处理 HTTPS 终止 | |
