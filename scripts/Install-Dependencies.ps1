# GitMentor MVP 依赖安装脚本 (PowerShell版本)
# 使用方法: .\Install-Dependencies.ps1 [-Force] [-SkipOptional]

param(
    [switch]$Force,
    [switch]$SkipOptional
)

$ErrorActionPreference = "Continue"

Write-Host "📦 GitMentor MVP 依赖安装脚本" -ForegroundColor Green
Write-Host "=" * 50 -ForegroundColor Gray

# 检查是否有管理员权限
function Test-Administrator {
    $currentUser = [Security.Principal.WindowsIdentity]::GetCurrent()
    $principal = New-Object Security.Principal.WindowsPrincipal($currentUser)
    return $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

$isAdmin = Test-Administrator
if (-not $isAdmin) {
    Write-Host "⚠️  注意：未以管理员身份运行，某些安装可能失败" -ForegroundColor Yellow
    Write-Host "   建议以管理员身份运行 PowerShell" -ForegroundColor Cyan
    Write-Host ""
}

# 检查 winget 是否可用
function Test-Winget {
    try {
        winget --version | Out-Null
        return $true
    }
    catch {
        return $false
    }
}

$hasWinget = Test-Winget
if (-not $hasWinget) {
    Write-Host "❌ winget 不可用，将尝试手动安装方式" -ForegroundColor Red
    Write-Host "   请考虑安装 App Installer 以获得 winget 支持" -ForegroundColor Yellow
}

# 安装 Node.js
Write-Host "📋 检查 Node.js..." -ForegroundColor Yellow
try {
    $nodeVersion = node --version 2>$null
    if ($nodeVersion -and -not $Force) {
        Write-Host "   ✅ Node.js $nodeVersion 已安装" -ForegroundColor Green
    } else {
        throw "需要安装"
    }
}
catch {
    Write-Host "   🔧 安装 Node.js..." -ForegroundColor Cyan
    if ($hasWinget) {
        try {
            winget install OpenJS.NodeJS --accept-package-agreements --accept-source-agreements
            Write-Host "   ✅ Node.js 安装完成" -ForegroundColor Green
        }
        catch {
            Write-Host "   ❌ winget 安装失败，请手动安装" -ForegroundColor Red
            Write-Host "   💡 下载地址: https://nodejs.org/" -ForegroundColor Cyan
        }
    } else {
        Write-Host "   ❌ 无法自动安装，请手动安装 Node.js" -ForegroundColor Red
        Write-Host "   💡 下载地址: https://nodejs.org/" -ForegroundColor Cyan
    }
}

# 安装 Rust
Write-Host "📋 检查 Rust..." -ForegroundColor Yellow
try {
    $rustVersion = rustc --version 2>$null
    if ($rustVersion -and -not $Force) {
        Write-Host "   ✅ $rustVersion 已安装" -ForegroundColor Green
    } else {
        throw "需要安装"
    }
}
catch {
    Write-Host "   🔧 安装 Rust..." -ForegroundColor Cyan
    if ($hasWinget) {
        try {
            winget install Rustlang.Rustup --accept-package-agreements --accept-source-agreements
            Write-Host "   ✅ Rust 安装完成" -ForegroundColor Green
        }
        catch {
            Write-Host "   ❌ winget 安装失败，尝试直接下载安装..." -ForegroundColor Yellow
            try {
                # 下载并运行 rustup-init.exe
                $rustupUrl = "https://win.rustup.rs/x86_64"
                $rustupPath = "$env:TEMP\rustup-init.exe"
                
                Write-Host "   📥 下载 rustup-init.exe..." -ForegroundColor Cyan
                Invoke-WebRequest -Uri $rustupUrl -OutFile $rustupPath
                
                Write-Host "   🔧 运行 rustup 安装程序..." -ForegroundColor Cyan
                Start-Process -FilePath $rustupPath -ArgumentList "-y" -Wait
                
                Write-Host "   ✅ Rust 安装完成" -ForegroundColor Green
                Remove-Item $rustupPath -ErrorAction SilentlyContinue
            }
            catch {
                Write-Host "   ❌ 自动安装失败，请手动安装" -ForegroundColor Red
                Write-Host "   💡 访问: https://rustup.rs/" -ForegroundColor Cyan
            }
        }
    } else {
        Write-Host "   ❌ 无法自动安装，请手动安装 Rust" -ForegroundColor Red
        Write-Host "   💡 访问: https://rustup.rs/" -ForegroundColor Cyan
    }
}

# 安装 Git
Write-Host "📋 检查 Git..." -ForegroundColor Yellow
try {
    $gitVersion = git --version 2>$null
    if ($gitVersion -and -not $Force) {
        Write-Host "   ✅ $gitVersion 已安装" -ForegroundColor Green
    } else {
        throw "需要安装"
    }
}
catch {
    Write-Host "   🔧 安装 Git..." -ForegroundColor Cyan
    if ($hasWinget) {
        try {
            winget install Git.Git --accept-package-agreements --accept-source-agreements
            Write-Host "   ✅ Git 安装完成" -ForegroundColor Green
        }
        catch {
            Write-Host "   ❌ winget 安装失败，请手动安装" -ForegroundColor Red
            Write-Host "   💡 下载地址: https://git-scm.com/" -ForegroundColor Cyan
        }
    } else {
        Write-Host "   ❌ 无法自动安装，请手动安装 Git" -ForegroundColor Red
        Write-Host "   💡 下载地址: https://git-scm.com/" -ForegroundColor Cyan
    }
}

# 可选：安装 Visual Studio Build Tools
if (-not $SkipOptional) {
    Write-Host "📋 检查 Visual Studio Build Tools (可选)..." -ForegroundColor Yellow
    try {
        $vsWhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
        if (Test-Path $vsWhere) {
            $buildTools = & $vsWhere -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -format json | ConvertFrom-Json
            if ($buildTools -and -not $Force) {
                Write-Host "   ✅ Visual Studio Build Tools 已安装" -ForegroundColor Green
            } else {
                throw "需要安装"
            }
        } else {
            throw "需要安装"
        }
    }
    catch {
        Write-Host "   🔧 安装 Visual Studio Build Tools..." -ForegroundColor Cyan
        if ($hasWinget) {
            try {
                winget install Microsoft.VisualStudio.2022.BuildTools --accept-package-agreements --accept-source-agreements
                Write-Host "   ✅ Visual Studio Build Tools 安装完成" -ForegroundColor Green
            }
            catch {
                Write-Host "   ❌ winget 安装失败，请手动安装" -ForegroundColor Red
                Write-Host "   💡 下载地址: https://visualstudio.microsoft.com/visual-cpp-build-tools/" -ForegroundColor Cyan
            }
        } else {
            Write-Host "   ❌ 无法自动安装，请手动安装 Visual Studio Build Tools" -ForegroundColor Red
            Write-Host "   💡 下载地址: https://visualstudio.microsoft.com/visual-cpp-build-tools/" -ForegroundColor Cyan
        }
    }
}

# 可选：安装 VS Code
if (-not $SkipOptional) {
    Write-Host "📋 检查 Visual Studio Code (可选)..." -ForegroundColor Yellow
    try {
        $codePath = Get-Command code -ErrorAction Stop
        if ($codePath -and -not $Force) {
            Write-Host "   ✅ Visual Studio Code 已安装" -ForegroundColor Green
        } else {
            throw "需要安装"
        }
    }
    catch {
        Write-Host "   🔧 安装 Visual Studio Code..." -ForegroundColor Cyan
        if ($hasWinget) {
            try {
                winget install Microsoft.VisualStudioCode --accept-package-agreements --accept-source-agreements
                Write-Host "   ✅ Visual Studio Code 安装完成" -ForegroundColor Green
            }
            catch {
                Write-Host "   ❌ winget 安装失败，请手动安装" -ForegroundColor Red
                Write-Host "   💡 下载地址: https://code.visualstudio.com/" -ForegroundColor Cyan
            }
        } else {
            Write-Host "   ❌ 无法自动安装，请手动安装 VS Code" -ForegroundColor Red
            Write-Host "   💡 下载地址: https://code.visualstudio.com/" -ForegroundColor Cyan
        }
    }
}

# 更新环境变量
Write-Host "📋 刷新环境变量..." -ForegroundColor Yellow
$env:Path = [System.Environment]::GetEnvironmentVariable("Path", "Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path", "User")

# 安装 Rust 目标平台
Write-Host "📋 配置 Rust 目标平台..." -ForegroundColor Yellow
try {
    rustup target add x86_64-pc-windows-msvc 2>$null
    Write-Host "   ✅ Windows MSVC 目标已添加" -ForegroundColor Green
}
catch {
    Write-Host "   ⚠️  无法添加 Rust 目标平台，请稍后手动运行：" -ForegroundColor Yellow
    Write-Host "      rustup target add x86_64-pc-windows-msvc" -ForegroundColor Cyan
}

# 总结
Write-Host ""
Write-Host "=" * 50 -ForegroundColor Gray
Write-Host "🎉 依赖安装脚本执行完成！" -ForegroundColor Green
Write-Host ""
Write-Host "📋 下一步操作：" -ForegroundColor Cyan
Write-Host "   1. 重启 PowerShell 终端以刷新环境变量" -ForegroundColor White
Write-Host "   2. 运行 .\Check-Environment.ps1 验证安装" -ForegroundColor White
Write-Host "   3. 运行 .\Setup-MVP.ps1 创建项目" -ForegroundColor White
Write-Host ""
Write-Host "💡 提示：如果遇到权限问题，请以管理员身份运行 PowerShell" -ForegroundColor Cyan
