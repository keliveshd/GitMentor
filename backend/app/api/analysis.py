"""
分析结果API
"""

from typing import List, Dict, Any, Optional
from fastapi import APIRouter, HTTPException, Query
from pydantic import BaseModel
import logging

from app.agents.quality_controller import quality_controller, ProcessStatus

router = APIRouter()
logger = logging.getLogger(__name__)

class AnalysisResultResponse(BaseModel):
    task_id: str
    status: str
    commit_hash: str
    commit_message: str
    analysis_result: Optional[Dict[str, Any]] = None
    review_result: Optional[Dict[str, Any]] = None
    error_message: Optional[str] = None
    retry_count: int
    created_at: float
    completed_at: Optional[float] = None

class QualityTrendResponse(BaseModel):
    average_quality_score: float
    sample_count: int
    dimension_averages: Dict[str, float]

class ProcessingStatsResponse(BaseModel):
    total_tasks: int
    approved_count: int
    rejected_count: int
    error_count: int
    pending_count: int
    approval_rate: float
    average_processing_time: float

@router.get("/analysis/results", response_model=List[AnalysisResultResponse])
async def get_analysis_results(
    status: Optional[str] = Query(None, description="筛选状态"),
    limit: int = Query(50, description="返回数量限制"),
    offset: int = Query(0, description="偏移量")
):
    """获取分析结果列表"""
    try:
        if status:
            try:
                status_enum = ProcessStatus(status)
                results = await quality_controller.get_results_by_status(status_enum)
            except ValueError:
                raise HTTPException(status_code=400, detail=f"无效的状态值: {status}")
        else:
            # 获取所有结果
            all_results = list(quality_controller.results.values())
            # 按创建时间倒序排序
            results = sorted(all_results, key=lambda x: x.created_at, reverse=True)
        
        # 应用分页
        paginated_results = results[offset:offset + limit]
        
        return [
            AnalysisResultResponse(
                task_id=result.task_id,
                status=result.status.value,
                commit_hash=result.commit_data.get("hash", ""),
                commit_message=result.commit_data.get("message", ""),
                analysis_result=result.analysis_result,
                review_result=result.review_result,
                error_message=result.error_message,
                retry_count=result.retry_count,
                created_at=result.created_at,
                completed_at=result.completed_at
            )
            for result in paginated_results
        ]
        
    except HTTPException:
        raise
    except Exception as e:
        logger.error(f"获取分析结果失败: {str(e)}")
        raise HTTPException(status_code=500, detail=f"获取分析结果失败: {str(e)}")

@router.get("/analysis/results/{task_id}")
async def get_analysis_result(task_id: str):
    """获取单个分析结果"""
    try:
        result = await quality_controller.get_result(task_id)
        if not result:
            raise HTTPException(status_code=404, detail="分析结果不存在")
        
        return AnalysisResultResponse(
            task_id=result.task_id,
            status=result.status.value,
            commit_hash=result.commit_data.get("hash", ""),
            commit_message=result.commit_data.get("message", ""),
            analysis_result=result.analysis_result,
            review_result=result.review_result,
            error_message=result.error_message,
            retry_count=result.retry_count,
            created_at=result.created_at,
            completed_at=result.completed_at
        )
        
    except HTTPException:
        raise
    except Exception as e:
        logger.error(f"获取分析结果失败: {str(e)}")
        raise HTTPException(status_code=500, detail=f"获取分析结果失败: {str(e)}")

@router.post("/analysis/results/{task_id}/retry")
async def retry_analysis(task_id: str):
    """重试分析任务"""
    try:
        result = await quality_controller.retry_failed_task(task_id)
        
        return {
            "message": "重试任务已提交",
            "new_task_id": result.task_id,
            "status": result.status.value
        }
        
    except ValueError as e:
        raise HTTPException(status_code=400, detail=str(e))
    except Exception as e:
        logger.error(f"重试分析任务失败: {str(e)}")
        raise HTTPException(status_code=500, detail=f"重试分析任务失败: {str(e)}")

