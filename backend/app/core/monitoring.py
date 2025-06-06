"""
监控和指标收集模块
"""

import time
import psutil
import logging
from typing import Dict, Any, List, Optional
from dataclasses import dataclass, asdict
from threading import Lock
from collections import defaultdict, deque

@dataclass
class MetricPoint:
    """指标数据点"""
    timestamp: float
    value: float
    tags: Dict[str, str] = None
    
    def __post_init__(self):
        if self.tags is None:
            self.tags = {}

class MetricsCollector:
    """指标收集器"""
    
    def __init__(self, max_points: int = 1000):
        self.max_points = max_points
        self.metrics: Dict[str, deque] = defaultdict(lambda: deque(maxlen=max_points))
        self.lock = Lock()
        self.logger = logging.getLogger("metrics_collector")
    
    def record_metric(self, name: str, value: float, tags: Dict[str, str] = None):
        """记录指标"""
        with self.lock:
            point = MetricPoint(
                timestamp=time.time(),
                value=value,
                tags=tags or {}
            )
            self.metrics[name].append(point)
    
    def get_metric_history(self, name: str, limit: int = 100) -> List[Dict[str, Any]]:
        """获取指标历史"""
        with self.lock:
            if name not in self.metrics:
                return []
            
            points = list(self.metrics[name])[-limit:]
            return [asdict(point) for point in points]
    
    def get_metric_summary(self, name: str, duration: int = 3600) -> Dict[str, Any]:
        """获取指标摘要"""
        with self.lock:
            if name not in self.metrics:
                return {}
            
            current_time = time.time()
            cutoff_time = current_time - duration
            
            # 筛选时间范围内的数据点
            points = [p for p in self.metrics[name] if p.timestamp >= cutoff_time]
            
            if not points:
                return {}
            
            values = [p.value for p in points]
            
            return {
                "count": len(values),
                "min": min(values),
                "max": max(values),
                "avg": sum(values) / len(values),
                "latest": values[-1] if values else 0,
                "duration": duration
            }
    
    def get_all_metrics(self) -> List[str]:
        """获取所有指标名称"""
        with self.lock:
            return list(self.metrics.keys())

class SystemMonitor:
    """系统监控器"""
    
    def __init__(self, metrics_collector: MetricsCollector):
        self.metrics = metrics_collector
        self.logger = logging.getLogger("system_monitor")
    
    def collect_system_metrics(self):
        """收集系统指标"""
        try:
            # CPU使用率
            cpu_percent = psutil.cpu_percent(interval=1)
            self.metrics.record_metric("system.cpu.usage", cpu_percent, {"unit": "percent"})
            
            # 内存使用情况
            memory = psutil.virtual_memory()
            self.metrics.record_metric("system.memory.usage", memory.percent, {"unit": "percent"})
            self.metrics.record_metric("system.memory.available", memory.available, {"unit": "bytes"})
            self.metrics.record_metric("system.memory.used", memory.used, {"unit": "bytes"})
            
            # 磁盘使用情况
            disk = psutil.disk_usage('/')
            disk_percent = (disk.used / disk.total) * 100
            self.metrics.record_metric("system.disk.usage", disk_percent, {"unit": "percent"})
            self.metrics.record_metric("system.disk.free", disk.free, {"unit": "bytes"})
            
            # 网络IO
            net_io = psutil.net_io_counters()
            self.metrics.record_metric("system.network.bytes_sent", net_io.bytes_sent, {"unit": "bytes"})
            self.metrics.record_metric("system.network.bytes_recv", net_io.bytes_recv, {"unit": "bytes"})
            
        except Exception as e:
            self.logger.error(f"收集系统指标失败: {str(e)}")
    
    def get_system_status(self) -> Dict[str, Any]:
        """获取系统状态"""
        try:
            return {
                "cpu": {
                    "usage_percent": psutil.cpu_percent(),
                    "count": psutil.cpu_count(),
                    "load_avg": psutil.getloadavg() if hasattr(psutil, 'getloadavg') else None
                },
                "memory": {
                    "total": psutil.virtual_memory().total,
                    "available": psutil.virtual_memory().available,
                    "used": psutil.virtual_memory().used,
                    "percent": psutil.virtual_memory().percent
                },
                "disk": {
                    "total": psutil.disk_usage('/').total,
                    "used": psutil.disk_usage('/').used,
                    "free": psutil.disk_usage('/').free,
                    "percent": (psutil.disk_usage('/').used / psutil.disk_usage('/').total) * 100
                },
                "processes": len(psutil.pids()),
                "boot_time": psutil.boot_time()
            }
        except Exception as e:
            self.logger.error(f"获取系统状态失败: {str(e)}")
            return {}

