"""
Agent管理器
"""

import asyncio
import logging
from typing import Dict, Any, List, Optional, Type
from .base_agent import BaseAgent, AgentInput, AgentOutput, AgentStatus
from .analyzer_agent import GitCommitAnalyzer
from .reviewer_agent import QualityReviewer

class AgentManager:
    """Agent管理器"""
    
    def __init__(self):
        self.agents: Dict[str, BaseAgent] = {}
        self.agent_classes: Dict[str, Type[BaseAgent]] = {
            "analyzer": GitCommitAnalyzer,
            "reviewer": QualityReviewer
        }
        self.task_queue = asyncio.Queue()
        self.running = False
        self.logger = logging.getLogger("agent_manager")
        self._worker_tasks: List[asyncio.Task] = []
    
    async def register_agent_class(self, agent_type: str, agent_class: Type[BaseAgent]):
        """注册Agent类"""
        self.agent_classes[agent_type] = agent_class
        self.logger.info(f"注册Agent类: {agent_type}")
    
    async def create_agent(self, agent_id: str, agent_type: str, config: Dict[str, Any]) -> BaseAgent:
        """创建Agent实例"""
        try:
            if agent_type not in self.agent_classes:
                raise ValueError(f"未知的Agent类型: {agent_type}")
            
            agent_class = self.agent_classes[agent_type]
            agent = agent_class(agent_id, config)
            
            self.agents[agent_id] = agent
            self.logger.info(f"创建Agent成功: {agent_id} ({agent_type})")
            
            return agent
            
        except Exception as e:
            self.logger.error(f"创建Agent失败: {agent_id} - {str(e)}")
            raise
    
    async def remove_agent(self, agent_id: str):
        """移除Agent"""
        if agent_id in self.agents:
            agent = self.agents[agent_id]
            agent.status = AgentStatus.STOPPED
            del self.agents[agent_id]
            self.logger.info(f"移除Agent: {agent_id}")
    
    def get_agent(self, agent_id: str) -> Optional[BaseAgent]:
        """获取Agent实例"""
        return self.agents.get(agent_id)
    
    def list_agents(self) -> List[str]:
        """列出所有Agent ID"""
        return list(self.agents.keys())
    
    def get_agents_by_type(self, agent_type: str) -> List[BaseAgent]:
        """根据类型获取Agent列表"""
        result = []
        for agent in self.agents.values():
            if agent_type in agent.get_capabilities():
                result.append(agent)
        return result
    
    async def execute_task(self, agent_id: str, input_data: AgentInput) -> AgentOutput:
        """执行单个任务"""
        agent = self.get_agent(agent_id)
        if not agent:
            raise ValueError(f"Agent不存在: {agent_id}")
        
        if agent.status == AgentStatus.ERROR:
            raise RuntimeError(f"Agent状态异常: {agent_id}")
        
        return await agent.execute_task(input_data)
    
    async def submit_task(self, agent_id: str, input_data: AgentInput) -> str:
        """提交任务到队列"""
        task_item = {
            "agent_id": agent_id,
            "input_data": input_data,
            "future": asyncio.Future()
        }
        
        await self.task_queue.put(task_item)
        self.logger.debug(f"任务已提交到队列: {input_data.task_id}")
        
        return input_data.task_id
    
    async def get_task_result(self, task_id: str, timeout: float = 30.0) -> AgentOutput:
        """获取任务结果（需要配合任务队列使用）"""
        # 这里需要实现任务结果的存储和检索机制
        # 简化实现，实际应该使用Redis或数据库
        pass
    
    async def start_workers(self, num_workers: int = 2):
        """启动工作线程"""
        if self.running:
            return
        
        self.running = True
        self._worker_tasks = []
        
        for i in range(num_workers):
            task = asyncio.create_task(self._worker_loop(f"worker-{i}"))
            self._worker_tasks.append(task)
        
        self.logger.info(f"启动了 {num_workers} 个工作线程")
    
    async def stop_workers(self):
        """停止工作线程"""
        self.running = False
        
        # 取消所有工作任务
        for task in self._worker_tasks:
            task.cancel()
        
        # 等待任务完成
        if self._worker_tasks:
            await asyncio.gather(*self._worker_tasks, return_exceptions=True)
        
        self._worker_tasks.clear()
        self.logger.info("所有工作线程已停止")
    
    async def _worker_loop(self, worker_name: str):
        """工作线程循环"""
        self.logger.info(f"工作线程启动: {worker_name}")
        
        while self.running:
            try:
                # 从队列获取任务
                task_item = await asyncio.wait_for(
                    self.task_queue.get(), 
                    timeout=1.0
                )
                
                agent_id = task_item["agent_id"]
                input_data = task_item["input_data"]
                future = task_item["future"]
                
                try:
                    # 执行任务
                    result = await self.execute_task(agent_id, input_data)
                    future.set_result(result)
                    
                except Exception as e:
                    future.set_exception(e)
                
                finally:
                    self.task_queue.task_done()
                
            except asyncio.TimeoutError:
                # 队列为空，继续循环
                continue
            except Exception as e:
                self.logger.error(f"工作线程错误 {worker_name}: {str(e)}")
                await asyncio.sleep(1)
        
        self.logger.info(f"工作线程停止: {worker_name}")
    
    async def health_check_all(self) -> Dict[str, bool]:
        """检查所有Agent健康状态"""
        results = {}
        
        for agent_id, agent in self.agents.items():
            try:
                results[agent_id] = await agent.health_check()
            except Exception as e:
                self.logger.error(f"Agent健康检查失败 {agent_id}: {str(e)}")
                results[agent_id] = False
        
        return results
    
    async def reload_agent_config(self, agent_id: str, new_config: Dict[str, Any]):
        """重新加载Agent配置"""
        agent = self.get_agent(agent_id)
        if not agent:
            raise ValueError(f"Agent不存在: {agent_id}")
        
        await agent.reload_config(new_config)
        self.logger.info(f"Agent配置重载成功: {agent_id}")
    
    def get_agent_status(self, agent_id: str) -> Optional[Dict[str, Any]]:
        """获取Agent状态"""
        agent = self.get_agent(agent_id)
        if not agent:
            return None
        
        return agent.get_status()
    
    def get_all_agent_status(self) -> Dict[str, Dict[str, Any]]:
        """获取所有Agent状态"""
        results = {}
        
        for agent_id, agent in self.agents.items():
            results[agent_id] = agent.get_status()
        
        return results
    
    async def get_system_metrics(self) -> Dict[str, Any]:
        """获取系统指标"""
        total_agents = len(self.agents)
        healthy_agents = sum(1 for result in (await self.health_check_all()).values() if result)
        
        # 聚合所有Agent的指标
        total_tasks = 0
        total_success = 0
        total_processing_time = 0.0
        
        for agent in self.agents.values():
            metrics = agent.metrics
            total_tasks += metrics.total_tasks
            total_success += metrics.successful_tasks
            total_processing_time += metrics.total_processing_time
        
        return {
            "total_agents": total_agents,
            "healthy_agents": healthy_agents,
            "health_rate": healthy_agents / total_agents if total_agents > 0 else 0,
            "total_tasks_processed": total_tasks,
            "overall_success_rate": total_success / total_tasks if total_tasks > 0 else 0,
            "average_processing_time": total_processing_time / total_tasks if total_tasks > 0 else 0,
            "queue_size": self.task_queue.qsize(),
            "workers_running": len(self._worker_tasks)
        }
    
    async def shutdown(self):
        """关闭管理器"""
        self.logger.info("开始关闭Agent管理器")
        
        # 停止工作线程
        await self.stop_workers()
        
        # 停止所有Agent
        for agent_id in list(self.agents.keys()):
            await self.remove_agent(agent_id)
        
        self.logger.info("Agent管理器已关闭")

# 全局Agent管理器实例
agent_manager = AgentManager()
