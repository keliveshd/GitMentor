"""
Agent管理API
"""

from typing import List, Dict, Any, Optional
from fastapi import APIRouter, HTTPException, BackgroundTasks
from pydantic import BaseModel
import logging

from app.agents.agent_manager import agent_manager
from app.agents.quality_controller import quality_controller
from app.services.ai_service import ai_service

router = APIRouter()
logger = logging.getLogger(__name__)

class AgentConfigRequest(BaseModel):
    agent_type: str
    config: Dict[str, Any]

class AnalysisRequest(BaseModel):
    commit_hash: str
    commit_message: str
    author_name: str
    author_email: str
    commit_date: str
    files_changed: int = 0
    insertions: int = 0
    deletions: int = 0

class AgentStatusResponse(BaseModel):
    agent_id: str
    status: str
    capabilities: List[str]
    metrics: Dict[str, Any]
    config_version: str

@router.get("/agents", response_model=List[AgentStatusResponse])
async def get_agents():
    """获取所有Agent状态"""
    try:
        agent_status = agent_manager.get_all_agent_status()
        
        return [
            AgentStatusResponse(
                agent_id=agent_id,
                status=status["status"],
                capabilities=status["capabilities"],
                metrics=status["metrics"],
                config_version=status["config_version"]
            )
            for agent_id, status in agent_status.items()
        ]
    except Exception as e:
        logger.error(f"获取Agent列表失败: {str(e)}")
        raise HTTPException(status_code=500, detail=f"获取Agent列表失败: {str(e)}")

@router.get("/agents/{agent_id}")
async def get_agent_status(agent_id: str):
    """获取单个Agent状态"""
    try:
        status = agent_manager.get_agent_status(agent_id)
        if not status:
            raise HTTPException(status_code=404, detail="Agent不存在")
        
        return status
    except HTTPException:
        raise
    except Exception as e:
        logger.error(f"获取Agent状态失败: {str(e)}")
        raise HTTPException(status_code=500, detail=f"获取Agent状态失败: {str(e)}")

@router.post("/agents/{agent_id}/config")
async def update_agent_config(agent_id: str, config_request: AgentConfigRequest):
    """更新Agent配置"""
    try:
        await agent_manager.reload_agent_config(agent_id, config_request.config)
        return {"message": "配置更新成功"}
    except ValueError as e:
        raise HTTPException(status_code=404, detail=str(e))
    except Exception as e:
        logger.error(f"更新Agent配置失败: {str(e)}")
        raise HTTPException(status_code=500, detail=f"更新Agent配置失败: {str(e)}")

@router.post("/agents/create")
async def create_agent(config_request: AgentConfigRequest):
    """创建新的Agent"""
    try:
        # 生成Agent ID
        agent_id = f"{config_request.agent_type}_{int(time.time())}"
        
        agent = await agent_manager.create_agent(
            agent_id, 
            config_request.agent_type, 
            config_request.config
        )
        
        return {
            "agent_id": agent_id,
            "message": "Agent创建成功",
            "status": agent.get_status()
        }
    except Exception as e:
        logger.error(f"创建Agent失败: {str(e)}")
        raise HTTPException(status_code=500, detail=f"创建Agent失败: {str(e)}")

@router.delete("/agents/{agent_id}")
async def remove_agent(agent_id: str):
    """移除Agent"""
    try:
        await agent_manager.remove_agent(agent_id)
        return {"message": "Agent移除成功"}
    except Exception as e:
        logger.error(f"移除Agent失败: {str(e)}")
        raise HTTPException(status_code=500, detail=f"移除Agent失败: {str(e)}")

@router.get("/agents/health")
async def check_agents_health():
    """检查所有Agent健康状态"""
    try:
        health_results = await agent_manager.health_check_all()
        return {
            "healthy_agents": sum(1 for healthy in health_results.values() if healthy),
            "total_agents": len(health_results),
            "details": health_results
        }
    except Exception as e:
        logger.error(f"健康检查失败: {str(e)}")
        raise HTTPException(status_code=500, detail=f"健康检查失败: {str(e)}")

@router.post("/agents/analyze")
async def analyze_commit(analysis_request: AnalysisRequest, background_tasks: BackgroundTasks):
    """分析Git提交"""
    try:
        # 构建提交数据
        commit_data = {
            "hash": analysis_request.commit_hash,
            "message": analysis_request.commit_message,
            "author_name": analysis_request.author_name,
            "author_email": analysis_request.author_email,
            "commit_date": analysis_request.commit_date,
            "files_changed": analysis_request.files_changed,
            "insertions": analysis_request.insertions,
            "deletions": analysis_request.deletions
        }
        
        # 提交到后台处理
        background_tasks.add_task(
            _process_commit_analysis_background,
            commit_data
        )
        
        return {
            "message": "分析任务已提交",
            "commit_hash": analysis_request.commit_hash
        }
        
    except Exception as e:
        logger.error(f"提交分析任务失败: {str(e)}")
        raise HTTPException(status_code=500, detail=f"提交分析任务失败: {str(e)}")

@router.get("/agents/metrics")
async def get_system_metrics():
    """获取系统指标"""
    try:
        agent_metrics = await agent_manager.get_system_metrics()
        quality_stats = await quality_controller.get_processing_statistics()
        
        return {
            "agent_metrics": agent_metrics,
            "quality_statistics": quality_stats,
            "timestamp": time.time()
        }
    except Exception as e:
        logger.error(f"获取系统指标失败: {str(e)}")
        raise HTTPException(status_code=500, detail=f"获取系统指标失败: {str(e)}")

@router.get("/agents/llm-clients")
async def get_llm_clients():
    """获取可用的LLM客户端"""
    try:
        clients = ai_service.get_available_clients()
        connection_status = await ai_service.test_all_connections()
        
        return {
            "available_clients": clients,
            "connection_status": connection_status
        }
    except Exception as e:
        logger.error(f"获取LLM客户端失败: {str(e)}")
        raise HTTPException(status_code=500, detail=f"获取LLM客户端失败: {str(e)}")

@router.post("/agents/llm-clients/test")
async def test_llm_connection(client_name: Optional[str] = None):
    """测试LLM连接"""
    try:
        if client_name:
            client = ai_service.get_client(client_name)
            result = await client.test_connection()
            return {"client": client_name, "connected": result}
        else:
            results = await ai_service.test_all_connections()
            return {"connection_results": results}
    except Exception as e:
        logger.error(f"测试LLM连接失败: {str(e)}")
        raise HTTPException(status_code=500, detail=f"测试LLM连接失败: {str(e)}")

async def _process_commit_analysis_background(commit_data: Dict[str, Any]):
    """后台处理提交分析"""
    try:
        result = await quality_controller.process_commit_analysis(commit_data)
        logger.info(f"提交分析完成: {result.task_id}, 状态: {result.status.value}")
    except Exception as e:
        logger.error(f"后台分析任务失败: {str(e)}")

# 需要导入time模块
import time
