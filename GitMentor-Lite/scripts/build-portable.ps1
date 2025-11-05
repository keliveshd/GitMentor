#!/usr/bin/env pwsh
# GitMentor Lite Portable 版本构建脚本
# 作者：Evilek
# 用途：在 MSI 构建完成后，创建便携版 zip 包

param(
    [string]$Version = "",
    [string]$SourceDir = "",
    [string]$OutputDir = ""
)

# 设置错误处理
$ErrorActionPreference = "Stop"

# 默认路径
if (-not $Version) {
    $Version = Get-Content -Path "package.json" | Select-String '"version"' | ForEach-Object { $_.Line -match '"version":\s*"([^"]+)"' | Out-Null; $matches[1] }
}

if (-not $SourceDir) {
    $SourceDir = "src-tauri/target/release"
}

if (-not $OutputDir) {
    $OutputDir = "src-tauri/target/release/bundle/portable"
}

Write-Host "=== 开始构建 Portable 版本 ===" -ForegroundColor Green
Write-Host "版本: $Version" -ForegroundColor Yellow
Write-Host "源目录: $SourceDir" -ForegroundColor Yellow
Write-Host "输出目录: $OutputDir" -ForegroundColor Yellow

# 创建输出目录
$OutputDir = Join-Path $OutputDir "GitMentorLite-$Version"
Write-Host "Portable 目录: $OutputDir" -ForegroundColor Yellow

if (Test-Path $OutputDir) {
    Remove-Item -Path $OutputDir -Recurse -Force
}

New-Item -ItemType Directory -Path $OutputDir -Force | Out-Null

# 复制可执行文件
Write-Host "`n[1/4] 复制可执行文件..." -ForegroundColor Cyan
$exeSource = Join-Path $SourceDir "GitMentorLite.exe"
if (Test-Path $exeSource) {
    Copy-Item -Path $exeSource -Destination $OutputDir
    Write-Host "  ✓ GitMentorLite.exe" -ForegroundColor Green
} else {
    Write-Error "未找到 GitMentorLite.exe"
}

# 复制 Git 二进制文件（如果存在）
Write-Host "`n[2/4] 复制 Git 二进制文件..." -ForegroundColor Cyan
$gitSource = Join-Path $SourceDir "git.exe"
if (Test-Path $gitSource) {
    Copy-Item -Path $gitSource -Destination $OutputDir
    Write-Host "  ✓ git.exe" -ForegroundColor Green
} else {
    Write-Host "  ⊘ 未找到 git.exe（可选）" -ForegroundColor Gray
}

# 复制资源文件
Write-Host "`n[3/4] 复制资源文件..." -ForegroundColor Cyan
$resourcesSource = Join-Path $SourceDir "resources"
if (Test-Path $resourcesSource) {
    Copy-Item -Path $resourcesSource -Destination (Join-Path $OutputDir "resources") -Recurse
    Write-Host "  ✓ resources/" -ForegroundColor Green
} else {
    Write-Host "  ⊘ 未找到 resources 目录" -ForegroundColor Gray
}

# 复制 PDB 文件（用于调试，可选）
Write-Host "`n[4/4] 复制调试文件（可选）..." -ForegroundColor Cyan
$pdbSource = Join-Path $SourceDir "GitMentorLite.pdb"
if (Test-Path $pdbSource) {
    Copy-Item -Path $pdbSource -Destination $OutputDir
    Write-Host "  ✓ GitMentorLite.pdb" -ForegroundColor Green
} else {
    Write-Host "  ⊘ 未找到 PDB 文件（可选）" -ForegroundColor Gray
}

# 创建启动脚本
Write-Host "`n创建启动脚本..." -ForegroundColor Cyan
$launcherScript = @"
@echo off
REM GitMentor Lite Portable Launcher
REM Version: $Version

echo 正在启动 GitMentor Lite...

REM 检查可执行文件是否存在
if not exist "GitMentorLite.exe" (
    echo 错误: 找不到 GitMentorLite.exe
    pause
    exit /b 1
)

REM 启动应用
start "" "GitMentorLite.exe"

REM 可选：退出启动器
REM exit
"@

$launcherScript | Out-File -FilePath (Join-Path $OutputDir "启动 GitMentor Lite.bat") -Encoding UTF8

# 创建 README
Write-Host "创建说明文档..." -ForegroundColor Cyan
$readmeContent = @"
# GitMentor Lite Portable 版本

## 版本
$Version

## 使用方法

### Windows 用户
1. 双击 `启动 GitMentor Lite.bat` 启动应用
2. 或者直接双击 `GitMentorLite.exe` 启动

### 注意事项
- 这是便携版，无需安装，解压即可使用
- 首次运行可能需要 Windows 防火墙允许
- 所有数据保存在用户目录中

## 文件说明
- `GitMentorLite.exe` - 主程序
- `git.exe` - Git 可执行文件（用于 Git 操作）
- `resources/` - 应用程序资源文件
- `启动 GitMentor Lite.bat` - 启动脚本（Windows）

## 故障排除
如果遇到问题，请检查：
1. 是否有杀毒软件拦截
2. 是否有足够的文件权限
3. 尝试以管理员身份运行

---
GitMentor Lite - 轻量级 Git 图形化工具
"@

$readmeContent | Out-File -FilePath (Join-Path $OutputDir "README.txt") -Encoding UTF8

# 创建 ZIP 包
Write-Host "`n创建 ZIP 包..." -ForegroundColor Cyan
$zipPath = Join-Path (Split-Path $OutputDir -Parent) "GitMentorLite-$Version-portable.zip"

# 使用 PowerShell 的 Compress-Archive
try {
    Compress-Archive -Path "$OutputDir\*" -DestinationPath $zipPath -CompressionLevel Optimal
    Write-Host "  ✓ 已创建: $zipPath" -ForegroundColor Green

    # 显示文件大小
    $size = (Get-Item $zipPath).Length
    $sizeMB = [math]::Round($size / 1MB, 2)
    Write-Host "  ✓ 文件大小: $sizeMB MB" -ForegroundColor Green
} catch {
    Write-Error "创建 ZIP 失败: $_"
    exit 1
}

Write-Host "`n=== Portable 版本构建完成 ===" -ForegroundColor Green
Write-Host "输出路径: $OutputDir" -ForegroundColor Yellow
Write-Host "ZIP 文件: $zipPath" -ForegroundColor Yellow
