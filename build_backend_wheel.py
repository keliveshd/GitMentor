#!/usr/bin/env python3
"""
GitMentor Backend Build Script (Using Wheels)
使用预编译wheel包避免编译问题
"""

import os
import sys
import shutil
import subprocess
import platform
from pathlib import Path

def setup_pip_config():
    """配置pip使用国内镜像和wheel包"""
    print("配置pip使用预编译包...")
    
    # 创建pip配置目录
    if platform.system() == "Windows":
        pip_dir = Path.home() / "pip"
    else:
        pip_dir = Path.home() / ".pip"
    
    pip_dir.mkdir(exist_ok=True)
    pip_conf = pip_dir / "pip.conf"
    
    pip_config = """[global]
index-url = https://pypi.tuna.tsinghua.edu.cn/simple/
trusted-host = pypi.tuna.tsinghua.edu.cn
timeout = 120
prefer-binary = true
only-binary = :all:
"""
    
    with open(pip_conf, "w", encoding="utf-8") as f:
        f.write(pip_config)
    
    print(f"pip配置已写入: {pip_conf}")

def main():
    print("GitMentor Backend Build Tool (Wheel Only)")
    print("=" * 50)
    
    # 配置pip
    setup_pip_config()
    
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
    
    # 升级pip和安装wheel
    print("\n升级pip和安装wheel...")
    try:
        subprocess.check_call([
            sys.executable, "-m", "pip", "install", "--upgrade", 
            "pip", "wheel", "setuptools",
            "-i", "https://pypi.tuna.tsinghua.edu.cn/simple/",
            "--prefer-binary"
        ])
    except subprocess.CalledProcessError as e:
        print(f"警告: pip升级失败: {e}")
    
    # Install essential dependencies with specific versions that have wheels
    print("\n安装核心依赖（仅使用wheel包）...")
    try:
        # 使用已知有wheel包的版本
        essential_packages = [
            "fastapi==0.104.1",
            "uvicorn==0.24.0", 
            "pydantic==2.4.2",  # 使用较早版本，更可能有wheel
            "httpx==0.25.2",
            "python-multipart==0.0.6",
            "aiofiles==23.2.1",
            "python-dotenv==1.0.0",
            "pyyaml==6.0.1",
            "pyinstaller==6.1.0"  # 使用较稳定版本
        ]
        
        for package in essential_packages:
            print(f"安装 {package}...")
            subprocess.check_call([
                sys.executable, "-m", "pip", "install", package,
                "-i", "https://pypi.tuna.tsinghua.edu.cn/simple/",
                "--prefer-binary",
                "--only-binary=all",
                "--no-compile"
            ])
            
    except subprocess.CalledProcessError as e:
        print(f"错误: 依赖安装失败: {e}")
        print("尝试使用备用方案...")
        
        # 备用方案：逐个安装，跳过失败的包
        backup_packages = [
            "fastapi",
            "uvicorn", 
            "pydantic",
            "pyinstaller"
        ]
        
        for package in backup_packages:
            try:
                print(f"备用安装 {package}...")
                subprocess.check_call([
                    sys.executable, "-m", "pip", "install", package,
                    "-i", "https://pypi.tuna.tsinghua.edu.cn/simple/",
                    "--prefer-binary"
                ])
            except subprocess.CalledProcessError:
                print(f"跳过 {package}")
                continue
    
    print("依赖安装完成")
    
    # Create simplified PyInstaller spec file
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
        "uvicorn.logging",
        "pydantic",
        "pydantic.main",
        "pydantic.fields",
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

if __name__ == "__main__":
    main()
