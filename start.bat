@echo off
chcp 65001 >nul

echo 🚀 GitMentor 启动脚本
echo ====================

REM 检查Python版本
echo 📋 检查环境...
python --version >nul 2>&1
if %errorlevel% neq 0 (
    echo ❌ Python未安装或版本不兼容
    pause
    exit /b 1
) else (
    for /f "tokens=*" %%i in ('python --version') do echo ✅ Python: %%i
)

REM 检查Node.js版本
node --version >nul 2>&1
if %errorlevel% neq 0 (
    echo ❌ Node.js未安装
    pause
    exit /b 1
) else (
    for /f "tokens=*" %%i in ('node --version') do echo ✅ Node.js: %%i
)

REM 创建数据目录
echo 📁 创建数据目录...
if not exist "backend\data" mkdir backend\data
if not exist "backend\logs" mkdir backend\logs

REM 安装后端依赖
echo 📦 安装后端依赖...
cd backend

if not exist "venv" (
    echo 创建Python虚拟环境...
    python -m venv venv
)

echo 激活虚拟环境...
call venv\Scripts\activate.bat

echo 安装Python依赖...
pip install -r requirements.txt
if %errorlevel% neq 0 (
    echo ❌ 后端依赖安装失败
    pause
    exit /b 1
)

echo ✅ 后端依赖安装完成

REM 返回项目根目录
cd ..

REM 安装前端依赖
echo 📦 安装前端依赖...
if not exist "node_modules" (
    npm install
    if %errorlevel% neq 0 (
        echo ❌ 前端依赖安装失败
        pause
        exit /b 1
    )
    echo ✅ 前端依赖安装完成
) else (
    echo ✅ 前端依赖已存在
)

REM 启动服务
echo 🚀 启动服务...

REM 启动后端（后台运行）
echo 启动后端服务...
cd backend
call venv\Scripts\activate.bat
start /b python main.py > logs\app.log 2>&1

REM 等待后端启动
timeout /t 3 /nobreak >nul

REM 检查后端是否启动成功
curl -s http://localhost:8000/api/health >nul 2>&1
if %errorlevel% equ 0 (
    echo ✅ 后端启动成功 (http://localhost:8000)
) else (
    echo ❌ 后端启动失败，请检查日志
    echo 📋 日志位置: backend\logs\app.log
    pause
    exit /b 1
)

REM 返回项目根目录启动前端
cd ..
echo 启动前端服务...

echo.
echo 🎉 GitMentor 启动完成！
echo ====================
echo 📱 前端界面: http://localhost:1420
echo 🔧 后端API: http://localhost:8000
echo 📚 API文档: http://localhost:8000/docs
echo.
echo 💡 按任意键打开前端界面...
echo 📋 后端日志: backend\logs\app.log
echo.

REM 打开浏览器
start http://localhost:1420

REM 启动前端开发服务器
npm run dev

pause
