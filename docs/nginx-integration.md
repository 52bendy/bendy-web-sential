# 技术方案：Nginx 模块化集成

> 文档版本：v1.2
> 创建日期：2026-04-24
> 状态：待审核

---

## 0. 模块化设计原则

**核心原则：bend y-web-sential 是主体，Nginx 是插件。**

| 场景 | 请求链路 | 说明 |
|---|---|---|
| 无 Nginx（默认） | 用户 → Gateway(8080) → 后端 | 项目直接当网关使用 |
| 有 Nginx | 用户 → Nginx(443) → Gateway(8080) → 后端 | 链路拉长，多一层控制 |

- **接入 Nginx**：请求链路拉长（多一层控制）
- **去掉 Nginx**：项目直接当网关使用，不受影响

Nginx 做成**开关形式**，通过环境变量控制，默认关闭。前期不作为重点，后期再启用。

---

## 1. 开关配置

### 1.1 环境变量

```env
# .env
BWS_ENABLE_NGINX=false   # 默认关闭
BWS_NGINX_HTTP_PORT=80
BWS_NGINX_HTTPS_PORT=443
BWS_GATEWAY_BIND=0.0.0.0  # Nginx关闭时保持0.0.0.0（公网可访问）
```

### 1.2 启动逻辑

```
BWS_ENABLE_NGINX=false
    └── Gateway 监听 0.0.0.0:8080，直接对外服务（当前状态）

BWS_ENABLE_NGINX=true
    └── Nginx 监听 0.0.0.0:80/443
    └── Gateway 改为 127.0.0.1:8080（仅本机）
    └── Admin API 改为 127.0.0.1:3000（仅本机）
```

---

## 2. Nginx 模块功能

Nginx 作为插件，提供以下可选功能：

| 功能 | 说明 | Phase |
|---|---|---|
| HTTPS 终止 | SSL 证书管理 | 插件期 |
| 静态资源托管 | Admin UI 静态文件 | Phase 3 |
| 安全响应头 | X-Frame-Options, CSP, HSTS | 插件期 |
| 入口限流 | Phase 2 限流前置 | Phase 2 |
| WebSocket | Phase 3 实时监控 | Phase 3 |
| 日志格式化 | JSON 日志 → 审计系统 | 插件期 |

---

## 3. 架构图

### 3.1 无 Nginx（当前）

```
[用户] → (公网) → [Gateway:8080] → [后端服务]
                    ↓
              [Admin API:3000]
```

### 3.2 有 Nginx（后期）

```
[用户] → (公网) → [Nginx:443] → [Gateway:8080] → [后端服务]
                              ↓
                        [Admin API:3000]
                            ↓
                      [Admin UI /admin/*]
```

---

## 4. Nginx 配置模板

### 4.1 配置文件结构

```
deploy/
└── nginx/
    ├── Dockerfile          # 可选：Docker 部署
    ├── gateway.conf         # Gateway 反向代理
    ├── admin-api.conf      # Admin API 反向代理
    ├── admin-ui.conf       # Admin UI 静态托管
    ├── security-headers.conf # 安全响应头
    └── docker-compose.yml   # 可选：一键部署
```

### 4.2 Gateway 反向代理 (gateway.conf)

```nginx
server {
    listen 443 ssl;
    server_name _;

    # SSL（自签开发 / Let's Encrypt 生产）
    ssl_certificate /etc/nginx/ssl/cert.pem;
    ssl_certificate_key /etc/nginx/ssl/key.pem;

    include snippets/security-headers.conf;

    access_log /var/log/nginx/gateway.access.log;
    error_log /var/log/nginx/gateway.error.log;

    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_http_version 1.1;

        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;

        proxy_connect_timeout 10s;
        proxy_send_timeout 60s;
        proxy_read_timeout 60s;

        # WebSocket (Phase 3)
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
    }
}
```

### 4.3 安全响应头 (security-headers.conf)

```nginx
add_header X-Frame-Options "SAMEORIGIN" always;
add_header X-Content-Type-Options "nosniff" always;
add_header X-XSS-Protection "1; mode=block" always;
add_header Referrer-Policy "no-referrer-when-downgrade" always;
add_header Strict-Transport-Security "max-age=2592000; includeSubDomains" always;
```

### 4.4 Admin UI 静态托管 (admin-ui.conf)

```nginx
server {
    listen 443 ssl;
    server_name admin.*;

    include snippets/security-headers.conf;

    location /admin/ {
        alias /var/www/bendy-admin/;
        try_files $uri $uri/ /admin/index.html;
    }

    location / {
        proxy_pass http://127.0.0.1:3000;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

---

## 5. 端口管理

| 端口 | 场景 | 服务 | 暴露 |
|---|---|---|---|
| 80 | Nginx开 | Nginx HTTP | 公网 |
| 443 | Nginx开 | Nginx HTTPS | 公网 |
| 8080 | 始终 | Gateway | Nginx关:公网 / Nginx开:本机 |
| 3000 | 始终 | Admin API | 仅本机 |

---

## 6. 实现计划

| 任务 | 说明 | 优先级 | 阶段 |
|---|---|---|---|
| 6.1 | 环境变量开关实现 | P0 | 插件期 |
| 6.2 | Gateway 本机绑定（当 Nginx 开启时） | P0 | 插件期 |
| 6.3 | Nginx 配置模板（gateway.conf） | P1 | 插件期 |
| 6.4 | 安全响应头配置 | P1 | 插件期 |
| 6.5 | SSL 证书方案（自签/Let's Encrypt） | P1 | 插件期 |
| 6.6 | Admin UI 静态托管配置 | P2 | Phase 3 |
| 6.7 | Docker Compose 一键部署 | P2 | 插件期 |
| 6.8 | 更新文档和部署说明 | P2 | 插件期 |

预计工时：**2-3 小时**

---

## 7. 风险与注意事项

1. **端口冲突**：如服务器已占用 80/443，需先清理
2. **Gateway 监听变更**：Nginx 开启时需改为 127.0.0.1 绑定
3. **SSL 证书**：开发阶段用自签证书，浏览器会有安全提示
4. **Cloudflare 用户**：需开启 `remoteip_module` 还原真实 IP

---

## 8. 相关文档

- [README.md](../README.md) — 项目概述
- [plan.md](../plan.md) — 开发计划
- [maintain.md](../maintain.md) — 版本维护记录