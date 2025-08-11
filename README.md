# GitMentor Lite

一个基于 Tauri + Vue 3 + Rust 的桌面工具，用 AI 生成高质量 Git 提交信息，支持 12+ 主流 AI 提供商与可配置模板。

## 功能速览
- 一键生成提交信息（支持模板与分层提交）
- Git 状态/暂存/撤销/历史/分支管理
- 首次启动引导与多提供商配置（OpenAI、Ollama、Anthropic、智谱、Deepseek、Gemini、DashScope、豆包、SiliconFlow、OpenRouter、Together、Groq）
- 对话记录与进度流式事件

## 技术栈
- 前端：Vue 3 + TypeScript + Vite + Element Plus
- 桌面：Tauri v2
- 后端：Rust（git2、reqwest、tokio）

## 快速开始
1. 安装依赖
   - Node.js 16+
   - Rust 1.70+
   - Git 2.30+
2. 开发运行
   - 在 GitMentor-Lite 目录执行：
     - npm run tauri:dev（开发）
     - npm run tauri:build（打包）
3. 首次使用
   - 打开应用后按引导依次：选择提供商 → 配置密钥 → 测试连接 → 选择仓库

## 常用脚本
- GitMentor-Lite/kill-port-1420.bat：清理 Tauri dev 端口占用
- GitMentor-Lite/diagnose_and_start.bat：诊断与启动检查
- GitMentor-Lite/cleanup_unused_icons.bat：清理多余图标，仅保留 12 个必要图标

## 项目结构（简）
- GitMentor-Lite/
  - src/ 前端页面与组件（图标位于 public/icons）
  - src-tauri/ Rust 后端、命令与配置（tauri.conf.json）

## 注意事项
- 图标：仅使用 public/icons 下的 12 个 webp 文件；src/assets/static-webp 为第三方图标全集，已在 .gitignore 中忽略，避免仓库臃肿。
- 配置与日志：src-tauri/.config/*.json 及 *.log 已加入 .gitignore，避免泄露与污染。
- 内置 Git：如需打包内置 Git sidecar，请参考 GitMentor-Lite/src-tauri/binaries/README.md。

## 许可证
MIT

