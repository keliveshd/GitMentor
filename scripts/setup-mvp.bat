@echo off
setlocal enabledelayedexpansion

REM GitMentor MVP 快速搭建脚本 (Windows版本)
REM 使用方法: setup-mvp.bat [项目名称]

set PROJECT_NAME=%1
if "%PROJECT_NAME%"=="" set PROJECT_NAME=GitMentor-MVP

echo 🚀 开始创建 GitMentor MVP 项目: %PROJECT_NAME%

REM 检查必需的工具
echo 📋 检查环境依赖...

where node >nul 2>nul
if %errorlevel% neq 0 (
    echo ❌ Node.js 未安装，请先安装 Node.js 16+
    echo    下载地址: https://nodejs.org/
    pause
    exit /b 1
)

where npm >nul 2>nul
if %errorlevel% neq 0 (
    echo ❌ npm 未安装
    pause
    exit /b 1
)

where rustc >nul 2>nul
if %errorlevel% neq 0 (
    echo ❌ Rust 未安装，请先安装 Rust
    echo    安装命令: winget install Rustlang.Rustup
    echo    或访问: https://rustup.rs/
    pause
    exit /b 1
)

where cargo >nul 2>nul
if %errorlevel% neq 0 (
    echo ❌ Cargo 未安装
    pause
    exit /b 1
)

echo ✅ 环境检查通过

REM 创建项目
echo 📁 创建 Tauri 项目...
call npm create tauri-app@latest "%PROJECT_NAME%" --template vue-ts --yes

cd "%PROJECT_NAME%"

REM 安装前端依赖
echo 📦 安装前端依赖...
call npm install element-plus @element-plus/icons-vue pinia

REM 配置 Rust 依赖
echo 🦀 配置 Rust 依赖...
(
echo [package]
echo name = "gitmentor-mvp"
echo version = "0.1.0"
echo edition = "2021"
echo.
echo [build-dependencies]
echo tauri-build = { version = "2.0", features = [] }
echo.
echo [dependencies]
echo tauri = { version = "2.0", features = ["shell-open", "dialog-open"] }
echo git2 = "0.18"
echo reqwest = { version = "0.11", features = ["json"] }
echo serde = { version = "1.0", features = ["derive"] }
echo serde_json = "1.0"
echo tokio = { version = "1.0", features = ["full"] }
echo anyhow = "1.0"
echo handlebars = "4.5"
echo toml = "0.8"
echo async-trait = "0.1"
echo dirs = "5.0"
) > src-tauri\Cargo.toml

REM 创建目录结构
echo 📂 创建项目结构...
mkdir src-tauri\src\commands 2>nul
mkdir src-tauri\src\core 2>nul
mkdir src-tauri\src\types 2>nul
mkdir src\components 2>nul
mkdir src\stores 2>nul
mkdir src\types 2>nul
mkdir config 2>nul

REM 创建 Rust 模块文件
echo 📝 创建 Rust 代码文件...

REM types/mod.rs
echo pub mod git_types; > src-tauri\src\types\mod.rs

REM types/git_types.rs
(
echo use serde::{Deserialize, Serialize};
echo.
echo #[derive^(Debug, Clone, Serialize, Deserialize^)]
echo pub struct FileStatus {
echo     pub path: String,
echo     pub status: String,
echo     pub selected: bool,
echo }
echo.
echo #[derive^(Debug, Clone, Serialize, Deserialize^)]
echo pub struct GitStatusResult {
echo     pub files: Vec^<FileStatus^>,
echo     pub branch: String,
echo     pub has_changes: bool,
echo }
echo.
echo #[derive^(Debug, Clone, Serialize, Deserialize^)]
echo pub struct CommitRequest {
echo     pub selected_files: Vec^<String^>,
echo     pub additional_context: Option^<String^>,
echo }
echo.
echo #[derive^(Debug, Clone, Serialize, Deserialize^)]
echo pub struct CommitMessageResult {
echo     pub message: String,
echo     pub confidence: f32,
echo     pub processing_time_ms: u64,
echo }
) > src-tauri\src\types\git_types.rs

REM core/mod.rs
(
echo pub mod git_engine;
echo pub mod llm_client;
echo pub mod template_engine;
) > src-tauri\src\core\mod.rs

REM commands/mod.rs
(
echo pub mod git_commands;
echo pub mod config_commands;
) > src-tauri\src\commands\mod.rs

REM 创建配置文件
echo ⚙️ 创建配置文件...

(
echo [llm]
echo provider = "ollama"
echo base_url = "http://localhost:11434"
echo model = "llama2"
echo api_key = ""
echo timeout_seconds = 30
echo.
echo [template]
echo template_file = ""
) > config\app.toml

REM 创建 TypeScript 类型文件
echo 📝 创建前端类型文件...

(
echo export interface FileStatus {
echo   path: string
echo   status: string
echo   selected: boolean
echo }
echo.
echo export interface GitStatusResult {
echo   files: FileStatus[]
echo   branch: string
echo   has_changes: boolean
echo }
echo.
echo export interface CommitRequest {
echo   selected_files: string[]
echo   additional_context?: string
echo }
echo.
echo export interface CommitMessageResult {
echo   message: string
echo   confidence: number
echo   processing_time_ms: number
echo }
) > src\types\git.ts

REM 更新 package.json 脚本
echo 📦 更新 package.json...
call npm pkg set scripts.tauri:dev="tauri dev"
call npm pkg set scripts.tauri:build="tauri build"

echo ✅ GitMentor MVP 项目创建完成！
echo.
echo 📋 下一步操作：
echo 1. cd %PROJECT_NAME%
echo 2. 根据文档完成剩余的代码实现
echo 3. npm run tauri:dev  # 启动开发服务器
echo.
echo 📚 详细实现指南请参考: docs/mvp-implementation-guide.md
echo.
echo 🎉 开始你的 GitMentor MVP 开发之旅吧！
pause
