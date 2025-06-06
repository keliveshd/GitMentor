#!/usr/bin/env python3
"""
GitMentor完整打包脚本
将前端和后端打包成单个可执行文件
"""

import os
import sys
import shutil
import subprocess
import platform
import json
from pathlib import Path

def check_requirements():
    """检查构建环境"""
    print("🔍 检查构建环境...")
    
    # 检查Python版本
    if sys.version_info < (3, 8):
        print("❌ 需要Python 3.8或更高版本")
        return False
    
    # 检查Node.js
    try:
        result = subprocess.run(["node", "--version"], capture_output=True, text=True)
        if result.returncode != 0:
            print("❌ Node.js未安装")
            return False
        print(f"✅ Node.js: {result.stdout.strip()}")
    except FileNotFoundError:
        print("❌ Node.js未安装")
        return False
    
    # 检查Rust
    try:
        result = subprocess.run(["rustc", "--version"], capture_output=True, text=True)
        if result.returncode != 0:
            print("❌ Rust未安装")
            return False
        print(f"✅ Rust: {result.stdout.strip()}")
    except FileNotFoundError:
        print("❌ Rust未安装，请访问 https://rustup.rs/ 安装")
        return False
    
    print(f"✅ Python: {sys.version}")
    print(f"✅ 操作系统: {platform.system()} {platform.release()}")
    
    return True

def install_dependencies():
    """安装构建依赖"""
    print("\n📦 安装构建依赖...")
    
    # 安装前端依赖
    print("安装前端依赖...")
    try:
        subprocess.check_call(["npm", "install"], cwd=".")
        print("✅ 前端依赖安装成功")
    except subprocess.CalledProcessError as e:
        print(f"❌ 前端依赖安装失败: {e}")
        return False
    
    # 安装后端依赖
    print("安装后端依赖...")
    try:
        subprocess.check_call([
            sys.executable, "-m", "pip", "install", 
            "-r", "backend/requirements.txt"
        ])
        print("✅ 后端依赖安装成功")
    except subprocess.CalledProcessError as e:
        print(f"❌ 后端依赖安装失败: {e}")
        return False
    
    # 安装PyInstaller
    print("安装PyInstaller...")
    try:
        subprocess.check_call([sys.executable, "-m", "pip", "install", "pyinstaller"])
        print("✅ PyInstaller安装成功")
    except subprocess.CalledProcessError as e:
        print(f"❌ PyInstaller安装失败: {e}")
        return False
    
    return True

def build_frontend():
    """构建前端"""
    print("\n🎨 构建前端...")
    
    try:
        subprocess.check_call(["npm", "run", "build"])
        print("✅ 前端构建成功")
        return True
    except subprocess.CalledProcessError as e:
        print(f"❌ 前端构建失败: {e}")
        return False

def build_backend():
    """构建后端"""
    print("\n🔨 构建后端...")
    
    # 创建PyInstaller spec文件
    spec_content = '''
# -*- mode: python ; coding: utf-8 -*-

import os
from pathlib import Path

block_cipher = None

# 获取后端目录
backend_dir = Path("backend")

# 收集所有Python文件和数据
a = Analysis(
    [str(backend_dir / "main.py")],
    pathex=[str(backend_dir)],
    binaries=[],
    datas=[
        (str(backend_dir / "app"), "app"),
    ],
    hiddenimports=[
        "fastapi",
        "uvicorn",
        "uvicorn.lifespan.on",
        "uvicorn.lifespan.off", 
        "uvicorn.protocols.websockets.auto",
        "uvicorn.protocols.http.auto",
        "uvicorn.protocols.http.h11_impl",
        "uvicorn.protocols.http.httptools_impl",
        "uvicorn.protocols.websockets.wsproto_impl",
        "uvicorn.protocols.websockets.websockets_impl",
        "uvicorn.logging",
        "pydantic",
        "sqlalchemy",
        "psutil",
        "httpx",
        "yaml",
        "gitpython",
        "pandas",
        "multipart",
        "aiofiles",
        "python_multipart",
        "email_validator",
    ],
    hookspath=[],
    hooksconfig={},
    runtime_hooks=[],
    excludes=[],
    win_no_prefer_redirects=False,
    win_private_assemblies=False,
    cipher=block_cipher,
    noarchive=False,
)

pyz = PYZ(a.pure, a.zipped_data, cipher=block_cipher)

exe = EXE(
    pyz,
    a.scripts,
    a.binaries,
    a.zipfiles,
    a.datas,
    [],
    name="gitmentor-backend",
    debug=False,
    bootloader_ignore_signals=False,
    strip=False,
    upx=True,
    upx_exclude=[],
    runtime_tmpdir=None,
    console=False,
    disable_windowed_traceback=False,
    argv_emulation=False,
    target_arch=None,
    codesign_identity=None,
    entitlements_file=None,
)
'''
    
    # 写入spec文件
    with open("gitmentor-backend.spec", "w", encoding="utf-8") as f:
        f.write(spec_content)
    
    try:
        # 使用PyInstaller构建
        subprocess.check_call([
            "pyinstaller", 
            "--clean",
            "--noconfirm",
            "gitmentor-backend.spec"
        ])
        
        # 移动可执行文件
        dist_dir = Path("dist")
        backend_dir = Path("backend")
        
        if platform.system() == "Windows":
            exe_name = "gitmentor-backend.exe"
        else:
            exe_name = "gitmentor-backend"
        
        src_exe = dist_dir / exe_name
        dst_exe = backend_dir / exe_name
        
        if src_exe.exists():
            # 确保目标目录存在
            backend_dir.mkdir(exist_ok=True)
            shutil.move(str(src_exe), str(dst_exe))
            print(f"✅ 后端可执行文件: {dst_exe}")
            return True
        else:
            print(f"❌ 找不到构建的可执行文件: {src_exe}")
            return False
            
    except subprocess.CalledProcessError as e:
        print(f"❌ 后端构建失败: {e}")
        return False