@router.get("/analysis/statistics", response_model=ProcessingStatsResponse)
async def get_processing_statistics():
    """获取处理统计信息"""
    try:
        stats = await quality_controller.get_processing_statistics()
        
        return ProcessingStatsResponse(
            total_tasks=stats["total_tasks"],
            approved_count=stats["approved_count"],
            rejected_count=stats["rejected_count"],
            error_count=stats["error_count"],
            pending_count=stats["pending_count"],
            approval_rate=stats["approval_rate"],
            average_processing_time=stats["average_processing_time"]
        )
        
    except Exception as e:
        logger.error(f"获取处理统计失败: {str(e)}")
        raise HTTPException(status_code=500, detail=f"获取处理统计失败: {str(e)}")

@router.get("/analysis/quality-trends", response_model=QualityTrendResponse)
async def get_quality_trends(hours: int = Query(24, description="时间范围（小时）")):
    """获取质量趋势数据"""
    try:
        if hours <= 0 or hours > 168:  # 最多7天
            raise HTTPException(status_code=400, detail="时间范围必须在1-168小时之间")
        
        trends = await quality_controller.get_quality_trends(hours)
        
        return QualityTrendResponse(
            average_quality_score=trends["average_quality_score"],
            sample_count=trends["sample_count"],
            dimension_averages=trends["dimension_averages"]
        )
        
    except HTTPException:
        raise
    except Exception as e:
        logger.error(f"获取质量趋势失败: {str(e)}")
        raise HTTPException(status_code=500, detail=f"获取质量趋势失败: {str(e)}")

@router.get("/analysis/commit/{commit_hash}")
async def get_commit_analysis(commit_hash: str):
    """根据提交哈希获取分析结果"""
    try:
        # 在所有结果中查找匹配的提交哈希
        matching_results = []
        for result in quality_controller.results.values():
            if result.commit_data.get("hash", "").startswith(commit_hash):
                matching_results.append(result)
        
        if not matching_results:
            raise HTTPException(status_code=404, detail="未找到该提交的分析结果")
        
        # 返回最新的结果
        latest_result = max(matching_results, key=lambda x: x.created_at)
        
        return AnalysisResultResponse(
            task_id=latest_result.task_id,
            status=latest_result.status.value,
            commit_hash=latest_result.commit_data.get("hash", ""),
            commit_message=latest_result.commit_data.get("message", ""),
            analysis_result=latest_result.analysis_result,
            review_result=latest_result.review_result,
            error_message=latest_result.error_message,
            retry_count=latest_result.retry_count,
            created_at=latest_result.created_at,
            completed_at=latest_result.completed_at
        )
        
    except HTTPException:
        raise
    except Exception as e:
        logger.error(f"获取提交分析失败: {str(e)}")
        raise HTTPException(status_code=500, detail=f"获取提交分析失败: {str(e)}")

@router.delete("/analysis/results")
async def cleanup_old_results(max_age_hours: int = Query(24, description="最大保留时间（小时）")):
    """清理旧的分析结果"""
    try:
        if max_age_hours <= 0:
            raise HTTPException(status_code=400, detail="保留时间必须大于0")
        
        await quality_controller.cleanup_old_results(max_age_hours)
        
        return {"message": f"已清理超过 {max_age_hours} 小时的旧结果"}
        
    except HTTPException:
        raise
    except Exception as e:
        logger.error(f"清理旧结果失败: {str(e)}")
        raise HTTPException(status_code=500, detail=f"清理旧结果失败: {str(e)}")

@router.get("/analysis/dashboard")
async def get_analysis_dashboard():
    """获取分析仪表板数据"""
    try:
        # 获取统计信息
        stats = await quality_controller.get_processing_statistics()
        
        # 获取质量趋势
        trends = await quality_controller.get_quality_trends(24)
        
        # 获取最近的结果
        recent_results = []
        all_results = list(quality_controller.results.values())
        sorted_results = sorted(all_results, key=lambda x: x.created_at, reverse=True)
        
        for result in sorted_results[:10]:  # 最近10个结果
            recent_results.append({
                "task_id": result.task_id,
                "status": result.status.value,
                "commit_hash": result.commit_data.get("hash", "")[:8],
                "commit_message": result.commit_data.get("message", "")[:100],
                "quality_score": result.review_result.get("overall_score") if result.review_result else None,
                "created_at": result.created_at
            })
        
        return {
            "statistics": stats,
            "quality_trends": trends,
            "recent_results": recent_results,
            "timestamp": time.time()
        }
        
    except Exception as e:
        logger.error(f"获取仪表板数据失败: {str(e)}")
        raise HTTPException(status_code=500, detail=f"获取仪表板数据失败: {str(e)}")

# 需要导入time模块
import time
