"""
监控API
"""

from typing import List, Dict, Any, Optional
from fastapi import APIRouter, HTTPException, Query
import logging

from app.core.monitoring import metrics_collector, system_monitor, app_monitor
from app.core.cache_manager import cache_manager
from app.agents.agent_manager import agent_manager

router = APIRouter()
logger = logging.getLogger(__name__)

@router.get("/monitoring/health")
async def get_health_status():
    """获取系统健康状态"""
    try:
        # 获取系统状态
        system_status = system_monitor.get_system_status()
        
        # 获取应用健康状态
        app_health = app_monitor.get_application_health()
        
        # 获取Agent健康状态
        agent_health = await agent_manager.health_check_all()
        healthy_agents = sum(1 for status in agent_health.values() if status)
        total_agents = len(agent_health)
        
        # 获取缓存状态
        cache_stats = cache_manager.get_stats()
        
        # 综合健康评分
        health_score = 100
        
        # 系统资源检查
        if system_status.get("cpu", {}).get("usage_percent", 0) > 80:
            health_score -= 20
        if system_status.get("memory", {}).get("percent", 0) > 85:
            health_score -= 20
        if system_status.get("disk", {}).get("percent", 0) > 90:
            health_score -= 15
        
        # 应用状态检查
        if app_health.get("error_rate", 0) > 5:
            health_score -= 25
        if app_health.get("agent_success_rate", 100) < 90:
            health_score -= 20
        
        # Agent状态检查
        if total_agents > 0 and (healthy_agents / total_agents) < 0.8:
            health_score -= 15
        
        overall_status = "healthy"
        if health_score < 70:
            overall_status = "critical"
        elif health_score < 85:
            overall_status = "degraded"
        
        return {
            "status": overall_status,
            "health_score": max(0, health_score),
            "timestamp": system_status.get("boot_time"),
            "system": system_status,
            "application": app_health,
            "agents": {
                "healthy": healthy_agents,
                "total": total_agents,
                "details": agent_health
            },
            "cache": cache_stats
        }
    except Exception as e:
        logger.error(f"获取健康状态失败: {str(e)}")
        raise HTTPException(status_code=500, detail=f"获取健康状态失败: {str(e)}")

@router.get("/monitoring/metrics")
async def get_metrics(
    metric_name: Optional[str] = Query(None, description="指标名称"),
    duration: int = Query(3600, description="时间范围（秒）")
):
    """获取监控指标"""
    try:
        if metric_name:
            # 获取特定指标
            history = metrics_collector.get_metric_history(metric_name, limit=100)
            summary = metrics_collector.get_metric_summary(metric_name, duration)
            
            return {
                "metric_name": metric_name,
                "summary": summary,
                "history": history
            }
        else:
            # 获取所有指标概览
            all_metrics = metrics_collector.get_all_metrics()
            metrics_summary = {}
            
            for name in all_metrics:
                metrics_summary[name] = metrics_collector.get_metric_summary(name, duration)
            
            return {
                "available_metrics": all_metrics,
                "summary": metrics_summary
            }
    except Exception as e:
        logger.error(f"获取监控指标失败: {str(e)}")
        raise HTTPException(status_code=500, detail=f"获取监控指标失败: {str(e)}")

@router.get("/monitoring/system")
async def get_system_metrics():
    """获取系统指标"""
    try:
        # 收集当前系统指标
        system_monitor.collect_system_metrics()
        
        # 获取系统状态
        system_status = system_monitor.get_system_status()
        
        # 获取最近的系统指标趋势
        cpu_trend = metrics_collector.get_metric_history("system.cpu.usage", 20)
        memory_trend = metrics_collector.get_metric_history("system.memory.usage", 20)
        disk_trend = metrics_collector.get_metric_history("system.disk.usage", 20)
        
        return {
            "current_status": system_status,
            "trends": {
                "cpu_usage": cpu_trend,
                "memory_usage": memory_trend,
                "disk_usage": disk_trend
            }
        }
    except Exception as e:
        logger.error(f"获取系统指标失败: {str(e)}")
        raise HTTPException(status_code=500, detail=f"获取系统指标失败: {str(e)}")

