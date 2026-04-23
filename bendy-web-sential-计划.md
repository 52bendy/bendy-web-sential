# bendy-web-sential 技术方案 v0.1

> 基于 Rust 的 Web 流量控制平台 — 管理多域名流量，支持限流、熔断、流量转移、监控。

---

## 一、核心定位

管理多域名流量，支持限流、熔断、流量转移、监控。能将线上请求（某一路径）下的请求通过流控直接转移到静态网页或其他任意资源路径，实现流量完整控制，能监控路径流量请求、并发、流量大小。

---

## 二、技术调研

### 2.1 核心框架选型

| 方案 | 优点 | 缺点 | 推荐度 |
|---|---|---|---|
| **Pingora** (Cloudflare 开源) | 经过 4000万QPS 生产验证，完整proxy框架，gRPC/WebSocket/TLS 支持 | 上手较重，配置复杂 | ⭐⭐⭐ 备选 |
| **Axum + Tower** | Tokio 生态核心，tower 中间件生态丰富，限流/熔断开箱即用，灵活度高 | 需要自己组合各组件 | ⭐⭐⭐⭐⭐ 推荐 |
| **Ratatui / Axum 组合** | 轻量简洁，配合 tower 生态做流量控制 | 不如 Pingora 生产级 | ⭐⭐⭐ |

**结论：选择 `Axum` + `Tower` 生态**

- Axum 是最流行的 Rust Web 框架，基于 Tokio
- Tower 生态有成熟的 `RateLimiter`、`CircuitBreaker`、`LoadBalancer` 中间件
- 轻量灵活，适合先快速迭代再加固

### 2.2 关键依赖库

| 类别 | 库 | 用途 |
|---|---|---|
| HTTP Server | `axum` | Web 服务端 |
| HTTP Client | `reqwest` | 转发请求到上游 |
| 异步 Runtime | `tokio` | 核心异步 runtime |
| Tower 中间件 | `tower` | 请求处理中间件基座 |
| 限流 | `tower::RateLimit` / `governor` | 令牌桶/滑动窗口限流 |
| 熔断 | `tower::CircuitBreaker` | 基于成功率的熔断 |
| 重试 | `tower::Retries` | 请求重试 |
| 配置热加载 | `tokio::sync::watch` | 无停机配置更新 |
| 指标采集 | `axum-prometheus` / `metrics` | Prometheus 指标 |
| WebSocket | `tokio-tungstenite` | 实时推送监控数据 |
| 持久化 | `serde` + `rusqlite` | 配置存储（SQLite 数据库）|

### 2.3 系统架构

```
                                    ┌─────────────────┐
                                    │  前端 (HTML/CSS) │
                                    │  管理后台        │
                                    │  实时监控面板    │
                                    └───────┬─────────┘
                                            │ WebSocket + REST API
┌──────────────┐   ┌─────────────────────────────┴──────────────┐
│   Cloudflare  │   │           Bendy-Web-Sential                │
│   / DNS       │   │  ┌──────────────┐  ┌──────────────────┐   │
│   (HTTPS)     │──▶│  │  Gateway     │  │  Admin API       │   │
│              │   │  │  (Axum)       │  │  (Axum)          │   │
└──────────────┘   │  └──────┬───────┘  └──────────────────┘   │
                   │         │                                   │
                   │  ┌──────▼───────┐  ┌──────────────────┐   │
                   │  │  Router      │  │  Config Store    │   │
                   │  │  (基于域名/   │  │  (JSON文件 +    │   │
                   │  │   路径匹配)   │  │   热加载)        │   │
                   │  └──────┬───────┘  └──────────────────┘   │
                   │         │                                   │
                   │  ┌──────▼───────┐  ┌──────────────────┐   │
                   │  │  Middlewares  │  │  Metrics Store  │   │
                   │  │  限流/熔断/   │  │  (内存+导出)     │   │
                   │  │  重试/日志   │  └──────────────────┘   │
                   │  └──────┬───────┘                          │
                   │         │                                   │
                   │  ┌──────▼───────┐  ┌──────────────────┐   │
                   │  │  Upstream     │  │  实时连接池      │   │
                   │  │  Proxy Pool   │  │  (健康检查)     │   │
                   │  └──────┬───────┘  └──────────────────┘   │
                   └─────────┼───────────────────────────────────┘
                             │
              ┌──────────────┼──────────────┐
              │              │              │
        ┌─────▼─────┐  ┌────▼────┐  ┌─────▼─────┐
        │  后端服务A │  │  静态文件│  │  外部API  │
        │  替换路径  │  │  /fallback│ │  重定向   │
        └───────────┘  └─────────┘  └───────────┘
```

### 2.4 核心功能模块

