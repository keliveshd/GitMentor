# GitMentor - 纯Rust客户端程序开发计划

## 项目概述

GitMentor是一个基于AI技术的Git提交分析工具，采用纯Rust架构实现高性能、零依赖的桌面应用程序。项目从零开始搭建，专注于Windows和macOS平台支持。

### 核心价值
- 自动化分析Git提交历史
- 智能整理和汇总贡献者工作内容
- 生成可视化报告和洞察
- 提供代码质量和效率评估
- 零依赖部署，单一可执行文件

## 技术架构设计

### 新架构：纯Rust一体化设计
```
┌─────────────────────────────────────────────────────────────┐
│                    Tauri Frontend                          │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────┐ │
│  │   Dashboard     │  │   Settings      │  │   Reports   │ │
│  │   Component     │  │   Component     │  │  Component  │ │
│  └─────────────────┘  └─────────────────┘  └─────────────┘ │
│                    Vue 3 + TypeScript                      │
└─────────────────────────────────────────────────────────────┘
                              │
                              │ Tauri IPC
                              │
┌─────────────────────────────────────────────────────────────┐
│                   Rust Backend (集成)                       │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────┐ │
│  │   Git Engine    │  │   Agent System  │  │   Storage   │ │
│  │   (git2-rs)     │  │   (AI Agents)   │  │  Manager    │ │
│  └─────────────────┘  └─────────────────┘  └─────────────┘ │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────┐ │
│  │   LLM Client    │  │   Cache System  │  │   Config    │ │
│  │   (reqwest)     │  │   (moka)        │  │  Manager    │ │
│  └─────────────────┘  └─────────────────┘  └─────────────┘ │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────┐ │
│  │   Database      │  │   Monitoring    │  │   Utils     │ │
│  │   (sqlx)        │  │   (tracing)     │  │  (crypto)   │ │
│  └─────────────────┘  └─────────────────┘  └─────────────┘ │
└─────────────────────────────────────────────────────────────┘
                              │
                              │ HTTPS API Calls
                              │
┌─────────────────────────────────────────────────────────────┐
│                    LLM Services                             │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────┐ │
│  │   Local LLM     │  │   OpenAI API    │  │  Anthropic  │ │
│  │   (Ollama)      │  │                 │  │     API     │ │
│  └─────────────────┘  └─────────────────┘  └─────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

### 技术栈选择

#### 前端技术栈 (Tauri + Vue 3)
- **Tauri 2.0**: 跨平台桌面应用框架，Rust后端集成
- **Vue 3**: 渐进式JavaScript框架，组合式API
- **TypeScript**: 类型安全的JavaScript超集
- **Element Plus**: Vue 3 企业级UI组件库
- **Tailwind CSS**: 实用优先的CSS框架
- **Pinia**: Vue 3 官方状态管理库
- **Vue Router**: 官方路由管理
- **Chart.js + Vue-Chartjs**: 数据可视化图表库
- **Vite**: 现代化前端构建工具

**选择理由**:
- Tauri提供原生性能和极小体积
- Vue 3组合式API与TypeScript完美集成
- Element Plus提供丰富的企业级组件
- 完全集成的Rust后端，无需外部依赖

#### Rust后端技术栈 (完全集成)
- **git2**: 高性能Git操作库，libgit2的Rust绑定
- **sqlx**: 异步SQL工具包，编译时SQL检查
- **tokio**: 异步运行时，高性能并发处理
- **reqwest**: 现代HTTP客户端，支持async/await
- **serde**: 序列化/反序列化框架
- **moka**: 高性能缓存库，支持TTL和LRU
- **handlebars**: 模板引擎，用于提示词管理
- **tracing**: 结构化日志和监控
- **anyhow/thiserror**: 错误处理
- **config/figment**: 配置管理
- **ring**: 加密库，用于API密钥安全存储

**选择理由**:
- 零运行时依赖，静态编译
- 内存安全和并发安全保证
- 极致性能，特别是Git操作和数据处理
- 丰富的异步生态系统
- 编译时错误检查，减少运行时问题

#### AI集成策略
- **统一LLM客户端**: 支持多个LLM提供商
- **OpenAI API**: GPT-4/GPT-3.5支持，使用reqwest实现
- **Anthropic API**: Claude支持，自实现客户端
- **Ollama**: 本地LLM运行环境，HTTP API调用
- **提示词管理**: 基于handlebars的模板系统
- **API密钥安全**: 使用ring加密存储

## 从零开始分阶段实施计划

### 第一阶段：项目基础搭建 (2周)

#### 目标
建立完整的开发环境和基础架构，实现Tauri + Rust的基础框架

#### Week 1: 环境搭建和项目初始化

**Day 1-2: 开发环境准备**
```bash
# 安装Rust工具链
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add x86_64-pc-windows-msvc    # Windows目标
rustup target add x86_64-apple-darwin       # macOS Intel目标
rustup target add aarch64-apple-darwin      # macOS Apple Silicon目标

