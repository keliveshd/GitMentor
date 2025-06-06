# GitMentor 构建故障排除指南

## 🚨 常见问题解决方案

### 1. 批处理文件编码问题

**问题**: 执行 `build_simple.bat` 时出现乱码或命令无法识别
```
'/b' 不是内部或外部命令
'寤哄墠绔?' 不是内部或外部命令
```

**解决方案**:
使用新的 `build_windows.bat` 文件：
```bash
build_windows.bat
```

### 2. Rust 未安装

**问题**: 
```
ERROR: Rust not installed
```

**解决方案**:
1. 访问 https://rustup.rs/
2. 下载并运行安装程序
3. 重启命令提示符
4. 验证安装: `rustc --version`

**或者使用PowerShell安装**:
```powershell
# 在PowerShell中运行
Invoke-WebRequest -Uri "https://win.rustup.rs/" -OutFile "rustup-init.exe"
.\rustup-init.exe
```

### 3. Python 版本问题

**问题**:
```
ERROR: Python not installed
```

**解决方案**:
1. 下载Python 3.8+: https://python.org/downloads/
2. 安装时勾选 "Add Python to PATH"
3. 验证安装: `python --version`

### 4. Node.js 版本问题

**问题**:
```
ERROR: Node.js not installed
```

**解决方案**:
1. 下载Node.js 16+: https://nodejs.org/
2. 安装LTS版本
3. 验证安装: `node --version`

### 5. 依赖安装失败

**问题**:
```
ERROR: Frontend dependencies installation failed
ERROR: Backend dependencies installation failed
```

**解决方案**:

**前端依赖问题**:
```bash
# 清理缓存
npm cache clean --force
# 删除node_modules
rmdir /s node_modules
del package-lock.json
# 重新安装
npm install
```

**后端依赖问题**:
```bash
# 升级pip
python -m pip install --upgrade pip
# 重新安装依赖
cd backend
pip install -r requirements.txt --force-reinstall
```

### 6. PyInstaller 构建失败

**问题**:
```
ERROR: Backend build failed
```

**解决方案**:

**方法1: 手动安装PyInstaller**
```bash
pip install pyinstaller --upgrade
```

**方法2: 使用虚拟环境**
```bash
python -m venv build_env
build_env\Scripts\activate
pip install -r backend\requirements.txt
pip install pyinstaller
python build_backend_simple.py
```

**方法3: 检查隐藏导入**
如果仍然失败，编辑 `build_backend_simple.py`，在 `hiddenimports` 列表中添加缺失的模块。

### 7. Tauri 构建失败

**问题**:
```
ERROR: Tauri build failed
```

**解决方案**:

**检查Rust工具链**:
```bash
rustup update
rustc --version
cargo --version
```

**清理Rust缓存**:
```bash
cd src-tauri
cargo clean
cd ..
```

**手动构建**:
```bash
npm run tauri build -- --verbose
```

### 8. 权限问题

**问题**: 文件访问被拒绝或权限不足

**解决方案**:
1. 以管理员身份运行命令提示符
2. 检查防病毒软件是否阻止
3. 确保项目目录有写权限

### 9. 端口占用问题

**问题**: 后端启动时端口8000被占用

**解决方案**:
```bash
# 查找占用端口的进程
netstat -ano | findstr :8000
# 结束进程 (替换PID)
taskkill /PID <PID> /F
```

### 10. 内存不足

**问题**: 构建过程中内存不足

**解决方案**:
1. 关闭其他应用程序
2. 增加虚拟内存
3. 分步构建：
   ```bash
   # 只构建前端
   npm run build
   
   # 只构建后端
   python build_backend_simple.py
   
   # 最后构建Tauri
   npm run tauri build
   ```

## 🔧 手动构建步骤

如果自动化脚本失败，可以手动执行以下步骤：

### 步骤1: 环境检查
```bash
python --version
node --version
rustc --version
```

### 步骤2: 安装依赖
```bash
# 前端依赖
npm install

# 后端依赖
cd backend
pip install -r requirements.txt
pip install pyinstaller
cd ..
```

### 步骤3: 构建前端
```bash
npm run build
```

### 步骤4: 构建后端
```bash
python build_backend_simple.py
```

### 步骤5: 构建Tauri应用
```bash
npm run tauri build
```

## 📋 验证构建结果

构建完成后，检查以下文件是否存在：

1. **前端构建**: `dist/` 目录
2. **后端可执行文件**: `backend/gitmentor-backend.exe`
3. **Tauri应用**: `src-tauri/target/release/GitMentor.exe`
4. **安装包**: `src-tauri/target/release/bundle/msi/`

## 🆘 获取帮助

如果问题仍然存在：

1. **查看详细日志**: 使用 `--verbose` 参数
2. **检查系统要求**: 确保满足最低系统要求
3. **重启系统**: 有时环境变量需要重启生效
4. **使用虚拟环境**: 避免依赖冲突

## 📞 技术支持

### 常用调试命令
```bash
# 检查环境
python --version
node --version
rustc --version
npm --version

# 检查依赖
pip list
npm list

# 清理缓存
npm cache clean --force
pip cache purge
cargo clean
```

### 日志文件位置
- NPM日志: `%APPDATA%\npm-cache\_logs\`
- Python日志: 控制台输出
- Rust日志: `src-tauri/target/`

---

**提示**: 大多数构建问题都是由于环境配置不正确导致的。请确保按照顺序安装所有必需的工具。
