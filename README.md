# GitMentor Lite

一个基于 Tauri + Vue 3 + Rust 的跨平台桌面应用，用 AI 生成高质量的 Git 提交信息，并集成常用 Git 操作面板、"分层提交"工作流、Gitflow 管理与远程仓库支持。支持多家 AI 提供商与可配置模板，帮助你在保证规范的同时显著提升提交效率。

---

## 简介（中文）

GitMentor Lite 专注于"提交信息自动化 + Git 面板 + Gitflow 工作流"。前端使用 Vue 3 + Element Plus，后端以 Tauri v2 + Rust 实现，通过 git2 与多家 AI Provider 协同完成：

- AI 生成规范化提交信息（可模板化、参数化）
- 分层提交会话编排
- Gitflow 工作流管理
- 远程仓库克隆与管理
- 智能分支切换与仓库操作
- 变更查看、暂存、提交、撤销等常用 Git 操作

### 最新功能更新（2025年10月）

- ✨ **Gitflow 完整支持**：特性分支、发布分支、热修分支的完整生命周期管理
- 🌐 **远程仓库管理**：支持仓库克隆、远程配置、分支推送与拉取
- 🧠 **智能分支切换**：基于分支所有者的智能分支管理与切换
- 📊 **增强型 Git 面板**：更完善的分支历史与操作界面
- 🔧 **Git 引擎优化**：更强大的 Git 操作能力与错误处理

### 功能特性

- **提交信息生成**
  - 模板与两段式处理（语言、max_tokens、temperature、emoji、类型等）
  - 支持"分层提交"工作流
  - 智能提交消息分析与优化建议

- **Git 面板**
  - Diff/Stage/Commit/Revert
  - 分支历史与可视化
  - 暂存区管理与撤销操作

- **Gitflow 工作流管理**
  - 特性分支（feature）创建与管理
  - 发布分支（release）与版本管理
  - 热修分支（hotfix）与紧急修复
  - Gitflow 可视化仪表板与进度跟踪

- **远程仓库支持**
  - Git 仓库克隆
  - 远程仓库配置与管理
  - 动态仓库名解析
  - 分支推送与拉取操作

- **智能分支操作**
  - 智能分支切换
  - 分支所有者管理
  - 基于上下文的分支建议

- **多提供商支持**（已实现并注册）
  - OpenAI、Ollama、智谱(Zhipu)、Anthropic、DashScope、Doubao（豆包）、Gemini、Deepseek
  - SiliconCloud、Together、OpenRouter 等

- **对话记录与进度流**
  - 记录 AI 请求/响应与会话，便于调试与审计
  - 实时流式输出显示
  - 详细的操作日志与错误追踪

- **模板版本化管理**
  - 提交模板版本控制
  - 模板自定义与管理
  - 模板历史记录与回滚

- **桌面端体验**
  - Tauri v2，轻量、安全；Windows 默认 MSI 打包，可捆绑 Git sidecar（可选）
  - 跨平台支持（Windows/Mac/Linux）

### 技术栈

- **桌面**：Tauri v2（opener、dialog、shell 插件）
- **前端**：Vue 3 + TypeScript + Vite + Element Plus
- **后端**：Rust（git2、reqwest、tokio、serde、handlebars、uuid、regex 等）
- **辅助**：@git-diff-view、vue-diff、jsdiff、pinia、vue-router
- **AI 集成**：多提供商支持，统一接口设计

### 环境要求

- Node.js 16+
- Rust 1.70+
- Git 2.30+
- Windows/Mac/Linux（以 Tauri 支持为准；默认打包目标为 Windows MSI）

### 安装与运行

1. **克隆仓库**

```bash
git clone <repo-url>
cd GitMentor
```

2. **安装依赖**

```bash
cd GitMentor-Lite
npm install
```

3. **开发运行**

```bash
npm run tauri:dev
```

- 启动 Vite 开发服务器（http://localhost:1420）与 Tauri 窗口
- 若端口占用，可运行 `GitMentor-Lite/kill-port-1420.bat` 清理

4. **构建打包**

```bash
npm run tauri:build
```

- Windows 默认生成 MSI 安装包
- 内置 Git sidecar：参考 `GitMentor-Lite/src-tauri/binaries/README.md`

**首次使用引导**：选择提供商 → 配置密钥 → 测试连接 → 选择仓库

### 使用方法

#### 基本使用流程

1. **选择或克隆仓库**
   - 从本地选择现有仓库
   - 或克隆远程仓库到本地

2. **生成提交信息**
   - 查看变更内容
   - 选择模板与参数
   - 点击"一键生成"，预览后提交

3. **Gitflow 工作流**
   - 启动 Gitflow 向导
   - 创建特性分支
   - 跟踪分支状态与进度
   - 完成分支合并

#### 高级功能

- **分层提交**：AI 按层给出建议，用户可编辑确认
- **远程同步**：推送/拉取分支到远程仓库
- **模板管理**：创建与管理自定义提交模板
- **对话历史**：查看 AI 生成历史与调试信息