# 安装Node.js和前端工具
npm install -g @tauri-apps/cli
npm install -g typescript
```

**Day 3-4: 项目初始化**
```bash
# 创建Tauri项目
npm create tauri-app@latest GitMentor --template vue-ts
cd GitMentor

# 配置基础依赖
# 设置项目结构
# 配置开发环境
```

**Day 5-7: 基础架构代码**
- 实现基础的Rust模块结构
- 设置tracing日志系统
- 配置管理系统(config/figment)
- 基础的Tauri命令框架
- 错误处理系统(anyhow/thiserror)

#### Week 2: 核心模块框架

**Day 8-10: Git引擎基础**
- 集成git2库
- 实现Repository管理
- 基础提交信息提取
- 文件变更分析

**Day 11-12: 数据库设计**
- SQLite数据库设计
- sqlx集成和迁移系统
- 基础数据模型定义
- 数据库连接池配置

**Day 13-14: 前端基础框架**
- Vue 3 + TypeScript项目结构
- Element Plus集成
- 基础路由和状态管理(Pinia)
- Tauri命令调用封装

#### 可交付成果
- ✅ 完整的开发环境和工具链
- ✅ 基础项目结构和模块框架
- ✅ Git仓库基础读取功能
- ✅ 数据库连接和基础操作
- ✅ 前后端通信框架
- ✅ 基础UI界面和路由系统

### 第二阶段：Git分析引擎 (3周)

#### 目标
实现完整的Git操作和基础分析功能，建立LLM集成基础

#### Week 3: Git操作核心功能

**Day 15-17: 高级Git操作**
- 实现完整的提交分析功能
- 差异(diff)分析和统计
- 分支信息和合并历史
- 文件变更模式识别
- 贡献者统计和分析

**Day 18-19: 仓库监控和缓存**
- 文件系统监控(notify)
- 智能缓存系统(moka)
- LRU缓存策略
- 缓存命中率统计
- 性能优化

**Day 20-21: Tauri命令集成**
- Git操作的Tauri命令封装
- 异步操作处理
- 错误处理和用户反馈
- 进度报告机制

#### Week 4: LLM集成基础

**Day 22-24: LLM客户端实现**
- 统一LLM客户端接口设计
- reqwest HTTP客户端配置
- API密钥安全存储(ring加密)
- 请求限流和重试机制
- 错误处理和降级策略

**Day 25-26: OpenAI集成**
- OpenAI API客户端实现
- Chat Completions API集成
- 流式响应处理
- 令牌计数和成本控制
- API响应缓存

**Day 27-28: 提示词管理系统**
- handlebars模板引擎集成
- 动态提示词加载
- 模板版本管理
- 上下文数据注入
- 提示词优化和A/B测试

#### Week 5: Agent系统基础

**Day 29-31: Agent基础架构**
- Agent trait定义
- 异步Agent执行框架
- Agent生命周期管理
- 配置热重载
- 健康检查机制

**Day 32-35: AnalyzerAgent实现**
- Git提交分析Agent
- 语义理解和分类
- 代码质量评估
- 影响范围分析
- 置信度计算

#### 可交付成果
- ✅ 完整的Git分析引擎
- ✅ 高性能缓存系统
- ✅ LLM客户端基础框架
- ✅ 提示词管理系统
- ✅ Agent系统基础架构
- ✅ 基础的提交分析功能

### 第三阶段：AI Agent系统完善 (3周)

#### 目标
实现完整的双重审核AI Agent系统和质量控制机制

#### Week 6: ReviewerAgent和质量控制

**Day 36-38: ReviewerAgent实现**
- 质量审核Agent设计
- 多维度质量评估
- 准确性、完整性、一致性评分
- 审核决策逻辑
- 改进建议生成

**Day 39-40: 质量控制器**
- 双重审核流程控制
- Agent执行编排
- 重试和错误恢复
- 质量阈值管理
- 审核结果聚合

**Day 41-42: Agent管理系统**
- Agent注册和发现
- 动态配置更新
- 性能监控和指标
- 负载均衡
- 健康检查和故障转移

#### Week 7: 存储和报告系统

**Day 43-45: 存储管理器**
- 双重存储策略(SQLite + Markdown)
- 数据一致性保证
- 批量操作优化
- 数据压缩和归档
- 备份和恢复机制

**Day 46-47: Markdown报告生成**
- 结构化报告模板
- 动态内容生成
- 层级目录组织
- 搜索和索引
- 版本控制集成

**Day 48-49: 数据可视化基础**
- Chart.js集成
- 实时数据更新
- 交互式图表
- 自定义图表类型
- 导出功能

#### Week 8: 高级功能和优化

**Day 50-52: 缓存和性能优化**
- 多层缓存策略
- 预加载和预计算
- 内存使用优化
- 并发处理优化
- 性能监控和调优

**Day 53-54: 配置和安全**
- 配置热重载
- API密钥轮换
- 数据加密存储
- 访问控制
- 审计日志

**Day 55-56: 错误处理和监控**
- 全局错误处理
- 结构化日志
- 性能指标收集
- 告警机制
- 故障诊断工具

#### 可交付成果
- ✅ 完整的双重审核AI Agent系统
- ✅ 质量控制和流程编排
- ✅ 双重存储系统(数据库+文件)
- ✅ Markdown报告生成
- ✅ 基础数据可视化
- ✅ 性能优化和监控系统

### 第四阶段：用户界面和体验优化 (2周)

#### 目标
完善用户界面，实现完整的用户体验和高级分析功能

#### Week 9: 前端界面完善

**Day 57-59: 核心界面组件**
- 仓库管理界面
- 分析结果展示
- Agent状态监控
- 配置管理界面
- 实时日志查看

**Day 60-61: 数据可视化增强**
- 提交时间线图表
- 贡献者活跃度热力图
- 代码量变化趋势
- 质量评分趋势
- 交互式仪表板

**Day 62-63: 用户体验优化**
- 响应式设计
- 加载状态和进度指示
- 错误提示和用户引导
- 快捷键支持
- 主题和个性化设置

#### Week 10: 高级功能和发布准备

**Day 64-66: 高级分析功能**
- 团队协作分析
- 代码质量趋势
- 效率指标计算
- 自定义分析规则
- 批量处理和导出

**Day 67-68: 系统集成测试**
- 端到端测试
- 性能基准测试
- 内存泄漏检测
- 跨平台兼容性测试
- 用户接受度测试

**Day 69-70: 发布准备**
- 构建脚本优化
- 安装包制作
- 文档完善
- 部署指南
- 版本发布流程

#### 可交付成果
- ✅ 完整的用户界面和交互体验
- ✅ 丰富的数据可视化功能
- ✅ 高级分析和报告功能
- ✅ 跨平台构建和部署
- ✅ 完整的文档和用户指南
- ✅ 生产就绪的应用程序

## 项目里程碑和时间线

### 总体时间规划 (10周)

| 阶段 | 时间 | 主要目标 | 关键交付物 |
|------|------|----------|------------|
| 第一阶段 | Week 1-2 | 基础架构搭建 | 开发环境、基础框架、Git引擎 |
| 第二阶段 | Week 3-5 | Git分析引擎 | LLM集成、Agent系统、分析功能 |
| 第三阶段 | Week 6-8 | AI系统完善 | 双重审核、存储系统、报告生成 |
| 第四阶段 | Week 9-10 | 界面和发布 | 用户界面、测试、发布准备 |

### 关键里程碑

- **里程碑1** (Week 2结束): 基础架构完成，Git基础功能可用
- **里程碑2** (Week 5结束): AI分析引擎完成，基础Agent系统运行
- **里程碑3** (Week 8结束): 完整的双重审核系统，存储和报告功能
- **里程碑4** (Week 10结束): 生产就绪的应用程序，跨平台发布

### 风险缓解计划

#### 高风险项目
1. **LLM API集成复杂性** - 预留额外1周时间用于API调试
2. **Rust异步编程复杂性** - 分阶段实现，先同步后异步
3. **跨平台构建问题** - 早期建立CI/CD流程

#### 应急方案
- 保持功能的最小可行版本(MVP)
- 关键功能优先，高级功能可延后
- 建立回滚机制和版本控制

## 技术挑战和解决方案

### 1. 大型仓库性能问题
**挑战**: 处理包含大量提交历史的仓库时性能下降
**解决方案**: 
- 实现增量分析和缓存机制
- 使用后台任务处理大数据量
- 分页和懒加载技术

### 2. AI API成本控制
**挑战**: 频繁的AI API调用可能产生高额费用
**解决方案**:
- 智能缓存AI分析结果
- 批量处理减少API调用
- 提供本地LLM选项

### 3. 跨平台兼容性
**挑战**: 确保在Windows、macOS、Linux上正常运行
**解决方案**:
- 使用Tauri的跨平台能力
- 充分测试各平台特性
- 平台特定的优化和适配

### 4. 数据安全和隐私
**挑战**: 保护用户代码和API密钥安全
**解决方案**:
- 本地数据加密存储
- 安全的API密钥管理
- 可选的完全离线模式

### 5. Git仓库复杂性
**挑战**: 处理复杂的Git历史和分支结构
**解决方案**:
- 使用成熟的GitPython库
- 实现健壮的错误处理
- 支持多种Git工作流

## 项目里程碑

- **里程碑1** (第1阶段结束): 基础原型完成
- **里程碑2** (第2阶段结束): AI分析功能上线
- **里程碑3** (第3阶段结束): 完整报告系统
- **里程碑4** (第4阶段结束): 高级功能完成
- **里程碑5** (第5阶段结束): 产品发布就绪

## 资源需求

### 开发环境
- Windows 10/11 (主要开发环境)
- Node.js 18+ 和 npm/yarn
- Python 3.9+
- Rust 1.70+
- Git 2.30+

### 外部服务
- OpenAI API访问权限
- Anthropic API访问权限 (可选)
- Ollama本地部署环境

### 开发工具
- VS Code + Tauri扩展
- PyCharm/VS Code (Python开发)
- Git客户端
- 设计工具 (Figma等)

## 风险评估

### 高风险
- AI API服务稳定性和成本
- 大型仓库性能问题
- 跨平台兼容性问题

### 中风险
- 用户界面复杂度
- 数据安全要求
- 第三方依赖更新

### 低风险
- 基础Git操作
- 本地数据存储
- 基础UI组件

## 成功指标

### 技术指标
- 支持10万+提交的仓库分析
- 应用启动时间 < 3秒
- 内存使用 < 500MB
- 跨平台100%兼容

### 用户体验指标
- 用户操作响应时间 < 1秒
- 错误率 < 1%
- 用户满意度 > 4.5/5
- 文档完整度 > 90%

## 详细技术规范

### 前端技术栈详细配置

#### Tauri配置
```json
{
  "tauri": {
    "allowlist": {
      "all": false,
      "shell": {
        "all": false,
        "sidecar": true
      },
      "fs": {
        "all": false,
        "readFile": true,
        "writeFile": true,
        "readDir": true,
        "exists": true
      },
      "dialog": {
        "all": false,
        "open": true,
        "save": true
      }
    }
  }
}
```

#### 核心依赖包
```json
{
  "dependencies": {
    "vue": "^3.3.0",
    "@tauri-apps/api": "^2.0.0",
    "vue-router": "^4.2.0",
    "pinia": "^2.1.0",
    "element-plus": "^2.4.0",
    "@element-plus/icons-vue": "^2.1.0",
    "chart.js": "^4.4.0",
    "vue-chartjs": "^5.2.0",
    "axios": "^1.5.0",
    "@vueuse/core": "^10.4.0"
  },
  "devDependencies": {
    "@vitejs/plugin-vue": "^4.4.0",
    "@vue/tsconfig": "^0.4.0",
    "typescript": "^5.0.0",
    "tailwindcss": "^3.3.0",
    "autoprefixer": "^10.4.0",
    "postcss": "^8.4.0",
    "unplugin-auto-import": "^0.16.0",
    "unplugin-vue-components": "^0.25.0"
  }
}
```

### 后端技术栈详细配置

#### Python依赖
```txt
fastapi==0.103.0
uvicorn==0.23.0
gitpython==3.1.32
sqlalchemy==2.0.20
sqlite3
pydantic==2.3.0
httpx==0.24.0
python-dotenv==1.0.0
langchain==0.0.300
openai==0.28.0
anthropic==0.3.0
ollama==0.1.0
pandas==2.1.0
numpy==1.24.0
matplotlib==3.7.0
jinja2==3.1.2
reportlab==4.0.4
```

#### 纯Rust架构项目结构
```
GitMentor/
├── src-tauri/                    # Tauri Rust后端
│   ├── src/
│   │   ├── main.rs              # 应用入口
│   │   ├── lib.rs               # 库入口
│   │   ├── commands/            # Tauri命令
│   │   │   ├── mod.rs
│   │   │   ├── git_commands.rs
│   │   │   ├── analysis_commands.rs
│   │   │   └── config_commands.rs
│   │   ├── core/                # 核心功能模块
│   │   │   ├── mod.rs
│   │   │   ├── git_engine.rs    # Git操作引擎
│   │   │   ├── database.rs      # 数据库管理
│   │   │   ├── cache.rs         # 缓存管理
│   │   │   └── config.rs        # 配置管理
│   │   ├── agents/              # AI Agent系统
│   │   │   ├── mod.rs
│   │   │   ├── base_agent.rs    # Agent基类
│   │   │   ├── analyzer_agent.rs
│   │   │   ├── reviewer_agent.rs
│   │   │   └── agent_manager.rs
│   │   ├── llm/                 # LLM集成
│   │   │   ├── mod.rs
│   │   │   ├── client.rs        # 统一LLM客户端
│   │   │   ├── openai.rs        # OpenAI实现
│   │   │   ├── anthropic.rs     # Anthropic实现
│   │   │   └── ollama.rs        # Ollama实现
│   │   ├── storage/             # 存储管理
│   │   │   ├── mod.rs
│   │   │   ├── database_manager.rs
│   │   │   ├── file_manager.rs
│   │   │   └── markdown_generator.rs
│   │   ├── utils/               # 工具函数
│   │   │   ├── mod.rs
│   │   │   ├── crypto.rs        # 加密工具
│   │   │   ├── file_utils.rs
│   │   │   └── time_utils.rs
│   │   └── types/               # 类型定义
│   │       ├── mod.rs
│   │       ├── git_types.rs
│   │       ├── agent_types.rs
│   │       └── config_types.rs
│   ├── tauri.conf.json          # Tauri配置
│   ├── Cargo.toml               # Rust依赖
│   ├── build.rs                 # 构建脚本
│   └── icons/                   # 应用图标
├── src/                         # Vue前端
│   ├── main.ts
│   ├── App.vue
│   ├── views/                   # 页面组件
│   │   ├── Dashboard.vue
│   │   ├── RepositoryConfig.vue
│   │   ├── AnalysisResults.vue
│   │   ├── AgentManagement.vue
│   │   └── Settings.vue
│   ├── components/              # 通用组件
│   │   ├── GitRepositoryCard.vue
│   │   ├── AnalysisChart.vue
│   │   ├── AgentStatusCard.vue
│   │   └── ConfigEditor.vue
│   ├── composables/             # Vue组合式函数
│   │   ├── useGitOperations.ts
│   │   ├── useAnalysis.ts
│   │   └── useConfig.ts
│   ├── stores/                  # Pinia状态管理
│   │   ├── git.ts
│   │   ├── analysis.ts
│   │   └── config.ts
│   ├── types/                   # TypeScript类型
│   │   ├── git.ts
│   │   ├── analysis.ts
│   │   └── config.ts
│   └── utils/                   # 前端工具
│       ├── api.ts
│       └── format.ts
├── docs/                        # 项目文档
├── tests/                       # 测试文件
│   ├── rust_tests/              # Rust单元测试
│   └── integration_tests/       # 集成测试
├── scripts/                     # 构建脚本
│   ├── build.sh                 # Unix构建脚本
│   └── build.bat                # Windows构建脚本
├── config/                      # 配置文件模板
│   ├── default.toml
│   └── prompts/
│       ├── analyzer_prompts.toml
│       └── reviewer_prompts.toml
└── migrations/                  # 数据库迁移文件
    ├── 001_initial.sql
    ├── 002_agents.sql
    └── 003_analysis_results.sql
