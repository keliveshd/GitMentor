# GitMentor MVP 快速搭建脚本 (PowerShell版本)
# 使用方法: .\Setup-MVP.ps1 [-ProjectName "GitMentor-MVP"]

param(
    [string]$ProjectName = "GitMentor-MVP"
)

# 设置错误处理
$ErrorActionPreference = "Stop"

Write-Host "🚀 开始创建 GitMentor MVP 项目: $ProjectName" -ForegroundColor Green

# 检查必需的工具
Write-Host "📋 检查环境依赖..." -ForegroundColor Yellow

function Test-Command {
    param([string]$Command)
    try {
        Get-Command $Command -ErrorAction Stop | Out-Null
        return $true
    }
    catch {
        return $false
    }
}

if (-not (Test-Command "node")) {
    Write-Host "❌ Node.js 未安装，请先安装 Node.js 16+" -ForegroundColor Red
    Write-Host "   下载地址: https://nodejs.org/" -ForegroundColor Yellow
    Write-Host "   或使用 winget: winget install OpenJS.NodeJS" -ForegroundColor Yellow
    exit 1
}

if (-not (Test-Command "npm")) {
    Write-Host "❌ npm 未安装" -ForegroundColor Red
    exit 1
}

if (-not (Test-Command "rustc")) {
    Write-Host "❌ Rust 未安装，请先安装 Rust" -ForegroundColor Red
    Write-Host "   安装命令: winget install Rustlang.Rustup" -ForegroundColor Yellow
    Write-Host "   或访问: https://rustup.rs/" -ForegroundColor Yellow
    exit 1
}

if (-not (Test-Command "cargo")) {
    Write-Host "❌ Cargo 未安装" -ForegroundColor Red
    exit 1
}

Write-Host "✅ 环境检查通过" -ForegroundColor Green

# 创建项目
Write-Host "📁 创建 Tauri 项目..." -ForegroundColor Yellow
try {
    npm create tauri-app@latest $ProjectName --template vue-ts --yes
    if ($LASTEXITCODE -ne 0) {
        throw "npm create 命令失败"
    }
}
catch {
    Write-Host "❌ 创建 Tauri 项目失败: $_" -ForegroundColor Red
    exit 1
}

Set-Location $ProjectName

# 安装前端依赖
Write-Host "📦 安装前端依赖..." -ForegroundColor Yellow
try {
    npm install element-plus "@element-plus/icons-vue" pinia
    if ($LASTEXITCODE -ne 0) {
        throw "npm install 命令失败"
    }
}
catch {
    Write-Host "❌ 安装前端依赖失败: $_" -ForegroundColor Red
    exit 1
}

# 配置 Rust 依赖
Write-Host "🦀 配置 Rust 依赖..." -ForegroundColor Yellow

$cargoToml = @"
[package]
name = "gitmentor-mvp"
version = "0.1.0"
edition = "2021"

[build-dependencies]
tauri-build = { version = "2.0", features = [] }

