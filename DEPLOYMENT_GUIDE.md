# GitMentor 部署运行手册

## 📋 系统要求

### 基础环境
- **操作系统**: Windows 10/11, macOS 10.15+, Ubuntu 18.04+
- **Python**: 3.8+ (推荐 3.9 或 3.10)
- **Node.js**: 16.0+ (推荐 18.x LTS)
- **内存**: 最低 4GB，推荐 8GB+
- **磁盘空间**: 最低 2GB 可用空间

### 必需软件
- Git
- Python 3.8+
- Node.js 16+
- npm 或 yarn

## 🚀 快速启动

### 1. 克隆项目
```bash
git clone <repository-url>
cd GitMentor
```

### 2. 后端启动
```bash
# 进入后端目录
cd backend

# 安装Python依赖
pip install fastapi uvicorn pydantic sqlite3 psutil

# 启动后端服务
python main.py
```

### 3. 前端启动
```bash
# 新开终端，进入项目根目录
cd GitMentor

# 安装前端依赖
npm install

# 启动开发服务器
npm run dev
```

### 4. 访问应用
- **前端界面**: http://localhost:1420
- **后端API**: http://localhost:8000
- **API文档**: http://localhost:8000/docs

## 📦 详细安装步骤

### 步骤1: 环境准备

#### Python环境
```bash
# 检查Python版本
python --version
# 或
python3 --version

# 如果版本低于3.8，请升级Python
```

#### Node.js环境
```bash
# 检查Node.js版本
node --version

# 检查npm版本
npm --version
```

### 步骤2: 后端部署

#### 2.1 创建虚拟环境（推荐）
```bash
# 创建虚拟环境
python -m venv gitmentor-env

# 激活虚拟环境
# Windows:
gitmentor-env\Scripts\activate
# macOS/Linux:
source gitmentor-env/bin/activate
```

#### 2.2 安装后端依赖
```bash
cd backend

# 方式1: 使用requirements.txt（如果存在）
pip install -r requirements.txt

# 方式2: 手动安装核心依赖
pip install fastapi==0.104.1
pip install uvicorn[standard]==0.24.0
pip install pydantic==2.5.0
pip install psutil==5.9.6
```

#### 2.3 数据库初始化
```bash
# 创建数据目录
mkdir -p data

# 启动应用（会自动创建数据库）
python main.py
```

#### 2.4 验证后端启动
```bash
# 检查后端是否正常运行
curl http://localhost:8000/api/health
# 应该返回: {"status": "healthy", "timestamp": "..."}
```

### 步骤3: 前端部署

#### 3.1 安装前端依赖
```bash
# 回到项目根目录
cd ..

# 安装依赖
npm install

# 如果遇到网络问题，可以使用国内镜像
npm install --registry=https://registry.npmmirror.com
```

#### 3.2 启动开发服务器
```bash
# 启动开发模式
npm run dev

# 应该看到类似输出：
# Local:   http://localhost:1420/
# Network: use --host to expose
```

#### 3.3 生产环境构建
```bash
# 构建生产版本
npm run build

# 构建完成后，dist目录包含生产文件
```

## ⚙️ 配置说明

### 后端配置

#### 环境变量
创建 `backend/.env` 文件：
```env
# 数据库配置
DATABASE_URL=sqlite:///./data/gitmentor.db

# API配置
API_HOST=0.0.0.0
API_PORT=8000

# LLM配置（可选）
OPENAI_API_KEY=your_openai_api_key
ANTHROPIC_API_KEY=your_anthropic_api_key

# 日志级别
LOG_LEVEL=INFO
```

#### 数据库配置
```python
# backend/app/core/database.py 中的配置
DATABASE_URL = "sqlite:///./data/gitmentor.db"
```

### 前端配置

#### 环境变量
创建 `.env.local` 文件：
```env
# API基础URL
VITE_API_BASE_URL=http://localhost:8000

# 应用标题
VITE_APP_TITLE=GitMentor

# 开发模式配置
VITE_DEV_MODE=true
```