```

### Rust依赖配置

#### Cargo.toml
```toml
[package]
name = "gitmentor"
version = "1.0.0"
edition = "2021"

[dependencies]
# Tauri核心
tauri = { version = "2.0", features = ["shell-open"] }
tauri-build = { version = "2.0", features = [] }

# Git操作
git2 = "0.18"

# 数据库
sqlx = { version = "0.7", features = ["sqlite", "runtime-tokio-rustls", "chrono", "migrate"] }

# 异步运行时
tokio = { version = "1.0", features = ["full"] }

# HTTP客户端 (LLM API调用)
reqwest = { version = "0.11", features = ["json", "rustls-tls"] }

# 序列化
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# 配置管理
config = "0.14"
figment = { version = "0.10", features = ["toml", "json", "env"] }

# 缓存
moka = { version = "0.12", features = ["future"] }

# 模板引擎 (提示词)
handlebars = "4.5"

# 日志
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# 错误处理
anyhow = "1.0"
thiserror = "1.0"

# 时间处理
chrono = { version = "0.4", features = ["serde"] }

# 文件系统
walkdir = "2.4"
notify = "6.1"

# 加密 (API密钥存储)
ring = "0.17"
base64 = "0.21"

# 异步trait
async-trait = "0.1"

# UUID生成
uuid = { version = "1.6", features = ["v4", "serde"] }
```

### 数据库设计

#### 核心表结构 (适配Rust/sqlx)
```sql
-- 仓库信息表
CREATE TABLE repositories (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    path TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    remote_url TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    last_analyzed DATETIME,
    total_commits INTEGER DEFAULT 0,
    enabled BOOLEAN DEFAULT TRUE
);

