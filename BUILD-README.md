# GitMentor Windows 打包指南

本文档介绍如何在Windows平台上构建和打包GitMentor应用程序。

## 📋 系统要求

### 必需工具
- **Node.js 18+** - JavaScript运行时环境
- **Rust 1.70+** - 系统编程语言
- **Cargo** - Rust包管理器（随Rust安装）
- **Git** - 版本控制工具（推荐）

### 推荐工具
- **Visual Studio Build Tools** - Windows C++编译器
- **PowerShell 5.0+** - 增强的脚本执行体验

## 🚀 快速开始

### 方案一：自动安装（推荐新用户）
```batch
# 1. 自动安装所有依赖
install-dependencies.bat

# 2. 验证环境
check-environment.bat

# 3. 开始构建
quick-build.bat
```

### 方案二：手动检查（推荐有经验用户）
```batch
# 1. 检查构建环境
check-environment.bat

# 2. 快速构建（适合日常开发）
quick-build.bat

# 3. 完整构建（生产环境）
build-windows-package.bat
```

### 方案三：PowerShell增强体验
```powershell
# 一键完整构建（推荐）
.\Build-GitMentor-Windows.ps1 -Clean -OpenOutput
```

## 📜 脚本说明

### install-dependencies.bat
**功能**: 自动安装构建依赖
- 自动检测并安装Node.js、Rust等工具
- 使用winget包管理器（Windows 10 1709+）
- 安装项目前端依赖
- 验证安装结果

**使用场景**:
- 首次设置开发环境
- 快速配置新的开发机器

### check-environment.bat
**功能**: 检查构建环境是否完整
- 验证Node.js、Rust、Cargo等工具安装状态
- 检查项目结构和依赖
- 提供安装建议

**使用场景**: 
- 首次设置开发环境
- 构建失败时的故障排除

### quick-build.bat
**功能**: 快速构建脚本
- 最小化的环境检查
- 自动安装前端依赖
- 执行标准构建流程

**使用场景**:
- 日常开发和测试
- 快速验证代码更改

### build-windows-package.bat
**功能**: 完整的Windows打包脚本
- 全面的环境检查
- 支持清理构建缓存
- 生成多种格式的安装包
- 详细的错误处理和提示

**参数**:
- `--debug`: 构建调试版本
- `--clean`: 清理缓存后重新构建
- `--skip-deps`: 跳过依赖检查
- `--verbose`: 显示详细信息
- `--help`: 显示帮助信息

**使用示例**:
```batch
# 标准构建
build-windows-package.bat

# 清理后重新构建
build-windows-package.bat --clean

# 构建调试版本
build-windows-package.bat --debug
```

### Build-GitMentor-Windows.ps1
**功能**: PowerShell版本的完整构建脚本
- 更好的错误处理和用户体验
- 彩色输出和进度提示
- 支持打开输出目录

**参数**:
- `-BuildMode <release|debug>`: 构建模式
- `-Clean`: 清理构建缓存
- `-SkipDeps`: 跳过依赖检查
- `-Verbose`: 详细输出
- `-OpenOutput`: 构建完成后打开输出目录
- `-Help`: 显示帮助

**使用示例**:
```powershell
# 标准构建
.\Build-GitMentor-Windows.ps1

# 清理后重新构建并打开输出目录
.\Build-GitMentor-Windows.ps1 -Clean -OpenOutput

# 调试模式构建
.\Build-GitMentor-Windows.ps1 -BuildMode debug -Verbose
```

## 📁 构建输出

构建成功后，将在以下位置生成文件：

### 发布版本 (Release)
```
GitMentor-Lite/src-tauri/target/release/
├── gitmentor-lite.exe          # 可执行文件
└── bundle/
    ├── msi/
    │   └── *.msi              # MSI安装包
    └── nsis/
        └── *.exe              # NSIS安装包
```

### 调试版本 (Debug)
```
GitMentor-Lite/src-tauri/target/debug/
├── gitmentor-lite.exe          # 调试版可执行文件
└── bundle/
    └── ...                    # 调试版安装包
```

## 🔧 故障排除

### 常见问题

1. **"Node.js 未安装"**
   - 从 https://nodejs.org/ 下载并安装Node.js 18+

2. **"Rust 未安装"**
   ```batch
   # 使用winget安装
   winget install Rustlang.Rustup
   
   # 或访问 https://rustup.rs/
   ```

3. **"Tauri CLI 未安装"**
   ```batch
   cargo install tauri-cli
   ```

4. **构建失败**
   - 运行 `build-windows-package.bat --clean` 清理后重试
   - 检查TypeScript错误：`npm run build`
   - 检查Rust编译错误：`cargo check`

5. **权限问题**
   - 以管理员身份运行PowerShell
   - 设置执行策略：`Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser`

### 调试技巧

1. **查看详细错误信息**
   ```batch
   build-windows-package.bat --verbose
   ```

2. **单独测试前端构建**
   ```batch
   cd GitMentor-Lite
   npm run build
   ```

3. **单独测试Rust编译**
   ```batch
   cd GitMentor-Lite
   cargo check
   ```

## 📦 部署建议

### 可执行文件部署
- 将 `gitmentor-lite.exe` 复制到目标机器
- 确保目标机器有必要的运行时库
- 建议创建快捷方式到桌面或开始菜单

### 安装包部署
- **MSI包**: 适合企业环境，支持静默安装
- **NSIS包**: 适合个人用户，安装体验更好

### 系统要求
- Windows 10 1903+ 或 Windows 11
- .NET Framework 4.7.2+（通常已预装）
- Visual C++ Redistributable（如需要）

## 🔄 持续集成

对于自动化构建，建议：

1. 使用 `build-windows-package.bat --clean --skip-deps` 
2. 预先安装所有依赖
3. 设置适当的环境变量
4. 保存构建产物到指定目录

## 📞 支持

如果遇到问题：
1. 首先运行 `check-environment.bat` 检查环境
2. 查看本文档的故障排除部分
3. 检查项目的GitHub Issues
4. 提交新的Issue并附上详细的错误信息
