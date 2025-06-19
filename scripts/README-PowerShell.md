# GitMentor MVP PowerShell 脚本使用指南

本文档介绍如何使用 PowerShell 脚本来管理 GitMentor MVP 项目。

## 📁 脚本文件说明

| 脚本文件 | 功能描述 | 使用场景 |
|---------|----------|----------|
| `Check-Environment.ps1` | 检查开发环境 | 首次安装前检查 |
| `Install-Dependencies.ps1` | 自动安装依赖 | 环境准备 |
| `Setup-MVP.ps1` | 创建项目结构 | 项目初始化 |
| `Build-MVP.ps1` | 构建和运行项目 | 开发和构建 |
| `GitMentor-MVP.psm1` | PowerShell 模块 | 便捷命令集合 |

## 🚀 快速开始

### 方法1: 使用独立脚本 (推荐新手)

```powershell
# 1. 检查环境
.\scripts\Check-Environment.ps1

# 2. 安装依赖 (如果需要)
.\scripts\Install-Dependencies.ps1

# 3. 创建项目
.\scripts\Setup-MVP.ps1 -ProjectName "MyGitMentor"

# 4. 启动开发
cd MyGitMentor
..\scripts\Build-MVP.ps1 -Mode dev
```

### 方法2: 使用 PowerShell 模块 (推荐高级用户)

```powershell
# 1. 导入模块
Import-Module .\scripts\GitMentor-MVP.psm1

# 2. 查看帮助
Show-GitMentorHelp

# 3. 检查环境
Test-GitMentorEnvironment

# 4. 创建项目
New-GitMentorProject -ProjectName "MyGitMentor"

# 5. 启动开发
Start-GitMentorDev
```

## 📋 详细脚本说明

### Check-Environment.ps1

**功能**: 检查开发环境是否满足要求

**参数**:
- `-Fix`: 尝试自动修复环境问题

**使用示例**:
```powershell
# 基本检查
.\scripts\Check-Environment.ps1

# 检查并尝试自动修复
.\scripts\Check-Environment.ps1 -Fix
```

**检查项目**:
- PowerShell 版本 (≥5.0)
- Node.js (≥16.0)
- npm (≥8.0)
- Rust (≥1.70)
- Cargo
- Git (≥2.30)
- Visual Studio Build Tools (可选)
- 网络连接

### Install-Dependencies.ps1

**功能**: 自动安装缺失的开发工具

**参数**:
- `-Force`: 强制重新安装
- `-SkipOptional`: 跳过可选组件

**使用示例**:
```powershell
# 标准安装
.\scripts\Install-Dependencies.ps1

# 强制重新安装所有工具
.\scripts\Install-Dependencies.ps1 -Force

# 只安装必需工具
.\scripts\Install-Dependencies.ps1 -SkipOptional
```

**安装内容**:
- Node.js (通过 winget)
- Rust (通过 winget 或直接下载)
- Git (通过 winget)
- Visual Studio Build Tools (可选)
- Visual Studio Code (可选)

### Setup-MVP.ps1

**功能**: 创建 GitMentor MVP 项目结构

**参数**:
- `-ProjectName`: 项目名称 (默认: "GitMentor-MVP")

**使用示例**:
```powershell
# 使用默认名称
.\scripts\Setup-MVP.ps1

# 指定项目名称
.\scripts\Setup-MVP.ps1 -ProjectName "MyAwesomeGitTool"
```

**创建内容**:
- Tauri + Vue 3 项目结构
- Rust 依赖配置
- 前端依赖安装
- 基础代码文件
- 配置文件模板

### Build-MVP.ps1

**功能**: 构建和运行项目

**参数**:
- `-Mode`: 构建模式 ("dev" 或 "build")
- `-Clean`: 清理构建缓存

**使用示例**:
```powershell
# 开发模式
.\scripts\Build-MVP.ps1 -Mode dev

# 构建生产版本
.\scripts\Build-MVP.ps1 -Mode build

# 清理并重新构建
.\scripts\Build-MVP.ps1 -Mode build -Clean
```

## 🔧 PowerShell 模块使用

### 导入模块

```powershell
# 导入模块
Import-Module .\scripts\GitMentor-MVP.psm1

# 查看可用命令
Get-Command -Module GitMentor-MVP
```

### 主要命令

#### 环境管理
```powershell
# 检查环境
Test-GitMentorEnvironment

# 查看项目状态
Get-GitMentorStatus
```

#### 项目管理
```powershell
# 创建新项目
New-GitMentorProject -ProjectName "MyProject"

# 启动开发服务器
Start-GitMentorDev

# 启动开发服务器 (详细日志)
Start-GitMentorDev -Verbose

# 构建项目
Build-GitMentorProject

# 构建生产版本
Build-GitMentorProject -Release

# 清理并构建
Build-GitMentorProject -Release -Clean
```

