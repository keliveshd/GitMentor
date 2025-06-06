"""
质量控制器
"""

import uuid
import time
import asyncio
import logging
from typing import Dict, Any, Optional, List
from dataclasses import dataclass
from enum import Enum
from .base_agent import AgentInput, AgentOutput
from .agent_manager import agent_manager
from ..storage.storage_manager import storage_manager

class ProcessStatus(Enum):
    """处理状态枚举"""
    PENDING = "pending"
    ANALYZING = "analyzing"
    REVIEWING = "reviewing"
    APPROVED = "approved"
    REJECTED = "rejected"
    ERROR = "error"

@dataclass
class ProcessResult:
    """处理结果"""
    task_id: str
    status: ProcessStatus
    commit_data: Dict[str, Any]
    analysis_result: Optional[Dict[str, Any]] = None
    review_result: Optional[Dict[str, Any]] = None
    error_message: Optional[str] = None
    retry_count: int = 0
    created_at: float = None
    completed_at: Optional[float] = None
    
    def __post_init__(self):
        if self.created_at is None:
            self.created_at = time.time()

class RetryPolicy:
    """重试策略"""
    
    def __init__(self, config: Dict[str, Any]):
        self.max_retries = config.get("max_retries", 3)
        self.base_delay = config.get("base_delay", 1.0)
        self.max_delay = config.get("max_delay", 60.0)
        self.exponential_base = config.get("exponential_base", 2.0)
    
    def should_retry(self, attempt: int, error: Exception) -> bool:
        """判断是否应该重试"""
        if attempt >= self.max_retries:
            return False
        
        # 某些错误不应该重试
        if isinstance(error, ValueError):
            return False
        
        return True
    
    def get_delay(self, attempt: int) -> float:
        """获取重试延迟时间"""
        delay = self.base_delay * (self.exponential_base ** attempt)
        return min(delay, self.max_delay)
    
    async def execute(self, func, *args, **kwargs):
        """执行带重试的函数"""
        last_error = None
        
        for attempt in range(self.max_retries + 1):
            try:
                return await func(*args, **kwargs)
            except Exception as e:
                last_error = e
                
                if not self.should_retry(attempt, e):
                    break
                
                if attempt < self.max_retries:
                    delay = self.get_delay(attempt)
                    await asyncio.sleep(delay)
        
        raise last_error

