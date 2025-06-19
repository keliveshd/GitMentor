# GitMentor MVP 构建脚本 (PowerShell版本)
# 使用方法: .\Build-MVP.ps1 [-Mode "dev"|"build"] [-Clean]

param(
    [ValidateSet("dev", "build")]
    [string]$Mode = "dev",
    [switch]$Clean
)

$ErrorActionPreference = "Stop"

Write-Host "🔨 GitMentor MVP 构建脚本" -ForegroundColor Green
Write-Host "模式: $Mode" -ForegroundColor Yellow

# 检查是否在项目根目录
if (-not (Test-Path "package.json") -or -not (Test-Path "src-tauri")) {
    Write-Host "❌ 请在项目根目录运行此脚本" -ForegroundColor Red
    exit 1
}

# 清理构建缓存
if ($Clean) {
    Write-Host "🧹 清理构建缓存..." -ForegroundColor Yellow
    
    if (Test-Path "node_modules") {
        Remove-Item -Recurse -Force "node_modules"
        Write-Host "   ✅ 删除 node_modules" -ForegroundColor Green
    }
    
    if (Test-Path "src-tauri\target") {
        Remove-Item -Recurse -Force "src-tauri\target"
        Write-Host "   ✅ 删除 Rust target 目录" -ForegroundColor Green
    }
    
    if (Test-Path "dist") {
        Remove-Item -Recurse -Force "dist"
        Write-Host "   ✅ 删除前端构建目录" -ForegroundColor Green
    }
    
    Write-Host "🧹 清理完成" -ForegroundColor Green
}

# 检查依赖
Write-Host "📋 检查依赖..." -ForegroundColor Yellow

if (-not (Test-Path "node_modules")) {
    Write-Host "📦 安装前端依赖..." -ForegroundColor Yellow
    npm install
    if ($LASTEXITCODE -ne 0) {
        Write-Host "❌ 前端依赖安装失败" -ForegroundColor Red
        exit 1
    }
}

# 检查 Rust 工具链
try {
    $rustVersion = rustc --version
    Write-Host "   ✅ Rust: $rustVersion" -ForegroundColor Green
}
catch {
    Write-Host "❌ Rust 未安装或不可用" -ForegroundColor Red
    exit 1
}

# 检查 Tauri CLI
try {
    $tauriVersion = cargo tauri --version 2>$null
    if (-not $tauriVersion) {
        Write-Host "📦 安装 Tauri CLI..." -ForegroundColor Yellow
        cargo install tauri-cli
        if ($LASTEXITCODE -ne 0) {
            Write-Host "❌ Tauri CLI 安装失败" -ForegroundColor Red
            exit 1
        }
    }
    Write-Host "   ✅ Tauri CLI 已安装" -ForegroundColor Green
}
catch {
    Write-Host "❌ Tauri CLI 检查失败" -ForegroundColor Red
    exit 1
}

# 执行构建
switch ($Mode) {
    "dev" {
        Write-Host "🚀 启动开发服务器..." -ForegroundColor Green
        Write-Host "   按 Ctrl+C 停止服务器" -ForegroundColor Yellow
        Write-Host ""
        
        try {
            npm run tauri:dev
        }
        catch {
            Write-Host "❌ 开发服务器启动失败" -ForegroundColor Red
            exit 1
        }
    }
    
    "build" {
        Write-Host "🏗️ 构建生产版本..." -ForegroundColor Green
        
        # 构建前端
        Write-Host "📦 构建前端..." -ForegroundColor Yellow
        npm run build
        if ($LASTEXITCODE -ne 0) {
            Write-Host "❌ 前端构建失败" -ForegroundColor Red
            exit 1
        }
        
        # 构建 Tauri 应用
        Write-Host "🦀 构建 Tauri 应用..." -ForegroundColor Yellow
        npm run tauri:build
        if ($LASTEXITCODE -ne 0) {
            Write-Host "❌ Tauri 应用构建失败" -ForegroundColor Red
            exit 1
        }
        
        Write-Host "✅ 构建完成！" -ForegroundColor Green
        Write-Host ""
        Write-Host "📁 构建产物位置：" -ForegroundColor Cyan
        
        # 查找构建产物
        $bundlePath = "src-tauri\target\release\bundle"
        if (Test-Path $bundlePath) {
            $msiFiles = Get-ChildItem -Path "$bundlePath\msi" -Filter "*.msi" -ErrorAction SilentlyContinue
            $exeFiles = Get-ChildItem -Path "src-tauri\target\release" -Filter "*.exe" -ErrorAction SilentlyContinue
            
            if ($msiFiles) {
                Write-Host "   🎁 MSI 安装包: $($msiFiles[0].FullName)" -ForegroundColor White
            }
            
            if ($exeFiles) {
                Write-Host "   📱 可执行文件: $($exeFiles[0].FullName)" -ForegroundColor White
            }
        }
    }
}

Write-Host ""
Write-Host "🎉 操作完成！" -ForegroundColor Green