## 🔧 故障排除

### 常见问题

#### 1. 后端启动失败
```bash
# 检查端口占用
netstat -an | grep 8000
# 或
lsof -i :8000

# 更换端口
uvicorn main:app --host 0.0.0.0 --port 8001
```

#### 2. 前端构建失败
```bash
# 清理缓存
npm cache clean --force

# 删除node_modules重新安装
rm -rf node_modules package-lock.json
npm install
```

#### 3. 数据库权限问题
```bash
# 确保data目录有写权限
chmod 755 data/
```

#### 4. Python依赖冲突
```bash
# 使用虚拟环境隔离依赖
python -m venv fresh-env
source fresh-env/bin/activate  # Linux/Mac
# 或
fresh-env\Scripts\activate     # Windows
pip install -r requirements.txt
```

### 日志查看

#### 后端日志
```bash
# 查看应用日志
tail -f backend/logs/app.log

# 或直接在控制台查看
python main.py --log-level DEBUG
```

#### 前端日志
```bash
# 开发模式下在浏览器控制台查看
# 或查看构建日志
npm run build --verbose
```

## 🌐 生产环境部署

### 使用Docker（推荐）

#### 1. 创建Dockerfile
```dockerfile
# 后端Dockerfile
FROM python:3.10-slim

WORKDIR /app
COPY backend/ .
RUN pip install -r requirements.txt

EXPOSE 8000
CMD ["python", "main.py"]
```

#### 2. 创建docker-compose.yml
```yaml
version: '3.8'
services:
  backend:
    build: .
    ports:
      - "8000:8000"
    volumes:
      - ./data:/app/data
    environment:
      - DATABASE_URL=sqlite:///./data/gitmentor.db
  
  frontend:
    image: nginx:alpine
    ports:
      - "80:80"
    volumes:
      - ./dist:/usr/share/nginx/html
```

#### 3. 部署命令
```bash
# 构建并启动
docker-compose up -d

# 查看日志
docker-compose logs -f
```

### 传统部署

#### 1. 后端部署
```bash
# 使用gunicorn（生产WSGI服务器）
pip install gunicorn
gunicorn main:app -w 4 -k uvicorn.workers.UvicornWorker --bind 0.0.0.0:8000
```

#### 2. 前端部署
```bash
# 构建静态文件
npm run build

# 使用nginx服务静态文件
# 配置nginx.conf指向dist目录
```

## 📊 监控和维护

### 健康检查
```bash
# 检查系统健康状态
curl http://localhost:8000/api/monitoring/health

# 检查应用指标
curl http://localhost:8000/api/monitoring/metrics
```

### 数据备份
```bash
# 备份SQLite数据库
cp data/gitmentor.db data/gitmentor_backup_$(date +%Y%m%d).db

# 备份Markdown文件
tar -czf markdown_backup_$(date +%Y%m%d).tar.gz data/markdown/
```

### 日志轮转
```bash
# 设置logrotate（Linux）
sudo nano /etc/logrotate.d/gitmentor
```

## 🔐 安全配置

### API安全
```python
# 在main.py中添加CORS配置
from fastapi.middleware.cors import CORSMiddleware

app.add_middleware(
    CORSMiddleware,
    allow_origins=["http://localhost:1420"],  # 生产环境改为实际域名
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)
```

### 数据库安全
```bash
# 设置数据库文件权限
chmod 600 data/gitmentor.db
```

## 📞 技术支持

### 获取帮助
1. 查看项目文档: `PROJECT_SUMMARY.md`
2. 检查API文档: http://localhost:8000/docs
3. 查看系统日志定位问题
4. 使用健康检查接口诊断系统状态

### 性能优化
1. 启用缓存: 确保缓存管理器正常工作
2. 监控系统资源: 使用监控API查看系统状态
3. 数据库优化: 定期清理过期数据
4. 前端优化: 使用生产构建版本

---

**注意**: 首次启动时，系统会自动创建数据库表结构和必要的目录。请确保应用有足够的文件系统权限。