@router.get("/monitoring/application")
async def get_application_metrics():
    """获取应用指标"""
    try:
        # 获取应用健康状态
        app_health = app_monitor.get_application_health()
        
        # 获取请求指标趋势
        request_duration = metrics_collector.get_metric_summary("app.request.duration", 3600)
        request_count = metrics_collector.get_metric_summary("app.request.count", 3600)
        error_count = metrics_collector.get_metric_summary("app.request.errors", 3600)
        
        # 获取Agent执行指标
        agent_duration = metrics_collector.get_metric_summary("agent.execution.duration", 3600)
        agent_success = metrics_collector.get_metric_summary("agent.execution.success", 3600)
        agent_confidence = metrics_collector.get_metric_summary("agent.execution.confidence", 3600)
        
        return {
            "health": app_health,
            "requests": {
                "duration": request_duration,
                "count": request_count,
                "errors": error_count
            },
            "agents": {
                "execution_duration": agent_duration,
                "success_count": agent_success,
                "average_confidence": agent_confidence
            }
        }
    except Exception as e:
        logger.error(f"获取应用指标失败: {str(e)}")
        raise HTTPException(status_code=500, detail=f"获取应用指标失败: {str(e)}")

@router.get("/monitoring/cache")
async def get_cache_metrics():
    """获取缓存指标"""
    try:
        # 清理过期缓存
        expired_count = cache_manager.cleanup_expired()
        
        # 获取缓存统计
        cache_stats = cache_manager.get_stats()
        
        # 获取缓存项信息
        cache_info = cache_manager.get_cache_info()
        
        return {
            "statistics": cache_stats,
            "expired_cleaned": expired_count,
            "top_items": cache_info[:10],  # 前10个最常访问的项
            "total_items": len(cache_info)
        }
    except Exception as e:
        logger.error(f"获取缓存指标失败: {str(e)}")
        raise HTTPException(status_code=500, detail=f"获取缓存指标失败: {str(e)}")

@router.post("/monitoring/cache/clear")
async def clear_cache():
    """清空缓存"""
    try:
        cache_manager.clear()
        return {"message": "缓存已清空"}
    except Exception as e:
        logger.error(f"清空缓存失败: {str(e)}")
        raise HTTPException(status_code=500, detail=f"清空缓存失败: {str(e)}")

@router.get("/monitoring/quality")
async def get_quality_metrics(duration: int = Query(3600, description="时间范围（秒）")):
    """获取质量指标"""
    try:
        # 获取质量分数趋势
        overall_scores = metrics_collector.get_metric_summary("quality.overall_score", duration)
        
        # 获取各维度分数
        dimensions = ["accuracy", "completeness", "consistency", "clarity"]
        dimension_scores = {}
        
        for dimension in dimensions:
            dimension_scores[dimension] = metrics_collector.get_metric_summary(
                "quality.dimension_score", duration
            )
        
        # 获取最近的质量分数历史
        quality_history = metrics_collector.get_metric_history("quality.overall_score", 50)
        
        return {
            "overall_quality": overall_scores,
            "dimension_scores": dimension_scores,
            "quality_history": quality_history,
            "duration": duration
        }
    except Exception as e:
        logger.error(f"获取质量指标失败: {str(e)}")
        raise HTTPException(status_code=500, detail=f"获取质量指标失败: {str(e)}")

@router.get("/monitoring/alerts")
async def get_alerts():
    """获取告警信息"""
    try:
        alerts = []
        
        # 检查系统资源告警
        system_status = system_monitor.get_system_status()
        
        if system_status.get("cpu", {}).get("usage_percent", 0) > 80:
            alerts.append({
                "level": "warning",
                "type": "system",
                "message": f"CPU使用率过高: {system_status['cpu']['usage_percent']:.1f}%",
                "timestamp": time.time()
            })
        
        if system_status.get("memory", {}).get("percent", 0) > 85:
            alerts.append({
                "level": "warning", 
                "type": "system",
                "message": f"内存使用率过高: {system_status['memory']['percent']:.1f}%",
                "timestamp": time.time()
            })
        
        # 检查应用告警
        app_health = app_monitor.get_application_health()
        
        if app_health.get("error_rate", 0) > 5:
            alerts.append({
                "level": "error",
                "type": "application",
                "message": f"错误率过高: {app_health['error_rate']:.1f}%",
                "timestamp": time.time()
            })
        
        if app_health.get("agent_success_rate", 100) < 90:
            alerts.append({
                "level": "warning",
                "type": "agent",
                "message": f"Agent成功率过低: {app_health['agent_success_rate']:.1f}%",
                "timestamp": time.time()
            })
        
        return {
            "alerts": alerts,
            "total_count": len(alerts)
        }
    except Exception as e:
        logger.error(f"获取告警信息失败: {str(e)}")
        raise HTTPException(status_code=500, detail=f"获取告警信息失败: {str(e)}")

# 需要导入time模块
import time
