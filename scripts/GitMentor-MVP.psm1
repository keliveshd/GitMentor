# GitMentor MVP PowerShell 模块
# 提供便捷的项目管理命令

# 全局变量
$script:ProjectRoot = $null
$script:ConfigPath = $null

# 初始化模块
function Initialize-GitMentorMVP {
    [CmdletBinding()]
    param()
    
    $script:ProjectRoot = Get-Location
    $script:ConfigPath = Join-Path $script:ProjectRoot "config"
    
    Write-Host "🚀 GitMentor MVP 模块已加载" -ForegroundColor Green
    Write-Host "   项目根目录: $script:ProjectRoot" -ForegroundColor Cyan
}

# 检查环境
function Test-GitMentorEnvironment {
    [CmdletBinding()]
    param()
    
    Write-Host "🔍 检查 GitMentor MVP 环境..." -ForegroundColor Yellow
    
    $checks = @(
        @{ Name = "Node.js"; Command = "node"; Args = "--version"; MinVersion = "16.0.0" },
        @{ Name = "npm"; Command = "npm"; Args = "--version"; MinVersion = "8.0.0" },
        @{ Name = "Rust"; Command = "rustc"; Args = "--version"; MinVersion = "1.70.0" },
        @{ Name = "Cargo"; Command = "cargo"; Args = "--version"; MinVersion = "1.70.0" },
        @{ Name = "Git"; Command = "git"; Args = "--version"; MinVersion = "2.30.0" }
    )
    
    $allPassed = $true
    
    foreach ($check in $checks) {
        try {
            $output = & $check.Command $check.Args 2>$null
            if ($output) {
                Write-Host "   ✅ $($check.Name): $output" -ForegroundColor Green
            } else {
                Write-Host "   ❌ $($check.Name): 未安装" -ForegroundColor Red
                $allPassed = $false
            }
        }
        catch {
            Write-Host "   ❌ $($check.Name): 未安装" -ForegroundColor Red
            $allPassed = $false
        }
    }
    
    return $allPassed
}

# 创建新项目
function New-GitMentorProject {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory = $false)]
        [string]$ProjectName = "GitMentor-MVP",
        
        [Parameter(Mandatory = $false)]
        [string]$Path = "."
    )
    
    Write-Host "📁 创建 GitMentor MVP 项目: $ProjectName" -ForegroundColor Green
    
    # 检查环境
    if (-not (Test-GitMentorEnvironment)) {
        Write-Host "❌ 环境检查失败，请先安装必需的工具" -ForegroundColor Red
        return $false
    }
    
    # 创建项目
    try {
        Set-Location $Path
        npm create tauri-app@latest $ProjectName --template vue-ts --yes
        Set-Location $ProjectName
        
        # 安装依赖
        Write-Host "📦 安装前端依赖..." -ForegroundColor Yellow
        npm install element-plus "@element-plus/icons-vue" pinia
        
        Write-Host "✅ 项目创建成功！" -ForegroundColor Green
        Write-Host "   项目位置: $(Get-Location)" -ForegroundColor Cyan
        
        return $true
    }
    catch {
        Write-Host "❌ 项目创建失败: $_" -ForegroundColor Red
        return $false
    }
}

# 启动开发服务器
function Start-GitMentorDev {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory = $false)]
        [switch]$Verbose
    )
    
    if ($Verbose) {
        $env:RUST_LOG = "debug"
    }
    
    Write-Host "🚀 启动 GitMentor MVP 开发服务器..." -ForegroundColor Green
    
    try {
        npm run tauri:dev
    }
    catch {
        Write-Host "❌ 开发服务器启动失败: $_" -ForegroundColor Red
    }
}

# 构建项目
function Build-GitMentorProject {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory = $false)]
        [switch]$Release,
        
        [Parameter(Mandatory = $false)]
        [switch]$Clean
    )
    
    if ($Clean) {
        Write-Host "🧹 清理构建缓存..." -ForegroundColor Yellow
        Remove-Item -Recurse -Force "node_modules" -ErrorAction SilentlyContinue
        Remove-Item -Recurse -Force "src-tauri\target" -ErrorAction SilentlyContinue
        Remove-Item -Recurse -Force "dist" -ErrorAction SilentlyContinue
        
        Write-Host "📦 重新安装依赖..." -ForegroundColor Yellow
        npm install
    }
    
    if ($Release) {
        Write-Host "🏗️ 构建生产版本..." -ForegroundColor Green
        npm run tauri:build
    } else {
        Write-Host "🏗️ 构建开发版本..." -ForegroundColor Green
        npm run build
    }
}

# 配置管理
function Get-GitMentorConfig {
    [CmdletBinding()]
    param()
    
    $configFile = Join-Path $script:ConfigPath "app.toml"
    
    if (Test-Path $configFile) {
        Get-Content $configFile | Write-Host
    } else {
        Write-Host "❌ 配置文件不存在: $configFile" -ForegroundColor Red
    }
}

