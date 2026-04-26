# bendy-web-sential 版本维护记录

> 记录每一次版本迭代的内容、修改点、变更方式

---

## 版本规范

版本号格式：**大版本.中版本.小版本**（如 `1.0.0`）

| 版本类型 | 触发条件 | 更新方式 |
|---|---|---|
| **大版本** | 重大功能更新、架构重构 | 开发者告知后更新 |
| **中版本** | 功能发布上线 | 开发者告知后更新 |
| **小版本** | 每次 commit 提交 | 自动化维护 |

---

## 版本历史

| 版本 | 日期 | 类型 | 更新内容 | 状态 |
|---|---|---|---|---|
| 0.1.0 | 2026-04-23 | 中 | 初始技术方案制定 | ✅ 已完成 |
| 0.2.0 | 2026-04-23 | 中 | Phase 1 核心基础构建 | ✅ 已完成 |
| 0.3.0 | 2026-04-24 | 中 | Phase 2 流量控制模块 | ✅ 已完成 |
| 0.4.0 | 2026-04-24 | 中 | Phase 3 管理后台与 UI | ✅ 已完成 |
| 0.5.0 | 2026-04-24 | 中 | Phase 4 安全与生产加固 | ✅ 已完成 |
| 0.6.0 | 2026-04-24 | 中 | Phase 5 自动化与灾备 | ✅ 已完成 |
| 0.1.3 | 2026-04-25 | 中 | Alpha 收尾 + Beta 启动 | ✅ 已完成 |

---

## v0.1.3 — 2026-04-25

**类型：** 中版本

**触发说明：**
> Alpha 阶段所有计划功能已完成，项目正式进入 Beta 测试阶段。

**变更内容：**
- Frontend: 完善路由管理界面（展开行显示认证/限流/健康检查配置）
- Frontend: 补全 rewrites 翻译键（i18n）
- Frontend: 新增 Rewrites 改写规则管理页面
- Backend: 修复 Axum 0.6 路径参数语法（`:id` 而非 `{id}`）
- Backend: 修复更新路由/域名时未传字段被清空的 BUG（增量更新）
- Backend: 3000端口集成静态文件服务（前端直接由后端托管）
- Backend: 新增 Rewrite Rules API 和缓存
- Backend: 新增 Upstream 负载均衡 CRUD API
- Backend: 新增健康检查和负载均衡逻辑
- Database: 新增 rewrite_rules 表
- Database: 新增性能索引

**如何修改：**
> 从 `main` 分支拉取 `feat/v0.1.3-enhancements` 开发，完成后合并回 `main`

**完成的 Commits：**
- `2061d79` feat: v0.1.3 - 完善路由管理界面和修复API BUG
- `90b719d` chore: bump version to 0.1.3

**遗留问题：**
- 无

---

## v0.6.0 — 2026-04-24

**类型：** 中版本

**触发说明：**
> Phase 5 自动化与灾备阶段完成

**变更内容：**
- 三环境管理（development/staging/production）
- 数据库定时备份策略（backup.sh）
- 备份恢复流程文档化
- TOTP 密钥备份恢复流程
- 依赖安全扫描集成（cargo audit）
- 高危漏洞禁止发布拦截

**如何修改：**
> 从 `dev` 分支拉取功能分支开发，完成后合并回 `dev`

**完成的 Commits：**
- `bd8993b` feat(automation): phase 5 automation & disaster recovery
- `575150d` chore(release): v0.6.0 Phase 5 completed

**遗留问题：**
- 无

---

## v0.5.0 — 2026-04-24

**类型：** 中版本

**触发说明：**
> Phase 4 安全与生产加固阶段完成

**变更内容：**
- TOTP 生成与验证（totp-rs crate）
- TOTP 密钥加密存储（AES 加密）
- Token 吊销机制（内存黑名单）
- 敏感操作二次验证接口
- 用户输入校验防 XSS / SQL 注入
- Prometheus 指标导出（`/metrics` 端点）
- 流量/并发/错误率实时统计
- Dockerfile 编写
- docker-compose.yml 编写

**如何修改：**
> 从 `dev` 分支拉取功能分支开发，完成后合并回 `dev`

**完成的 Commits：**
- `5f1763d` feat(security): phase 4 security & production hardening
- `1a13952` chore(release): v0.5.0 Phase 4 completed

**遗留问题：**
- 无

---

## v0.4.0 — 2026-04-24

**类型：** 中版本

**触发说明：**
> Phase 3 管理后台与 UI 阶段完成

**变更内容：**
- React 前端项目初始化（Vite + TypeScript + Tailwind）
- 黑白灰三色主题 + 白天/夜间模式切换
- i18next 中/英双语国际化
- 路由规则可视化配置界面
- 域名管理界面
- 日志审计界面
- Admin API RESTful 接口完善
- 指标数据聚合查询接口

**如何修改：**
> 从 `dev` 分支拉取功能分支开发，完成后合并回 `dev`

