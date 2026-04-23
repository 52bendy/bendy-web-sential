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

---

## 更新记录模板

每次版本更新时，按以下格式追加记录：

```markdown
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

**Phase 1 延后至后续 Phase 的内容（不做 ≠ 漏做）：**
- 前端 UI（Phase 3）
- i18next 国际化（Phase 3）
- 三环境 .env 分离（Phase 5）
- 完整审计日志接入业务（Phase 4）
- TOTP 二次验证（Phase 4）
- Prometheus 指标导出（Phase 4）
- 自动化脚本（Phase 5）
- 容器化 Dockerfile（Phase 4）

**如何修改：**
> 从 `dev` 分支拉取 `feat/phase1-core` 开发，完成后合并回 `dev`

**做的什么：**
- [Phase 1 开发中...]

**遗留问题：**
- 无（所有延后项已记录在 plan.md 各 Phase 中）