-- 提交信息表
CREATE TABLE commits (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    repo_id INTEGER NOT NULL,
    hash TEXT NOT NULL UNIQUE,
    author_name TEXT NOT NULL,
    author_email TEXT NOT NULL,
    commit_date DATETIME NOT NULL,
    message TEXT NOT NULL,
    files_changed INTEGER DEFAULT 0,
    insertions INTEGER DEFAULT 0,
    deletions INTEGER DEFAULT 0,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (repo_id) REFERENCES repositories (id) ON DELETE CASCADE
);

-- Agent执行记录表
CREATE TABLE agent_executions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    task_id TEXT NOT NULL,
    agent_type TEXT NOT NULL, -- 'analyzer', 'reviewer'
    commit_id INTEGER NOT NULL,
    input_data_hash TEXT NOT NULL,
    output_data TEXT NOT NULL,
    confidence_score REAL,
    processing_time_ms INTEGER,
    status TEXT NOT NULL, -- 'success', 'failed', 'timeout'
    error_message TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (commit_id) REFERENCES commits (id) ON DELETE CASCADE
);

-- 质量控制记录表
CREATE TABLE quality_control_records (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    task_id TEXT NOT NULL UNIQUE,
    commit_id INTEGER NOT NULL,
    analyzer_execution_id INTEGER,
    reviewer_execution_id INTEGER,
    final_status TEXT NOT NULL, -- 'approved', 'rejected', 'pending'
    overall_quality_score REAL,
    dimension_scores TEXT, -- JSON格式存储各维度分数
    retry_count INTEGER DEFAULT 0,
    completed_at DATETIME,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (commit_id) REFERENCES commits (id) ON DELETE CASCADE,
    FOREIGN KEY (analyzer_execution_id) REFERENCES agent_executions (id),
    FOREIGN KEY (reviewer_execution_id) REFERENCES agent_executions (id)
);

