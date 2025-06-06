#!/bin/bash

# GitMentor 快速启动脚本

echo "🚀 GitMentor 启动脚本"
echo "===================="

# 检查Python版本
echo "📋 检查环境..."
python_version=$(python3 --version 2>/dev/null || python --version 2>/dev/null)
if [ $? -eq 0 ]; then
    echo "✅ Python: $python_version"
else
    echo "❌ Python未安装或版本不兼容"
    exit 1
fi

# 检查Node.js版本
node_version=$(node --version 2>/dev/null)
if [ $? -eq 0 ]; then
    echo "✅ Node.js: $node_version"
else
    echo "❌ Node.js未安装"
    exit 1
fi

# 创建数据目录
echo "📁 创建数据目录..."
mkdir -p backend/data
mkdir -p backend/logs

# 安装后端依赖
echo "📦 安装后端依赖..."
cd backend
if [ ! -d "venv" ]; then
    echo "创建Python虚拟环境..."
    python3 -m venv venv 2>/dev/null || python -m venv venv
fi

echo "激活虚拟环境..."
source venv/bin/activate

echo "安装Python依赖..."
pip install -r requirements.txt

if [ $? -ne 0 ]; then
    echo "❌ 后端依赖安装失败"
    exit 1
fi

echo "✅ 后端依赖安装完成"

# 返回项目根目录
cd ..

# 安装前端依赖
echo "📦 安装前端依赖..."
if [ ! -d "node_modules" ]; then
    npm install
    if [ $? -ne 0 ]; then
        echo "❌ 前端依赖安装失败"
        exit 1
    fi
    echo "✅ 前端依赖安装完成"
else
    echo "✅ 前端依赖已存在"
fi

# 启动服务
echo "🚀 启动服务..."

# 启动后端（后台运行）
echo "启动后端服务..."
cd backend
source venv/bin/activate
nohup python main.py > logs/app.log 2>&1 &
backend_pid=$!
echo "后端PID: $backend_pid"

# 等待后端启动
sleep 3

# 检查后端是否启动成功
if curl -s http://localhost:8000/api/health > /dev/null; then
    echo "✅ 后端启动成功 (http://localhost:8000)"
else
    echo "❌ 后端启动失败"
    kill $backend_pid 2>/dev/null
    exit 1
fi

# 返回项目根目录启动前端
cd ..
echo "启动前端服务..."
npm run dev &
frontend_pid=$!

echo ""
echo "🎉 GitMentor 启动完成！"
echo "===================="
echo "📱 前端界面: http://localhost:1420"
echo "🔧 后端API: http://localhost:8000"
echo "📚 API文档: http://localhost:8000/docs"
echo ""
echo "💡 使用 Ctrl+C 停止服务"
echo "📋 后端日志: backend/logs/app.log"
echo ""

# 等待用户中断
trap 'echo ""; echo "🛑 正在停止服务..."; kill $backend_pid $frontend_pid 2>/dev/null; echo "✅ 服务已停止"; exit 0' INT

# 保持脚本运行
wait