**完成的 Commits：**
- `cb0c55d` chore(release): v0.4.0 Phase 3 completed
- `97fe4ad` merge: feat/phase3-admin-ui into dev
- `ec9e62e` feat(admin-ui): add React frontend with Tailwind and management interfaces

**遗留问题：**
- 无

---

## v0.3.0 — 2026-04-24

**类型：** 中版本

**触发说明：**
> Phase 2 流量控制模块完成

**变更内容：**
- 限流中间件（基于 IP/全局/路径的速率限制）
- 熔断中间件（基于请求成功率的熔断逻辑）
- 重试策略（指数退避）

**如何修改：**
> 从 `dev` 分支拉取功能分支开发，完成后合并回 `dev`

**完成的 Commits：**
- `c13622f` merge: feat/phase2-traffic-control into dev
- `5cd0413` feat(traffic-control): add rate limiting, circuit breaker, and retry middleware

**遗留问题：**
- 无

---

## v0.2.0 — 2026-04-23

**类型：** 中版本

**触发说明：**
> Phase 1 业务审核通过，开始核心基础构建

**变更内容：**
- 技术方案 v0.4 评审确认
- Phase 1 范围调整：先聚焦后端核心网关，前端延后
- 数据库精简为 4 张表（去掉 bws_metrics 表，指标用内存+Prometheus）
- 三环境配置延后至 Phase 5
- i18next 延后至 Phase 3 前端阶段
- TOTP 二次验证延后至 Phase 4
- 端口调整为 Gateway:8080 / Admin API:3000
- 静态文件/Fallback/redirect 合并到路由配置
- 审计日志表建好但不接业务，Phase 4 再接入
- JWT 简单认证 Phase 1 先做，Phase 4 再加固 TOTP

**Phase 1 实际范围（必做）：**
- Rust 项目初始化 + 核心依赖
- SQLite 连接 + 迁移系统（4 张表：bws_domains, bws_routes, bws_admin_users, bws_audit_log）
- Gateway 8080 监听
- 域名 + 路径路由转发
- action: proxy / redirect / static 三种动作
- 结构化日志（tracing）
- JWT 简单认证
- `.env` 配置 + `.env.example`
- `.gitignore`
- Git main/dev 分支

**如何修改：**
> 从 `dev` 分支拉取 `feat/phase1-core` 开发，完成后合并回 `dev`

**完成的 Commits：**
- `5df11e0` feat: Phase 1.1 project init and env config
- `1d2958b` feat: Phase 1.5 README documentation
- `ef658c9` fix: axum 0.8 path syntax for route params and wildcard

**遗留问题：**
- 无（所有延后项已记录在 plan.md 各 Phase 中）

---

## 运维操作指南

### 备份与恢复

```bash
# 手动备份（数据库 + TOTP 密钥加密备份）
./scripts/backup.sh

# 列出可用备份
./scripts/backup.sh --list

# 恢复数据库
./scripts/backup.sh --restore backups/bws_db_20240115_120000.sqlite

# 从加密备份提取 TOTP 密钥
./scripts/backup.sh --restore backups/bws_totp_key_20240115_120000.enc
```

环境变量控制备份行为：
- `BWS_BACKUP_DIR` — 备份目录（默认: `./backups`）
- `BWS_BACKUP_RETENTION_DAYS` — 保留天数（默认: 7）
- `BWS_BACKUP_ENCRYPTION_PASSPHRASE` — TOTP 密钥加密密码

### TOTP 密钥管理

```bash
# 生成新密钥
openssl rand -base64 32

# 迁移到新服务器：备份 + 恢复 TOTP 密钥
# 1. 旧服务器: ./scripts/backup.sh
# 2. 复制 .enc 文件到新服务器
# 3. 新服务器: ./scripts/backup.sh --restore <file>
# 4. 更新 .env 中的 BWS_TOTP_AES_KEY
```

**注意**: 更换 TOTP AES 密钥后，所有已注册用户的 TOTP 将失效，需重新注册。

### 环境切换

| 环境 | 配置文件 | 启动方式 |
|------|---------|----------|
| 开发 | `.env.development` | `cargo run` |
| Staging | `.env.staging` | `cargo run` |
| 生产 | `.env.production` | `docker-compose up -d` |

### 版本升级

1. `./scripts/backup.sh` 备份
2. `git pull origin dev`
3. `cargo build --release`
4. 重启服务
5. `./scripts/ci-build.sh` 验证

### 安全扫描

```bash
cargo install cargo-audit
cargo audit
```

---

## 更新记录模板

每次版本更新时，按以下格式追加记录。

```markdown
## v{VERSION} — {DATE}

**类型：** 大/中/小版本

**触发说明：**
> {一句话说明触发原因}

**变更内容：**
- {变更点}

**如何修改：**
> {从哪个分支拉取，开发完成后合并到哪里}

**完成的 Commits：**
- `{hash}` {subject}

**遗留问题：**
- {遗留问题列表}
```