### API（Tauri 命令）

前端通过 `@tauri-apps/api.invoke` 调用：

#### 基础命令
- `greet`

#### Git 操作
- `select_repository`、`get_git_status`、`stage_files`、`commit_changes`、`revert_files`、`generate_commit_message`
- `clone_repository`、`configure_remote`、`push_branch`、`pull_branch`

#### 分支管理
- `create_feature_branch`、`merge_branch`、`switch_branch`
- `list_branches`、`get_branch_info`

#### Gitflow 工作流
- `init_gitflow`、`create_feature`、`create_release`、`create_hotfix`
- `get_gitflow_status`、`complete_gitflow_operation`

#### 模板管理
- `list_templates`、`create_template`、`update_template`
- `generate_from_template`

#### AI/分层提交
- `list_providers`、`update_provider_config`、`test_ai_connection`
- `get_layered_sessions`、`cancel_layered_commit`
- `get_conversation_records_by_session`

#### 调试与配置
- `get_debug_settings`、`set_debug_logs_enabled`、`update_debug_settings`
- `get_git_config`、`update_git_config`

完整参数与返回结构建议在 `docs/API.md` 中维护。

### 目录结构

```
GitMentor-Lite/
├── src/                          # 前端（Vue 3 + TS + Element Plus）
│   ├── components/               # Vue 组件
│   │   ├── gitflow/              # Gitflow 工作流组件
│   │   ├── LayeredCommitProgress.vue
│   │   └── ...
│   ├── pages/                    # 页面组件
│   ├── types/                    # TypeScript 类型定义
│   └── utils/                    # 工具函数
├── src-tauri/                    # Rust 后端与 Tauri 配置
│   ├── src/
│   │   ├── commands/             # Tauri 命令实现
│   │   │   ├── gitflow_commands.rs
│   │   │   ├── repository_commands.rs
│   │   │   └── ...
│   │   ├── core/                 # 核心业务逻辑
│   │   │   ├── git_engine.rs     # Git 操作引擎
│   │   │   ├── ai_manager.rs     # AI 管理
│   │   │   ├── gitflow_manager.rs # Gitflow 管理
│   │   │   └── ...
│   │   ├── providers/            # AI 提供商实现
│   │   └── templates/            # 模板系统
│   ├── capabilities/             # Tauri 权限配置
│   └── icons/                    # 应用图标
├── docs/                         # 项目文档
├── public/icons/                 # AI 提供商图标
└── package.json                  # 构建脚本
```

### 架构与数据流

- **前端（Vue）**通过 `invoke` 调用后端命令
- **后端（Tauri + Rust）**分发到核心模块：
  - `GitEngine`（git2）执行仓库操作
  - `GitflowManager` 管理 Gitflow 工作流
  - `RepositoryManager` 处理远程仓库操作
  - `AIManager` 通过 Provider 工厂选择具体提供商
  - `LLMClient/PromptManager` 处理统一参数与模板
  - `ConversationLogger` 记录请求/响应与会话
  - `LayeredCommitManager` 管理会话与取消
- **结果返回**前端预览，用户确认后执行操作

### 配置文件

- **位置**：`src-tauri/.config`（gitignored）
- **内容**：Provider 密钥、模板设置、运行时日志
- **安全**：密钥不提交，不公开暴露配置文件

### 常用脚本

- `GitMentor-Lite/kill-port-1420.bat`：清理 1420 端口占用
- `GitMentor-Lite/build-backend.bat`：构建后端
- `GitMentor-Lite/diagnose_and_start.bat`：诊断并启动应用

### 开发文档

- [Git仓库管理功能设计文档](GitMentor-Lite/docs/Git仓库管理功能设计文档.md)
- [TEMPLATE_VERSIONING](GitMentor-Lite/docs/TEMPLATE_VERSIONING.md)
- [Daily Report Enhancement](GitMentor-Lite/DAILY_REPORT_ENHANCEMENT.md)
- [Streaming Implementation](GitMentor-Lite/streaming-implementation-summary.md)

### 贡献指南

欢迎 Issue/PR。提交前请确保：

- 遵循现有代码风格与目录组织
- Rust 可编译通过；前端通过类型检查与构建
- 新功能（Gitflow、远程仓库等）请补充相应文档/示例
- 新 Provider 或命令请在工厂中注册并更新 API 文档

### 更新日志

#### v0.2.7 (2025-10-21)
- ✨ 新增完整的 Gitflow 工作流支持
- 🌐 新增远程仓库克隆与管理功能
- 🧠 新增智能分支切换与所有者管理
- 🔧 优化 Git 引擎功能并更新依赖
- 📊 增强日志文件路径处理
- 🛠️ 动态化 Git 远程仓库名解析

#### v0.2.6 及之前
- 基础 Git 操作面板
- AI 提交信息生成
- 分层提交工作流
- 多提供商支持

### 许可证

GPL-3.0 license

### 联系方式

- 作者：Evilek
- 项目地址：[GitHub Repository]
- 技术支持：请提交 Issue

---

*更新时间：2025年10月30日*
