@echo off
echo Manual Dependency Installation Script
echo =====================================

REM Configure pip to use wheels only
echo Configuring pip for wheel-only installation...
pip config set global.prefer-binary true
pip config set global.only-binary ":all:"
pip config set global.index-url https://pypi.tuna.tsinghua.edu.cn/simple/
pip config set global.trusted-host pypi.tuna.tsinghua.edu.cn

REM Upgrade pip first
echo Upgrading pip...
python -m pip install --upgrade pip -i https://pypi.tuna.tsinghua.edu.cn/simple/ --prefer-binary

REM Install dependencies one by one with timeout
echo Installing FastAPI...
timeout 60 python -m pip install fastapi==0.104.1 -i https://pypi.tuna.tsinghua.edu.cn/simple/ --prefer-binary --only-binary=all
if %errorlevel% neq 0 (
    echo FastAPI installation failed, trying without version constraint...
    timeout 60 python -m pip install fastapi -i https://pypi.tuna.tsinghua.edu.cn/simple/ --prefer-binary
)

echo Installing Uvicorn...
timeout 60 python -m pip install uvicorn -i https://pypi.tuna.tsinghua.edu.cn/simple/ --prefer-binary --only-binary=all
if %errorlevel% neq 0 (
    echo Uvicorn installation failed, trying basic version...
    timeout 60 python -m pip install uvicorn -i https://pypi.tuna.tsinghua.edu.cn/simple/ --prefer-binary
)

echo Installing Pydantic (this might take a while)...
timeout 120 python -m pip install pydantic==2.4.2 -i https://pypi.tuna.tsinghua.edu.cn/simple/ --prefer-binary --only-binary=all
if %errorlevel% neq 0 (
    echo Pydantic 2.4.2 failed, trying 2.3.0...
    timeout 120 python -m pip install pydantic==2.3.0 -i https://pypi.tuna.tsinghua.edu.cn/simple/ --prefer-binary --only-binary=all
    if %errorlevel% neq 0 (
        echo Pydantic 2.3.0 failed, trying latest...
        timeout 120 python -m pip install pydantic -i https://pypi.tuna.tsinghua.edu.cn/simple/ --prefer-binary
    )
)

echo Installing PyInstaller...
timeout 60 python -m pip install pyinstaller -i https://pypi.tuna.tsinghua.edu.cn/simple/ --prefer-binary --only-binary=all

echo.
echo Dependency installation completed!
echo Now you can run: python build_backend_wheel.py
echo.

pause
