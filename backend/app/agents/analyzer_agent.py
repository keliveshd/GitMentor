"""
Git提交分析Agent
"""

import json
import time
import asyncio
import logging
from typing import Dict, Any, List
from .base_agent import BaseAgent, AgentInput, AgentOutput, AgentProcessingError
from ..services.ai_service import ai_service
from ..services.prompt_manager import prompt_manager

class GitCommitAnalyzer(BaseAgent):
    """Git提交分析Agent"""
    
    def __init__(self, agent_id: str, config: Dict[str, Any]):
        super().__init__(agent_id, config)
        self.llm_client_name = config.get("llm_client", None)
        self.prompt_template_name = config.get("prompt_template", "git_commit_analyzer")
        self.max_retries = config.get("max_retries", 3)
        self.timeout = config.get("timeout", 30)
    
    def validate_config(self, config: Dict[str, Any]) -> bool:
        """验证配置有效性"""
        required_fields = ["llm_client", "prompt_template"]
        for field in required_fields:
            if field not in config:
                raise ValueError(f"配置缺少必需字段: {field}")
        
        # 验证LLM客户端是否存在
        if config["llm_client"] not in ai_service.get_available_clients():
            raise ValueError(f"LLM客户端不存在: {config['llm_client']}")
        
        # 验证Prompt模板是否存在
        if not prompt_manager.get_template(config["prompt_template"]):
            raise ValueError(f"Prompt模板不存在: {config['prompt_template']}")
        
        return True
    
    def get_capabilities(self) -> List[str]:
        """返回Agent能力列表"""
        return [
            "commit_analysis",
            "semantic_understanding", 
            "change_categorization",
            "impact_assessment",
            "confidence_scoring"
        ]
    
    async def process(self, input_data: AgentInput) -> AgentOutput:
        """处理Git提交分析"""
        start_time = time.time()
        
        try:
            # 1. 验证输入数据
            commit_data = self._validate_input(input_data.data)
            
            # 2. 预处理提交数据
            processed_data = await self._preprocess_commit_data(commit_data)
            
            # 3. 构建提示词
            prompt = await self._build_prompt(processed_data)
            
            # 4. 调用LLM分析
            llm_response = await self._call_llm_with_retry(prompt)
            
            # 5. 后处理结果
            analysis_result = await self._postprocess_result(llm_response, processed_data)
            
            # 6. 计算置信度
            confidence = self._calculate_confidence(analysis_result, processed_data)
            
            processing_time = time.time() - start_time
            
            return AgentOutput(
                task_id=input_data.task_id,
                result=analysis_result,
                confidence=confidence,
                processing_time=processing_time,
                metadata={
                    "agent_version": self.config.get("version", "1.0"),
                    "llm_client": self.llm_client_name,
                    "prompt_template": self.prompt_template_name
                }
            )
            
        except Exception as e:
            self.logger.error(f"分析失败: {str(e)}")
            raise AgentProcessingError(f"分析失败: {str(e)}")
    
    def _validate_input(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """验证输入数据"""
        required_fields = ["hash", "message", "author_name", "author_email", "commit_date"]
        
        for field in required_fields:
            if field not in data:
                raise ValueError(f"输入数据缺少必需字段: {field}")
        
        return data
    
    async def _preprocess_commit_data(self, commit_data: Dict[str, Any]) -> Dict[str, Any]:
        """预处理提交数据"""
        return {
            "commit_hash": commit_data.get("hash", "")[:8],  # 短哈希
            "commit_message": commit_data.get("message", "").strip(),
            "author_name": commit_data.get("author_name", ""),
            "author_email": commit_data.get("author_email", ""),
            "commit_date": commit_data.get("commit_date", ""),
            "files_changed": commit_data.get("files_changed", 0),
            "insertions": commit_data.get("insertions", 0),
            "deletions": commit_data.get("deletions", 0),
            "file_changes": commit_data.get("file_changes", [])
        }
    
    async def _build_prompt(self, data: Dict[str, Any]) -> str:
        """构建分析提示词"""
        try:
            return prompt_manager.format_template(self.prompt_template_name, **data)
        except Exception as e:
            raise ValueError(f"构建提示词失败: {str(e)}")
    
    async def _call_llm_with_retry(self, prompt: str) -> Dict[str, Any]:
        """带重试的LLM调用"""
        last_error = None
        
        for attempt in range(self.max_retries):
            try:
                response = await ai_service.complete(
                    prompt=prompt,
                    client_name=self.llm_client_name,
                    max_tokens=self.config.get("max_tokens", 1000),
                    temperature=self.config.get("temperature", 0.3)
                )
                return response
                
            except Exception as e:
                last_error = e
                self.logger.warning(f"LLM调用失败 (尝试 {attempt + 1}/{self.max_retries}): {str(e)}")
                
                if attempt < self.max_retries - 1:
                    # 等待后重试
                    await asyncio.sleep(2 ** attempt)  # 指数退避
        
        raise AgentProcessingError(f"LLM调用失败，已重试 {self.max_retries} 次: {str(last_error)}")
    
    async def _postprocess_result(self, llm_response: Dict[str, Any], commit_data: Dict[str, Any]) -> Dict[str, Any]:
        """后处理LLM结果"""
        try:
            content = llm_response.get("content", "")
            
            # 尝试解析JSON结果
            try:
                # 提取JSON部分
                json_start = content.find('{')
                json_end = content.rfind('}') + 1
                
                if json_start >= 0 and json_end > json_start:
                    json_content = content[json_start:json_end]
                    result = json.loads(json_content)
                else:
                    # 如果没有找到JSON，使用默认结构
                    result = self._create_fallback_result(content, commit_data)
                
            except json.JSONDecodeError:
                # JSON解析失败，创建备用结果
                result = self._create_fallback_result(content, commit_data)
            
            # 验证和标准化结果
            return self._normalize_result(result, commit_data)
            
        except Exception as e:
            self.logger.error(f"结果后处理失败: {str(e)}")
            return self._create_fallback_result("", commit_data)
    
    def _create_fallback_result(self, content: str, commit_data: Dict[str, Any]) -> Dict[str, Any]:
        """创建备用分析结果"""
        # 基于提交消息的简单分析
        message = commit_data.get("commit_message", "").lower()
        
        # 简单的类型判断
        if any(word in message for word in ["fix", "bug", "error", "issue"]):
            category = "bugfix"
        elif any(word in message for word in ["add", "new", "feature", "implement"]):
            category = "feature"
        elif any(word in message for word in ["refactor", "clean", "improve"]):
            category = "refactor"
        elif any(word in message for word in ["doc", "readme", "comment"]):
            category = "docs"
        elif any(word in message for word in ["test", "spec"]):
            category = "test"
        else:
            category = "chore"
        
        # 简单的影响级别判断
        files_changed = commit_data.get("files_changed", 0)
        if files_changed > 10:
            impact_level = "high"
        elif files_changed > 3:
            impact_level = "medium"
        else:
            impact_level = "low"
        
        return {
            "summary": content[:100] if content else commit_data.get("commit_message", "")[:100],
            "category": category,
            "impact_level": impact_level,
            "confidence_score": 0.5,  # 低置信度
            "description": content if content else "自动生成的分析结果"
        }
    
    def _normalize_result(self, result: Dict[str, Any], commit_data: Dict[str, Any]) -> Dict[str, Any]:
        """标准化分析结果"""
        # 确保必需字段存在
        normalized = {
            "summary": result.get("summary", "")[:200],  # 限制长度
            "category": self._normalize_category(result.get("category", "chore")),
            "impact_level": self._normalize_impact_level(result.get("impact_level", "low")),
            "confidence_score": max(0.0, min(1.0, float(result.get("confidence_score", 0.5)))),
            "description": result.get("description", "")[:500],  # 限制长度
            "files_affected": commit_data.get("file_changes", [])[:10]  # 限制数量
        }
        
        return normalized
    
    def _normalize_category(self, category: str) -> str:
        """标准化类型分类"""
        valid_categories = ["feature", "bugfix", "refactor", "docs", "style", "test", "chore"]
        category_lower = category.lower()
        
        if category_lower in valid_categories:
            return category_lower
        
        # 映射常见的变体
        category_mapping = {
            "feat": "feature",
            "fix": "bugfix", 
            "bug": "bugfix",
            "doc": "docs",
            "documentation": "docs",
            "tests": "test",
            "testing": "test"
        }
        
        return category_mapping.get(category_lower, "chore")
    
    def _normalize_impact_level(self, impact_level: str) -> str:
        """标准化影响级别"""
        valid_levels = ["low", "medium", "high"]
        level_lower = impact_level.lower()
        
        if level_lower in valid_levels:
            return level_lower
        
        # 映射常见的变体
        level_mapping = {
            "small": "low",
            "minor": "low",
            "moderate": "medium",
            "major": "high",
            "large": "high",
            "critical": "high"
        }
        
        return level_mapping.get(level_lower, "low")
    
    def _calculate_confidence(self, result: Dict[str, Any], commit_data: Dict[str, Any]) -> float:
        """计算置信度"""
        base_confidence = result.get("confidence_score", 0.5)
        
        # 根据数据质量调整置信度
        adjustments = 0.0
        
        # 提交消息质量
        message = commit_data.get("commit_message", "")
        if len(message) > 10:
            adjustments += 0.1
        if len(message) > 50:
            adjustments += 0.1
        
        # 文件变更信息
        if commit_data.get("files_changed", 0) > 0:
            adjustments += 0.1
        
        # 结果完整性
        if result.get("summary") and result.get("description"):
            adjustments += 0.1
        
        final_confidence = min(1.0, base_confidence + adjustments)
        return round(final_confidence, 2)
    
    async def _custom_health_check(self) -> bool:
        """自定义健康检查"""
        try:
            # 检查LLM客户端连接
            client = ai_service.get_client(self.llm_client_name)
            if not await client.test_connection():
                return False
            
            # 检查Prompt模板
            template = prompt_manager.get_template(self.prompt_template_name)
            if not template:
                return False
            
            return True
            
        except Exception as e:
            self.logger.error(f"健康检查失败: {str(e)}")
            return False
