# GitMentor MVP 环境检查脚本 (PowerShell版本)
# 使用方法: .\Check-Environment.ps1 [-Fix]

param(
    [switch]$Fix
)

$ErrorActionPreference = "Continue"

Write-Host "🔍 GitMentor MVP 环境检查" -ForegroundColor Green
Write-Host "=" * 50 -ForegroundColor Gray

$allGood = $true

# 检查 PowerShell 版本
Write-Host "📋 PowerShell 版本检查..." -ForegroundColor Yellow
$psVersion = $PSVersionTable.PSVersion
if ($psVersion.Major -ge 5) {
    Write-Host "   ✅ PowerShell $($psVersion.ToString()) - 支持" -ForegroundColor Green
} else {
    Write-Host "   ❌ PowerShell $($psVersion.ToString()) - 需要 5.0+" -ForegroundColor Red
    $allGood = $false
}

# 检查 Node.js
Write-Host "📋 Node.js 检查..." -ForegroundColor Yellow
try {
    $nodeVersion = node --version 2>$null
    if ($nodeVersion) {
        $versionNumber = [version]($nodeVersion -replace 'v', '')
        if ($versionNumber.Major -ge 16) {
            Write-Host "   ✅ Node.js $nodeVersion - 支持" -ForegroundColor Green
        } else {
            Write-Host "   ⚠️  Node.js $nodeVersion - 建议升级到 16+" -ForegroundColor Yellow
        }
    } else {
        throw "Node.js 未找到"
    }
}
catch {
    Write-Host "   ❌ Node.js 未安装" -ForegroundColor Red
    if ($Fix) {
        Write-Host "   🔧 尝试安装 Node.js..." -ForegroundColor Cyan
        try {
            winget install OpenJS.NodeJS
            Write-Host "   ✅ Node.js 安装完成，请重启终端" -ForegroundColor Green
        }
        catch {
            Write-Host "   ❌ 自动安装失败，请手动安装: https://nodejs.org/" -ForegroundColor Red
        }
    } else {
        Write-Host "   💡 安装命令: winget install OpenJS.NodeJS" -ForegroundColor Cyan
        Write-Host "   💡 或访问: https://nodejs.org/" -ForegroundColor Cyan
    }
    $allGood = $false
}

# 检查 npm
Write-Host "📋 npm 检查..." -ForegroundColor Yellow
try {
    $npmVersion = npm --version 2>$null
    if ($npmVersion) {
        Write-Host "   ✅ npm $npmVersion - 支持" -ForegroundColor Green
    } else {
        throw "npm 未找到"
    }
}
catch {
    Write-Host "   ❌ npm 未安装 (通常随 Node.js 一起安装)" -ForegroundColor Red
    $allGood = $false
}

# 检查 Rust
Write-Host "📋 Rust 检查..." -ForegroundColor Yellow
try {
    $rustVersion = rustc --version 2>$null
    if ($rustVersion) {
        $versionMatch = $rustVersion -match "rustc (\d+\.\d+)"
        if ($versionMatch) {
            $rustVer = [version]$matches[1]
            if ($rustVer -ge [version]"1.70") {
                Write-Host "   ✅ $rustVersion - 支持" -ForegroundColor Green
            } else {
                Write-Host "   ⚠️  $rustVersion - 建议升级到 1.70+" -ForegroundColor Yellow
                if ($Fix) {
                    Write-Host "   🔧 更新 Rust..." -ForegroundColor Cyan
                    rustup update
                }
            }
        }
    } else {
        throw "Rust 未找到"
    }
}
catch {
    Write-Host "   ❌ Rust 未安装" -ForegroundColor Red
    if ($Fix) {
        Write-Host "   🔧 尝试安装 Rust..." -ForegroundColor Cyan
        try {
            winget install Rustlang.Rustup
            Write-Host "   ✅ Rust 安装完成，请重启终端" -ForegroundColor Green
        }
        catch {
            Write-Host "   ❌ 自动安装失败，请手动安装" -ForegroundColor Red
        }
    } else {
        Write-Host "   💡 安装命令: winget install Rustlang.Rustup" -ForegroundColor Cyan
        Write-Host "   💡 或访问: https://rustup.rs/" -ForegroundColor Cyan
    }
    $allGood = $false
}

