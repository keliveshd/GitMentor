"""
仓库配置API
"""

from typing import List, Dict, Any, Optional
from fastapi import APIRouter, HTTPException
from pydantic import BaseModel
import logging

from app.config.repository_config import repository_config_manager
from app.config.agent_config import agent_config_manager

router = APIRouter()
logger = logging.getLogger(__name__)

class RepositoryConfigRequest(BaseModel):
    name: str
    path: str
    type: str = "git"
    enabled: bool = True
    agents: Dict[str, str] = {}
    user_mapping: Dict[str, Any] = {}
    analysis_settings: Dict[str, Any] = {}

class UserMappingRequest(BaseModel):
    user_mapping: Dict[str, Any]

class AgentConfigRequest(BaseModel):
    agents: Dict[str, str]

@router.get("/repository-configs")
async def get_repository_configs():
    """获取所有仓库配置"""
    try:
        repositories = repository_config_manager.list_repositories()
        configs = []
        
        for repo_name in repositories:
            config = repository_config_manager.get_repository(repo_name)
            if config:
                configs.append(config)
        
        return {"repositories": configs}
    except Exception as e:
        logger.error(f"获取仓库配置失败: {str(e)}")
        raise HTTPException(status_code=500, detail=f"获取仓库配置失败: {str(e)}")

@router.get("/repository-configs/{repo_name}")
async def get_repository_config(repo_name: str):
    """获取单个仓库配置"""
    try:
        config = repository_config_manager.get_repository(repo_name)
        if not config:
            raise HTTPException(status_code=404, detail="仓库配置不存在")
        
        return config
    except HTTPException:
        raise
    except Exception as e:
        logger.error(f"获取仓库配置失败: {str(e)}")
        raise HTTPException(status_code=500, detail=f"获取仓库配置失败: {str(e)}")

@router.post("/repository-configs")
async def create_repository_config(config_request: RepositoryConfigRequest):
    """创建仓库配置"""
    try:
        config_data = config_request.dict()
        
        # 设置默认的分析设置
        if not config_data.get("analysis_settings"):
            config_data["analysis_settings"] = {
                "auto_analysis": True,
                "batch_size": 10,
                "retry_limit": 3,
                "quality_threshold": 0.85
            }
        
        # 设置默认的Agent配置
        if not config_data.get("agents"):
            config_data["agents"] = {
                "analyzer": "default_analyzer",
                "reviewer": "default_reviewer"
            }
        
        success = repository_config_manager.add_repository(config_data)
        if not success:
            raise HTTPException(status_code=400, detail="创建仓库配置失败")
        
        return {"message": "仓库配置创建成功", "name": config_request.name}
    except HTTPException:
        raise
    except Exception as e:
        logger.error(f"创建仓库配置失败: {str(e)}")
        raise HTTPException(status_code=500, detail=f"创建仓库配置失败: {str(e)}")

@router.put("/repository-configs/{repo_name}")
async def update_repository_config(repo_name: str, config_request: RepositoryConfigRequest):
    """更新仓库配置"""
    try:
        config_data = config_request.dict()
        config_data["name"] = repo_name  # 确保名称一致
        
        success = repository_config_manager.update_repository(repo_name, config_data)
        if not success:
            raise HTTPException(status_code=400, detail="更新仓库配置失败")
        
        return {"message": "仓库配置更新成功"}
    except HTTPException:
        raise
    except Exception as e:
        logger.error(f"更新仓库配置失败: {str(e)}")
        raise HTTPException(status_code=500, detail=f"更新仓库配置失败: {str(e)}")

@router.delete("/repository-configs/{repo_name}")
async def delete_repository_config(repo_name: str):
    """删除仓库配置"""
    try:
        success = repository_config_manager.remove_repository(repo_name)
        if not success:
            raise HTTPException(status_code=404, detail="仓库配置不存在")
        
        return {"message": "仓库配置删除成功"}
    except HTTPException:
        raise
    except Exception as e:
        logger.error(f"删除仓库配置失败: {str(e)}")
        raise HTTPException(status_code=500, detail=f"删除仓库配置失败: {str(e)}")

@router.post("/repository-configs/{repo_name}/enable")
async def enable_repository(repo_name: str):
    """启用仓库"""
    try:
        success = repository_config_manager.enable_repository(repo_name)
        if not success:
            raise HTTPException(status_code=404, detail="仓库配置不存在")
        
        return {"message": "仓库已启用"}
    except HTTPException:
        raise
    except Exception as e:
        logger.error(f"启用仓库失败: {str(e)}")
        raise HTTPException(status_code=500, detail=f"启用仓库失败: {str(e)}")

