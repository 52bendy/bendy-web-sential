# Backend Research v0.7.0 — Technical Survey

**Date**: 2026-04-24  
**Agent**: Backend Agent  
**Project**: bendy-web-sential

---

## 1. Docker 可 Docker 化程度评估

### 当前状态：✅ 已支持

项目已有 `Dockerfile`，多阶段构建：

```dockerfile
# Stage 1: Builder
FROM rust:1.80-slim AS builder
RUN cargo build --release

# Stage 2: Runtime
FROM debian:bookworm-slim
COPY binary + ca-certificates
ENTRYPOINT ["/app/bendy-web-sential"]
```

### 评估结果

| 项目 | 状态 | 说明 |
|------|------|------|
| 基础镜像 | ✅ | rust:1.80-slim (multi-stage) |
| 系统依赖 | ✅ | pkg-config, libssl-dev 已安装 |
| 数据持久化 | ✅ | VOLUME ["/data"] + SQLite |
| 环境变量 | ✅ | .env 支持 via dotenvy |
| 端口暴露 | ✅ | EXPOSE 8080 8081 |
| 健康检查 | ⚠️ | **缺失** — 需要实现 /health 接口 |
| 非 root 运行 | ❌ | 未配置 — 建议添加 USER |

### 建议改进

1. **添加 HEALTCHECK**（Dockerfile）：
```dockerfile
HEALTHCHECK --interval=30s --timeout=3s CMD curl -f http://localhost:8081/health || exit 1
```

2. **非 root 用户**（可选但建议）：
```dockerfile
RUN groupadd -r appgroup && useradd -r -g appgroup appuser
COPY --chown=appuser:appgroup ...
USER appuser
```

---

## 2. K8s Health Check / Probe 机制

### K8s Probe 类型

| Probe | 用途 | 失败后果 |
|-------|------|----------|
| `livenessProbe` | 进程存活判断 | 重启容器 |
| `readinessProbe` | 流量接收判断 | 移除 Service |
| `startupProbe` | 启动完成判断 | 阻塞其他 probes |

### 在 Rust/Axum 中的实现方案

#### 1. Liveness Probe — 简单存活检查

```rust
// 不依赖任何外部资源，只要进程响应即可
async fn liveness() -> &'static str {
    "ok"
}
```

#### 2. Readiness Probe — 流量接收检查

需要验证：
- ✅ DB 连接池可用（执行 `SELECT 1`）
- ✅ Circuit Breaker 非 Open 状态
- ✅ 配置加载完成

```rust
async fn readiness(State(state): State<AppState>) -> Result<&'static str, AppError> {
    // 1. 检查 DB 连接
    let conn = state.db.lock().map_err(|_| AppError::InternalError)?;
    conn.execute("SELECT 1", [])?;
    drop(conn);
    
    // 2. 检查 Circuit Breaker
    let cb = state.circuit_breaker.metrics().await;
    if cb.state == CircuitState::Open {
        return Err(AppError::ServiceUnavailable);
    }
    
    Ok("ok")
}
```

#### 3. Startup Probe — 启动完成

类似 Liveness，首次调用时检查配置加载：

```rust
static STARTUP_COMPLETE: AtomicBool = AtomicBool::new(false);

async fn startup() -> &'static str {
    if STARTUP_COMPLETE.load(Ordering::SeqCst) {
        "ok"
    } else {
        // 可以返回错误状态码让 K8s 持续探测
        StatusCode::SERVICE_UNAVAILABLE
    }
}
```

---

## 3. 流量图数据采集方案

### 需求分析

- **Ingress（入口请求）**：所有进入 API 的请求
- **Egress（出口响应）**：所有从 API 返回的响应
- **时间窗口**：最近 24 小时，按小时聚合
- **指标**：bytes（请求/响应体大小）、requests（请求数）

### 数据源方案对比

| 方案 | 优点 | 缺点 |
|------|------|------|
| **基于 bws_audit_log** | 已有点击审计表，数据结构清晰 | 需要记录 bytes 字段 |
| **基于请求日志** | 可捕获所有请求，含完整元数据 | 需要修改日志中间件 |
| **独立 traffic_logs 表** | 专用，性能好 | 需要新增表和迁移 |
| **基于 Prometheus metrics** | 已有 metrics 框架 | 难以按时间聚合 |

