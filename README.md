# GitMentor Lite

一个基于 Tauri + Vue 3 + Rust 的跨平台桌面应用，用 AI 生成高质量的 Git 提交信息，并集成常用 Git 操作面板与“分层提交”工作流。支持多家 AI 提供商与可配置模板，帮助你在保证规范的同时显著提升提交效率。

---

## 简介（中文）

GitMentor Lite 专注于“提交信息自动化 + Git 面板”。前端使用 Vue 3 + Element Plus，后端以 Tauri v2 + Rust 实现，通过 git2 与多家 AI Provider 协同完成：

- 一键生成规范化提交信息（可模板化、参数化）
- 分层提交会话编排
- 变更查看、暂存、提交、撤销等常用 Git 操作

### 功能特性

- 提交信息生成
  - 模板与两段式处理（语言、max_tokens、temperature、emoji、类型等）
  - 支持“分层提交”工作流
- Git 面板
  - Diff/Stage/Commit/Revert
  - 历史/分支（以 UI 为准，持续完善中）
- 多提供商支持（已实现并注册）
  - OpenAI、Ollama、智谱(Zhipu)、Anthropic、DashScope、Doubao（豆包）、Gemini、Deepseek
- 对话记录与进度流
  - 记录 AI 请求/响应与会话，便于调试与审计
- 桌面端体验
  - Tauri v2，轻量、安全；Windows 默认 MSI 打包，可捆绑 Git sidecar（可选）

### 技术栈

- 桌面：Tauri v2（opener、dialog、shell 插件）
- 前端：Vue 3 + TypeScript + Vite + Element Plus
- 后端：Rust（git2、reqwest、tokio、serde、handlebars、uuid、regex 等）
- 辅助：@git-diff-view、vue-diff、jsdiff、pinia、vue-router

### 环境要求

- Node.js 16+
- Rust 1.70+
- Git 2.30+
- Windows/Mac/Linux（以 Tauri 支持为准；默认打包目标为 Windows MSI）

### 安装与运行

1. 克隆仓库

- git clone <repo-url>
- cd GitMentor

2. 安装依赖

- cd GitMentor-Lite
- npm install

3. 开发运行

- npm run tauri:dev
  - 启动 Vite 开发服务器（http://localhost:1420）与 Tauri 窗口
  - 若端口占用，可运行 GitMentor-Lite/kill-port-1420.bat 清理

4. 构建打包

- npm run tauri:build
  - Windows 默认生成 MSI 安装包
  - 内置 Git sidecar：参考 GitMentor-Lite/src-tauri/binaries/README.md

首次使用引导：选择提供商 → 配置密钥 → 测试连接 → 选择仓库

### 使用方法

- 生成提交信息

  1. 选择仓库并查看变更
  2. 在生成视图选择模板与参数
  3. 点击“一键生成”，预览后提交

- 分层提交

  1. 进入“分层提交”模式
  2. AI 按层给出建议，用户可编辑确认
  3. 可随时取消会话

- Git 操作
  - 在面板中执行暂存/撤销/提交
  - 历史/分支按 UI 为准（迭代中）

### API（Tauri 命令，部分）

前端通过 `@tauri-apps/api.invoke` 调用：

- 基础：`greet`
- Git：`select_repository`、`get_git_status`、`stage_files`、`commit_changes`、`revert_files`、`generate_commit_message`
- 调试：`get_debug_settings`、`set_debug_logs_enabled`、`update_debug_settings`
- Git 配置：`get_git_config`、`update_git_config`
- AI/分层提交：`list_providers`、`update_provider_config`、`remove_provider_config`、`get_layered_sessions`、`get_conversation_records_by_session`、`check_and_process_file_tokens`、`cancel_layered_commit`、`check_first_time_setup`、`test_ai_connection`

建议在 docs/API.md 中维护完整参数与返回结构。

### 目录结构

```text
GitMentor-Lite/
  src/                # 前端（Vue 3 + TS + Element Plus；icons 在 public/icons）
  src-tauri/          # Rust 后端与 Tauri 配置
    src/core/         # AI 管理、模板、Git 引擎、分层提交、会话记录等
    tauri.conf.json   # Tauri 构建/打包配置（devUrl、externalBin 等）
  package.json        # 脚本（dev/build/tauri:dev/tauri:build）
```

### 架构与数据流

- 前端（Vue）通过 `invoke` 调用后端命令
- 后端（Tauri + Rust）分发到核心模块：
  - GitEngine（git2）执行仓库操作
  - AIManager 通过 Provider 工厂选择具体提供商
  - LLMClient/PromptManager 处理统一参数与模板
  - ConversationLogger 记录请求/响应与会话
  - LayeredCommitManager 管理会话与取消
- 结果返回前端预览，用户确认后提交

### 常用脚本

- GitMentor-Lite/kill-port-1420.bat：清理 1420 端口占用

### 贡献指南

欢迎 Issue/PR。提交前请确保：

- 遵循现有代码风格与目录组织
- Rust 可编译通过；前端通过类型检查与构建
- 新 Provider 或命令请补充文档/示例，并在工厂中注册

### 许可证

GPL-3.0 license
