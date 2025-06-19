# GitMentor MVP Project Setup Script (PowerShell)
# Usage: .\Setup-MVP-Simple.ps1 [-ProjectName "GitMentor-MVP"]

param(
    [string]$ProjectName = "GitMentor-MVP"
)

$ErrorActionPreference = "Stop"

Write-Host "Creating GitMentor MVP project: $ProjectName" -ForegroundColor Green

# Check required tools
Write-Host "Checking environment dependencies..." -ForegroundColor Yellow

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
    Write-Host "ERROR: Node.js not installed, please install Node.js 16+" -ForegroundColor Red
    Write-Host "   Download: https://nodejs.org/" -ForegroundColor Yellow
    Write-Host "   Or use: winget install OpenJS.NodeJS" -ForegroundColor Yellow
    exit 1
}

if (-not (Test-Command "npm")) {
    Write-Host "ERROR: npm not installed" -ForegroundColor Red
    exit 1
}

if (-not (Test-Command "rustc")) {
    Write-Host "ERROR: Rust not installed, please install Rust" -ForegroundColor Red
    Write-Host "   Install command: winget install Rustlang.Rustup" -ForegroundColor Yellow
    Write-Host "   Or visit: https://rustup.rs/" -ForegroundColor Yellow
    exit 1
}

if (-not (Test-Command "cargo")) {
    Write-Host "ERROR: Cargo not installed" -ForegroundColor Red
    exit 1
}

Write-Host "OK: Environment check passed" -ForegroundColor Green

# Create project
Write-Host "Creating Tauri project..." -ForegroundColor Yellow
try {
    npm create tauri-app@latest $ProjectName --template vue-ts --yes
    if ($LASTEXITCODE -ne 0) {
        throw "npm create command failed"
    }
}
catch {
    Write-Host "ERROR: Failed to create Tauri project: $_" -ForegroundColor Red
    exit 1
}

Set-Location $ProjectName

# Install frontend dependencies
Write-Host "Installing frontend dependencies..." -ForegroundColor Yellow
try {
    npm install element-plus "@element-plus/icons-vue" pinia
    if ($LASTEXITCODE -ne 0) {
        throw "npm install command failed"
    }
}
catch {
    Write-Host "ERROR: Failed to install frontend dependencies: $_" -ForegroundColor Red
    exit 1
}

# Configure Rust dependencies
Write-Host "Configuring Rust dependencies..." -ForegroundColor Yellow

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

# Create directory structure
Write-Host "Creating project structure..." -ForegroundColor Yellow

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

# Create Rust module files
Write-Host "Creating Rust code files..." -ForegroundColor Yellow

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

# Create configuration files
Write-Host "Creating configuration files..." -ForegroundColor Yellow

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
Please generate a concise and clear commit message based on the following Git change information:

Branch: {{branch}}
Number of changed files: {{file_count}}

File change details:
{{#each files}}
- {{this.status}}: {{this.path}}
{{/each}}

{{#if diff_summary}}
Change summary:
{{diff_summary}}
{{/if}}

Please generate a commit message in the following format:
- First line: concise title (no more than 50 characters)
- Blank line
- Detailed description (if needed)

The commit message should:
1. Start with a verb (such as: Add, Fix, Update, Remove, etc.)
2. Concisely and clearly describe the change content
3. If it's a bug fix, explain what problem was fixed
4. If it's a new feature, explain what functionality was added

Example format:
Add: New user login functionality

Implemented JWT-based user authentication system, including login, registration and password reset functionality.
Added user session management and permission verification middleware.
"""
"@

$promptsToml | Out-File -FilePath "config\prompts.toml" -Encoding UTF8

# Create TypeScript type files
Write-Host "Creating frontend type files..." -ForegroundColor Yellow

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

# Update package.json
Write-Host "Updating package.json..." -ForegroundColor Yellow
npm pkg set scripts.tauri:dev="tauri dev"
npm pkg set scripts.tauri:build="tauri build"

Write-Host "OK: GitMentor MVP project created successfully!" -ForegroundColor Green
Write-Host ""
Write-Host "Next steps:" -ForegroundColor Cyan
Write-Host "1. cd $ProjectName" -ForegroundColor White
Write-Host "2. Complete the remaining code implementation according to the documentation" -ForegroundColor White
Write-Host "3. npm run tauri:dev  # Start development server" -ForegroundColor White
Write-Host ""
Write-Host "Detailed implementation guide: docs/mvp-implementation-guide.md" -ForegroundColor Cyan
Write-Host ""
Write-Host "Start your GitMentor MVP development journey!" -ForegroundColor Green
