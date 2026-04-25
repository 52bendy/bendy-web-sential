# bendy-web-sential

[English](#english) | [中文](#中文) | [Русский](#русский)

---

## English

### Overview

**bendy-web-sential** is a Rust-based Web traffic control platform that manages multi-domain routing, rate limiting, circuit breaking, and traffic monitoring.

bendy-web-sential acts as an intelligent gateway layer in front of your services. It routes incoming requests based on domain and path patterns to different backends (proxy, redirect, or static files), with built-in support for traffic control, authentication, and observability.

### Features

- **Multi-Domain Routing** — Route traffic by domain and path patterns
- **Traffic Control** — Rate limiting (per-IP and global), circuit breaker, retry logic
- **Authentication** — JWT, API Key, TOTP 2FA, role-based access control
- **Traffic Monitoring** — Real-time metrics, Prometheus exporter
- **Kubernetes Ready** — Service discovery and integration
- **Admin UI** — Modern React-based dashboard

### Tech Stack

| Component | Technology |
|---|---|
| Language | Rust (stable) |
| Web Framework | Axum + Tower |
| Async Runtime | Tokio |
| Database | SQLite (rusqlite) |
| Authentication | JWT + bcrypt |
| Logging | tracing + JSON |
| HTTP Client | reqwest |
| Frontend | React 18 + Vite |

### Quick Start

```bash
# Clone and enter project
cd bendy-web-sential

# Copy env config
cp .env.example .env

# Build
cargo build --release

# Run
cargo run --release
```

Server starts on two ports:
- **Gateway**: `http://localhost:8080` — receives traffic
- **Admin API**: `http://localhost:3000` — management interface

### Default Admin Account

On first start, a default admin user is created:

```
Username: admin
Password: bendy2024
```

**Change this password immediately in production.**

### Docker Deployment

```bash
# Generate secrets
export BWS_JWT_SECRET=$(openssl rand -base64 32)
export BWS_TOTP_AES_KEY=$(openssl rand -base64 32)

# Start services
docker-compose up -d
```

### License

MIT License

---

## 中文

### 概述

**bendy-web-sential** 是一个基于 Rust 的 Web 流量控制平台，支持多域名路由、限流熔断、流量监控等功能。

bendy-web-sential 作为智能网关层，部署在服务前端，根据域名和路径将请求路由到不同的后端（代理、重定向或静态文件），内置流量控制、认证鉴权和可观测性支持。

### 核心功能

- **多域名路由** — 支持按域名和路径模式路由流量
- **流量控制** — 限流（按 IP 和全局）、熔断器、重试机制
- **认证鉴权** — JWT、API Key、TOTP 二次验证、RBAC 角色权限
- **流量监控** — 实时指标、Prometheus 导出器
- **Kubernetes 集成** — 服务发现与集成
- **管理后台** — React + Vite 现代管理界面

### 技术栈

| 组件 | 技术 |
|---|---|
| 语言 | Rust (stable) |
| Web 框架 | Axum + Tower |
| 异步运行时 | Tokio |
| 数据库 | SQLite (rusqlite) |
| 认证 | JWT + bcrypt |
| 日志 | tracing + JSON |
| HTTP 客户端 | reqwest |
| 前端 | React 18 + Vite |

### 快速开始

```bash
# 克隆并进入项目
cd bendy-web-sential

# 复制环境配置
cp .env.example .env

# 编译
cargo build --release

# 运行
cargo run --release
```

服务器启动两个端口：
- **网关端口**: `http://localhost:8080` — 接收流量
- **管理 API**: `http://localhost:3000` — 管理接口

### 默认管理员账号

首次启动时自动创建默认管理员：

```
用户名: admin
密码: bendy2024
```

**生产环境请立即修改密码。**

### Docker 部署

```bash
# 生成密钥
export BWS_JWT_SECRET=$(openssl rand -base64 32)
export BWS_TOTP_AES_KEY=$(openssl rand -base64 32)

# 启动服务
docker-compose up -d
```

### 开发计划

| 版本 | 功能 | 状态 |
|--------|---------|--------|
| 0.1.1 | 认证与鉴权 (JWT / API Key / RBAC) | ✅ 已完成 |
| 0.1.2 | 路由级限流 | 🔄 进行中 |
| 0.1.3 | 负载均衡 + 健康检查 | 📋 规划中 |
| 0.1.4 | 请求/响应改写 | 📋 规划中 |
| 0.1.5 | 可观测性 | 📋 规划中 |
| 0.2.0 | 协议扩展 (WebSocket / gRPC) | 📋 规划中 |

详细计划请查看 [plan.md](plan.md)。

### 版本历史

| 版本 | 日期 | 更新内容 |
|--------|---------|--------|
| [v0.1.1](https://github.com/52bendy/bendy-web-sential/releases/tag/v0.1.1) | 2026-04-25 | 修复 GitHub OAuth 登录竞态条件问题 |
| v0.1.0 | 2026-04-24 | 域名托管服务 + SSO Token 登录 |

### 许可证

MIT License

---

## Русский

### Обзор

**bendy-web-sential** — это платформа управления веб-трафиком на базе Rust, поддерживающая маршрутизацию по нескольким доменам, ограничение скорости, автоматические выключатели и мониторинг трафика.

bendy-web-sential выступает интеллектуальным шлюзом перед вашими сервисами. Он маршрутизирует входящие запросы на основе домена и шаблонов пути к различным бэкендам (прокси, перенаправление или статические файлы), со встроенной поддержкой управления трафиком, аутентификации и наблюдаемости.

### Возможности

- **Маршрутизация по доменам** — Маршрутизация трафика по домену и шаблонам пути
- **Управление трафиком** — Ограничение скорости (по IP и глобальное), автоматический выключатель, логика повторных попыток
- **Аутентификация** — JWT, API Key, TOTP 2FA, контроль доступа на основе ролей
- **Мониторинг трафика** — Метрики в реальном времени, экспортер Prometheus
- **Готовность к Kubernetes** — Обнаружение сервисов и интеграция
- **Админ-панель** — Современный веб-интерфейс на React

### Технологический стек

| Компонент | Технология |
|---|---|
| Язык | Rust (stable) |
| Веб-фреймворк | Axum + Tower |
| Асинхронная среда | Tokio |
| База данных | SQLite (rusqlite) |
| Аутентификация | JWT + bcrypt |
| Логирование | tracing + JSON |
| HTTP-клиент | reqwest |
| Фронтенд | React 18 + Vite |

### Быстрый старт

```bash
# Клонируйте и перейдите в проект
cd bendy-web-sential

# Скопируйте конфигурацию окружения
cp .env.example .env

# Сборка
cargo build --release

# Запуск
cargo run --release
```

Сервер запускается на двух портах:
- **Шлюз**: `http://localhost:8080` — приём трафика
- **Админ API**: `http://localhost:3000` — интерфейс управления

### Учётная запись администратора по умолчанию

При первом запуске создаётся учётная запись администратора по умолчанию:

```
Имя пользователя: admin
Пароль: bendy2024
```

**Немедленно измените этот пароль в производственной среде.**

### Развёртывание с Docker

```bash
# Генерация секретов
export BWS_JWT_SECRET=$(openssl rand -base64 32)
export BWS_TOTP_AES_KEY=$(openssl rand -base64 32)

# Запуск сервисов
docker-compose up -d
```

### План разработки

| Версия | Функция | Статус |
|--------|---------|--------|
| 0.1.1 | Аутентификация и авторизация (JWT / API Key / RBAC) | ✅ Завершено |
| 0.1.2 | Ограничение скорости на уровне маршрута | 🔄 В процессе |
| 0.1.3 | Балансировка нагрузки + Проверка работоспособности | 📋 Запланировано |
| 0.1.4 | Перезапись запросов/ответов | 📋 Запланировано |
| 0.1.5 | Наблюдаемость | 📋 Запланировано |
| 0.2.0 | Расширение протоколов (WebSocket / gRPC) | 📋 Запланировано |

Подробный план см. в [plan.md](plan.md).

### История версий

| Версия | Дата | Изменения |
|--------|---------|--------|
| [v0.1.1](https://github.com/52bendy/bendy-web-sential/releases/tag/v0.1.1) | 2026-04-25 | Исправлена проблема гонки при входе через GitHub OAuth |
| v0.1.0 | 2026-04-24 | Сервис хостинга доменов + Вход через SSO токен |

### Лицензия

MIT License
