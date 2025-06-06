#!/usr/bin/env python3
"""
GitMentor Backend Server
基于FastAPI的Git分析后端服务
"""

import sys
from pathlib import Path
from fastapi import FastAPI, Request
from fastapi.middleware.cors import CORSMiddleware
from fastapi.responses import JSONResponse
import uvicorn

# 添加app目录到Python路径
sys.path.append(str(Path(__file__).parent / "app"))

from app.api.repositories import router as repositories_router
from app.api.commits import router as commits_router
from app.api.health import router as health_router
from app.api.agents import router as agents_router
from app.api.analysis import router as analysis_router
from app.api.repository_config import router as repository_config_router
from app.api.agent_config import router as agent_config_router
from app.api.monitoring import router as monitoring_router
from app.core.database import init_db
from app.core.initialization import startup_initialization, shutdown_cleanup

# 创建FastAPI应用
app = FastAPI(
    title="GitMentor API",
    description="Git提交分析工具后端API",
    version="0.1.0"
)

# 配置CORS
app.add_middleware(
    CORSMiddleware,
    allow_origins=["http://localhost:1420"],  # Tauri前端地址
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# 注册路由
app.include_router(health_router, prefix="/api", tags=["健康检查"])
app.include_router(repositories_router, prefix="/api", tags=["仓库管理"])
app.include_router(commits_router, prefix="/api", tags=["提交分析"])
app.include_router(agents_router, prefix="/api", tags=["Agent管理"])
app.include_router(analysis_router, prefix="/api", tags=["分析结果"])
app.include_router(repository_config_router, prefix="/api", tags=["仓库配置"])
app.include_router(agent_config_router, prefix="/api", tags=["Agent配置"])
app.include_router(monitoring_router, prefix="/api", tags=["监控"])

@app.on_event("startup")
async def startup_event():
    """应用启动时初始化"""
    try:
        # 初始化数据库
        init_db()
        print("✅ 数据库初始化成功")

        # 初始化AI服务和Agent (可选)
        try:
            await startup_initialization()
            print("✅ AI服务初始化成功")
        except Exception as e:
            print(f"⚠️ AI服务初始化失败，但继续启动: {e}")

    except Exception as e:
        print(f"❌ 应用初始化失败: {e}")

@app.on_event("shutdown")
async def shutdown_event():
    """应用关闭时清理"""
    try:
        await shutdown_cleanup()
        print("✅ 应用清理完成")
    except Exception as e:
        print(f"❌ 应用清理失败: {e}")

@app.exception_handler(Exception)
async def global_exception_handler(request: Request, exc: Exception):
    """全局异常处理"""
    return JSONResponse(
        status_code=500,
        content={"detail": f"内部服务器错误: {str(exc)}"}
    )

if __name__ == "__main__":
    # 开发模式运行
    uvicorn.run(
        "main:app",
        host="127.0.0.1",
        port=8000,
        reload=True,
        log_level="info"
    )
