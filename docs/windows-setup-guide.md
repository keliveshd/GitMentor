# GitMentor MVP - Windows 安装指南

本指南专门为Windows用户提供详细的安装和配置步骤。

## 🎯 系统要求

- **操作系统**: Windows 10 (版本 1903+) 或 Windows 11
- **PowerShell**: 5.0+ (Windows 10/11 内置)
- **内存**: 至少 4GB RAM (推荐 8GB+)
- **存储**: 至少 2GB 可用空间
- **网络**: 稳定的互联网连接

## 🚀 快速开始 (3步完成)

### 步骤1: 准备PowerShell环境

1. **以管理员身份运行PowerShell**
   - 按 `Win + X`，选择 "Windows PowerShell (管理员)"
   - 或搜索 "PowerShell"，右键选择 "以管理员身份运行"

2. **设置执行策略** (如果需要)
   ```powershell
   # 检查当前执行策略
   Get-ExecutionPolicy
   
   # 如果显示 "Restricted"，需要更改为 "RemoteSigned"
   Set-ExecutionPolicy RemoteSigned -Scope CurrentUser
   ```

3. **导航到项目目录**
   ```powershell
   # 假设你已经下载了 GitMentor 项目
   cd "C:\path\to\GitMentor"
   ```

### 步骤2: 自动环境检查和安装

```powershell
# 检查环境 (必需)
.\scripts\Check-Environment.ps1

# 如果有缺失的工具，自动安装
.\scripts\Install-Dependencies.ps1

# 重启 PowerShell 以刷新环境变量
```

### 步骤3: 创建和运行项目

```powershell
# 创建 MVP 项目
.\scripts\Setup-MVP.ps1 -ProjectName "GitMentor-MVP"

# 进入项目目录
cd GitMentor-MVP

# 启动开发服务器
..\scripts\Build-MVP.ps1 -Mode dev
```

## 📋 详细安装步骤

### 手动安装依赖 (如果自动安装失败)

#### 1. 安装 Node.js

**方法1: 使用 winget (推荐)**
```powershell
winget install OpenJS.NodeJS
```

**方法2: 手动下载**
1. 访问 https://nodejs.org/
2. 下载 LTS 版本 (推荐 18.x 或更高)
3. 运行安装程序，保持默认设置
4. 重启 PowerShell

**验证安装:**
```powershell
node --version
npm --version
```

#### 2. 安装 Rust

**方法1: 使用 winget (推荐)**
```powershell
winget install Rustlang.Rustup
```

**方法2: 手动安装**
1. 访问 https://rustup.rs/
2. 下载 `rustup-init.exe`
3. 运行安装程序，选择默认安装
4. 重启 PowerShell

**验证安装:**
```powershell
rustc --version
cargo --version
```

**配置 Rust 目标:**
```powershell
rustup target add x86_64-pc-windows-msvc
```

#### 3. 安装 Git

**方法1: 使用 winget (推荐)**
```powershell
winget install Git.Git
```

**方法2: 手动下载**
1. 访问 https://git-scm.com/
2. 下载 Windows 版本
3. 运行安装程序，推荐设置：
   - 选择 "Git from the command line and also from 3rd-party software"
   - 选择 "Use Windows' default console window"

**验证安装:**
```powershell
git --version
```

#### 4. 安装 Visual Studio Build Tools (可选但推荐)

**方法1: 使用 winget**
```powershell
winget install Microsoft.VisualStudio.2022.BuildTools
```

**方法2: 手动安装**
1. 访问 https://visualstudio.microsoft.com/visual-cpp-build-tools/
2. 下载 "Build Tools for Visual Studio 2022"
3. 运行安装程序，选择：
   - "C++ build tools"
   - "Windows 10/11 SDK"

## 🔧 项目配置

### 配置 LLM 服务

#### 选项1: 使用 Ollama (本地AI，推荐)

1. **安装 Ollama**
   ```powershell
   winget install Ollama.Ollama
   ```

2. **下载模型**
   ```powershell
   ollama pull llama2
   # 或其他模型: ollama pull codellama, ollama pull mistral
   ```

3. **启动服务**
   ```powershell
   ollama serve
   ```