#### 配置管理
```powershell
# 查看当前配置
Get-GitMentorConfig

# 设置 LLM 提供商
Set-GitMentorConfig -Key "provider" -Value "ollama" -Section "llm"

# 设置 API 密钥
Set-GitMentorConfig -Key "api_key" -Value "your-key" -Section "llm"

# 设置模型
Set-GitMentorConfig -Key "model" -Value "llama2" -Section "llm"
```

#### Ollama 管理
```powershell
# 启动 Ollama 服务
Start-OllamaService

# 安装模型
Install-OllamaModel -ModelName "llama2"
Install-OllamaModel -ModelName "codellama"
Install-OllamaModel -ModelName "mistral"
```

## 🐛 故障排除

### 常见错误和解决方案

#### 1. 执行策略错误
```
错误: 无法加载文件，因为在此系统上禁止运行脚本
```

**解决方案**:
```powershell
Set-ExecutionPolicy RemoteSigned -Scope CurrentUser
```

#### 2. 模块导入失败
```
错误: 找不到指定的模块
```

**解决方案**:
```powershell
# 使用完整路径导入
Import-Module "C:\full\path\to\scripts\GitMentor-MVP.psm1"

# 或先切换到项目目录
cd "C:\path\to\GitMentor"
Import-Module .\scripts\GitMentor-MVP.psm1
```

#### 3. winget 不可用
```
错误: 'winget' 不是内部或外部命令
```

**解决方案**:
1. 更新 Windows 到最新版本
2. 从 Microsoft Store 安装 "App Installer"
3. 或手动下载安装工具

#### 4. 权限不足
```
错误: 拒绝访问
```

**解决方案**:
```powershell
# 以管理员身份运行 PowerShell
Start-Process PowerShell -Verb RunAs
```

### 调试技巧

#### 启用详细输出
```powershell
# 设置详细输出
$VerbosePreference = "Continue"

# 运行脚本
.\scripts\Check-Environment.ps1 -Verbose
```

#### 查看错误详情
```powershell
# 查看最后一个错误
$Error[0] | Format-List * -Force

# 查看错误堆栈
$Error[0].ScriptStackTrace
```

#### 网络问题诊断
```powershell
# 测试网络连接
Test-NetConnection registry.npmjs.org -Port 443
Test-NetConnection crates.io -Port 443

# 配置代理 (如果需要)
$env:HTTP_PROXY = "http://proxy.company.com:8080"
$env:HTTPS_PROXY = "http://proxy.company.com:8080"
```

## 💡 最佳实践

### 1. 项目组织
```powershell
# 推荐的目录结构
C:\Development\
├── GitMentor\              # 主项目目录
│   ├── scripts\           # PowerShell 脚本
│   └── docs\             # 文档
└── Projects\              # 生成的项目
    ├── GitMentor-MVP\     # MVP 项目
    └── GitMentor-Full\    # 完整版项目
```

### 2. 环境隔离
```powershell
# 为不同项目使用不同的 Node.js 版本
# 安装 nvm-windows
winget install CoreyButler.NVMforWindows

# 使用特定版本
nvm install 18.17.0
nvm use 18.17.0
```

### 3. 性能优化
```powershell
# 并行构建
$env:CARGO_BUILD_JOBS = "4"  # 根据 CPU 核心数调整

# 启用增量编译
$env:CARGO_INCREMENTAL = "1"

# 使用本地缓存
npm config set cache "C:\npm-cache"
```

### 4. 安全考虑
```powershell
# 不要在脚本中硬编码 API 密钥
# 使用环境变量
$env:OPENAI_API_KEY = "your-api-key"

# 或使用 Windows 凭据管理器
cmdkey /add:openai /user:api /pass:your-api-key
```

## 🎯 快速参考

### 常用命令组合

**完整的项目创建流程**:
```powershell
Import-Module .\scripts\GitMentor-MVP.psm1
Test-GitMentorEnvironment
New-GitMentorProject -ProjectName "MyProject"
cd MyProject
Start-GitMentorDev
```

**配置 Ollama**:
```powershell
Start-OllamaService
Install-OllamaModel -ModelName "llama2"
Set-GitMentorConfig -Key "provider" -Value "ollama"
Set-GitMentorConfig -Key "model" -Value "llama2"
```

**构建发布版本**:
```powershell
Build-GitMentorProject -Release -Clean
```

这些 PowerShell 脚本将大大简化 Windows 用户的开发体验！