class QualityController:
    """质量控制器"""
    
    def __init__(self, config: Dict[str, Any] = None):
        self.config = config or {}
        self.retry_policy = RetryPolicy(self.config.get("retry_policy", {}))
        self.approval_threshold = self.config.get("approval_threshold", 0.85)
        self.processing_timeout = self.config.get("processing_timeout", 300)  # 5分钟
        self.logger = logging.getLogger("quality_controller")
        
        # 存储处理结果（实际应该使用数据库）
        self.results: Dict[str, ProcessResult] = {}
    
    def _generate_task_id(self) -> str:
        """生成任务ID"""
        return str(uuid.uuid4())
    
    async def process_commit_analysis(self, commit_data: Dict[str, Any]) -> ProcessResult:
        """处理提交分析的完整流程"""
        task_id = self._generate_task_id()
        
        # 创建处理结果
        result = ProcessResult(
            task_id=task_id,
            status=ProcessStatus.PENDING,
            commit_data=commit_data
        )
        
        self.results[task_id] = result
        
        try:
            self.logger.info(f"开始处理提交分析: {task_id}")
            
            # 第一阶段：分析
            result.status = ProcessStatus.ANALYZING
            analysis_result = await self._run_analysis_phase(task_id, commit_data)
            result.analysis_result = analysis_result.result
            
            # 第二阶段：审核
            result.status = ProcessStatus.REVIEWING
            review_result = await self._run_review_phase(task_id, commit_data, analysis_result)
            result.review_result = review_result.result
            
            # 第三阶段：最终决策和存储
            final_status = await self._make_final_decision(review_result.result)
            result.status = final_status

            # 存储结果
            if final_status == ProcessStatus.APPROVED:
                await storage_manager.store_approved_analysis(commit_data, analysis_result, review_result)
            else:
                await storage_manager.store_rejected_analysis(commit_data, analysis_result, review_result)

            result.completed_at = time.time()

            self.logger.info(f"提交分析完成: {task_id}, 状态: {final_status.value}")

            return result
            
        except Exception as e:
            result.status = ProcessStatus.ERROR
            result.error_message = str(e)
            result.completed_at = time.time()
            
            self.logger.error(f"提交分析失败: {task_id} - {str(e)}")
            return result
    
    async def _run_analysis_phase(self, task_id: str, commit_data: Dict[str, Any]) -> AgentOutput:
        """运行分析阶段"""
        # 获取分析Agent
        analyzer_agents = agent_manager.get_agents_by_type("commit_analysis")
        if not analyzer_agents:
            raise RuntimeError("没有可用的分析Agent")
        
        analyzer = analyzer_agents[0]  # 使用第一个可用的分析Agent
        
        # 创建输入数据
        analyzer_input = AgentInput(
            task_id=task_id,
            data=commit_data,
            context={"phase": "analysis"},
            metadata={}
        )
        
        # 执行分析
        return await self.retry_policy.execute(
            analyzer.execute_task, analyzer_input
        )
    
    async def _run_review_phase(self, task_id: str, commit_data: Dict[str, Any], 
                               analysis_result: AgentOutput) -> AgentOutput:
        """运行审核阶段"""
        # 获取审核Agent
        reviewer_agents = agent_manager.get_agents_by_type("quality_assessment")
        if not reviewer_agents:
            raise RuntimeError("没有可用的审核Agent")
        
        reviewer = reviewer_agents[0]  # 使用第一个可用的审核Agent
        
        # 创建输入数据
        reviewer_input = AgentInput(
            task_id=task_id,
            data={
                "original_commit": commit_data,
                "analysis_result": analysis_result.result
            },
            context={"phase": "review"},
            metadata={"analysis_agent_id": analysis_result.metadata.get("agent_version")}
        )
        
        # 执行审核
        return await self.retry_policy.execute(
            reviewer.execute_task, reviewer_input
        )
    
    async def _make_final_decision(self, review_result: Dict[str, Any]) -> ProcessStatus:
        """做出最终决策"""
        approved = review_result.get("approved", False)
        overall_score = review_result.get("overall_score", 0.0)
        
        if approved and overall_score >= self.approval_threshold:
            return ProcessStatus.APPROVED
        else:
            return ProcessStatus.REJECTED
    
    async def get_result(self, task_id: str) -> Optional[ProcessResult]:
        """获取处理结果"""
        return self.results.get(task_id)
    
    async def get_results_by_status(self, status: ProcessStatus) -> List[ProcessResult]:
        """根据状态获取结果列表"""
        return [result for result in self.results.values() if result.status == status]
    
    async def retry_failed_task(self, task_id: str) -> ProcessResult:
        """重试失败的任务"""
        result = self.results.get(task_id)
        if not result:
            raise ValueError(f"任务不存在: {task_id}")
        
        if result.status not in [ProcessStatus.ERROR, ProcessStatus.REJECTED]:
            raise ValueError(f"任务状态不允许重试: {result.status}")
        
        # 增加重试计数
        result.retry_count += 1
        
        # 重新处理
        return await self.process_commit_analysis(result.commit_data)
    
    async def get_processing_statistics(self) -> Dict[str, Any]:
        """获取处理统计信息"""
        total_tasks = len(self.results)
        
        if total_tasks == 0:
            return {
                "total_tasks": 0,
                "approved_count": 0,
                "rejected_count": 0,
                "error_count": 0,
                "pending_count": 0,
                "approval_rate": 0.0,
                "average_processing_time": 0.0
            }
        
        status_counts = {}
        total_processing_time = 0.0
        completed_tasks = 0
        
        for result in self.results.values():
            status = result.status.value
            status_counts[status] = status_counts.get(status, 0) + 1
            
            if result.completed_at:
                processing_time = result.completed_at - result.created_at
                total_processing_time += processing_time
                completed_tasks += 1
        
        approved_count = status_counts.get("approved", 0)
        rejected_count = status_counts.get("rejected", 0)
        error_count = status_counts.get("error", 0)
        pending_count = total_tasks - approved_count - rejected_count - error_count
        
        return {
            "total_tasks": total_tasks,
            "approved_count": approved_count,
            "rejected_count": rejected_count,
            "error_count": error_count,
            "pending_count": pending_count,
            "approval_rate": approved_count / (approved_count + rejected_count) if (approved_count + rejected_count) > 0 else 0.0,
            "average_processing_time": total_processing_time / completed_tasks if completed_tasks > 0 else 0.0
        }
    
    async def cleanup_old_results(self, max_age_hours: int = 24):
        """清理旧的处理结果"""
        current_time = time.time()
        max_age_seconds = max_age_hours * 3600
        
        old_task_ids = []
        for task_id, result in self.results.items():
            if current_time - result.created_at > max_age_seconds:
                old_task_ids.append(task_id)
        
        for task_id in old_task_ids:
            del self.results[task_id]
        
        if old_task_ids:
            self.logger.info(f"清理了 {len(old_task_ids)} 个旧的处理结果")
    
    async def get_quality_trends(self, hours: int = 24) -> Dict[str, Any]:
        """获取质量趋势数据"""
        current_time = time.time()
        start_time = current_time - (hours * 3600)
        
        # 筛选时间范围内的结果
        recent_results = [
            result for result in self.results.values()
            if result.created_at >= start_time and result.status == ProcessStatus.APPROVED
        ]
        
        if not recent_results:
            return {
                "average_quality_score": 0.0,
                "quality_trend": [],
                "dimension_averages": {}
            }
        
        # 计算平均质量分数
        total_score = 0.0
        dimension_totals = {}
        
        for result in recent_results:
            if result.review_result:
                score = result.review_result.get("overall_score", 0.0)
                total_score += score
                
                # 聚合维度分数
                dimension_scores = result.review_result.get("dimension_scores", {})
                for dim, score in dimension_scores.items():
                    if dim not in dimension_totals:
                        dimension_totals[dim] = []
                    dimension_totals[dim].append(score)
        
        count = len(recent_results)
        average_quality_score = total_score / count if count > 0 else 0.0
        
        # 计算维度平均值
        dimension_averages = {}
        for dim, scores in dimension_totals.items():
            dimension_averages[dim] = sum(scores) / len(scores) if scores else 0.0
        
        return {
            "average_quality_score": round(average_quality_score, 3),
            "sample_count": count,
            "dimension_averages": dimension_averages
        }

# 全局质量控制器实例
quality_controller = QualityController()
