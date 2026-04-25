# GitHub Agents 研究报告

**调研日期:** 2026-04-25
**调研人:** Claude Code

---

## 一、GitHub 现有 AI Agents 相关产品

### 1. GitHub Copilot Agent Mode

GitHub Copilot 的 Agent 模式允许开发者将任务分配给 AI Agent，这些 Agent **在后台自主规划、探索和执行工作**。

#### 核心功能

| 功能 | 说明 |
|------|------|
| **代码补全与编辑** | 在编辑器中启用 Agent 模式处理复杂任务 |
| **终端集成** | 在终端中规划、构建和执行工作流 |
| **多模型支持** | 支持 Copilot、Claude (Anthropic)、OpenAI Codex 等 |
| **MCP 集成** | 连接外部工具（我们项目已配置 `.mcp.json`） |

#### 定价

| 方案 | 价格 | Agent 限制 |
|------|------|-----------|
| **Free** | $0 | 50 requests/月，2000 completions/月 |
| **Pro** | $10/月 | 无限 Agent 模式和聊天，300 premium requests |
| **Pro+** | $39/月 | 全部 Pro 功能 + Claude Opus 4.7 + 5× premium requests |

### 2. GitHub Spark

GitHub Spark 是一个 AI 驱动的全栈应用构建平台。

#### 核心功能

- 使用自然语言、可视化控件或代码创建应用
- 实时预览，边构建边更新
- 一键发布，内置 GitHub 认证
- 内置 AI 功能（聊天机器人、内容生成、自动化）
- 与 GitHub Copilot 和 VS Code/Codespaces 集成

#### 定价

- 包含在 Copilot Pro+ ($39/月，最多 375 条消息)
- 或 Copilot Enterprise ($39/人/月，最多 250 条消息)

### 3. GitHub Models

用于管理和比较 Prompt 的 AI 模型平台。

### 4. MCP (Model Context Protocol)

**MCP 是一个开放协议**，用于将 AI 应用连接到外部系统。

```
AI 应用 ←→ MCP ←→ 外部工具/数据源
```

#### 类比

MCP 就像 AI 应用的 **USB-C 接口** — 标准化方式连接外部系统。

#### 我们项目的 MCP 配置

项目根目录已存在 `.mcp.json`，说明项目已支持 MCP 集成。

---

## 二、我们的项目能否使用？

### ✅ 可以使用的部分

| 产品/功能 | 适用场景 | 难度 |
|-----------|----------|------|
| **GitHub Copilot (Free)** | 代码补全、基础辅助 | ⭐ 零门槛 |
| **GitHub Copilot Pro ($10/月)** | Agent 模式处理复杂任务 | ⭐ 低门槛 |
| **MCP 服务器** | 连接外部工具（数据库、API 等） | ⭐⭐ 中等 |
| **GitHub Actions + AI** | CI/CD 流程中集成 AI | ⭐⭐⭐ 需要配置 |

### ❌ 不太适用的部分

| 产品/功能 | 原因 |
|-----------|------|
| **GitHub Spark** | 用于创建独立应用，与我们的网关项目定位不同 |
| **GitHub Models** | 主要是 Prompt 管理和测试平台 |

---

## 三、如何使用

### 方案 A: 使用 GitHub Copilot Agent Mode（推荐）

1. **升级到 Pro+ ($39/月)**
   - 获取 Claude Opus 4.7 模型
   - 无限 Agent 模式和聊天

2. **在工作流中调用 Agent**
   ```bash
   # 在 VS Code 中
   /agent plan: 重构 auth 模块，添加 SSO 支持

   # 在终端中
   gh copilot suggest "添加 JWT 刷新机制"
   ```

3. **在项目中配置 MCP**
   ```json
   // .mcp.json 已存在，可扩展
   {
     "servers": {
       "database": {
         "command": "npx",
         "args": ["@modelcontextprotocol/server-sqlite"]
       }
     }
   }
   ```

### 方案 B: 使用 MCP 连接我们的 API

我们项目是一个 API 网关，可以通过 MCP 暴露工具给 AI Agent：

```typescript
// 创建 MCP 服务器暴露我们的管理 API
import { McpServer } from "@modelcontextprotocol/sdk/server";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio";

const server = new McpServer({
  name: "bendy-web-sential-admin",
  version: "1.0.0",
});

server.tool(
  "create-domain",
  "Create a new domain configuration",
  { name: z.string(), target: z.string() },
  async ({ name, target }) => {
    const response = await fetch("http://localhost:3000/api/v1/domains", {
      method: "POST",
      headers: { "Authorization": `Bearer ${process.env.BWS_TOKEN}` },
      body: JSON.stringify({ name, target }),
    });
    return response.json();
  }
);
```

### 方案 C: GitHub Actions + AI 自动化

在 CI/CD 中集成 AI Agent：

```yaml
# .github/workflows/ai-review.yml
name: AI Code Review
on: [pull_request]

jobs:
  review:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: AI Review
        uses: cognitive/ai-review-action@v1
        with:
          model: "claude-opus-4"
          api-key: ${{ secrets.ANTHROPIC_API_KEY }}
```

---

## 四、建议的实施路径

### 短期（1-2 周）

1. **团队成员升级到 Copilot Pro+** ($39/月/人)
   - 获取 Claude Opus 4.7 模型
   - Agent 模式加速开发

2. **配置 MCP 连接开发数据库**
   - 现有 `.mcp.json` 可直接扩展
   - 参考 [MCP 官方文档](https://modelcontextprotocol.io/)

### 中期（1 个月）

1. **创建项目专属 MCP 服务器**
   - 暴露 bendy-web-sential 管理 API
   - 让 AI Agent 可以直接查询/修改路由配置

2. **在 GitHub Actions 中集成 AI 审查**
   - PR 自动 AI 审查
   - 自动生成 CHANGELOG

### 长期（3 个月+）

1. **探索 GitHub Spark**
   - 如果需要创建独立的管理 App
   - 使用自然语言构建管理界面原型

2. **多 Agent 协作**
   - 一个 Agent 处理前端，一个处理后端
   - 通过 MCP 协调任务

---

## 五、参考资料

- [GitHub Copilot 官方](https://github.com/features/copilot)
- [GitHub Copilot 定价](https://github.com/features/copilot/plans)
- [GitHub Spark](https://github.com/features/spark)
- [MCP 官方文档](https://modelcontextprotocol.io/)
- [我们项目的 MCP 配置](../.mcp.json)

---

## 六、结论

| 问题 | 答案 |
|------|------|
| **能否使用 GitHub Agents?** | ✅ 能，且有多种集成方式 |
| **我们适合用哪种?** | Copilot Pro+ ($39/月) + MCP 扩展 |
| **集成难度?** | 低，零代码即可享受基础功能 |
| **最佳实践?** | 从 Copilot Pro 开始，根据需要升级到 Pro+ |

### 行动建议

1. **立即行动**: 团队成员试用 Copilot Free，体验 Agent 模式
2. **下周决策**: 评估是否升级到 Pro+，开启 Agent 模式增强开发
3. **本月规划**: 扩展 MCP 配置，连接项目 API 到 AI Agent
