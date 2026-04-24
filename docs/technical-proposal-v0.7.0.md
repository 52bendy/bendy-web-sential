# bendy-web-sential 技术方案 v0.7.0

**项目**: bendy-web-sential API 网关与监控平台
**版本**: v0.7.0
**日期**: 2026-04-24
**状态**: 第二轮开发完成

---

## 目录

1. [架构概览](#1-架构概览)
2. [新增接口](#2-新增接口)
3. [前端功能](#3-前端功能)
4. [DevOps 配置](#4-devops-配置)
5. [数据库变更](#5-数据库变更)
6. [技术选型决策](#6-技术选型决策)

---

## 1. 架构概览

```
                                    ┌─────────────────────┐
                                    │   bendy-web-sential  │
                                    │      v0.7.0         │
                                    └─────────────────────┘
                                               │
                    ┌──────────────────────────┼──────────────────────────┐
                    │                          │                          │
              ┌─────┴─────┐              ┌─────┴─────┐              ┌─────┴─────┐
              │  Gateway  │              │   Admin   │              │ Frontend  │
              │   :8080   │              │   :3000   │              │  :5173    │
              └───────────┘              └───────────┘              └───────────┘
                    │                          │                          │
              proxy requests            API endpoints              React SPA
              circuit breaker           auth/metrics              Vite dev

                         ┌───────────────┐
                         │    SQLite     │
                         │  WAL + FK     │
                         └───────────────┘
```

### 技术栈

| 层级 | 技术 | 说明 |
|------|------|------|
| 后端 | Rust + Axum 0.8 | 异步 Web 框架 |
| 数据库 | SQLite (rusqlite 0.33) | WAL 模式，外键约束 |
| 前端 | React 18 + Vite 5 + TailwindCSS | SPA 应用 |
| 状态管理 | Zustand | 布局/主题/认证状态 |
| 数据获取 | TanStack Query | API 数据缓存与刷新 |
| 图表 | Recharts 2.12 | 流量可视化 |
| 国际化 | i18next | 中英文支持 |
| 容器化 | Docker + K8s | 生产部署 |

---

## 2. 新增接口

### 2.1 流量图数据接口

**`GET /api/v1/traffic`**

返回最近 24 小时的流量数据，按小时聚合。

```json
{
  "code": 0,
  "message": "ok",
  "data": {
    "ingress": [
      { "time": "2026-04-24T00:00:00Z", "bytes": 1024, "requests": 42 },
      { "time": "2026-04-24T01:00:00Z", "bytes": 2048, "requests": 38 }
    ],
    "egress": [
      { "time": "2026-04-24T00:00:00Z", "bytes": 8192, "requests": 42 },
      { "time": "2026-04-24T01:00:00Z", "bytes": 12288, "requests": 38 }
    ],
    "total_ingress_bytes": 3072,
    "total_egress_bytes": 20480
  }
}
```

**实现文件**: `src/api/traffic.rs`

**数据源**: `bws_traffic_metrics` 表，按 `direction` 分组聚合

---

### 2.2 K8s 容器心跳接口

**`GET /api/v1/k8s/health`**

返回容器健康状态和系统资源使用情况。

```json
{
  "code": 0,
  "message": "ok",
  "data": {
    "status": "healthy",
    "uptime_seconds": 3600,
    "memory_usage_bytes": 52428800,
    "memory_total_bytes": 268435456,
    "cpu_usage_percent": 15.5
  }
}
```

**状态判断逻辑**:
- `unhealthy`: memory_percent >= 95% 或 cpu_usage >= 95%
- `degraded`: memory_percent >= 80% 或 cpu_usage >= 80%
- `healthy`: 其他情况

**实现文件**: `src/api/k8s.rs` (使用 `sysinfo = "0.32"` crate)

---

### 2.3 K8s Probe 处理接口

**`POST /api/v1/k8s/probe`**

处理 K8s 的 liveness/readiness/startup 探针。

**请求**:
```json
{ "type": "liveness" | "readiness" | "startup" }
```

**响应**:
```json
{ "code": 0, "message": "ok" }
```

**Probe 类型说明**:

| Probe 类型 | 检查内容 | 失败后果 |
|-----------|---------|---------|
| `liveness` | 进程存活 | 容器重启 |
| `readiness` | DB 连接 + Circuit Breaker 非 Open | 从 Service 移除 |
| `startup` | 启动完成 | 阻塞其他探针 |

---

### 2.4 用户资料接口

**`GET /api/v1/auth/me`** (已更新)

返回当前登录用户信息。

```json
{
  "code": 0,
  "message": "ok",
  "data": {
    "id": 1,
    "username": "admin",
    "avatar": null,
    "role": "superadmin"
  }
}
```

**要求**: 请求头 `Authorization: Bearer <token>`

---

**`PUT /api/v1/auth/me`** (新增)

更新当前用户资料。

**请求**:
```json
{
  "username": "new_name",
  "avatar": "https://example.com/avatar.png"
}
```

**响应**: 同 GET /api/v1/auth/me

---

## 3. 前端功能

### 3.1 菜单布局可配置化

支持三种布局，通过 `useLayoutStore` (Zustand) 管理：

```typescript
type MenuPosition = 'top' | 'left' | 'bottom';
```

**组件**:

| 布局 | 组件 | 特点 |
|------|------|------|
| 顶部 | `TopNavbar.tsx` | 横向导航，深色模式切换，语言切换 |
| 左侧 | `Sidebar.tsx` | 可折叠侧边栏，用户信息 + 退出登录 |
| 底部 | `BottomNavbar.tsx` | 固定底部标签栏，适合移动端 |

**存储**: localStorage (`bws_menu_position`, `bws_sidebar_collapsed`)

---

### 3.2 用户头像功能

- **显示**: Navbar/Sidebar/BottomNavbar 右下角显示头像
- **获取**: `GET /api/v1/auth/me` 返回 `avatar` 字段
- **回退**: 无头像时显示用户名首字母缩写（蓝色圆形背景）
- **修改**: Settings 页面表单支持头像 URL 输入

**实现组件**:

- `UserAvatar` 内置于三个 Navbar 组件
- `UserProfileForm` 在 Settings 页面

---

### 3.3 仪表盘流量图

**组件**: `TrafficChart.tsx` (Recharts AreaChart)

**功能**:

- 24 小时流量趋势（按小时）
- 两条曲线：Ingress (蓝色) / Egress (绿色)
- 实时刷新：30 秒间隔
- 字节格式化：自动显示 B/KB/MB/GB

**数据获取**:

```typescript
const { data: trafficData } = useQuery({
  queryKey: ['traffic'],
  queryFn: async () => {
    const { data } = await api.get('/v1/traffic');
    return data;
  },
  refetchInterval: 30000,
});
```

---

## 4. DevOps 配置

### 4.1 Docker 配置更新

**修复项**:

| 文件 | 修复前 | 修复后 |
|------|--------|--------|
| `Dockerfile` | `BWS_ADMIN_PORT=8081` | `BWS_ADMIN_PORT=3000` |
| `Dockerfile` | `EXPOSE 8080 8081` | `EXPOSE 8080 3000` |
| `docker-compose.yml` | `8081:8081` | `3000:3000` |
| `docker-compose.yml` healthcheck | port 8081 | port 3000 |

**建议改进** (参考 `docs/devops-research-v0.7.0.md`):
- 升级 Rust 镜像: 1.80-slim → 1.85-slim
- 添加 HEALTHCHECK 指令
- 添加 .dockerignore
- 使用 Cargo 缓存优化构建

---

### 4.2 Kubernetes 配置

**文件**:
- `k8s/deployment.yaml` — 双容器部署 (backend + frontend)，完整 Probe 配置
- `k8s/service.yaml` — ClusterIP + LoadBalancer 服务
- `k8s/ingress.yaml` — Nginx Ingress 路由
- `k8s/configmap.yaml` — 环境变量和前端配置
- `k8s/secrets.yaml` — JWT/TOTP 密钥模板

**Probe 配置** (后端):

```yaml
livenessProbe:
  httpGet:
    path: /health
    port: 3000
  initialDelaySeconds: 10
  periodSeconds: 30
  timeoutSeconds: 5
  failureThreshold: 3

readinessProbe:
  httpGet:
    path: /api/v1/k8s/health
    port: 3000
  initialDelaySeconds: 5
  periodSeconds: 10
  timeoutSeconds: 3
  failureThreshold: 3
```

---

## 5. 数据库变更

### 迁移 003: 用户头像字段

**文件**: `migrations/003_user_avatar.sql`

```sql
ALTER TABLE bws_admin_users ADD COLUMN avatar TEXT;
```

**注册**: `src/db/mod.rs` 迁移数组添加 `("003_avatar", ...)`

---

## 6. 技术选型决策

### 6.1 电路 breaker 双 RwLock 模式

**问题**: tokio 的 `RwLock::read()` 返回 Future，不能在同步上下文中调用

**解决方案**: 分离同步/异步状态

```rust
pub struct CircuitBreaker {
    // 同步 RwLock — 用于 metrics()（从 sync context 调用）
    state: Arc<StdRwLock<CircuitState>>,
    // 异步 RwLock — 用于 state transitions（从 async context 调用）
    async_state: Arc<TokioRwLock<CircuitState>>,
    // ...
}
```

**结果**: `metrics()` 改为 `async fn`，避免 panic

---

### 6.2 Vite Proxy IPv6 修复

**问题**: `target: 'http://localhost:3000'` 解析到 `::1` (IPv6)，但后端绑定 IPv4

**修复**: `target: 'http://127.0.0.1:3000'`

---

### 6.3 前端容器化方案

**推荐**: Nginx 多阶段构建

```dockerfile
# Build stage
FROM node:18-alpine AS builder
RUN npm ci && npm run build

# Production stage
FROM nginx:alpine
COPY --from=builder /app/dist /usr/share/nginx/html
COPY nginx.conf /etc/nginx/conf.d/default.conf
```

**优点**: 镜像小 (~30MB)，无需 Node.js 运行时

---

### 6.4 系统资源获取

**选用**: `sysinfo = "0.32"`

| 需求 | 实现 |
|------|------|
| 内存使用 | `sys.used_memory()` / `sys.total_memory()` |
| CPU 使用 | `sys.global_cpu_usage()` |
| 进程启动时间 | `sys.process(pid).uptime()` |

---

## 附录 A: 完整接口清单

| 方法 | 路径 | 说明 | 认证 |
|------|------|------|------|
| POST | `/api/v1/auth/login` | 登录 (可选 TOTP) | 否 |
| POST | `/api/v1/auth/logout` | 登出 | 否 |
| GET | `/api/v1/auth/me` | 当前用户信息 | Bearer |
| PUT | `/api/v1/auth/me` | 更新用户资料 | Bearer |
| GET | `/api/v1/traffic` | 流量数据 | Bearer |
| GET | `/api/v1/k8s/health` | K8s 健康检查 | Bearer |
| POST | `/api/v1/k8s/probe` | K8s 探针 | Bearer |
| GET | `/api/v1/domains` | 域名列表 | Bearer |
| POST | `/api/v1/domains` | 创建域名 | Bearer |
| GET | `/api/v1/routes` | 路由列表 | Bearer |
| POST | `/api/v1/routes` | 创建路由 | Bearer |
| GET | `/api/v1/audit` | 审计日志 | Bearer |
| GET | `/api/v1/metrics` | 系统指标 | Bearer |
| GET | `/metrics` | Prometheus 格式 | 否 |
| GET | `/health` | 存活检查 | 否 |

---

## 附录 B: 文件变更清单

### 后端新增
- `src/api/traffic.rs` — 流量图 API
- `src/api/k8s.rs` — K8s 健康检查 API
- `migrations/002_traffic_metrics.sql` — 流量表
- `migrations/003_user_avatar.sql` — 头像字段
- `docs/backend-research-v0.7.0.md` — 后端技术调研

### 后端修改
- `src/api/auth.rs` — me/update_profile 实现
- `src/api/mod.rs` — 添加 traffic, k8s 模块
- `src/main.rs` — 注册新路由
- `src/db/mod.rs` — 添加 002, 003 迁移
- `Cargo.toml` — sysinfo = "0.32"
- `Dockerfile` — 端口统一 3000
- `docker-compose.yml` — 端口修复

### 前端新增
- `src/components/TopNavbar.tsx`
- `src/components/Sidebar.tsx`
- `src/components/BottomNavbar.tsx`
- `src/components/TrafficChart.tsx`
- `docs/frontend-research-v0.7.0.md`

### 前端修改
- `src/types/index.ts` — User, TrafficData, MenuPosition 类型
- `src/store/index.ts` — useLayoutStore
- `src/components/Layout.tsx` — 根据 menuPosition 渲染
- `src/pages/Dashboard.tsx` — 流量统计卡片 + 图
- `src/pages/Settings.tsx` — 布局选择 + 用户资料

### DevOps
- `k8s/deployment.yaml` — 双容器 + Probe
- `k8s/service.yaml`
- `k8s/ingress.yaml`
- `k8s/configmap.yaml`
- `k8s/secrets.yaml`
- `docs/devops-research-v0.7.0.md`

---

**版本**: v0.7.0
**下次更新**: 待流量数据采集中间件实现