```
bendy-web-sential/
├── src/
│   ├── main.rs              # 入口，双端口监听（Gateway:80/443, Admin:8080）
│   ├── gateway/
│   │   ├── mod.rs
│   │   ├── router.rs         # 域名+路径路由匹配
│   │   ├── proxy.rs         # 请求转发核心逻辑
│   │   └── upstream.rs      # 上游连接池 + 健康检查
│   ├── middleware/
│   │   ├── mod.rs
│   │   ├── rate_limiter.rs  # 限流（基于 IP/路径/域名）
│   │   ├── circuit_breaker.rs # 熔断
│   │   ├── retry.rs         # 重试策略
│   │   └── log.rs           # 请求日志
│   ├── config/
│   │   ├── mod.rs
│   │   ├── loader.rs        # JSON 配置加载
│   │   └── hot_reload.rs    # 热更新机制
│   ├── api/
│   │   ├── mod.rs
│   │   ├── admin.rs         # 管理 API（CRUD 路由/规则）
│   │   ├── metrics.rs       # 指标查询 API
│   │   └── websocket.rs     # 实时推送 WebSocket
│   ├── metrics/
│   │   ├── mod.rs
│   │   └── prometheus.rs    # Prometheus 格式输出
│   └── types.rs             # 共享类型定义
├── config/
│   └── default.json         # 默认配置
├── admin/                   # 前端 (HTML+CSS+JS)
│   ├── index.html
│   ├── styles.css
│   └── app.js
├── Cargo.toml
└── README.md
```

### 2.5 路由配置模型（JSON）

```json
{
  "domains": [
    {
      "domain": "blog.polofox.com",
      "routes": [
        {
          "path": "/api/v2/*",
          "target": "http://localhost:3000",
          "rate_limit": {
            "requests": 100,
            "window_secs": 60,
            "scope": "ip"
          },
          "circuit_breaker": {
            "failure_threshold": 5,
            "recovery_secs": 30
          },
          "retry": {
            "max_attempts": 3,
            "backoff_ms": 100
          }
        },
        {
          "path": "/legacy/*",
          "action": "redirect",
          "target": "https://newsite.polofox.com/legacy/"
        },
        {
          "path": "/maintenance",
          "action": "static",
          "target": "/var/www/maintenance.html"
        }
      ]
    },
    {
      "domain": "api.polofox.com",
      "routes": [
        {
          "path": "/",
          "target": "http://localhost:8080",
          "rate_limit": {
            "requests": 1000,
            "window_secs": 60,
            "scope": "global"
          }
        }
      ]
    }
  ]
}
```

### 2.6 监控指标

| 指标名 | 类型 | 说明 |
|---|---|---|
| `bendy_requests_total` | Counter | 总请求数（按域名、路径、状态码） |
| `bendy_request_duration_seconds` | Histogram | 请求耗时 |
| `bendy_active_connections` | Gauge | 当前活跃连接 |
| `bendy_upstream_errors_total` | Counter | 上游错误数 |
| `bendy_rate_limited_total` | Counter | 被限流拦截的请求 |
| `bendy_circuit_breaker_open_total` | Counter | 熔断开启次数 |
| `bendy_traffic_bytes` | Counter | 流量字节数 |

---

## 三、技术选型总结

| 组件 | 选型 | 理由 |
|---|---|---|
| 语言 | Rust | 高性能、低内存、安全并发 |
| Web框架 | **Axum** | 轻量、基于 Tower、活跃社区 |
| 限流 | **Tower RateLimit** + **Governor** | 生产级令牌桶实现 |
| 熔断 | **Tower CircuitBreaker** | 基于成功率的自适应熔断 |
| 代理转发 | **reqwest** + 手写路由 | 灵活控制每个请求的转发 |
| 配置热加载 | 文件监控 + Tokio watch channel | 无停机更新配置 |
| 数据库 | `rusqlite` | SQLite 存储配置和路由规则 |
| 前端 | Tailwind CSS + shadcn/ui | 样式核心 + UI 组件库 |
| 国际化 | i18next | 从第一天抽离文案 |
| 测试 | Vitest (前端) + Rust 标准框架 (后端) | 核心安全模块覆盖率 ≥80% |
| 配置存储 | SQLite 数据库 | 简单可靠，支持 CRUD |

---

## 四、开发计划

### Phase 1 — 核心骨架（今天）
- [ ] 项目初始化，`Cargo.toml` 依赖
- [ ] 双端口启动（Gateway + Admin API）
- [ ] 基础路由匹配（域名+路径 → 上游）
- [ ] 静态文件服务（fallback 页面）

### Phase 2 — 流量控制
- [ ] 限流中间件（IP/全局限流）
- [ ] 熔断中间件
- [ ] 重试策略

### Phase 3 — 管理后台
- [ ] Admin REST API（路由 CRUD）
- [ ] 原生 HTML 管理前端
- [ ] 实时监控 WebSocket 推送

### Phase 4 — 生产化
- [ ] Prometheus 指标导出
- [ ] 配置热加载
- [ ] 日志结构化输出
- [ ] 连接池 + 健康检查

---

## 五、风险与备选

| 风险 | 应对 |
|---|---|
| Rust 限流库不够灵活 | 用 `governor` crate 自己实现，比 Tower 更细粒度 |
| 高并发下内存问题 | 上线前用 `dhat` 分析内存分配 |
| 前端不够美观 | Tailwind CSS + shadcn/ui + 黑白灰三色主题 |
| 多语言切换成本 | i18next 从第一天抽离文案 |

---

## 六、评审记录

- **v0.1** — 初始方案
- **v0.3** — 2026-04-23 评审确认：
  - ✅ 双端口架构（Gateway:80 + Admin:8080）
  - ✅ 存储使用 SQLite 数据库（替代 JSON 配置文件）
  - ✅ HTTPS 终止交给 Cloudflare，网关只处理 HTTP
  - ✅ 管理后台设计风格：**黑白灰三色主题**，极简高对比度
  - ✅ **多语言支持**（中/英双语切换）
  - ✅ **白天/夜间模式切换**
  - ⏳ Phase 1 优先级待定，等完整方案审阅后再定