@router.post("/repository-configs/{repo_name}/disable")
async def disable_repository(repo_name: str):
    """禁用仓库"""
    try:
        success = repository_config_manager.disable_repository(repo_name)
        if not success:
            raise HTTPException(status_code=404, detail="仓库配置不存在")
        
        return {"message": "仓库已禁用"}
    except HTTPException:
        raise
    except Exception as e:
        logger.error(f"禁用仓库失败: {str(e)}")
        raise HTTPException(status_code=500, detail=f"禁用仓库失败: {str(e)}")

@router.get("/repository-configs/{repo_name}/user-mapping")
async def get_user_mapping(repo_name: str):
    """获取用户映射"""
    try:
        user_mapping = repository_config_manager.get_user_mapping(repo_name)
        return {"user_mapping": user_mapping}
    except Exception as e:
        logger.error(f"获取用户映射失败: {str(e)}")
        raise HTTPException(status_code=500, detail=f"获取用户映射失败: {str(e)}")

@router.put("/repository-configs/{repo_name}/user-mapping")
async def update_user_mapping(repo_name: str, mapping_request: UserMappingRequest):
    """更新用户映射"""
    try:
        success = repository_config_manager.update_user_mapping(repo_name, mapping_request.user_mapping)
        if not success:
            raise HTTPException(status_code=404, detail="仓库配置不存在")
        
        return {"message": "用户映射更新成功"}
    except HTTPException:
        raise
    except Exception as e:
        logger.error(f"更新用户映射失败: {str(e)}")
        raise HTTPException(status_code=500, detail=f"更新用户映射失败: {str(e)}")

@router.get("/repository-configs/{repo_name}/agents")
async def get_repository_agents(repo_name: str):
    """获取仓库的Agent配置"""
    try:
        agent_config = repository_config_manager.get_agent_config(repo_name)
        return {"agents": agent_config}
    except Exception as e:
        logger.error(f"获取Agent配置失败: {str(e)}")
        raise HTTPException(status_code=500, detail=f"获取Agent配置失败: {str(e)}")

@router.put("/repository-configs/{repo_name}/agents")
async def update_repository_agents(repo_name: str, agent_request: AgentConfigRequest):
    """更新仓库的Agent配置"""
    try:
        success = repository_config_manager.update_agent_config(repo_name, agent_request.agents)
        if not success:
            raise HTTPException(status_code=404, detail="仓库配置不存在")
        
        return {"message": "Agent配置更新成功"}
    except HTTPException:
        raise
    except Exception as e:
        logger.error(f"更新Agent配置失败: {str(e)}")
        raise HTTPException(status_code=500, detail=f"更新Agent配置失败: {str(e)}")

@router.post("/repository-configs/{repo_name}/validate-path")
async def validate_repository_path(repo_name: str):
    """验证仓库路径"""
    try:
        config = repository_config_manager.get_repository(repo_name)
        if not config:
            raise HTTPException(status_code=404, detail="仓库配置不存在")
        
        path = config.get("path", "")
        is_valid = repository_config_manager.validate_repository_path(path)
        
        return {
            "valid": is_valid,
            "path": path,
            "message": "路径有效" if is_valid else "路径无效或不存在"
        }
    except HTTPException:
        raise
    except Exception as e:
        logger.error(f"验证仓库路径失败: {str(e)}")
        raise HTTPException(status_code=500, detail=f"验证仓库路径失败: {str(e)}")

@router.get("/repository-configs/enabled")
async def get_enabled_repositories():
    """获取启用的仓库列表"""
    try:
        enabled_repos = repository_config_manager.get_enabled_repositories()
        return {"repositories": enabled_repos}
    except Exception as e:
        logger.error(f"获取启用仓库失败: {str(e)}")
        raise HTTPException(status_code=500, detail=f"获取启用仓库失败: {str(e)}")

@router.get("/agent-types")
async def get_available_agent_types():
    """获取可用的Agent类型"""
    try:
        agent_types = agent_config_manager.list_agent_types()
        return {"agent_types": agent_types}
    except Exception as e:
        logger.error(f"获取Agent类型失败: {str(e)}")
        raise HTTPException(status_code=500, detail=f"获取Agent类型失败: {str(e)}")