[dependencies]
tauri = { version = "2.0", features = ["shell-open", "dialog-open"] }
git2 = "0.18"
reqwest = { version = "0.11", features = ["json"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.0", features = ["full"] }
anyhow = "1.0"
handlebars = "4.5"
toml = "0.8"
async-trait = "0.1"
dirs = "5.0"
"@

$cargoToml | Out-File -FilePath "src-tauri\Cargo.toml" -Encoding UTF8

# 创建目录结构
Write-Host "📂 创建项目结构..." -ForegroundColor Yellow

$directories = @(
    "src-tauri\src\commands",
    "src-tauri\src\core", 
    "src-tauri\src\types",
    "src\components",
    "src\stores",
    "src\types",
    "config"
)

foreach ($dir in $directories) {
    New-Item -ItemType Directory -Path $dir -Force | Out-Null
}

# 创建 Rust 模块文件
Write-Host "📝 创建 Rust 代码文件..." -ForegroundColor Yellow

# types/mod.rs
"pub mod git_types;" | Out-File -FilePath "src-tauri\src\types\mod.rs" -Encoding UTF8

# types/git_types.rs
$gitTypesRs = @"
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileStatus {
    pub path: String,
    pub status: String,
    pub selected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitStatusResult {
    pub files: Vec<FileStatus>,
    pub branch: String,
    pub has_changes: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitRequest {
    pub selected_files: Vec<String>,
    pub additional_context: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitMessageResult {
    pub message: String,
    pub confidence: f32,
    pub processing_time_ms: u64,
}
"@

$gitTypesRs | Out-File -FilePath "src-tauri\src\types\git_types.rs" -Encoding UTF8

# core/mod.rs
$coreModRs = @"
pub mod git_engine;
pub mod llm_client;
pub mod template_engine;
"@

$coreModRs | Out-File -FilePath "src-tauri\src\core\mod.rs" -Encoding UTF8

# commands/mod.rs
$commandsModRs = @"
pub mod git_commands;
pub mod config_commands;
"@

$commandsModRs | Out-File -FilePath "src-tauri\src\commands\mod.rs" -Encoding UTF8

# 创建配置文件
Write-Host "⚙️ 创建配置文件..." -ForegroundColor Yellow

$appToml = @"
[llm]
provider = "ollama"
base_url = "http://localhost:11434"
model = "llama2"
api_key = ""
timeout_seconds = 30

[template]
template_file = ""
"@

$appToml | Out-File -FilePath "config\app.toml" -Encoding UTF8

$promptsToml = @"
[commit_message_template]
content = """
请根据以下Git变更信息生成一个简洁、清晰的提交消息：

分支: {{branch}}
变更文件数量: {{file_count}}

文件变更详情:
{{#each files}}
- {{this.status}}: {{this.path}}
{{/each}}

{{#if diff_summary}}
变更摘要:
{{diff_summary}}
{{/if}}

请生成一个符合以下格式的提交消息：
- 第一行：简洁的标题（不超过50字符）
- 空行  
- 详细描述（如果需要）

提交消息应该：
1. 使用动词开头（如：Add, Fix, Update, Remove等）
2. 简洁明了地描述变更内容
3. 如果是bug修复，说明修复了什么问题
4. 如果是新功能，说明添加了什么功能
5. 使用中文描述

示例格式：
Add: 新增用户登录功能

实现了基于JWT的用户认证系统，包括登录、注册和密码重置功能。
添加了用户会话管理和权限验证中间件。
"""
"@

$promptsToml | Out-File -FilePath "config\prompts.toml" -Encoding UTF8

# 创建 TypeScript 类型文件
Write-Host "📝 创建前端类型文件..." -ForegroundColor Yellow

$gitTs = @"
export interface FileStatus {
  path: string
  status: string
  selected: boolean
}

export interface GitStatusResult {
  files: FileStatus[]
  branch: string
  has_changes: boolean
}

export interface CommitRequest {
  selected_files: string[]
  additional_context?: string
}

export interface CommitMessageResult {
  message: string
  confidence: number
  processing_time_ms: number
}
"@

$gitTs | Out-File -FilePath "src\types\git.ts" -Encoding UTF8

# 更新 package.json
Write-Host "📦 更新 package.json..." -ForegroundColor Yellow
npm pkg set scripts.tauri:dev="tauri dev"
npm pkg set scripts.tauri:build="tauri build"

Write-Host "✅ GitMentor MVP 项目创建完成！" -ForegroundColor Green
Write-Host ""
Write-Host "📋 下一步操作：" -ForegroundColor Cyan
Write-Host "1. cd $ProjectName" -ForegroundColor White
Write-Host "2. 根据文档完成剩余的代码实现" -ForegroundColor White
Write-Host "3. npm run tauri:dev  # 启动开发服务器" -ForegroundColor White
Write-Host ""
Write-Host "📚 详细实现指南请参考: docs/mvp-implementation-guide.md" -ForegroundColor Cyan
Write-Host ""
Write-Host "🎉 开始你的 GitMentor MVP 开发之旅吧！" -ForegroundColor Green
