# GitMentor MVP - AI提交消息生成器

一个基于Tauri + Vue 3 + Rust的最小可行版本，专注于使用AI生成Git提交消息。

## 🎯 核心功能

- **📁 文件夹选择器** - 选择Git仓库目录
- **📊 Git状态检测** - 显示文件变更状态（修改/新增/删除/未跟踪）
- **☑️ 文件选择界面** - 选择要提交的文件
- **🤖 AI生成按钮** - 一键生成提交消息
- **🔌 LLM服务集成** - 支持Ollama和OpenAI兼容API
- **📝 可配置提示词** - 外部配置文件管理模板

## 🛠️ 技术栈

- **前端**: Vue 3 + TypeScript + Element Plus + Tauri
- **后端**: Rust + git2 + reqwest + handlebars
- **AI服务**: Ollama (本地) / OpenAI兼容API (远程)

## 🚀 快速开始

### 环境要求

- Windows 10/11
- PowerShell 5.0+
- Node.js 16+
- Rust 1.70+
- Git 2.30+

### 方法1: 自动化安装 (推荐)

**步骤1: 检查环境**
```powershell
# 以管理员身份运行 PowerShell
.\scripts\Check-Environment.ps1
```

**步骤2: 安装依赖 (如果需要)**
```powershell
# 自动安装缺失的工具
.\scripts\Install-Dependencies.ps1
```

**步骤3: 创建项目**
```powershell
# 创建 GitMentor MVP 项目
.\scripts\Setup-MVP.ps1 -ProjectName "GitMentor-MVP"
```

### 方法2: 手动创建

```powershell
# 1. 创建Tauri项目
npm create tauri-app@latest GitMentor-MVP --template vue-ts
cd GitMentor-MVP

# 2. 安装前端依赖
npm install element-plus "@element-plus/icons-vue" pinia

# 3. 配置Rust依赖 (参考 docs/mvp-implementation-guide.md)

# 4. 实现核心代码 (参考实现指南)
```

## 📖 实现指南

详细的实现步骤和代码示例请参考：
- [MVP实现指南](docs/mvp-implementation-guide.md)

## ⚙️ 配置说明

### LLM服务配置

编辑 `config/app.toml`:

```toml
[llm]
provider = "ollama"  # "ollama" 或 "openai"
base_url = "http://localhost:11434"
model = "llama2"
api_key = ""  # OpenAI API密钥（如果使用OpenAI）
timeout_seconds = 30
```

### 提示词模板配置

编辑 `config/prompts.toml` 来自定义提示词模板。

## 🎮 使用方法

1. **启动应用**
   ```powershell
   # 使用构建脚本 (推荐)
   .\scripts\Build-MVP.ps1 -Mode dev

   # 或直接使用 npm
   npm run tauri:dev
   ```

2. **选择Git仓库**
   - 点击"选择仓库文件夹"按钮
   - 选择一个包含Git仓库的目录

3. **选择要提交的文件**
   - 查看检测到的文件变更
   - 勾选要包含在本次提交中的文件

4. **生成提交消息**
   - 点击"生成提交消息"按钮
   - 等待AI分析并生成提交消息

5. **使用生成的消息**
   - 复制生成的提交消息
   - 在你的Git客户端中使用

## 🔧 开发命令

### PowerShell 脚本 (推荐)

```powershell
# 环境检查
.\scripts\Check-Environment.ps1

# 安装依赖
.\scripts\Install-Dependencies.ps1

# 开发模式
.\scripts\Build-MVP.ps1 -Mode dev

# 构建生产版本
.\scripts\Build-MVP.ps1 -Mode build

# 清理并重新构建
.\scripts\Build-MVP.ps1 -Mode build -Clean
```

### 传统 npm 命令

```powershell
# 开发模式
npm run tauri:dev

# 构建生产版本
npm run tauri:build

# 仅构建前端
npm run build
```

## 📁 项目结构

```
GitMentor-MVP/
├── src-tauri/              # Rust后端
│   ├── src/
│   │   ├── commands/       # Tauri命令
│   │   ├── core/          # 核心功能模块
│   │   └── types/         # 类型定义
│   └── Cargo.toml
├── src/                   # Vue前端
│   ├── components/        # Vue组件
│   ├── stores/           # Pinia状态管理
│   └── types/            # TypeScript类型
├── config/               # 配置文件
│   ├── app.toml         # 应用配置
│   └── prompts.toml     # 提示词模板
└── docs/                # 文档
```

## 🤖 支持的AI服务

### Ollama (推荐)

1. 安装Ollama: https://ollama.ai/
2. 下载模型: `ollama pull llama2`
3. 启动服务: `ollama serve`
4. 配置base_url为: `http://localhost:11434`

### OpenAI兼容API

支持任何OpenAI兼容的API服务：
- OpenAI官方API
- Azure OpenAI
- 其他兼容服务

配置示例：
```toml
[llm]
provider = "openai"
base_url = "https://api.openai.com"
model = "gpt-3.5-turbo"
api_key = "your-api-key-here"
```

## 🐛 故障排除

### 常见问题

1. **Rust编译错误**
   - 确保Rust版本 >= 1.70
   - 运行 `rustup update`

2. **Git操作失败**
   - 确保选择的目录是有效的Git仓库
   - 检查Git仓库是否有待提交的变更

3. **AI服务连接失败**
   - 检查Ollama服务是否运行
   - 验证API密钥和base_url配置

4. **前端构建错误**
   - 删除 `node_modules` 重新安装
   - 确保Node.js版本 >= 16

### 调试模式

启用详细日志：
```powershell
# 设置环境变量并启动
$env:RUST_LOG="debug"
npm run tauri:dev

# 或使用构建脚本
$env:RUST_LOG="debug"
.\scripts\Build-MVP.ps1 -Mode dev
```

## 🔮 后续扩展

这个MVP可以扩展为完整的GitMentor系统：

- 数据库存储和历史记录
- 双重审核Agent系统
- 更丰富的Git操作
- 团队协作功能
- 高级分析和报告

## 📄 许可证

MIT License

## 🤝 贡献

欢迎提交Issue和Pull Request！

---

**开始你的AI驱动的Git工作流程吧！** 🚀
