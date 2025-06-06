# GitMentor 打包部署指南

## 📦 打包概述

GitMentor使用Tauri框架将Vue 3前端和FastAPI后端打包成单个可执行文件，实现开箱即用的客户交付体验。

## 🛠️ 环境准备

### 必需软件
1. **Python 3.8+** - 后端运行环境
2. **Node.js 16+** - 前端构建环境  
3. **Rust** - Tauri构建环境
4. **Git** - 版本控制

### 安装Rust
```bash
# Windows/macOS/Linux
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# 或访问 https://rustup.rs/
```

### 验证环境
```bash
python --version    # 应显示 3.8+
node --version      # 应显示 16+
rustc --version     # 应显示 Rust 版本
```

## 🚀 快速打包

### 方式1: 使用自动化脚本（推荐）

**Windows:**
```bash
build_simple.bat
```

**Linux/macOS:**
```bash
python build_release.py
```

### 方式2: 手动打包

#### 步骤1: 安装依赖
```bash
# 前端依赖
npm install

# 后端依赖
cd backend
pip install -r requirements.txt
pip install pyinstaller
cd ..
```

#### 步骤2: 构建前端
```bash
npm run build
```

#### 步骤3: 构建后端
```bash
python build_backend.py
```

#### 步骤4: 构建Tauri应用
```bash
npm run tauri build
```

## 📁 打包结果

### Windows
- **安装包**: `src-tauri/target/release/bundle/msi/GitMentor_0.1.0_x64_en-US.msi`
- **可执行文件**: `src-tauri/target/release/GitMentor.exe`

### macOS
- **安装包**: `src-tauri/target/release/bundle/dmg/GitMentor_0.1.0_x64.dmg`
- **应用包**: `src-tauri/target/release/bundle/macos/GitMentor.app`

## 🎯 打包特性

### 自包含特性
- ✅ 内嵌Python后端服务
- ✅ 内嵌Vue 3前端界面
- ✅ 内嵌SQLite数据库
- ✅ 自动创建数据目录
- ✅ 零配置启动

### 客户体验
- 🚀 双击即可运行
- 📱 原生应用界面
- 💾 数据持久化存储
- 🔧 无需安装依赖环境
- 📚 内置离线文档

## 🏗️ 技术架构

### 打包架构
```
GitMentor.exe/app
├── Tauri Runtime (Rust)
├── WebView (前端界面)
├── Python Backend (内嵌)
├── SQLite Database
└── Static Resources
```

### 启动流程
```
1. Tauri启动 → 2. 启动Python后端 → 3. 初始化数据库 → 4. 加载前端界面
```

## 📋 配置说明

### Tauri配置 (src-tauri/tauri.conf.json)
```json
{
  "productName": "GitMentor",
  "version": "0.1.0",
  "identifier": "com.gitmentor.app",
  "bundle": {
    "resources": [
      "../backend/**/*",
      "../docs/**/*"
    ],
    "externalBin": [
      "backend/gitmentor-backend"
    ]
  }
}
```

### 后端打包配置
- 使用PyInstaller将FastAPI应用打包成可执行文件
- 包含所有Python依赖和资源文件
- 支持Windows和macOS平台

## 🔧 自定义配置

### 修改应用信息
编辑 `src-tauri/tauri.conf.json`:
```json
{
  "productName": "您的应用名称",
  "version": "1.0.0",
  "identifier": "com.yourcompany.app"
}
```

### 修改应用图标
替换 `src-tauri/icons/` 目录下的图标文件:
- `icon.ico` - Windows图标
- `icon.icns` - macOS图标
- `icon.png` - 通用图标

### 添加启动参数
在 `src-tauri/src/lib.rs` 中修改后端启动参数。

## 🚨 常见问题

### 1. Rust编译失败
```bash
# 更新Rust工具链
rustup update

# 清理缓存
cargo clean
```

### 2. Python依赖缺失
```bash
# 重新安装依赖
pip install -r backend/requirements.txt --force-reinstall
```

### 3. 前端构建失败
```bash
# 清理缓存
npm cache clean --force
rm -rf node_modules package-lock.json
npm install
```

### 4. 打包体积过大
- 使用 `--strip` 参数移除调试信息
- 启用UPX压缩
- 排除不必要的依赖

### 5. 后端启动失败
- 检查防火墙设置
- 确保端口8000未被占用
- 查看应用日志

## 📊 性能优化

### 减小包体积
1. 移除未使用的Python包
2. 启用UPX压缩
3. 使用最小化的依赖

### 提升启动速度
1. 优化后端启动逻辑
2. 使用缓存机制
3. 延迟加载非关键组件

## 🔐 安全考虑

### 代码保护
- Python代码通过PyInstaller编译保护
- Rust代码原生编译
- 资源文件内嵌保护

### 数据安全
- 本地SQLite数据库
- 配置文件加密存储
- 敏感信息环境变量化

## 📈 分发策略

### Windows分发
1. 生成MSI安装包
2. 可选择便携版ZIP包
3. 支持静默安装参数

### macOS分发
1. 生成DMG磁盘映像
2. 代码签名和公证
3. 支持App Store分发

### 版本更新
- 内置更新检查机制
- 增量更新支持
- 自动下载安装

## 📞 技术支持

### 构建问题
1. 检查环境变量配置
2. 查看构建日志
3. 验证依赖版本

### 运行问题
1. 查看应用日志
2. 检查系统兼容性
3. 验证权限设置

---

**GitMentor** - 专业的AI代码分析工具，开箱即用的企业级解决方案 🚀