# 检查 Cargo
Write-Host "📋 Cargo 检查..." -ForegroundColor Yellow
try {
    $cargoVersion = cargo --version 2>$null
    if ($cargoVersion) {
        Write-Host "   ✅ $cargoVersion - 支持" -ForegroundColor Green
    } else {
        throw "Cargo 未找到"
    }
}
catch {
    Write-Host "   ❌ Cargo 未安装 (通常随 Rust 一起安装)" -ForegroundColor Red
    $allGood = $false
}

# 检查 Git
Write-Host "📋 Git 检查..." -ForegroundColor Yellow
try {
    $gitVersion = git --version 2>$null
    if ($gitVersion) {
        Write-Host "   ✅ $gitVersion - 支持" -ForegroundColor Green
    } else {
        throw "Git 未找到"
    }
}
catch {
    Write-Host "   ❌ Git 未安装" -ForegroundColor Red
    if ($Fix) {
        Write-Host "   🔧 尝试安装 Git..." -ForegroundColor Cyan
        try {
            winget install Git.Git
            Write-Host "   ✅ Git 安装完成，请重启终端" -ForegroundColor Green
        }
        catch {
            Write-Host "   ❌ 自动安装失败，请手动安装" -ForegroundColor Red
        }
    } else {
        Write-Host "   💡 安装命令: winget install Git.Git" -ForegroundColor Cyan
        Write-Host "   💡 或访问: https://git-scm.com/" -ForegroundColor Cyan
    }
    $allGood = $false
}

# 检查 Visual Studio Build Tools (可选但推荐)
Write-Host "📋 Visual Studio Build Tools 检查..." -ForegroundColor Yellow
try {
    $vsWhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
    if (Test-Path $vsWhere) {
        $buildTools = & $vsWhere -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -format json | ConvertFrom-Json
        if ($buildTools) {
            Write-Host "   ✅ Visual Studio Build Tools 已安装" -ForegroundColor Green
        } else {
            throw "Build Tools 未找到"
        }
    } else {
        throw "vswhere 未找到"
    }
}
catch {
    Write-Host "   ⚠️  Visual Studio Build Tools 未安装 (可选)" -ForegroundColor Yellow
    Write-Host "   💡 如果遇到编译错误，请安装 Visual Studio Build Tools" -ForegroundColor Cyan
    Write-Host "   💡 下载地址: https://visualstudio.microsoft.com/visual-cpp-build-tools/" -ForegroundColor Cyan
}

# 检查网络连接
Write-Host "📋 网络连接检查..." -ForegroundColor Yellow
try {
    $response = Test-NetConnection -ComputerName "registry.npmjs.org" -Port 443 -InformationLevel Quiet -WarningAction SilentlyContinue
    if ($response) {
        Write-Host "   ✅ npm registry 连接正常" -ForegroundColor Green
    } else {
        Write-Host "   ⚠️  npm registry 连接异常" -ForegroundColor Yellow
    }
}
catch {
    Write-Host "   ⚠️  网络连接检查失败" -ForegroundColor Yellow
}

try {
    $response = Test-NetConnection -ComputerName "crates.io" -Port 443 -InformationLevel Quiet -WarningAction SilentlyContinue
    if ($response) {
        Write-Host "   ✅ Rust crates.io 连接正常" -ForegroundColor Green
    } else {
        Write-Host "   ⚠️  Rust crates.io 连接异常" -ForegroundColor Yellow
    }
}
catch {
    Write-Host "   ⚠️  Rust registry 连接检查失败" -ForegroundColor Yellow
}

# 总结
Write-Host ""
Write-Host "=" * 50 -ForegroundColor Gray
if ($allGood) {
    Write-Host "🎉 环境检查完成 - 所有必需工具已安装！" -ForegroundColor Green
    Write-Host ""
    Write-Host "📋 下一步操作：" -ForegroundColor Cyan
    Write-Host "   1. 运行 .\Setup-MVP.ps1 创建项目" -ForegroundColor White
    Write-Host "   2. 根据文档完成代码实现" -ForegroundColor White
    Write-Host "   3. 运行 .\Build-MVP.ps1 -Mode dev 启动开发服务器" -ForegroundColor White
} else {
    Write-Host "❌ 环境检查失败 - 请安装缺失的工具" -ForegroundColor Red
    Write-Host ""
    Write-Host "💡 提示：" -ForegroundColor Cyan
    Write-Host "   - 运行 .\Check-Environment.ps1 -Fix 尝试自动修复" -ForegroundColor White
    Write-Host "   - 安装完成后请重启 PowerShell 终端" -ForegroundColor White
}

Write-Host ""
