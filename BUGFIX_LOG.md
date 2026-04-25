# Bugfix Log

## 2026-04-25: GitHub OAuth 登录后跳回登录页

### 问题现象
用户通过 GitHub OAuth 授权登录后，能短暂进入后台，但不到一秒就跳回登录页，形成无限循环。

### 根因分析
**前端竞态条件 (Race Condition)**

1. **Login.tsx**: 检测到 URL 中的 `token` 参数后，同步调用 `setToken()` 写入 localStorage，然后立即 `navigate('/')`
2. **SsoAutoLogin (App.tsx)**: 同时检测到 URL 中的 `token` 参数，调用 `/v1/auth/me` 验证 token
3. 如果 `/me` 请求返回 401，axios 拦截器 (`api.ts`) 会清除 localStorage 并重定向到 `/login`
4. 由于 `ProtectedRoute` 依赖 localStorage 中的 token 状态，而 Login.tsx 先写入 token 导致竞态

### 代码问题
```typescript
// Login.tsx - 同步写入 token，无服务器验证
if (token) {
  setToken(token);           // 直接写入 localStorage
  navigate('/');            // 立即导航
}

// App.tsx - 异步验证 token
api.post('/v1/auth/sso/exchange', { token })
  .then(async ({ data }) => {
    await api.get('/v1/auth/me', {...});  // 可能返回 401
    cleanUrl();
    setToken(finalToken);   // 如果上面 401，这里不会执行
    navigate('/', { replace: true });
  })
```

### 修复方案
1. **Login.tsx**: 移除 token 处理逻辑，只保留错误处理
2. **App.tsx**: 添加 cleanup 函数防止 pending 状态卡住

### 修改的文件
- `frontend/src/pages/Login.tsx` - 移除 token 处理，保留 error 处理
- `frontend/src/App.tsx` - 添加 useEffect cleanup

### 教训总结
1. **不要在多个组件中处理相同的 URL 参数** - 选择一个组件统一处理
2. **Token 应该只在服务器验证后才写入 localStorage** - 避免无效 token 被使用
3. **添加 cleanup 函数** - 防止组件卸载时异步操作导致状态不一致
4. **axios 拦截器的破坏性操作** - 401 清除 token 会导致页面刷新后状态丢失

### 预防措施
1. 在代码审查时检查是否有多个组件处理相同的 URL 参数
2. Token 相关操作应该有统一的入口点
3. 添加前端集成测试覆盖登录流程
