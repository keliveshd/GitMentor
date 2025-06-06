"""
Agent配置API
"""

from typing import List, Dict, Any, Optional
from fastapi import APIRouter, HTTPException
from pydantic import BaseModel
import logging

from app.config.agent_config import agent_config_manager
from app.agents.agent_manager import agent_manager

router = APIRouter()
logger = logging.getLogger(__name__)

class AgentConfigUpdateRequest(BaseModel):
    config: Dict[str, Any]

class AgentConfigImportRequest(BaseModel):
    config_data: str
    format: str = "yaml"

@router.get("/agent-configs")
async def get_agent_configs():
    """获取所有Agent配置"""
    try:
        agent_types = agent_config_manager.list_agent_types()
        configs = {}
        
        for agent_type in agent_types:
            config = agent_config_manager.get_config(agent_type)
            version = agent_config_manager.get_config_version(agent_type)
            configs[agent_type] = {
                "config": config,
                "version": version
            }
        
        return {"agent_configs": configs}
    except Exception as e:
        logger.error(f"获取Agent配置失败: {str(e)}")
        raise HTTPException(status_code=500, detail=f"获取Agent配置失败: {str(e)}")

@router.get("/agent-configs/{agent_type}")
async def get_agent_config(agent_type: str):
    """获取单个Agent配置"""
    try:
        config = agent_config_manager.get_config(agent_type)
        if not config:
            raise HTTPException(status_code=404, detail="Agent配置不存在")
        
        version = agent_config_manager.get_config_version(agent_type)
        
        return {
            "agent_type": agent_type,
            "config": config,
            "version": version
        }
    except HTTPException:
        raise
    except Exception as e:
        logger.error(f"获取Agent配置失败: {str(e)}")
        raise HTTPException(status_code=500, detail=f"获取Agent配置失败: {str(e)}")

@router.put("/agent-configs/{agent_type}")
async def update_agent_config(agent_type: str, config_request: AgentConfigUpdateRequest):
    """更新Agent配置"""
    try:
        success = agent_config_manager.update_config(agent_type, config_request.config)
        if not success:
            raise HTTPException(status_code=400, detail="更新Agent配置失败")
        
        # 如果有运行中的Agent实例，重新加载配置
        try:
            agent_id = f"default_{agent_type}"
            await agent_manager.reload_agent_config(agent_id, config_request.config)
            logger.info(f"Agent {agent_id} 配置已热重载")
        except Exception as e:
            logger.warning(f"Agent配置热重载失败: {str(e)}")
        
        return {"message": "Agent配置更新成功"}
    except HTTPException:
        raise
    except Exception as e:
        logger.error(f"更新Agent配置失败: {str(e)}")
        raise HTTPException(status_code=500, detail=f"更新Agent配置失败: {str(e)}")

@router.post("/agent-configs/{agent_type}/reload")
async def reload_agent_config(agent_type: str):
    """重新加载Agent配置"""
    try:
        success = agent_config_manager.reload_config(agent_type)
        if not success:
            raise HTTPException(status_code=404, detail="Agent配置不存在")
        
        # 重新加载运行中的Agent实例
        try:
            agent_id = f"default_{agent_type}"
            config = agent_config_manager.get_config(agent_type)
            if config:
                await agent_manager.reload_agent_config(agent_id, config)
                logger.info(f"Agent {agent_id} 配置已重新加载")
        except Exception as e:
            logger.warning(f"Agent实例重新加载失败: {str(e)}")
        
        return {"message": "Agent配置重新加载成功"}
    except HTTPException:
        raise
    except Exception as e:
        logger.error(f"重新加载Agent配置失败: {str(e)}")
        raise HTTPException(status_code=500, detail=f"重新加载Agent配置失败: {str(e)}")

@router.get("/agent-configs/{agent_type}/export")
async def export_agent_config(agent_type: str, format: str = "yaml"):
    """导出Agent配置"""
    try:
        config_data = agent_config_manager.export_config(agent_type, format)
        if not config_data:
            raise HTTPException(status_code=404, detail="Agent配置不存在")
        
        return {
            "agent_type": agent_type,
            "format": format,
            "config_data": config_data
        }
    except HTTPException:
        raise
    except Exception as e:
        logger.error(f"导出Agent配置失败: {str(e)}")
        raise HTTPException(status_code=500, detail=f"导出Agent配置失败: {str(e)}")

