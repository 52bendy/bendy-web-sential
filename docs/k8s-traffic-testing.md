# K8s 环境下流量图接口测试方案

## 测试目标
验证 `/api/v1/traffic` 接口在 Kubernetes 集群中的可用性、性能和稳定性。

## 前置条件

### 1. 部署确认
```bash
# 检查 Pod 状态
kubectl get pods -l app=bendy-web-sential

# 检查 Service 是否正常
kubectl get svc bendy-web-sential

# 确认 Pod 准备就绪
kubectl get pods -l app=bendy-web-sential -o jsonpath='{.items[*].status.conditions[?(@.type=="Ready")].status}'
```

### 2. 获取 Pod 名称
```bash
# 获取后端 Pod 名称
BACKEND_POD=$(kubectl get pods -l app=bendy-web-sential -o jsonpath='{.items[0].metadata.name}')
echo "Backend Pod: $BACKEND_POD"
```

---

## 测试方法

### 方法一：直接 Pod 内测试
```bash
# 进入后端 Pod 测试接口
kubectl exec -it $BACKEND_POD -c backend -- curl -s http://localhost:3000/api/v1/traffic | jq .
```

### 方法二：通过 Service 测试
```bash
# 获取 Service ClusterIP
SVC_IP=$(kubectl get svc bendy-web-sential -o jsonpath='{.spec.clusterIP}')

# 测试前端端口（通过 nginx/Vite）
curl -s http://${SVC_IP}:80/api/v1/traffic | jq .

# 测试 admin 端口
curl -s http://${SVC_IP}:3000/api/v1/traffic | jq .
```

### 方法三：创建测试 Job
```bash
cat << 'TESTEOF' | kubectl apply -f -
apiVersion: batch/v1
kind: Job
metadata:
  name: traffic-test
spec:
  ttlSecondsAfterFinished: 300
  template:
    spec:
      restartPolicy: Never
      containers:
      - name: test
        image: curlimages/curl:latest
        command:
        - sh
        - -c
        - |
          echo "Testing /api/v1/traffic..."
          curl -s http://bendy-web-sential:3000/api/v1/traffic
          echo ""
          echo "Testing with auth..."
          TOKEN=$(curl -s -X POST http://bendy-web-sential:3000/api/v1/auth/login \
            -H "Content-Type: application/json" \
            -d '{"username":"admin","password":"admin123"}' | jq -r '.data.token')
          curl -s -H "Authorization: Bearer $TOKEN" \
            http://bendy-web-sential:3000/api/v1/traffic | jq .
TESTEOF
```

---

## 流量数据模拟脚本

### 生成入口流量数据
```bash
#!/bin/bash
# generate-ingress.sh - 模拟入口流量数据

for i in {1..100}; do
  TIMESTAMP=$(date +%s)
  BYTES=$((RANDOM % 100000 + 1000))
  REQUESTS=$((RANDOM % 100 + 1))
  
  curl -X POST http://bendy-web-sential:3000/api/v1/traffic/ingress \
    -H "Content-Type: application/json" \
    -d "{\"timestamp\":$TIMESTAMP,\"bytes\":$BYTES,\"requests\":$REQUESTS}"
  
  sleep 0.1
done
```

### 生成出口流量数据
```bash
#!/bin/bash
# generate-egress.sh - 模拟出口流量数据

for i in {1..100}; do
  TIMESTAMP=$(date +%s)
  BYTES=$((RANDOM % 80000 + 500))
  REQUESTS=$((RANDOM % 50 + 1))
  
  curl -X POST http://bendy-web-sential:3000/api/v1/traffic/egress \
    -H "Content-Type: application/json" \
    -d "{\"timestamp\":$TIMESTAMP,\"bytes\":$BYTES,\"requests\":$REQUESTS}"
  
  sleep 0.1
done
```

### 持续压测脚本
```bash
#!/bin/bash
# load-test.sh - 持续请求测试

END_TIME=$(($(date +%s) + 3600))  # 运行1小时

while [ $(date +%s) -lt $END_TIME ]; do
  RESPONSE=$(curl -s -w "%{http_code}" -o /tmp/traffic.json \
    http://bendy-web-sential:3000/api/v1/traffic)
  
  if [ "$RESPONSE" = "200" ]; then
    echo "$(date): OK - $(cat /tmp/traffic.json | jq '.data.total_ingress_bytes') bytes"
  else
    echo "$(date): ERROR - HTTP $RESPONSE"
  fi
  
  sleep 5  # 每5秒请求一次
done
```

---

## 测试用例设计

### TC-001: 基础连接性测试
```gherkin
Scenario: 获取流量数据接口可用
  Given K8s Pod 运行正常
  When 调用 GET /api/v1/traffic
  Then 返回 HTTP 200
  And 响应包含 ingress 和 egress 数组
```

### TC-002: 认证保护测试
```gherkin
Scenario: 未认证请求应被拒绝
  Given 未提供 JWT Token
  When 调用 GET /api/v1/traffic
  Then 返回 HTTP 401 Unauthorized
```

### TC-003: 负载下的响应时间
```gherkin
Scenario: 高并发下接口响应时间
  Given 部署 10 个副本
  When 100 个并发请求同时到达
  Then 95% 的请求在 500ms 内响应
  And 错误率低于 1%
```

### TC-004: 扩缩容测试
```gherkin
Scenario: Pod 扩容时接口仍可用
  Given 当前副本数为 2
  When HPA 触发扩容到 5 副本
  Then 扩容期间接口可用性 >= 99%
  And 扩缩容完成时间 < 2 分钟
```

### TC-005: 健康检查集成
```gherkin
Scenario: Pod 不健康时从 Service 移除
  Given Pod 的 readinessProbe 失败
  When 检查 Service endpoints
  Then 该 Pod 不在 endpoints 列表中
  And 流量不被路由到该 Pod
```

---

## 性能指标收集

### Prometheus 查询示例
```promql
# 接口 QPS
rate(http_requests_total{service="bendy-web-sential", path="/api/v1/traffic"}[5m])

# 接口延迟 P99
histogram_quantile(0.99, rate(http_request_duration_seconds_bucket{path="/api/v1/traffic"}[5m]))

# 错误率
rate(http_requests_total{service="bendy-web-sential", path="/api/v1/traffic", status=~"5.."}[5m])
```

### Grafana Dashboard 配置
建议创建 Dashboard 包含：
- 流量图接口 QPS 趋势
- 响应时间分布 (P50/P95/P99)
- Ingress/Egress 流量字节数
- Pod 数量与 HPA 状态

---

## 故障排查

### 接口超时
```bash
# 检查 Pod 日志
kubectl logs -l app=bendy-web-sential -c backend --tail=100

# 检查网络策略
kubectl get networkpolicy -o wide

# 检查 CoreDNS
kubectl run dnsutils --image=tutum/dnsutils --restart=Never -it --rm -- dig bendy-web-sential.default.svc.cluster.local
```

### 高延迟
```bash
# 检查资源限制
kubectl top pods -l app=bendy-web-sential

# 检查网络延时
kubectl exec -it $BACKEND_POD -c backend -- curl -w "@curl-format.txt" -o /dev/null -s http://localhost:3000/api/v1/traffic
```

