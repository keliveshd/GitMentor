#!/usr/bin/env python3
"""
GitMentor Backend Fast Build Script
"""

import os
import sys
import shutil
import subprocess
import platform
from pathlib import Path

def main():
    print("GitMentor Backend Fast Build Tool")
    print("=" * 40)
    
    # Check Python version
    if sys.version_info < (3, 8):
        print("ERROR: Python 3.8+ required")
        sys.exit(1)
    
    print(f"Python version: {sys.version}")
    print(f"Operating system: {platform.system()} {platform.release()}")
    
    # Check backend directory
    if not os.path.exists("backend"):
        print("ERROR: backend directory not found")
        sys.exit(1)
    
    # Install minimal dependencies
    print("\nInstalling minimal dependencies...")
    try:
        # Install only essential packages
        essential_packages = [
            "fastapi==0.104.1",
            "uvicorn==0.24.0", 
            "pydantic==2.5.0",
            "httpx==0.25.2",
            "python-multipart==0.0.6",
            "aiofiles==23.2.1",
            "python-dotenv==1.0.0",
            "pyyaml==6.0.1",
            "psutil==5.9.6",
            "pyinstaller"
        ]
        
        for package in essential_packages:
            print(f"Installing {package}...")
            subprocess.check_call([
                sys.executable, "-m", "pip", "install", package
            ], stdout=subprocess.DEVNULL, stderr=subprocess.STDOUT)
            
    except subprocess.CalledProcessError as e:
        print(f"ERROR: Dependency installation failed: {e}")
        sys.exit(1)
    
    print("Dependencies installed successfully")
    
    # Create simplified PyInstaller spec file
    print("Creating PyInstaller spec file...")
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
        "psutil",
        "httpx",
        "yaml",
        "multipart",
        "aiofiles",
        "python_multipart",
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
        "notebook"
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
    console=False,
    disable_windowed_traceback=False,
    argv_emulation=False,
    target_arch=None,
    codesign_identity=None,
    entitlements_file=None,
)
'''
    
    with open("gitmentor-backend.spec", "w", encoding="utf-8") as f:
        f.write(spec_content)
    
    print("PyInstaller spec file created")
    
    # Build backend
    print("Building backend executable...")
    try:
        subprocess.check_call([
            "pyinstaller", 
            "--clean",
            "--noconfirm",
            "gitmentor-backend.spec"
        ])
        print("Backend build successful")
        
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
            print(f"Executable moved to: {dst_exe}")
        else:
            print(f"ERROR: Built executable not found: {src_exe}")
            sys.exit(1)
            
    except subprocess.CalledProcessError as e:
        print(f"ERROR: Backend build failed: {e}")
        sys.exit(1)
    
    # Cleanup
    print("Cleaning up temporary files...")
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
    
    print("Cleanup completed")
    print("\nBackend packaging completed!")
    print(f"Executable location: backend/{exe_name}")

if __name__ == "__main__":
    main()
