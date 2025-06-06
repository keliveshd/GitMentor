# GitMentor - AI Agent双重审核系统

GitMentor 是一个基于 AI Agent 的 Git 提交分析和质量控制系统，采用创新的双重审核机制确保代码分析的准确性和可靠性。

## ✨ 核心特性

- 🤖 **AI Agent双重审核**: Analyzer + Reviewer 双重质量保障
- 📊 **实时质量监控**: 全面的质量仪表板和趋势分析
- 🏗️ **多仓库支持**: 灵活的仓库配置和管理
- ⚡ **性能优化**: 智能缓存和系统监控
- 🎯 **配置驱动**: 热更新配置管理
- 📱 **现代化界面**: Vue 3 + Element Plus

## 🚀 一键启动

### 方式1: 使用启动脚本（推荐）

**Windows:**
```bash
start.bat
```

**Linux/macOS:**
```bash
chmod +x start.sh
./start.sh
```

### 方式2: 手动启动

#### 环境要求
- Python 3.8+
- Node.js 16+
- Git

#### 快速启动
```bash
# 1. 克隆项目
git clone <repository-url>
cd GitMentor

# 2. 启动后端
cd backend
pip install -r requirements.txt
python main.py

# 3. 启动前端（新终端）
cd ..
npm install
npm run dev
```

#### 访问应用
- 📱 **前端界面**: http://localhost:1420
- 🔧 **后端API**: http://localhost:8000
- 📚 **API文档**: http://localhost:8000/docs

## 📋 系统架构

### 核心组件
- **Analyzer Agent**: 分析Git提交并生成结构化总结
- **Reviewer Agent**: 审核分析质量并提供反馈
- **Quality Controller**: 协调双重审核流程
- **Storage Manager**: SQLite + Markdown双重存储
- **Config Manager**: 配置管理和热更新
- **Cache Manager**: 智能缓存优化
- **Monitor System**: 全方位系统监控

### 技术栈
- **前端**: Vue 3 + Element Plus + Vite
- **后端**: FastAPI + Python + SQLite
- **监控**: psutil + 自定义指标收集
- **缓存**: LRU内存缓存
- **配置**: YAML + 热更新

## 📦 客户交付版本

### 一键打包成可执行文件

**Windows:**
```bash
build_simple.bat
```

**Linux/macOS:**
```bash
python build_release.py
```

### 打包特性
- 🚀 **开箱即用**: 无需安装Python、Node.js等环境
- 📱 **原生应用**: 基于Tauri的原生桌面应用
- 💾 **自包含**: 内嵌后端API、前端界面、数据库
- 🔧 **零配置**: 双击即可运行，自动初始化
- 📚 **离线文档**: 内置完整使用说明

## 📖 详细文档

- 📋 **部署手册**: [DEPLOYMENT_GUIDE.md](DEPLOYMENT_GUIDE.md)
- 📦 **打包指南**: [PACKAGING_GUIDE.md](PACKAGING_GUIDE.md)
- 📊 **项目总结**: [PROJECT_SUMMARY.md](PROJECT_SUMMARY.md)
- 🔧 **API文档**: http://localhost:8000/docs (启动后访问)

## 🎯 核心工作流程

```
Git提交 → Analyzer Agent分析 → Reviewer Agent审核 → 质量决策 → 双重存储
                                                                ↓
                                                        SQLite + Markdown
```

## 🔧 配置说明

### LLM API配置（可选）
```bash
# 在 backend/.env 中配置
OPENAI_API_KEY=your_openai_api_key
ANTHROPIC_API_KEY=your_anthropic_api_key
```

### 仓库配置
通过Web界面 "仓库配置" 页面进行配置，支持：
- 多仓库管理
- Agent分配
- 分析设置
- 用户映射

## 📊 功能模块

### 已实现功能 ✅
- [x] AI Agent双重审核系统
- [x] 多仓库配置管理
- [x] 实时质量监控仪表板
- [x] 缓存和性能优化
- [x] 完整的Web管理界面
- [x] RESTful API接口
- [x] 配置热更新
- [x] 系统监控和告警

### 待扩展功能 🔄
- [ ] 图表可视化组件
- [ ] 用户权限管理
- [ ] CI/CD集成
- [ ] Docker容器化
- [ ] 数据导出功能

## 🤝 贡献指南

1. Fork 项目
2. 创建功能分支
3. 提交更改
4. 推送到分支
5. 创建 Pull Request

## 📄 许可证

MIT License

## 🆘 技术支持

- 📖 查看详细文档
- 🔍 使用健康检查接口诊断问题
- 📊 通过监控API查看系统状态
- 📝 查看应用日志定位问题

---

**GitMentor** - 让AI帮助您提升代码质量 🚀
