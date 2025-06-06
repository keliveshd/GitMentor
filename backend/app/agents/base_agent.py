"""
基础Agent抽象类
"""

import time
import logging
from abc import ABC, abstractmethod
from typing import Dict, Any, List
from dataclasses import dataclass
from enum import Enum

class AgentStatus(Enum):
    """Agent状态枚举"""
    IDLE = "idle"
    PROCESSING = "processing"
    ERROR = "error"
    STOPPED = "stopped"

@dataclass
class AgentInput:
    """Agent输入数据结构"""
    task_id: str
    data: Dict[str, Any]
    context: Dict[str, Any]
    metadata: Dict[str, Any]

@dataclass
class AgentOutput:
    """Agent输出数据结构"""
    task_id: str
    result: Dict[str, Any]
    confidence: float
    processing_time: float
    metadata: Dict[str, Any]
    status: str = "success"
    error_message: str = None

class AgentMetrics:
    """Agent性能指标"""
    def __init__(self):
        self.total_tasks = 0
        self.successful_tasks = 0
        self.failed_tasks = 0
        self.total_processing_time = 0.0
        self.average_confidence = 0.0
    
    def update(self, output: AgentOutput):
        """更新指标"""
        self.total_tasks += 1
        if output.status == "success":
            self.successful_tasks += 1
            self.average_confidence = (
                (self.average_confidence * (self.successful_tasks - 1) + output.confidence) 
                / self.successful_tasks
            )
        else:
            self.failed_tasks += 1
        self.total_processing_time += output.processing_time
    
    def get_success_rate(self) -> float:
        """获取成功率"""
        if self.total_tasks == 0:
            return 0.0
        return self.successful_tasks / self.total_tasks
    
    def get_average_processing_time(self) -> float:
        """获取平均处理时间"""
        if self.total_tasks == 0:
            return 0.0
        return self.total_processing_time / self.total_tasks

class BaseAgent(ABC):
    """基础Agent抽象类"""
    
    def __init__(self, agent_id: str, config: Dict[str, Any]):
        self.agent_id = agent_id
        self.config = config
        self.status = AgentStatus.IDLE
        self.metrics = AgentMetrics()
        self.logger = logging.getLogger(f"agent.{agent_id}")
        self._initialize()
    
    def _initialize(self):
        """初始化Agent"""
        try:
            self.validate_config(self.config)
            self.logger.info(f"Agent {self.agent_id} 初始化成功")
        except Exception as e:
            self.logger.error(f"Agent {self.agent_id} 初始化失败: {str(e)}")
            self.status = AgentStatus.ERROR
            raise
    
    @abstractmethod
    async def process(self, input_data: AgentInput) -> AgentOutput:
        """
        处理输入数据并返回结果
        
        Args:
            input_data: 输入数据
            
        Returns:
            AgentOutput: 处理结果
        """
        pass
    
    @abstractmethod
    def validate_config(self, config: Dict[str, Any]) -> bool:
        """
        验证配置有效性
        
        Args:
            config: 配置字典
            
        Returns:
            bool: 配置是否有效
        """
        pass
    
    @abstractmethod
    def get_capabilities(self) -> List[str]:
        """
        返回Agent能力列表
        
        Returns:
            List[str]: 能力列表
        """
        pass
    
    async def health_check(self) -> bool:
        """
        健康检查
        
        Returns:
            bool: 是否健康
        """
        try:
            # 基础健康检查
            if self.status == AgentStatus.ERROR:
                return False
            
            # 子类可以重写此方法添加更多检查
            return await self._custom_health_check()
        except Exception as e:
            self.logger.error(f"健康检查失败: {str(e)}")
            return False
    
    async def _custom_health_check(self) -> bool:
        """自定义健康检查，子类可重写"""
        return True
    
    async def reload_config(self, new_config: Dict[str, Any]):
        """
        重新加载配置
        
        Args:
            new_config: 新配置
        """
        try:
            # 验证新配置
            self.validate_config(new_config)
            
            # 保存旧配置以便回滚
            old_config = self.config.copy()
            
            try:
                # 应用新配置
                self.config = new_config
                await self._apply_config_changes()
                self.logger.info(f"Agent {self.agent_id} 配置重载成功")
            except Exception as e:
                # 回滚配置
                self.config = old_config
                self.logger.error(f"配置应用失败，已回滚: {str(e)}")
                raise
                
        except Exception as e:
            self.logger.error(f"配置重载失败: {str(e)}")
            raise
    
    async def _apply_config_changes(self):
        """应用配置变更，子类可重写"""
        pass
    
    def get_status(self) -> Dict[str, Any]:
        """
        获取Agent状态信息
        
        Returns:
            Dict: 状态信息
        """
        return {
            "agent_id": self.agent_id,
            "status": self.status.value,
            "capabilities": self.get_capabilities(),
            "metrics": {
                "total_tasks": self.metrics.total_tasks,
                "success_rate": self.metrics.get_success_rate(),
                "average_processing_time": self.metrics.get_average_processing_time(),
                "average_confidence": self.metrics.average_confidence
            },
            "config_version": self.config.get("version", "unknown")
        }
    
    async def execute_task(self, input_data: AgentInput) -> AgentOutput:
        """
        执行任务的包装方法，包含状态管理和指标更新
        
        Args:
            input_data: 输入数据
            
        Returns:
            AgentOutput: 处理结果
        """
        start_time = time.time()
        self.status = AgentStatus.PROCESSING
        
        try:
            # 执行实际处理
            output = await self.process(input_data)
            
            # 更新处理时间
            output.processing_time = time.time() - start_time
            
            # 更新指标
            self.metrics.update(output)
            
            # 重置状态
            self.status = AgentStatus.IDLE
            
            self.logger.info(
                f"任务 {input_data.task_id} 处理完成，"
                f"耗时 {output.processing_time:.2f}s，"
                f"置信度 {output.confidence:.2f}"
            )
            
            return output
            
        except Exception as e:
            # 创建错误输出
            error_output = AgentOutput(
                task_id=input_data.task_id,
                result={},
                confidence=0.0,
                processing_time=time.time() - start_time,
                metadata={"error": True},
                status="error",
                error_message=str(e)
            )
            
            # 更新指标
            self.metrics.update(error_output)
            
            # 重置状态
            self.status = AgentStatus.ERROR
            
            self.logger.error(f"任务 {input_data.task_id} 处理失败: {str(e)}")
            
            return error_output

class AgentProcessingError(Exception):
    """Agent处理异常"""
    pass
