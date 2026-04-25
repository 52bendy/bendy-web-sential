---
name: bws-test
description: Integration tests for bendy-web-sential gateway auth
context: fork
---

# BWS Integration Test Skill

You are a QA engineer testing the bendy-web-sential gateway authentication system. Your job is to verify the 0.1.1 auth feature works correctly by running integration tests against the running server.

## Test Environment

The server runs two ports:
- **Gateway**: `http://localhost:8080` — receives traffic
- **Admin API**: `http://localhost:3000` — management interface

Default credentials: `admin` / `bendy2024`

## Test Cases

### 0. Authentication Tests

1. **Login with valid credentials**
   ```bash
   curl -s -X POST http://localhost:3000/api/v1/auth/login \
     -H "Content-Type: application/json" \
     -d '{"username":"admin","password":"bendy2024"}'
   ```
   Expected: `{"code":0,"message":"ok","data":{"token":"eyJ...","expires_in":86400}}`

2. **Login with invalid credentials**
   ```bash
   curl -s -X POST http://localhost:3000/api/v1/auth/login \
     -H "Content-Type: application/json" \
     -d '{"username":"admin","password":"wrongpassword"}'
   ```
   Expected: code 1003 (Invalid credentials)

### 1. Domain & Route Setup

First, login to get a JWT token:
```bash
TOKEN=$(curl -s -X POST http://localhost:3000/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"bendy2024"}' | jq -r '.data.token')
```

Create a domain:
```bash
curl -s -X POST http://localhost:3000/api/v1/domains \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"domain":"test.example.com","active":1}'
```

Get the domain ID:
```bash
DOMAIN_ID=$(curl -s http://localhost:3000/api/v1/domains \
  -H "Authorization: Bearer $TOKEN" | jq -r '.data[0].id')
```

Create a route for proxy (no auth):
```bash
curl -s -X POST http://localhost:3000/api/v1/routes \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d "{\"domain_id\":$DOMAIN_ID,\"path_pattern\":\"/api\",\"action\":\"proxy\",\"target\":\"http://httpbin.org\",\"active\":1,\"priority\":100}"
```

### 2. Auth Strategy Tests

**Test A: JWT strategy without token → 401**
Create a route with `auth_strategy=jwt`:
```bash
ROUTE_ID=$(curl -s -X POST http://localhost:3000/api/v1/routes \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d "{\"domain_id\":$DOMAIN_ID,\"path_pattern\":\"/jwt-protected","action":"proxy","target":"http://httpbin.org","active":1,"priority":100,\"auth_strategy\":\"jwt\"}" | jq -r '.data.id')

# Request without token
curl -s -w "\nHTTP_STATUS:%{http_code}" http://localhost:8080/jwt-protected
```
Expected: 401 Unauthorized

**Test B: JWT strategy with valid token → pass through**
```bash
curl -s -w "\nHTTP_STATUS:%{http_code}" http://localhost:8080/jwt-protected \
  -H "Authorization: Bearer $TOKEN"
```
Expected: 200 (proxied to httpbin.org)

**Test C: API Key strategy without key → 401**
Create a route with `auth_strategy=api_key`:
```bash
curl -s -X POST http://localhost:3000/api/v1/routes \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d "{\"domain_id\":$DOMAIN_ID,\"path_pattern\":\"/apikey-protected","action":"proxy","target":"http://httpbin.org","active":1,\"priority\":100,\"auth_strategy\":\"api_key\"}"

# Request without API key
curl -s -w "\nHTTP_STATUS:%{http_code}" http://localhost:8080/apikey-protected
```
Expected: 401 Unauthorized

**Test D: API Key with valid key → pass through**
```bash
# Create an API key
RESP=$(curl -s -X POST http://localhost:3000/api/v1/keys \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"name":"test-key","role":"user"}')
API_KEY=$(echo $RESP | jq -r '.data.key')

curl -s -w "\nHTTP_STATUS:%{http_code}" http://localhost:8080/apikey-protected \
  -H "X-API-Key: $API_KEY"
```
Expected: 200 (proxied to httpbin.org)

**Test E: Revoked API Key → 401**
```bash
# Revoke the key
KEY_ID=$(curl -s http://localhost:3000/api/v1/keys \
  -H "Authorization: Bearer $TOKEN" | jq -r '.data[] | select(.name=="test-key") | .id')
curl -s -X DELETE http://localhost:3000/api/v1/keys/$KEY_ID \
  -H "Authorization: Bearer $TOKEN"

# Try using the revoked key
curl -s -w "\nHTTP_STATUS:%{http_code}" http://localhost:8080/apikey-protected \
  -H "X-API-Key: $API_KEY"
```
Expected: 401 Unauthorized

### 3. Cleanup

```bash
# Delete routes
curl -s -X DELETE "http://localhost:3000/api/v1/routes/$ROUTE_ID" \
  -H "Authorization: Bearer $TOKEN"

# Logout
curl -s -X POST http://localhost:3000/api/v1/auth/logout \
  -H "Authorization: Bearer $TOKEN"
```

## Execution

1. First ensure the server is running. If not, return an error and explain what needs to be done.
2. Run each test case and report results clearly.
3. For each test, show: command, expected result, actual result, PASS/FAIL.
4. Summarize: total tests, passed, failed.
5. If the server is not running, check if there's a built binary or suggest how to start it.