class ApplicationMonitor:
    """应用监控器"""
    
    def __init__(self, metrics_collector: MetricsCollector):
        self.metrics = metrics_collector
        self.logger = logging.getLogger("app_monitor")
        self.request_times = deque(maxlen=1000)
        self.error_counts = defaultdict(int)
    
    def record_request(self, endpoint: str, method: str, status_code: int, 
                      processing_time: float, user_agent: str = None):
        """记录请求指标"""
        tags = {
            "endpoint": endpoint,
            "method": method,
            "status": str(status_code),
            "status_class": f"{status_code // 100}xx"
        }
        
        # 记录响应时间
        self.metrics.record_metric("app.request.duration", processing_time, tags)
        
        # 记录请求计数
        self.metrics.record_metric("app.request.count", 1, tags)
        
        # 记录错误
        if status_code >= 400:
            self.metrics.record_metric("app.request.errors", 1, tags)
            self.error_counts[f"{method} {endpoint}"] += 1
    
    def record_agent_execution(self, agent_id: str, agent_type: str, 
                             processing_time: float, success: bool, confidence: float = None):
        """记录Agent执行指标"""
        tags = {
            "agent_id": agent_id,
            "agent_type": agent_type,
            "success": str(success)
        }
        
        # 记录执行时间
        self.metrics.record_metric("agent.execution.duration", processing_time, tags)
        
        # 记录执行计数
        self.metrics.record_metric("agent.execution.count", 1, tags)
        
        # 记录置信度
        if confidence is not None:
            self.metrics.record_metric("agent.execution.confidence", confidence, tags)
        
        # 记录成功/失败
        if success:
            self.metrics.record_metric("agent.execution.success", 1, tags)
        else:
            self.metrics.record_metric("agent.execution.failure", 1, tags)
    
    def record_quality_score(self, repository: str, commit_hash: str, 
                           overall_score: float, dimension_scores: Dict[str, float]):
        """记录质量分数"""
        tags = {
            "repository": repository,
            "commit": commit_hash[:8]
        }
        
        # 记录总体质量分数
        self.metrics.record_metric("quality.overall_score", overall_score, tags)
        
        # 记录各维度分数
        for dimension, score in dimension_scores.items():
            dimension_tags = {**tags, "dimension": dimension}
            self.metrics.record_metric("quality.dimension_score", score, dimension_tags)
    
    def get_application_health(self) -> Dict[str, Any]:
        """获取应用健康状态"""
        try:
            # 获取最近的错误率
            recent_errors = self.metrics.get_metric_summary("app.request.errors", 300)  # 5分钟
            recent_requests = self.metrics.get_metric_summary("app.request.count", 300)
            
            error_rate = 0
            if recent_requests.get("count", 0) > 0:
                error_rate = (recent_errors.get("count", 0) / recent_requests.get("count", 1)) * 100
            
            # 获取平均响应时间
            response_time = self.metrics.get_metric_summary("app.request.duration", 300)
            
            # 获取Agent执行统计
            agent_success = self.metrics.get_metric_summary("agent.execution.success", 300)
            agent_failure = self.metrics.get_metric_summary("agent.execution.failure", 300)
            
            agent_success_rate = 0
            total_agent_executions = agent_success.get("count", 0) + agent_failure.get("count", 0)
            if total_agent_executions > 0:
                agent_success_rate = (agent_success.get("count", 0) / total_agent_executions) * 100
            
            return {
                "status": "healthy" if error_rate < 5 and agent_success_rate > 90 else "degraded",
                "error_rate": round(error_rate, 2),
                "avg_response_time": round(response_time.get("avg", 0), 3),
                "agent_success_rate": round(agent_success_rate, 2),
                "total_requests": recent_requests.get("count", 0),
                "total_agent_executions": total_agent_executions,
                "top_errors": dict(list(self.error_counts.items())[:5])
            }
        except Exception as e:
            self.logger.error(f"获取应用健康状态失败: {str(e)}")
            return {"status": "unknown", "error": str(e)}

# 全局监控实例
metrics_collector = MetricsCollector()
system_monitor = SystemMonitor(metrics_collector)
app_monitor = ApplicationMonitor(metrics_collector)
