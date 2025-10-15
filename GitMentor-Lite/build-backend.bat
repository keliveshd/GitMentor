@echo off
cd /d D:\GitRepo\GitMentor\GitMentor-Lite\src-tauri
echo 正在编译 Rust 后端...
cargo check --release
if %ERRORLEVEL% NEQ 0 (
    echo 编译失败！
    pause
    exit /b 1
)
echo 编译成功！
pause