#!/bin/bash

# GitMentor MVP 快速搭建脚本
# 使用方法: ./setup-mvp.sh [项目名称]

set -e

PROJECT_NAME=${1:-"GitMentor-MVP"}
echo "🚀 开始创建 GitMentor MVP 项目: $PROJECT_NAME"

# 检查必需的工具
echo "📋 检查环境依赖..."

if ! command -v node &> /dev/null; then
    echo "❌ Node.js 未安装，请先安装 Node.js 16+"
    exit 1
fi

if ! command -v npm &> /dev/null; then
    echo "❌ npm 未安装"
    exit 1
fi

if ! command -v rustc &> /dev/null; then
    echo "❌ Rust 未安装，请先安装 Rust"
    echo "   安装命令: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

if ! command -v cargo &> /dev/null; then
    echo "❌ Cargo 未安装"
    exit 1
fi

echo "✅ 环境检查通过"

# 创建项目
echo "📁 创建 Tauri 项目..."
npm create tauri-app@latest "$PROJECT_NAME" --template vue-ts --yes

cd "$PROJECT_NAME"

# 安装前端依赖
echo "📦 安装前端依赖..."
npm install element-plus @element-plus/icons-vue pinia

# 配置 Rust 依赖
echo "🦀 配置 Rust 依赖..."
cat > src-tauri/Cargo.toml << 'EOF'
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
EOF

# 创建目录结构
echo "📂 创建项目结构..."
mkdir -p src-tauri/src/{commands,core,types}
mkdir -p src/components
mkdir -p src/stores
mkdir -p src/types
mkdir -p config

# 创建 Rust 模块文件
echo "📝 创建 Rust 代码文件..."

# types/mod.rs
cat > src-tauri/src/types/mod.rs << 'EOF'
pub mod git_types;
EOF

# types/git_types.rs
cat > src-tauri/src/types/git_types.rs << 'EOF'
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
EOF

# core/mod.rs
cat > src-tauri/src/core/mod.rs << 'EOF'
pub mod git_engine;
pub mod llm_client;
pub mod template_engine;
EOF

# commands/mod.rs
cat > src-tauri/src/commands/mod.rs << 'EOF'
pub mod git_commands;
pub mod config_commands;
EOF

# 创建配置文件
echo "⚙️ 创建配置文件..."

cat > config/app.toml << 'EOF'
[llm]
provider = "ollama"
base_url = "http://localhost:11434"
model = "llama2"
api_key = ""
timeout_seconds = 30

[template]
template_file = ""
EOF

cat > config/prompts.toml << 'EOF'
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
EOF

# 创建 TypeScript 类型文件
echo "📝 创建前端类型文件..."

cat > src/types/git.ts << 'EOF'
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
EOF

# 更新 package.json
echo "📦 更新 package.json..."
npm pkg set scripts.tauri:dev="tauri dev"
npm pkg set scripts.tauri:build="tauri build"

echo "✅ GitMentor MVP 项目创建完成！"
echo ""
echo "📋 下一步操作："
echo "1. cd $PROJECT_NAME"
echo "2. 根据文档完成剩余的代码实现"
echo "3. npm run tauri:dev  # 启动开发服务器"
echo ""
echo "📚 详细实现指南请参考: docs/mvp-implementation-guide.md"
echo ""
echo "🎉 开始你的 GitMentor MVP 开发之旅吧！"
