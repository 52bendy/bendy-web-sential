# Frontend Research v0.7.0

## 项目 Docker 化程度评估

### 当前状态
- **后端**: 已有完整的 Dockerfile 和 docker-compose.yml
  - 位置: `/myproject/rust/bendy-web-sential/`
  - Health check: 使用 `wget --spider` 检测 health 端点
  - 暴露端口: 8080 (gateway), 8081 (admin)
  - 数据卷: 持久化 SQLite 数据库
  
- **前端**: 暂无 Dockerfile
  - 技术栈: React 18 + Vite + TailwindCSS
  - 开发模式: Vite dev server (端口 5173)
  - 生产模式: `npm run build` 生成静态文件

### Docker 化建议

#### 前端 Dockerfile (多阶段构建)
```dockerfile
# Build stage
FROM node:20-alpine AS builder
WORKDIR /app
COPY package*.json ./
RUN npm ci
COPY . .
RUN npm run build

# Production stage
FROM nginx:alpine
COPY --from=builder /app/dist /usr/share/nginx/html
COPY nginx.conf /etc/nginx/conf.d/default.conf
EXPOSE 80
```

#### Nginx 配置 (SPA 支持)
```nginx
server {
    listen 80;
    root /usr/share/nginx/html;
    index index.html;
    
    location / {
        try_files $uri $uri/ /index.html;
    }
    
    location /api/ {
        proxy_pass http://backend:8081/api/;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
    }
}
```

#### docker-compose.yml 扩展
```yaml
services:
  frontend:
    build:
      context: ./frontend
      dockerfile: Dockerfile
    ports:
      - "80:80"
    depends_on:
      - bendy-web-sential
```

### 评估结论
- **Docker 化可行性**: 高
- **复杂度**: 低 (前端仅需 Nginx 部署静态文件)
- **建议**: 前后端分离部署，通过 Nginx 反向代理 API

---

## K8s Health Check / Probe 在前端的应用方式

### 前端特殊性
前端是纯静态资源服务，不像后端服务有进程监控需求。但在前端容器化部署场景中：

### K8s Probe 应用于前端的场景

#### 1. Liveness Probe
用于检测容器是否存活，前端通常不需要。但某些场景下：
- 健康检查端点: `/health` 返回 200
- 可以通过 sidecar container 或 cronjob 检测

#### 2. Readiness Probe
用于检测前端是否可以接收流量：
- Nginx 健康检查: `nginx -t` 或检测配置文件
- 静态资源完整性: 检测 index.html 是否存在

#### 3. Startup Probe
用于慢启动应用，前端通常不需要。

### 建议的前端 K8s 配置

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: bendy-web-sential-frontend
spec:
  selector:
    matchLabels:
      app: bendy-web-sential-frontend
  template:
    metadata:
      labels:
        app: bendy-web-sential-frontend
    spec:
      containers:
        - name: frontend
          image: bendy-web-sential-frontend:latest
          ports:
            - containerPort: 80
          readinessProbe:
            httpGet:
              path: /index.html
              port: 80
            initialDelaySeconds: 5
            periodSeconds: 10
          livenessProbe:
            httpGet:
              path: /index.html
              port: 80
            initialDelaySeconds: 15
            periodSeconds: 20
```

### 与后端探针的对比

| 探针类型 | 后端 (Rust/Axum) | 前端 (Nginx) |
|---------|------------------|--------------|
| Liveness | `/health` 返回进程状态 | 通常不需要 |
| Readiness | `/health` 返回服务就绪状态 | `/index.html` 存在 |
| Startup | 启动完成检测 | 通常不需要 |

---

## 菜单布局可配置化方案

### 设计目标
- 支持三种布局: 顶部 (top) / 左侧 (left) / 底部 (bottom)
- 用户偏好保存到 localStorage
- 布局切换即时生效，无需刷新页面

### Zustand Store 设计
```typescript
interface LayoutState {
  menuPosition: 'top' | 'left' | 'bottom';
  setMenuPosition: (position: 'top' | 'left' | 'bottom') => void;
  sidebarCollapsed: boolean;
  toggleSidebar: () => void;
}
```

### 组件结构
```
Layout
├── TopNavbar (menuPosition === 'top')
├── Sidebar (menuPosition === 'left')
│   ├── Logo
│   ├── NavItems (collapsible)
│   └── CollapseToggle
├── BottomNavbar (menuPosition === 'bottom')
└── MainContent
    └── <Outlet />
```

### 样式方案
- 左侧菜单: `w-64` 宽度，可折叠到 `w-16`
- 底部菜单: 固定底部，高度 48px
- 顶部菜单: 现有 Navbar 实现

---

## 仪表盘流量图技术选型

### 推荐方案: Recharts

#### 选型理由
| 特性 | Recharts | Chart.js |
|-----|---------|----------|
| React 集成 | 原生 React 组件 | 需要 wrapper |
| TypeScript | 完整类型定义 | 需额外类型包 |
| 动画支持 | 内置，易配置 | 内置 |
| 学习曲线 | 低 | 中 |
| 包大小 | ~30KB | ~200KB |
| 社区活跃度 | 高 (2023-2024) | 高但稳定 |

#### Recharts 优势
1. **声明式 API**: 完全基于 React 组件
2. **响应式**: 与 React 生态无缝集成
3. **轻量**: 相比 Chart.js 更轻量
4. **已安装**: 项目已依赖 `recharts@2.12.0`

### 流量图组件设计
```typescript
interface TrafficChartProps {
  data: TrafficData;
  loading?: boolean;
}

// 图表配置
- 类型: AreaChart (带填充的折线图)
- X轴: 时间 (格式化: HH:mm:ss)
- Y轴: 字节数 (格式化: KB/MB/GB)
- 两条线: Ingress (蓝色), Egress (绿色)
- 实时刷新: 30秒间隔
```

### 数据处理
```typescript
// 时间格式化
format(new Date(point.time), 'HH:mm:ss')

// 字节格式化
formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
  return `${(bytes / 1024 / 1024 / 1024).toFixed(1)} GB`;
}
```

---

## 接口契约确认

### GET /api/v1/auth/me
```typescript
// 响应
{
  code: 0,
  message: "ok",
  data: {
    id: number;
    username: string;
    avatar?: string;
    role: string;
  }
}
```

### PUT /api/v1/auth/me
```typescript
// 请求
{
  username?: string;
  avatar?: string;
}

// 响应
{
  code: 0,
  message: "ok",
  data: {
    id: number;
    username: string;
    avatar?: string;
    role: string;
  }
}
```

### GET /api/v1/traffic
```typescript
// 响应
{
  code: 0,
  message: "ok",
  data: {
    ingress: TrafficPoint[];
    egress: TrafficPoint[];
    total_ingress_bytes: number;
    total_egress_bytes: number;
  }
}

// TrafficPoint
{
  time: string;      // ISO timestamp
  bytes: number;
  requests: number;
}
```

---

## 实现计划

### Phase 1: 基础架构
1. [x] 更新类型定义
2. [x] 创建 Layout Store (Zustand)
3. [x] 重构 Layout 组件

### Phase 2: 组件实现
1. [x] 创建 Sidebar 组件
2. [x] 创建 BottomNavbar 组件
3. [x] 更新 Navbar 组件 (用户头像)

### Phase 3: 业务页面
1. [x] 更新 Dashboard (添加流量图)
2. [x] 更新 Settings (用户信息表单 + 布局切换)

### Phase 4: 国际化
1. [x] 添加相关翻译 key
