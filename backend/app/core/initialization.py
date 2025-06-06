"""
应用初始化模块
"""

import os
import logging
from typing import Dict, Any
from app.services.ai_service import ai_service, LLMProvider
from app.services.prompt_manager import prompt_manager
from app.agents.agent_manager import agent_manager

logger = logging.getLogger(__name__)

async def initialize_ai_services():
    """初始化AI服务"""
    try:
        # 初始化默认的LLM客户端配置
        default_configs = {
            "openai": {
                "api_key": os.getenv("OPENAI_API_KEY", ""),
                "model": "gpt-3.5-turbo",
                "base_url": "https://api.openai.com/v1",
                "default": True
            },
            "anthropic": {
                "api_key": os.getenv("ANTHROPIC_API_KEY", ""),
                "model": "claude-3-sonnet-20240229",
                "base_url": "https://api.anthropic.com/v1"
            },
            "local": {
                "base_url": os.getenv("LOCAL_LLM_URL", "http://localhost:11434"),
                "model": "llama2"
            }
        }
        
        # 注册可用的LLM客户端
        for name, config in default_configs.items():
            if name == "openai" and config["api_key"]:
                ai_service.register_client(name, LLMProvider.OPENAI, config)
                logger.info(f"注册OpenAI客户端成功")
            elif name == "anthropic" and config["api_key"]:
                ai_service.register_client(name, LLMProvider.ANTHROPIC, config)
                logger.info(f"注册Anthropic客户端成功")
            elif name == "local":
                try:
                    ai_service.register_client(name, LLMProvider.LOCAL, config)
                    logger.info(f"注册本地LLM客户端成功")
                except Exception as e:
                    logger.warning(f"注册本地LLM客户端失败: {str(e)}")
        
        # 检查是否有可用的客户端
        available_clients = ai_service.get_available_clients()
        if not available_clients:
            logger.warning("没有可用的LLM客户端，请配置API密钥")
        else:
            logger.info(f"可用的LLM客户端: {available_clients}")
        
    except Exception as e:
        logger.error(f"初始化AI服务失败: {str(e)}")
        raise

async def initialize_agents():
    """初始化默认Agent"""
    try:
        # 检查是否有可用的LLM客户端
        available_clients = ai_service.get_available_clients()
        if not available_clients:
            logger.warning("没有可用的LLM客户端，跳过Agent初始化")
            return
        
        default_client = available_clients[0]
        
        # 创建默认的分析Agent
        analyzer_config = {
            "llm_client": default_client,
            "prompt_template": "git_commit_analyzer",
            "max_tokens": 1000,
            "temperature": 0.3,
            "max_retries": 3,
            "timeout": 30,
            "version": "1.0"
        }
        
        try:
            await agent_manager.create_agent("default_analyzer", "analyzer", analyzer_config)
            logger.info("创建默认分析Agent成功")
        except Exception as e:
            logger.error(f"创建默认分析Agent失败: {str(e)}")
        
        # 创建默认的审核Agent
        reviewer_config = {
            "llm_client": default_client,
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
            "version": "1.0"
        }
        
        try:
            await agent_manager.create_agent("default_reviewer", "reviewer", reviewer_config)
            logger.info("创建默认审核Agent成功")
        except Exception as e:
            logger.error(f"创建默认审核Agent失败: {str(e)}")
        
        # 启动Agent工作线程
        await agent_manager.start_workers(num_workers=2)
        logger.info("Agent工作线程启动成功")
        
    except Exception as e:
        logger.error(f"初始化Agent失败: {str(e)}")
        raise

async def initialize_prompt_templates():
    """初始化Prompt模板"""
    try:
        # Prompt管理器会自动加载模板，如果没有则创建默认模板
        templates = prompt_manager.list_templates()
        logger.info(f"加载了 {len(templates)} 个Prompt模板: {templates}")
        
    except Exception as e:
        logger.error(f"初始化Prompt模板失败: {str(e)}")
        raise

async def startup_initialization():
    """应用启动时的完整初始化"""
    logger.info("开始应用初始化...")
    
    try:
        # 1. 初始化Prompt模板
        await initialize_prompt_templates()
        
        # 2. 初始化AI服务
        await initialize_ai_services()
        
        # 3. 初始化Agent
        await initialize_agents()
        
        logger.info("应用初始化完成")
        
    except Exception as e:
        logger.error(f"应用初始化失败: {str(e)}")
        # 不抛出异常，允许应用继续启动
        # 用户可以稍后手动配置

async def shutdown_cleanup():
    """应用关闭时的清理"""
    logger.info("开始应用清理...")
    
    try:
        # 关闭Agent管理器
        await agent_manager.shutdown()
        logger.info("Agent管理器已关闭")
        
    except Exception as e:
        logger.error(f"应用清理失败: {str(e)}")
    
    logger.info("应用清理完成")

def get_initialization_status() -> Dict[str, Any]:
    """获取初始化状态"""
    try:
        # 检查AI服务状态
        available_clients = ai_service.get_available_clients()
        
        # 检查Agent状态
        agent_status = agent_manager.get_all_agent_status()
        
        # 检查Prompt模板状态
        templates = prompt_manager.list_templates()
        
        return {
            "ai_services": {
                "available_clients": available_clients,
                "client_count": len(available_clients)
            },
            "agents": {
                "total_agents": len(agent_status),
                "agent_details": agent_status
            },
            "prompt_templates": {
                "total_templates": len(templates),
                "template_names": templates
            },
            "status": "initialized" if available_clients and agent_status else "partial"
        }
        
    except Exception as e:
        logger.error(f"获取初始化状态失败: {str(e)}")
        return {
            "status": "error",
            "error": str(e)
        }
