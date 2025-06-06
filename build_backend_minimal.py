#!/usr/bin/env python3
"""
GitMentor Backend Minimal Build Script
最小化依赖构建后端
"""

import os
import sys
import shutil
import subprocess
import platform
from pathlib import Path

def main():
    print("GitMentor Backend Minimal Build Tool")
    print("=" * 40)
    
    # Check Python version
    if sys.version_info < (3, 8):
        print("错误: 需要Python 3.8或更高版本")
        sys.exit(1)
    
    print(f"Python版本: {sys.version}")
    print(f"操作系统: {platform.system()} {platform.release()}")
    
    # Check backend directory
    if not os.path.exists("backend"):
        print("错误: 找不到backend目录")
        sys.exit(1)
    
    # Install only PyInstaller
    print("\n安装PyInstaller...")
    try:
        subprocess.check_call([
            sys.executable, "-m", "pip", "install", "pyinstaller==6.2.0",
            "-i", "https://pypi.tuna.tsinghua.edu.cn/simple/"
        ])
    except subprocess.CalledProcessError as e:
        print(f"错误: PyInstaller安装失败: {e}")
        sys.exit(1)
    
    print("PyInstaller安装成功")
    
    # Create minimal PyInstaller spec file
    print("创建PyInstaller配置文件...")
    spec_content = '''# -*- mode: python ; coding: utf-8 -*-

import os
from pathlib import Path

block_cipher = None

backend_dir = Path("backend")

a = Analysis(
    [str(backend_dir / "main.py")],
    pathex=[str(backend_dir)],
    binaries=[],
    datas=[
        (str(backend_dir / "app"), "app"),
    ],
    hiddenimports=[
        "fastapi",
        "fastapi.applications",
        "fastapi.routing",
        "fastapi.middleware",
        "fastapi.middleware.cors",
        "uvicorn",
        "uvicorn.main",
        "uvicorn.server",
        "uvicorn.config",
        "uvicorn.lifespan",
        "uvicorn.protocols",
        "uvicorn.protocols.http",
        "uvicorn.protocols.websockets",
        "uvicorn.logging",
        "pydantic",
        "pydantic.main",
        "pydantic.fields",
        "pydantic.types",
        "starlette",
        "starlette.applications",
        "starlette.routing",
        "starlette.middleware",
        "starlette.responses",
        "starlette.requests",
        "json",
        "sqlite3",
        "pathlib",
        "datetime",
        "typing",
        "asyncio",
    ],
    hookspath=[],
    hooksconfig={},
    runtime_hooks=[],
    excludes=[
        "pandas",
        "numpy", 
        "matplotlib",
        "scipy",
        "sklearn",
        "tensorflow",
        "torch",
        "jupyter",
        "notebook",
        "tkinter",
        "psutil",
        "gitpython",
        "httpx",
        "yaml",
        "multipart",
        "aiofiles",
    ],
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
    
    print("PyInstaller配置文件创建完成")
    
    # Build backend
    print("构建后端可执行文件...")
    try:
        subprocess.check_call([
            "pyinstaller", 
            "--clean",
            "--noconfirm",
            "gitmentor-backend.spec"
        ])
        print("后端构建成功")
        
        # Move executable to correct location
        dist_dir = Path("dist")
        backend_dir = Path("backend")
        
        if platform.system() == "Windows":
            exe_name = "gitmentor-backend.exe"
        else:
            exe_name = "gitmentor-backend"
        
        src_exe = dist_dir / exe_name
        dst_exe = backend_dir / exe_name
        
        if src_exe.exists():
            # Ensure backend directory exists
            backend_dir.mkdir(exist_ok=True)
            shutil.move(str(src_exe), str(dst_exe))
            print(f"可执行文件已移动到: {dst_exe}")
            
            # 验证文件大小
            file_size = dst_exe.stat().st_size / (1024 * 1024)  # MB
            print(f"可执行文件大小: {file_size:.1f} MB")
            
        else:
            print(f"错误: 找不到构建的可执行文件: {src_exe}")
            sys.exit(1)
            
    except subprocess.CalledProcessError as e:
        print(f"错误: 后端构建失败: {e}")
        sys.exit(1)
    
    # Cleanup
    print("清理临时文件...")
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
    
    print("清理完成")
    print("\n后端打包完成!")
    print(f"可执行文件位置: backend/{exe_name}")
    print("现在可以运行Tauri构建将后端打包到安装包中")

if __name__ == "__main__":
    main()