### 推荐方案：扩展 bws_audit_log + 新增 traffic_metrics 视图

1. **方案 A：扩展 audit_log**（推荐）
   - 添加可选字段：`request_bytes`, `response_bytes`
   - 在中间件中记录请求/响应大小
   - 按 `created_at` 按小时聚合

2. **方案 B：新建专用表**（备选）
   ```sql
   CREATE TABLE IF NOT EXISTS bws_traffic_metrics (
       id INTEGER PRIMARY KEY AUTOINCREMENT,
       endpoint TEXT NOT NULL,
       method TEXT NOT NULL,
       request_bytes INTEGER DEFAULT 0,
       response_bytes INTEGER DEFAULT 0,
       status_code INTEGER,
       duration_ms INTEGER,
       created_at TEXT NOT NULL
   );
   CREATE INDEX idx_traffic_time ON bws_traffic_metrics(created_at DESC);
   ```

### 中间件实现（基于 tower-http TraceLayer）

```rust
use tower_http::trace::{MakeClassifier, OnRequest, OnResponse};

// 在中间件中记录请求大小
fn on_request<B>(req: &Request<B>) {
    let request_bytes = req.body().size_hint().map(|h| h.upper().unwrap_or(0)).unwrap_or(0);
}

// 在中间件中记录响应大小
fn on_response(response: &Response<Body>, latency: Duration, span: &Span) {
    let response_bytes = response.body().size_hint().map(|h| h.upper().unwrap_or(0)).unwrap_or(0);
}
```

---

## 4. 系统资源获取库推荐

### 需求
- 获取内存使用情况（total/used）
- 获取 CPU 使用率
- 获取进程 uptime

### 库对比

| 库 | 版本 | 特点 | 适用场景 |
|----|------|------|----------|
| **sysinfo** | 0.32+ | 跨平台，API 简洁，支持进程级 CPU | ⭐ 推荐 |
| `/proc` 读取 | — | Linux 特有，直接读取文件 | 备选方案 |

### sysinfo 使用示例

```rust
use sysinfo::{System, pid, ProcessStatus};

pub fn get_system_info() -> (u64, u64, f64, u64) {
    let mut sys = System::new_all();
    sys.refresh_all();
    
    // 内存
    let total_memory = sys.total_memory();
    let used_memory = sys.used_memory();
    
    // CPU
    let cpu_usage = sys.global_cpu_usage();
    
    // Uptime（从进程）
    let current_pid = pid::this();
    let process = sys.process(current_pid);
    let uptime = process.map(|p| p.uptime()).unwrap_or(0);
    
    (used_memory, total_memory, cpu_usage, uptime)
}
```

### CPU 使用率注意事项

- `sys.global_cpu_usage()` 返回的是**系统整体** CPU 占用
- 需要**进程级** CPU 时，使用 `sys.process(pid).cpu_usage()`
- 首次调用返回 0.0，需要调用 `refresh_processes()` 后再获取

---

## 5. 接口设计

### GET /api/v1/traffic

返回最近 24 小时流量数据（按小时聚合）：

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

### GET /api/v1/k8s/health

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

### POST /api/v1/k8s/probe

请求：
```json
{ "type": "readiness" }
```

响应：
```json
{ "code": 0, "message": "ok" }
```

---

## 6. 实现计划

### Phase 1: 技术准备
- [ ] 添加 `sysinfo = "0.32"` 依赖
- [ ] 创建 `src/api/traffic.rs`
- [ ] 创建 `src/api/k8s.rs`
- [ ] 更新 `src/api/mod.rs`
- [ ] 更新 `src/main.rs` 路由注册

### Phase 2: 数据采集（可选扩展）
- [ ] 新增 `bws_traffic_metrics` 表（如果需要独立记录）
- [ ] 实现请求/响应大小中间件

### Phase 3: K8s 适配
- [ ] 更新 Dockerfile 添加 HEALTHCHECK
- [ ] 提供 K8s deployment.yaml 示例

---

## 参考资料

- [sysinfo crate docs](https://docs.rs/sysinfo/)
- [K8s Probe 官方文档](https://kubernetes.io/docs/tasks/configure-pod-container/configure-liveness-readiness-startup-probes/)
- [tower-http TraceLayer](https://docs.rs/tower-http/)