-- 配置表
CREATE TABLE settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    category TEXT DEFAULT 'general', -- 'general', 'llm', 'agent', 'storage'
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- 缓存表
CREATE TABLE cache_entries (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    expires_at DATETIME,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

### Tauri命令接口设计

#### 核心Tauri命令 (替代REST API)
```rust
// Git操作命令
#[tauri::command]
async fn open_repository(path: String) -> Result<String, String>;

#[tauri::command]
async fn get_repository_info(path: String) -> Result<RepositoryInfo, String>;

#[tauri::command]
async fn get_commit_history(repo_path: String, limit: usize) -> Result<Vec<CommitInfo>, String>;

#[tauri::command]
async fn analyze_commit(repo_path: String, commit_hash: String) -> Result<CommitAnalysis, String>;

// Agent操作命令
#[tauri::command]
async fn start_analysis(task_config: AnalysisConfig) -> Result<String, String>;

#[tauri::command]
async fn get_analysis_status(task_id: String) -> Result<AnalysisStatus, String>;

#[tauri::command]
async fn get_analysis_result(task_id: String) -> Result<AnalysisResult, String>;

// 配置管理命令
#[tauri::command]
async fn get_config() -> Result<AppConfig, String>;

#[tauri::command]
async fn update_config(config: AppConfig) -> Result<(), String>;

#[tauri::command]
async fn test_llm_connection(provider: String, api_key: String) -> Result<bool, String>;

// 存储和报告命令
#[tauri::command]
async fn get_stored_analyses(filter: AnalysisFilter) -> Result<Vec<StoredAnalysis>, String>;

#[tauri::command]
async fn export_analysis(task_id: String, format: String) -> Result<String, String>;

#[tauri::command]
async fn generate_report(config: ReportConfig) -> Result<String, String>;
```

## 开发环境搭建指南

### 1. 系统要求
- Windows 10/11 (推荐)
- 16GB+ RAM
- 100GB+ 可用磁盘空间
- 稳定的网络连接

### 2. 必需软件安装
```powershell
# 安装 Node.js (推荐使用 nvm-windows)
winget install OpenJS.NodeJS

# 安装 Python
winget install Python.Python.3.11

# 安装 Rust
winget install Rustlang.Rustup

# 安装 Git
winget install Git.Git

# 安装 VS Code
winget install Microsoft.VisualStudioCode
```

### 3. 开发工具配置
```powershell
# 安装 Tauri CLI
cargo install tauri-cli

# 安装 Python 包管理器
pip install pipenv

# 全局安装前端工具
npm install -g @tauri-apps/cli
npm install -g typescript
```

### 4. 项目初始化脚本
```powershell
# 创建项目目录
mkdir GitMentor
cd GitMentor

# 初始化 Tauri 项目
npm create tauri-app@latest . --template vue-ts

# 创建 Python 后端
mkdir backend
cd backend
pipenv install fastapi uvicorn gitpython sqlalchemy

# 返回项目根目录
cd ..

# 安装前端依赖
npm install
```

## 质量保证计划

### 代码质量标准
- TypeScript严格模式
- ESLint + Prettier配置
- Python Black + isort格式化
- 单元测试覆盖率 > 80%
- 集成测试覆盖核心功能

### 测试策略
```
测试金字塔:
┌─────────────────┐
│   E2E Tests     │  10%
├─────────────────┤
│ Integration     │  20%
│    Tests        │
├─────────────────┤
│   Unit Tests    │  70%
└─────────────────┘
```

### 性能基准
- 应用冷启动: < 3秒
- 仓库分析(1000提交): < 30秒
- UI响应时间: < 100ms
- 内存使用峰值: < 1GB

## 项目进度跟踪

### 技术决策记录
- **2024-01-XX**: 前端框架从React调整为Vue 3
- **2024-01-XX**: 包管理选择pip + requirements.txt
- **2024-01-XX**: UI组件库选择Element Plus

### 阶段进度状态

#### 第一阶段：基础架构搭建
- **状态**: 🟡 进行中
- **开始时间**: 2025-06-05
- **预计完成**: 待定
- **实际完成**: 待定
- **主要任务**:
  - [x] 环境检查和开发工具安装
  - [x] 项目初始化 (Tauri + Vue 3 + TypeScript)
  - [x] 基础UI框架 (Vue 3 + Element Plus + Tailwind CSS)
  - [x] Python后端基础架构 (FastAPI + SQLite + GitPython)
  - [x] Git集成模块 (仓库检测、提交历史、分支信息、贡献者统计)
  - [ ] 数据存储层完善
  - [ ] Tauri-Python集成

#### 第二阶段：AI分析引擎
- **状态**: ⚪ 未开始
- **开始时间**: 待定
- **预计完成**: 待定

#### 第三阶段：报告生成和可视化
- **状态**: ⚪ 未开始
- **开始时间**: 待定
- **预计完成**: 待定

#### 第四阶段：高级功能和优化
- **状态**: ⚪ 未开始
- **开始时间**: 待定
- **预计完成**: 待定

#### 第五阶段：用户体验和发布准备
- **状态**: ⚪ 未开始
- **开始时间**: 待定
- **预计完成**: 待定

### 学习点和最佳实践
*待记录*

### 遇到的问题和解决方案
*待记录*

---

*本开发计划将根据实际开发进度和用户反馈进行动态调整*
