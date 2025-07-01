# GitMentor VSCode 开发手册

## 🎯 项目概述

GitMentor-Lite 是一个基于 Tauri + Vue 3 + Rust 的桌面应用，用于AI驱动的Git提交消息生成。

## 🛠️ 开发环境要求

### 必需工具
- **VSCode** - 主要开发IDE
- **Node.js 18+** - 前端开发环境
- **Rust 1.70+** - 后端开发环境
- **Git 2.30+** - 版本控制
- **PowerShell 5.0+** - Windows终端

### 推荐VSCode扩展
```json
{
  "recommendations": [
    "rust-lang.rust-analyzer",
    "tauri-apps.tauri-vscode",
    "vue.volar",
    "bradlc.vscode-tailwindcss",
    "ms-vscode.powershell",
    "formulahendry.auto-rename-tag",
    "esbenp.prettier-vscode"
  ]
}
```

## 🚀 快速开始

### 1. 环境检查
在VSCode终端中运行：
```powershell
# 检查Node.js版本
node --version  # 应该 >= 18.0.0

# 检查Rust版本
rustc --version  # 应该 >= 1.70.0

# 检查Cargo版本
cargo --version

# 检查Git版本
git --version  # 应该 >= 2.30.0
```

### 2. 项目初始化
```powershell
# 进入项目目录
cd GitMentor-Lite

# 安装前端依赖
npm install

# 检查Tauri CLI（如果没有会自动安装）
cargo tauri --version
```

### 3. 启动开发模式

#### 方法1: 使用统一构建脚本（推荐）
```powershell
# 在项目根目录运行
.\build-windows-package.bat --dev
```

#### 方法2: 使用npm命令
```powershell
# 在GitMentor-Lite目录运行
cd GitMentor-Lite
npm run tauri:dev
```

#### 方法3: 使用VSCode任务（见下文配置）
- 按 `Ctrl+Shift+P`
- 输入 "Tasks: Run Task"
- 选择 "Start Dev Mode"

## ⚙️ VSCode配置

### 1. 工作区设置 (.vscode/settings.json)
```json
{
  "rust-analyzer.cargo.target": "x86_64-pc-windows-msvc",
  "rust-analyzer.checkOnSave.command": "clippy",
  "typescript.preferences.importModuleSpecifier": "relative",
  "vue.codeActions.enabled": true,
  "editor.formatOnSave": true,
  "editor.defaultFormatter": "esbenp.prettier-vscode",
  "[rust]": {
    "editor.defaultFormatter": "rust-lang.rust-analyzer"
  },
  "files.associations": {
    "*.toml": "toml"
  }
}
```

### 2. 任务配置 (.vscode/tasks.json)
```json
{
  "version": "2.0.0",
  "tasks": [
    {
      "label": "Start Dev Mode",
      "type": "shell",
      "command": "npm",
      "args": ["run", "tauri:dev"],
      "group": {
        "kind": "build",
        "isDefault": true
      },
      "options": {
        "cwd": "${workspaceFolder}/GitMentor-Lite"
      },
      "presentation": {
        "echo": true,
        "reveal": "always",
        "focus": false,
        "panel": "new"
      },
      "problemMatcher": []
    },
    {
      "label": "Build Release",
      "type": "shell",
      "command": "npm",
      "args": ["run", "tauri:build"],
      "group": "build",
      "options": {
        "cwd": "${workspaceFolder}/GitMentor-Lite"
      }
    },
    {
      "label": "Install Dependencies",
      "type": "shell",
      "command": "npm",
      "args": ["install"],
      "options": {
        "cwd": "${workspaceFolder}/GitMentor-Lite"
      }
    }
  ]
}
```

### 3. 启动配置 (.vscode/launch.json)
```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "name": "Debug Tauri App",
      "type": "node",
      "request": "launch",
      "cwd": "${workspaceFolder}/GitMentor-Lite",
      "program": "${workspaceFolder}/GitMentor-Lite/node_modules/@tauri-apps/cli/bin/tauri.js",
      "args": ["dev"],
      "console": "integratedTerminal"
    }
  ]
}
```

## 📁 项目结构说明

```
GitMentor-Lite/
├── src/                    # Vue 3 前端源码
│   ├── components/         # Vue组件
│   │   ├── FileItem.vue   # 文件项组件
│   │   └── GitPanel.vue   # Git面板组件
│   ├── assets/            # 静态资源
│   ├── App.vue            # 主应用组件
│   └── main.ts            # 前端入口
├── src-tauri/             # Rust 后端源码
│   ├── src/
│   │   ├── commands/      # Tauri命令
│   │   ├── core/          # 核心功能模块
│   │   ├── types/         # 类型定义
│   │   ├── lib.rs         # 库入口
│   │   └── main.rs        # 主程序入口
│   ├── Cargo.toml         # Rust依赖配置
│   └── tauri.conf.json    # Tauri配置
├── package.json           # 前端依赖配置
├── vite.config.ts         # Vite构建配置
└── tsconfig.json          # TypeScript配置
```

## 🔧 开发工作流

### 1. 日常开发流程
```powershell
# 1. 启动开发服务器
npm run tauri:dev

# 2. 编辑代码（自动热重载）
# - 前端代码：src/ 目录
# - 后端代码：src-tauri/src/ 目录

# 3. 查看实时变更
# 应用会自动重新编译和重启
```

### 2. 前端开发
- **技术栈**: Vue 3 + TypeScript + Element Plus
- **热重载**: 保存文件后自动刷新
- **调试**: 使用浏览器开发者工具

### 3. 后端开发
- **技术栈**: Rust + Tauri + git2
- **热重载**: 保存Rust文件后自动重新编译
- **调试**: 使用 `println!` 或 `dbg!` 宏

## 🐛 调试技巧

### 1. 启用详细日志
```powershell
# 设置环境变量
$env:RUST_LOG="debug"
npm run tauri:dev
```

### 2. 前端调试
- 在应用中按 `F12` 打开开发者工具
- 使用 `console.log()` 输出调试信息
- 在VSCode中设置断点调试

### 3. 后端调试
- 在Rust代码中使用 `println!("调试信息: {:?}", variable)`
- 查看VSCode终端输出
- 使用 `cargo check` 检查语法错误

## 📦 构建和打包

### 开发构建
```powershell
npm run tauri:dev
```

### 生产构建
```powershell
# 方法1: 使用统一脚本
.\build-windows-package.bat

# 方法2: 使用npm
npm run tauri:build
```

## 🔍 常见问题解决

### 1. Rust编译错误
```powershell
# 更新Rust工具链
rustup update

# 清理并重新构建
cargo clean
npm run tauri:dev
```

### 2. 前端依赖问题
```powershell
# 删除node_modules重新安装
Remove-Item -Recurse -Force node_modules
npm install
```

### 3. Tauri CLI问题
```powershell
# 重新安装Tauri CLI
cargo install tauri-cli --version "^2.0"
```

## 🎯 开发建议

1. **使用VSCode扩展** - 安装推荐的扩展提高开发效率
2. **启用自动保存** - 设置 `"files.autoSave": "afterDelay"`
3. **使用Git集成** - 利用VSCode内置的Git功能
4. **定期构建测试** - 确保生产构建正常工作
5. **查看日志输出** - 关注终端中的错误和警告信息

## 📚 相关文档

- [Tauri官方文档](https://tauri.app/)
- [Vue 3官方文档](https://vuejs.org/)
- [Rust官方文档](https://doc.rust-lang.org/)
- [Element Plus文档](https://element-plus.org/)

---

**开始您的GitMentor开发之旅！** 🚀
