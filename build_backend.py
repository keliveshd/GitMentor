#!/usr/bin/env python3
"""
GitMentor后端打包脚本
使用PyInstaller将FastAPI后端打包成可执行文件
"""

import os
import sys
import shutil
import subprocess
import platform
from pathlib import Path

def install_pyinstaller():
    """安装PyInstaller"""
    print("📦 安装PyInstaller...")
    try:
        subprocess.check_call([sys.executable, "-m", "pip", "install", "pyinstaller"])
        print("✅ PyInstaller安装成功")
    except subprocess.CalledProcessError as e:
        print(f"❌ PyInstaller安装失败: {e}")
        sys.exit(1)

def create_spec_file():
    """创建PyInstaller spec文件"""
    spec_content = '''
# -*- mode: python ; coding: utf-8 -*-

import os
from pathlib import Path

block_cipher = None

# 获取后端目录
backend_dir = Path("backend")

# 收集所有Python文件
a = Analysis(
    [str(backend_dir / "main.py")],
    pathex=[str(backend_dir)],
    binaries=[],
    datas=[
        (str(backend_dir / "app"), "app"),
        (str(backend_dir / "data"), "data"),
    ],
    hiddenimports=[
        "fastapi",
        "uvicorn",
        "pydantic",
        "sqlalchemy",
        "psutil",
        "httpx",
        "yaml",
        "gitpython",
        "pandas",
        "multipart",
        "aiofiles",
        "uvicorn.lifespan.on",
        "uvicorn.lifespan.off",
        "uvicorn.protocols.websockets.auto",
        "uvicorn.protocols.http.auto",
        "uvicorn.protocols.http.h11_impl",
        "uvicorn.protocols.http.httptools_impl",
        "uvicorn.protocols.websockets.wsproto_impl",
        "uvicorn.protocols.websockets.websockets_impl",
        "uvicorn.logging",
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
    console=True,
    disable_windowed_traceback=False,
    argv_emulation=False,
    target_arch=None,
    codesign_identity=None,
    entitlements_file=None,
)
'''
    
    with open("gitmentor-backend.spec", "w", encoding="utf-8") as f:
        f.write(spec_content)
    
    print("✅ PyInstaller spec文件创建成功")

def build_backend():
    """构建后端可执行文件"""
    print("🔨 开始构建后端...")
    
    try:
        # 使用spec文件构建
        subprocess.check_call([
            "pyinstaller", 
            "--clean",
            "--noconfirm",
            "gitmentor-backend.spec"
        ])
        print("✅ 后端构建成功")
        
        # 移动可执行文件到正确位置
        dist_dir = Path("dist")
        backend_dir = Path("backend")
        
        if platform.system() == "Windows":
            exe_name = "gitmentor-backend.exe"
        else:
            exe_name = "gitmentor-backend"
        
        src_exe = dist_dir / exe_name
        dst_exe = backend_dir / exe_name
        
        if src_exe.exists():
            shutil.move(str(src_exe), str(dst_exe))
            print(f"✅ 可执行文件移动到: {dst_exe}")
        else:
            print(f"❌ 找不到构建的可执行文件: {src_exe}")
            
    except subprocess.CalledProcessError as e:
        print(f"❌ 后端构建失败: {e}")
        sys.exit(1)

def cleanup():
    """清理临时文件"""
    print("🧹 清理临时文件...")
    
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
    print("🚀 GitMentor后端打包工具")
    print("=" * 40)
    
    # 检查Python版本
    if sys.version_info < (3, 8):
        print("❌ 需要Python 3.8或更高版本")
        sys.exit(1)
    
    print(f"✅ Python版本: {sys.version}")
    print(f"✅ 操作系统: {platform.system()} {platform.release()}")
    
    # 检查后端目录
    if not os.path.exists("backend"):
        print("❌ 找不到backend目录")
        sys.exit(1)
    
    # 安装依赖
    print("\n📦 安装构建依赖...")
    subprocess.check_call([
        sys.executable, "-m", "pip", "install", 
        "-r", "backend/requirements.txt"
    ])
    
    # 安装PyInstaller
    install_pyinstaller()
    
    # 创建spec文件
    create_spec_file()
    
    # 构建后端
    build_backend()
    
    # 清理临时文件
    cleanup()
    
    print("\n🎉 后端打包完成!")
    print("📁 可执行文件位置: backend/gitmentor-backend")

if __name__ == "__main__":
    main()