@router.post("/agent-configs/{agent_type}/import")
async def import_agent_config(agent_type: str, import_request: AgentConfigImportRequest):
    """导入Agent配置"""
    try:
        success = agent_config_manager.import_config(
            agent_type, 
            import_request.config_data, 
            import_request.format
        )
        if not success:
            raise HTTPException(status_code=400, detail="导入Agent配置失败")
        
        return {"message": "Agent配置导入成功"}
    except HTTPException:
        raise
    except Exception as e:
        logger.error(f"导入Agent配置失败: {str(e)}")
        raise HTTPException(status_code=500, detail=f"导入Agent配置失败: {str(e)}")

@router.get("/agent-configs/{agent_type}/history")
async def get_agent_config_history(agent_type: str):
    """获取Agent配置历史"""
    try:
        history = agent_config_manager.get_config_history(agent_type)
        return {
            "agent_type": agent_type,
            "history": history
        }
    except Exception as e:
        logger.error(f"获取Agent配置历史失败: {str(e)}")
        raise HTTPException(status_code=500, detail=f"获取Agent配置历史失败: {str(e)}")

@router.post("/agent-configs/{agent_type}/validate")
async def validate_agent_config(agent_type: str, config_request: AgentConfigUpdateRequest):
    """验证Agent配置"""
    try:
        from app.config.config_validator import ConfigValidator
        validator = ConfigValidator()
        
        is_valid = validator.validate_agent_config(agent_type, config_request.config)
        
        if is_valid:
            return {
                "valid": True,
                "message": "配置验证通过"
            }
        else:
            errors = validator.get_validation_errors("agent", config_request.config)
            return {
                "valid": False,
                "message": "配置验证失败",
                "errors": errors
            }
    except Exception as e:
        logger.error(f"验证Agent配置失败: {str(e)}")
        raise HTTPException(status_code=500, detail=f"验证Agent配置失败: {str(e)}")

@router.get("/agent-configs/{agent_type}/template")
async def get_agent_config_template(agent_type: str):
    """获取Agent配置模板"""
    try:
        if agent_type == "analyzer":
            template = {
                "name": "Git Commit Analyzer",
                "version": "1.0",
                "description": "分析Git提交记录并生成结构化总结",
                "llm_client": "openai",
                "prompt_template": "git_commit_analyzer",
                "max_tokens": 1000,
                "temperature": 0.3,
                "max_retries": 3,
                "timeout": 30,
                "capabilities": [
                    "commit_analysis",
                    "semantic_understanding",
                    "change_categorization",
                    "impact_assessment"
                ]
            }
        elif agent_type == "reviewer":
            template = {
                "name": "Quality Reviewer",
                "version": "1.0",
                "description": "审核分析结果的质量并提供反馈",
                "llm_client": "openai",
                "prompt_template": "quality_reviewer",
                "max_tokens": 800,
                "temperature": 0.2,
                "max_retries": 3,
                "approval_threshold": 0.85,
                "dimension_weights": {
                    "accuracy": 0.3,
                    "completeness": 0.25,
                    "consistency": 0.25,
                    "clarity": 0.2
                },
                "capabilities": [
                    "quality_assessment",
                    "accuracy_evaluation",
                    "completeness_check",
                    "consistency_validation"
                ]
            }
        else:
            raise HTTPException(status_code=404, detail="未知的Agent类型")
        
        return {
            "agent_type": agent_type,
            "template": template
        }
    except HTTPException:
        raise
    except Exception as e:
        logger.error(f"获取Agent配置模板失败: {str(e)}")
        raise HTTPException(status_code=500, detail=f"获取Agent配置模板失败: {str(e)}")

@router.get("/agent-types")
async def get_agent_types():
    """获取支持的Agent类型"""
    try:
        agent_types = agent_config_manager.list_agent_types()
        
        # 添加类型描述
        type_descriptions = {
            "analyzer": {
                "name": "分析器",
                "description": "分析Git提交记录并生成结构化总结",
                "capabilities": ["commit_analysis", "semantic_understanding", "change_categorization"]
            },
            "reviewer": {
                "name": "审核器", 
                "description": "审核分析结果的质量并提供反馈",
                "capabilities": ["quality_assessment", "accuracy_evaluation", "completeness_check"]
            }
        }
        
        result = []
        for agent_type in agent_types:
            info = type_descriptions.get(agent_type, {
                "name": agent_type.title(),
                "description": f"默认的{agent_type}配置",
                "capabilities": []
            })
            result.append({
                "type": agent_type,
                **info
            })
        
        return {"agent_types": result}
    except Exception as e:
        logger.error(f"获取Agent类型失败: {str(e)}")
        raise HTTPException(status_code=500, detail=f"获取Agent类型失败: {str(e)}")
