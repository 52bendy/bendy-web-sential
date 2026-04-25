---
name: devops
description: DevOps and deployment for bendy-web-sential. Use when working with Docker, Kubernetes, CI/CD, backups, or container orchestration.
when_to_use: Docker deployments, Kubernetes manifests, CI/CD pipelines, backup strategies, container troubleshooting.
---

# DevOps Guide

## Container Architecture

### Ports
| Port | Service | Description |
|------|---------|-------------|
| 8080 | Gateway | Receives external HTTP traffic |
| 3000 | Admin API | Management interface |
| 5173 | Frontend | Admin UI (development) |

### Environment Variables
```bash
BWS_GATEWAY_PORT=8080           # Gateway port
BWS_ADMIN_PORT=3000             # Admin API port
BWS_DATABASE_URL=data/bws.db    # SQLite database path
BWS_JWT_SECRET=<secret>        # JWT signing secret
BWS_JWT_EXPIRY_SECS=86400       # Token expiry (1 day)
BWS_LOG_LEVEL=info              # Log level
```

## Docker

### Images
- `bendy-web-sential:latest` - Backend Rust binary
- `bendy-web-sential-frontend:latest` - Frontend React build

### Run with Docker Compose
```bash
docker-compose up -d
docker-compose ps
docker-compose logs --tail=50
```

### Build
```bash
docker build -t bendy-web-sential .
```

## Kubernetes

### Manifests (k8s/)
- `deployment.yaml` - Main deployment with backend + frontend
- `service.yaml` - Service definitions
- `ingress.yaml` - Ingress routing
- `configmap.yaml` - Configuration
- `secrets.yaml` - Sensitive data (JWT_SECRET, TOTP_KEY)
- `hpa.yaml` - Horizontal Pod Autoscaler

### Health Checks
- Liveness: `/health` on port 3000
- Readiness: `/api/v1/k8s/health` on port 3000

### Deploy
```bash
kubectl apply -f k8s/
kubectl get pods -l app=bendy-web-sential
kubectl logs -l app=bendy-web-sential -c backend
```

## Backup

### Create Backup
```bash
./scripts/backup.sh
```

### Restore
```bash
./scripts/backup.sh --restore backups/bws_db_YYYYMMDD_HHMMSS.sqlite
```

## CI/CD

### Branch Strategy
- `main` - Production-ready code
- `dev` - Integration branch
- `feat/<name>` - Feature branches

### Commit Convention
```
feat:     New features
fix:      Bug fixes
chore:    Maintenance, deps, releases
test:     Test code
style:    Formatting changes
docs:     Documentation
```

### Release Flow
1. Feature branch → merge to dev
2. dev → merge to main + tag vX.Y.Z
3. Push to origin

## Scripts

```bash
./scripts/backup.sh          # Create/restore backups
./scripts/ci-build.sh        # CI build pipeline
./scripts/release.sh        # Release automation
```
