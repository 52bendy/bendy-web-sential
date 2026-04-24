# DevOps 技术调研文档 v0.7.0

**项目**: bendy-web-sential  
**版本**: v0.7.0  
**日期**: 2026-04-24  
**负责人**: DevOps Agent

---

## 目录
1. [K8s Health Check / Probe 配置最佳实践](#1-kubernetes-health-check--probe-配置最佳实践)
2. [Docker 多阶段构建优化方案](#2-docker-多阶段构建优化方案)
3. [前端容器化方案](#3-前端容器化方案-node18-alpine)
4. [当前 Dockerfile 评估与改进建议](#4-当前-dockerfile-评估与改进建议)

---

## 1. Kubernetes Health Check / Probe 配置最佳实践

### 探针类型对比

| 探针类型 | 用途 | 失败后果 | 典型场景 |
|---------|------|---------|---------|
| **startupProbe** | 应用启动检测 | 容器被 kill | 慢启动应用 |
| **livenessProbe** | 存活检测 | 容器重启 | 死锁检测 |
| **readinessProbe** | 就绪检测 | 从 Service 移除 | 依赖未就绪 |

### 推荐配置

#### 后端服务 (Rust Axum)
```yaml
livenessProbe:
  httpGet:
    path: /health
    port: 3000
  initialDelaySeconds: 10
  periodSeconds: 30
  timeoutSeconds: 5
  failureThreshold: 3
  successThreshold: 1

readinessProbe:
  httpGet:
    path: /api/v1/k8s/health
    port: 3000
  initialDelaySeconds: 5
  periodSeconds: 10
  timeoutSeconds: 3
  failureThreshold: 3
  successThreshold: 1

startupProbe:
  httpGet:
    path: /health
    port: 3000
  failureThreshold: 30
  periodSeconds: 10
```

#### 前端服务 (Vite)
```yaml
livenessProbe:
  httpGet:
    path: /
    port: 5173
  initialDelaySeconds: 15
  periodSeconds: 20

readinessProbe:
  httpGet:
    path: /
    port: 5173
  initialDelaySeconds: 5
  periodSeconds: 10
```

### 最佳实践要点

1. **initialDelaySeconds** 需大于应用启动时间
2. **periodSeconds** 不要过短，避免产生过多请求
3. **timeoutSeconds** 应小于 periodSeconds
4. **failureThreshold** × periodSeconds 应覆盖整个启动过程
5. readinessProbe 失败不应导致容器重启

### 参考资料
- [Kubernetes Probe Documentation](https://kubernetes.io/docs/tasks/configure-pod-container/configure-liveness-readiness-startup-probes/)
- [Configure Liveness, Readiness and Startup Probes](https://kubernetes.io/docs/concepts/workloads/pods/pod-lifecycle/#when-should-you-use-a-startup-probe)

---

## 2. Docker 多阶段构建优化方案

### 当前构建分析

**已有 Dockerfile** 存在问题：
1. Rust 基础镜像版本过时 (1.80 → 应升级到 1.85)
2. 未包含前端构建步骤
3. 缺少健康检查配置
4. 缺少 .dockerignore 优化

### 优化方案

#### 2.1 基础优化：使用 Cargo 缓存

```dockerfile
# Stage 1: Builder
FROM rust:1.85-slim AS builder

WORKDIR /app

# 复制依赖文件（利用 Docker 缓存）
COPY Cargo.toml Cargo.lock* ./
RUN mkdir -p src && echo "// placeholder" > src/lib.rs

# 下载依赖（仅在 Cargo.toml 变化时重新执行）
RUN cargo fetch

# 复制源代码并构建
COPY src ./src
RUN apt-get update && apt-get install -y pkg-config libssl-dev && \
    cargo build --release && \
    apt-get clean && rm -rf /var/lib/apt/lists/*

# Stage 2: Runtime
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates libssl3 && \
    apt-get clean && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/bendy-web-sential /app/bendy-web-sential
COPY --from=builder /app/.env.example /app/.env 2>/dev/null || true

ENV RUST_LOG=info
ENV BWS_DATABASE_URL=sqlite:///data/bws.db
ENV BWS_GATEWAY_PORT=8080
ENV BWS_ADMIN_PORT=3000

EXPOSE 8080 3000

VOLUME ["/data"]

HEALTHCHECK --interval=30s --timeout=10s --start-period=10s --retries=3 \
    CMD wget --spider -q http://localhost:3000/health || exit 1

ENTRYPOINT ["/app/bendy-web-sential"]
```

#### 2.2 高级优化：使用 cargo-chef

```dockerfile
# syntax=docker/dockerfile:1
FROM rust:1.85-slim AS chef
RUN cargo install cargo-chef

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path /recipe.json

FROM chef AS builder
COPY --from=planner /recipe.json /recipe.json
RUN cargo chef cook --recipe-path /recipe.json --release
COPY . .
RUN cargo build --release --recipe-path /recipe.json

FROM debian:bookworm-slim AS runtime
# ... 同上
COPY --from=builder /app/target/release/bendy-web-sential /app/bindy-web-sential
```

#### 2.3 构建优化参数

```bash
# 使用 sccache 加速构建
docker build \
    --build-arg RUSTC_WRAPPER=sccache \
    --build-arg CARGO_INCREMENTAL=0 \
    -t bendy-web-sential:latest .

# 或使用 cargo-nextest
RUN cargo test --no-run && cargo nextest run
```

### .dockerignore 优化
```gitignore
# 排除不需要的文件
.git/
target/
node_modules/
dist/
*.log
.env.local
coverage/
```

---

## 3. 前端容器化方案 (node:18-alpine)

### 推荐 Dockerfile

```dockerfile
# syntax=docker/dockerfile:1
# Stage 1: Build
FROM node:18-alpine AS builder

WORKDIR /app

# 复制 package 文件
COPY frontend/package*.json ./

# 安装依赖（利用缓存）
RUN npm ci --only=production=false

# 复制源代码
COPY frontend/ ./

# 构建
RUN npm run build

# Stage 2: Production
FROM node:18-alpine AS runner

WORKDIR /app

# 创建非 root 用户
RUN addgroup --system --gid 1001 nodejs && \
    adduser --system --uid 1001 vite

# 复制构建产物
COPY --from=builder --chown=vite:nodejs /app/dist ./dist
COPY --from=builder --chown=vite:nodejs /app/node_modules ./node_modules
COPY --from=builder --chown=vite:nodejs /app/package.json ./

USER vite

ENV NODE_ENV=production
ENV HOST=0.0.0.0
ENV PORT=5173

EXPOSE 5173

CMD ["node", "dist/server/"]
```

### 备选方案：使用 Nginx

```dockerfile
# Build stage
FROM node:18-alpine AS builder
WORKDIR /app
COPY frontend/package*.json ./
RUN npm ci
COPY frontend/ ./
RUN npm run build

# Production stage
FROM nginx:alpine
COPY --from=builder /app/dist /usr/share/nginx/html
COPY frontend/nginx.conf /etc/nginx/conf.d/default.conf
EXPOSE 5173
CMD ["nginx", "-g", "daemon off;"]
```

### Nginx 配置示例
```nginx
server {
    listen 5173;
    server_name _;
    root /usr/share/nginx/html;
    index index.html;

    # SPA fallback
    location / {
        try_files $uri $uri/ /index.html;
    }

    # API 代理
    location /api/ {
        proxy_pass http://bendy-web-sential:3000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_cache_bypass $http_upgrade;
    }
}
```

### 前端构建优化建议

1. **使用 pnpm** 替代 npm（更快、更省空间）
2. **启用构建缓存**
   ```bash
   docker build \
       --build-arg NPM_CONFIG_CACHE=/app/.npm \
       -t bendy-web-sential-frontend:latest .
   ```
3. **分析构建产物**
   ```bash
   npm run build -- --analyze
   ```

---

## 4. 当前 Dockerfile 评估与改进建议

### 当前状态评估

| 项目 | 当前状态 | 评分 | 建议 |
|-----|---------|------|------|
| Rust 版本 | 1.80-slim | ⚠️ | 升级到 1.85-slim |
| 前端构建 | ❌ 缺失 | 🔴 | 添加前端构建阶段 |
| 健康检查 | ❌ 缺失 | 🔴 | 添加 HEALTHCHECK |
| 多阶段构建 | ✅ 已有 | 🟢 | 保持 |
| .dockerignore | ❌ 缺失 | ⚠️ | 创建 |
| 端口配置 | ⚠️ 8081 vs 3000 | ⚠️ | 统一为 3000 |
| Cargo 缓存 | ⚠️ 部分 | 🟡 | 优化构建顺序 |

### 详细问题

#### 问题 1: Rust 版本过旧
```dockerfile
# 当前
FROM rust:1.80-slim AS builder
# 建议
FROM rust:1.85-slim AS builder
```

#### 问题 2: Admin 端口不一致
```dockerfile
# Dockerfile 中
ENV BWS_ADMIN_PORT=8081

# docker-compose.yml 中
BWS_ADMIN_PORT=8081

# K8s deployment 中
containerPort: 3000
```
**建议**: 统一使用 3000 端口

#### 问题 3: 缺少前端构建
当前 Dockerfile 只构建 Rust 后端，前端需单独构建并部署。

#### 问题 4: 缺少 .dockerignore
```bash
# 创建 .dockerignore
.git/
target/
*.log
.env
.env.local
node_modules/
dist/
```

### 改进优先级

| 优先级 | 改进项 | 难度 | 影响 |
|-------|--------|------|------|
| P0 | 端口统一为 3000 | 低 | 高 |
| P0 | 升级 Rust 到 1.85 | 低 | 中 |
| P0 | 添加 HEALTHCHECK | 低 | 高 |
| P1 | 前端容器化 | 中 | 高 |
| P1 | Cargo 缓存优化 | 中 | 中 |
| P2 | 使用 cargo-chef | 高 | 中 |

---

## 附录：完整优化后 Dockerfile

```dockerfile
# syntax=docker/dockerfile:1
# ============================================
# bendy-web-sential Dockerfile v2.0
# ============================================

# Stage 1: Builder
FROM rust:1.85-slim AS builder

WORKDIR /app

# 安装构建依赖
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# 复制依赖文件
COPY Cargo.toml Cargo.lock* ./

# 创建占位文件（利用缓存）
RUN mkdir -p src && touch src/lib.rs

# 下载并缓存依赖
RUN cargo fetch

# 复制源代码
COPY src ./src

# 构建
RUN cargo build --release

# 清理不需要的文件
RUN strip /app/target/release/bendy-web-sential

# Stage 2: Runtime
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# 从 builder 复制二进制
COPY --from=builder /app/target/release/bendy-web-sential /app/bindy-web-sential
COPY --from=builder /app/.env.example /app/.env 2>/dev/null || true

# 环境变量
ENV RUST_LOG=info
ENV BWS_DATABASE_URL=sqlite:///data/bws.db
ENV BWS_GATEWAY_PORT=8080
ENV BWS_ADMIN_PORT=3000

# 端口
EXPOSE 8080 3000

# 数据卷
VOLUME ["/data"]

# 健康检查
HEALTHCHECK --interval=30s --timeout=10s --start-period=10s --retries=3 \
    CMD wget --spider -q http://localhost:3000/health || exit 1

ENTRYPOINT ["/app/bindy-web-sential"]
```

---

## 通知

> 📤 **Backend Leader** 请查收
> 
> 技术调研文档已完成，包含：
> - Dockerfile 评审与改进建议（需确认端口统一为 3000）
> - K8s Health Check 最佳实践
> - 前端容器化方案
> 
> 请在 `devops-research-v0.7.0.md` 中确认并补充后端相关技术方案。

---

**文档版本**: v0.7.0  
**下次更新**: 等待 Backend Leader 确认后整合到技术方案总文档