def build_tauri_app():
    """构建Tauri应用"""
    print("\n📱 构建Tauri应用...")
    
    try:
        subprocess.check_call(["npm", "run", "tauri", "build"])
        print("✅ Tauri应用构建成功")
        return True
    except subprocess.CalledProcessError as e:
        print(f"❌ Tauri应用构建失败: {e}")
        return False

def create_docs():
    """创建离线文档"""
    print("\n📚 创建离线文档...")
    
    docs_dir = Path("docs")
    docs_dir.mkdir(exist_ok=True)
    
    # 复制项目文档
    docs_files = [
        "README.md",
        "PROJECT_SUMMARY.md", 
        "DEPLOYMENT_GUIDE.md"
    ]
    
    for doc_file in docs_files:
        if Path(doc_file).exists():
            shutil.copy(doc_file, docs_dir / doc_file)
    
    # 创建使用说明
    usage_guide = """# GitMentor 使用说明

## 快速开始

1. 双击运行 GitMentor 应用
2. 等待系统启动完成
3. 在仪表板中查看系统状态
4. 通过"仓库配置"添加要分析的Git仓库
5. 在"分析结果"中查看AI分析报告

## 主要功能

### 仓库管理
- 添加多个Git仓库
- 配置分析参数
- 启用/禁用仓库分析

### AI分析
- 自动分析Git提交
- 双重审核机制
- 质量评分和建议

### 监控面板
- 实时系统状态
- 质量趋势图表
- 性能指标监控

## 技术支持

如有问题，请查看：
1. 应用内的健康检查页面
2. 系统监控指标
3. 项目文档

---
GitMentor v1.0.0
"""
    
    with open(docs_dir / "使用说明.md", "w", encoding="utf-8") as f:
        f.write(usage_guide)
    
    print("✅ 离线文档创建完成")

def cleanup():
    """清理临时文件"""
    print("\n🧹 清理临时文件...")
    
    cleanup_paths = [
        "build",
        "dist",
        "gitmentor-backend.spec",
        "__pycache__"
    ]
    
    for path in cleanup_paths:
        if os.path.exists(path):
            if os.path.isdir(path):
                shutil.rmtree(path)
            else:
                os.remove(path)
    
    print("✅ 清理完成")

def main():
    """主函数"""
    print("🚀 GitMentor 完整打包工具")
    print("=" * 50)
    
    # 检查构建环境
    if not check_requirements():
        print("\n❌ 构建环境检查失败")
        sys.exit(1)
    
    # 安装依赖
    if not install_dependencies():
        print("\n❌ 依赖安装失败")
        sys.exit(1)
    
    # 构建前端
    if not build_frontend():
        print("\n❌ 前端构建失败")
        sys.exit(1)
    
    # 构建后端
    if not build_backend():
        print("\n❌ 后端构建失败")
        sys.exit(1)
    
    # 创建文档
    create_docs()
    
    # 构建Tauri应用
    if not build_tauri_app():
        print("\n❌ Tauri应用构建失败")
        sys.exit(1)
    
    # 清理临时文件
    cleanup()
    
    print("\n🎉 GitMentor 打包完成!")
    print("=" * 50)
    
    # 显示构建结果
    if platform.system() == "Windows":
        bundle_path = "src-tauri/target/release/bundle/msi"
        print(f"📦 Windows安装包: {bundle_path}")
    elif platform.system() == "Darwin":
        bundle_path = "src-tauri/target/release/bundle/dmg"
        print(f"📦 macOS安装包: {bundle_path}")
    
    print("\n✅ 可执行文件已生成，可以分发给客户使用！")

if __name__ == "__main__":
    main()