4. **配置应用** (编辑 `config/app.toml`)
   ```toml
   [llm]
   provider = "ollama"
   base_url = "http://localhost:11434"
   model = "llama2"
   api_key = ""
   timeout_seconds = 30
   ```

#### 选项2: 使用 OpenAI API

1. **获取 API 密钥**
   - 访问 https://platform.openai.com/
   - 创建账户并获取 API 密钥

2. **配置应用** (编辑 `config/app.toml`)
   ```toml
   [llm]
   provider = "openai"
   base_url = "https://api.openai.com"
   model = "gpt-3.5-turbo"
   api_key = "your-api-key-here"
   timeout_seconds = 30
   ```

### 自定义提示词模板

编辑 `config/prompts.toml` 文件来自定义AI提示词：

```toml
[commit_message_template]
content = """
你是一个专业的Git提交消息生成助手。请根据以下信息生成简洁、清晰的中文提交消息：

分支: {{branch}}
变更文件数量: {{file_count}}

文件变更详情:
{{#each files}}
- {{this.status}}: {{this.path}}
{{/each}}

要求：
1. 第一行不超过50个字符
2. 使用动词开头 (新增/修复/更新/删除/重构等)
3. 简洁明了地描述变更内容
4. 使用中文
"""
```

## 🎮 使用流程

### 开发模式

```powershell
# 启动开发服务器
.\scripts\Build-MVP.ps1 -Mode dev

# 或使用传统方式
npm run tauri:dev
```

### 构建生产版本

```powershell
# 构建可执行文件
.\scripts\Build-MVP.ps1 -Mode build

# 构建产物位置:
# - 可执行文件: src-tauri\target\release\gitmentor-mvp.exe
# - MSI安装包: src-tauri\target\release\bundle\msi\*.msi
```

### 清理和重新构建

```powershell
# 清理所有构建缓存并重新构建
.\scripts\Build-MVP.ps1 -Mode build -Clean
```

## 🐛 常见问题解决

### 1. PowerShell 执行策略错误

**错误信息**: "无法加载文件，因为在此系统上禁止运行脚本"

**解决方案**:
```powershell
Set-ExecutionPolicy RemoteSigned -Scope CurrentUser
```

### 2. Rust 编译错误

**错误信息**: "linker 'link.exe' not found"

**解决方案**:
1. 安装 Visual Studio Build Tools
2. 或安装完整的 Visual Studio Community

### 3. Node.js 版本过低

**错误信息**: "Node.js version 14.x is not supported"

**解决方案**:
```powershell
# 卸载旧版本
winget uninstall OpenJS.NodeJS

# 安装最新版本
winget install OpenJS.NodeJS
```

### 4. 网络连接问题

**症状**: npm 或 cargo 下载失败

**解决方案**:
```powershell
# 配置 npm 镜像 (中国用户)
npm config set registry https://registry.npmmirror.com/

# 配置 Rust 镜像
$env:RUSTUP_DIST_SERVER="https://rsproxy.cn"
$env:RUSTUP_UPDATE_ROOT="https://rsproxy.cn/rustup"
```

### 5. 防火墙或杀毒软件阻止

**症状**: 编译或运行时被阻止

**解决方案**:
1. 将项目目录添加到杀毒软件白名单
2. 临时关闭实时保护进行构建
3. 配置防火墙允许 Tauri 应用

## 📊 性能优化建议

### 开发环境优化

1. **使用 SSD 存储项目**
2. **增加虚拟内存** (如果物理内存不足)
3. **关闭不必要的后台程序**
4. **使用 Windows Terminal** 替代传统 PowerShell

### 构建优化

```powershell
# 并行构建 (利用多核CPU)
$env:CARGO_BUILD_JOBS="4"  # 根据CPU核心数调整

# 启用增量编译
$env:CARGO_INCREMENTAL="1"
```

## 🎉 完成！

现在你已经成功在 Windows 上设置了 GitMentor MVP 开发环境！

**下一步**:
1. 阅读 [MVP实现指南](mvp-implementation-guide.md) 了解代码实现
2. 开始开发你的第一个功能
3. 测试 AI 提交消息生成功能

**获取帮助**:
- 查看项目文档
- 检查 GitHub Issues
- 参考 Tauri 官方文档