function Set-GitMentorConfig {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory = $true)]
        [string]$Key,
        
        [Parameter(Mandatory = $true)]
        [string]$Value,
        
        [Parameter(Mandatory = $false)]
        [string]$Section = "llm"
    )
    
    $configFile = Join-Path $script:ConfigPath "app.toml"
    
    if (-not (Test-Path $configFile)) {
        Write-Host "❌ 配置文件不存在: $configFile" -ForegroundColor Red
        return
    }
    
    # 简单的 TOML 配置更新 (实际项目中应使用专门的 TOML 库)
    $content = Get-Content $configFile
    $newContent = @()
    $inSection = $false
    $keyUpdated = $false
    
    foreach ($line in $content) {
        if ($line -match "^\[$Section\]") {
            $inSection = $true
            $newContent += $line
        }
        elseif ($line -match "^\[.*\]" -and $inSection) {
            if (-not $keyUpdated) {
                $newContent += "$Key = `"$Value`""
                $keyUpdated = $true
            }
            $inSection = $false
            $newContent += $line
        }
        elseif ($inSection -and $line -match "^$Key\s*=") {
            $newContent += "$Key = `"$Value`""
            $keyUpdated = $true
        }
        else {
            $newContent += $line
        }
    }
    
    if ($inSection -and -not $keyUpdated) {
        $newContent += "$Key = `"$Value`""
    }
    
    $newContent | Set-Content $configFile
    Write-Host "✅ 配置已更新: $Section.$Key = $Value" -ForegroundColor Green
}

# Ollama 管理
function Start-OllamaService {
    [CmdletBinding()]
    param()
    
    Write-Host "🤖 启动 Ollama 服务..." -ForegroundColor Green
    
    try {
        Start-Process -FilePath "ollama" -ArgumentList "serve" -NoNewWindow
        Write-Host "✅ Ollama 服务已启动" -ForegroundColor Green
    }
    catch {
        Write-Host "❌ Ollama 启动失败，请确保已安装 Ollama" -ForegroundColor Red
        Write-Host "   安装命令: winget install Ollama.Ollama" -ForegroundColor Cyan
    }
}

function Install-OllamaModel {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory = $true)]
        [string]$ModelName
    )
    
    Write-Host "📥 下载 Ollama 模型: $ModelName" -ForegroundColor Green
    
    try {
        ollama pull $ModelName
        Write-Host "✅ 模型下载完成: $ModelName" -ForegroundColor Green
    }
    catch {
        Write-Host "❌ 模型下载失败: $_" -ForegroundColor Red
    }
}

# 项目状态检查
function Get-GitMentorStatus {
    [CmdletBinding()]
    param()
    
    Write-Host "📊 GitMentor MVP 项目状态" -ForegroundColor Green
    Write-Host "=" * 40 -ForegroundColor Gray
    
    # 检查项目文件
    $files = @(
        "package.json",
        "src-tauri\Cargo.toml",
        "src-tauri\tauri.conf.json",
        "config\app.toml"
    )
    
    foreach ($file in $files) {
        if (Test-Path $file) {
            Write-Host "   ✅ $file" -ForegroundColor Green
        } else {
            Write-Host "   ❌ $file (缺失)" -ForegroundColor Red
        }
    }
    
    # 检查依赖
    Write-Host ""
    Write-Host "📦 依赖状态:" -ForegroundColor Yellow
    
    if (Test-Path "node_modules") {
        $packageCount = (Get-ChildItem "node_modules" -Directory).Count
        Write-Host "   ✅ Node.js 依赖: $packageCount 个包" -ForegroundColor Green
    } else {
        Write-Host "   ❌ Node.js 依赖未安装" -ForegroundColor Red
    }
    
    if (Test-Path "src-tauri\target") {
        Write-Host "   ✅ Rust 构建缓存存在" -ForegroundColor Green
    } else {
        Write-Host "   ⚠️  Rust 构建缓存不存在 (首次构建会较慢)" -ForegroundColor Yellow
    }
}

# 快速帮助
function Show-GitMentorHelp {
    [CmdletBinding()]
    param()
    
    Write-Host "🎯 GitMentor MVP PowerShell 模块帮助" -ForegroundColor Green
    Write-Host "=" * 50 -ForegroundColor Gray
    Write-Host ""
    Write-Host "📋 可用命令:" -ForegroundColor Yellow
    Write-Host "   Test-GitMentorEnvironment    - 检查开发环境" -ForegroundColor White
    Write-Host "   New-GitMentorProject         - 创建新项目" -ForegroundColor White
    Write-Host "   Start-GitMentorDev           - 启动开发服务器" -ForegroundColor White
    Write-Host "   Build-GitMentorProject       - 构建项目" -ForegroundColor White
    Write-Host "   Get-GitMentorConfig          - 查看配置" -ForegroundColor White
    Write-Host "   Set-GitMentorConfig          - 设置配置" -ForegroundColor White
    Write-Host "   Start-OllamaService          - 启动 Ollama 服务" -ForegroundColor White
    Write-Host "   Install-OllamaModel          - 安装 Ollama 模型" -ForegroundColor White
    Write-Host "   Get-GitMentorStatus          - 查看项目状态" -ForegroundColor White
    Write-Host "   Show-GitMentorHelp           - 显示此帮助" -ForegroundColor White
    Write-Host ""
    Write-Host "💡 示例用法:" -ForegroundColor Cyan
    Write-Host "   New-GitMentorProject -ProjectName 'MyProject'" -ForegroundColor Gray
    Write-Host "   Start-GitMentorDev -Verbose" -ForegroundColor Gray
    Write-Host "   Build-GitMentorProject -Release -Clean" -ForegroundColor Gray
    Write-Host "   Set-GitMentorConfig -Key 'model' -Value 'llama2'" -ForegroundColor Gray
}

# 导出函数
Export-ModuleMember -Function @(
    'Initialize-GitMentorMVP',
    'Test-GitMentorEnvironment',
    'New-GitMentorProject',
    'Start-GitMentorDev',
    'Build-GitMentorProject',
    'Get-GitMentorConfig',
    'Set-GitMentorConfig',
    'Start-OllamaService',
    'Install-OllamaModel',
    'Get-GitMentorStatus',
    'Show-GitMentorHelp'
)

# 模块加载时自动初始化
Initialize-GitMentorMVP
