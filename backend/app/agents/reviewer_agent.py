"""
质量审核Agent
"""

import json
import time
import asyncio
import logging
from typing import Dict, Any, List
from .base_agent import BaseAgent, AgentInput, AgentOutput, AgentProcessingError
from ..services.ai_service import ai_service
from ..services.prompt_manager import prompt_manager

class QualityReviewer(BaseAgent):
    """质量审核Agent"""
    
    def __init__(self, agent_id: str, config: Dict[str, Any]):
        super().__init__(agent_id, config)
        self.llm_client_name = config.get("llm_client", None)
        self.prompt_template_name = config.get("prompt_template", "quality_reviewer")
        self.approval_threshold = config.get("approval_threshold", 0.85)
        self.dimension_weights = config.get("dimension_weights", {
            "accuracy": 0.3,
            "completeness": 0.25,
            "consistency": 0.25,
            "clarity": 0.2
        })
        self.max_retries = config.get("max_retries", 3)
    
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
        
        # 验证阈值范围
        threshold = config.get("approval_threshold", 0.85)
        if not 0.0 <= threshold <= 1.0:
            raise ValueError("approval_threshold必须在0.0-1.0范围内")
        
        return True
    
    def get_capabilities(self) -> List[str]:
        """返回Agent能力列表"""
        return [
            "quality_assessment",
            "accuracy_evaluation", 
            "completeness_check",
            "consistency_validation",
            "clarity_assessment",
            "feedback_generation"
        ]
    
    async def process(self, input_data: AgentInput) -> AgentOutput:
        """处理质量审核"""
        start_time = time.time()
        
        try:
            # 1. 验证输入数据
            review_data = self._validate_input(input_data.data)
            
            # 2. 预处理审核数据
            processed_data = await self._preprocess_review_data(review_data)
            
            # 3. 构建审核提示词
            prompt = await self._build_review_prompt(processed_data)
            
            # 4. 调用LLM进行审核
            llm_response = await self._call_llm_with_retry(prompt)
            
            # 5. 后处理审核结果
            review_result = await self._postprocess_review_result(llm_response, processed_data)
            
            # 6. 生成最终决策
            final_decision = await self._make_final_decision(review_result)
            
            processing_time = time.time() - start_time
            
            return AgentOutput(
                task_id=input_data.task_id,
                result=final_decision,
                confidence=final_decision.get("confidence", 0.8),
                processing_time=processing_time,
                metadata={
                    "agent_version": self.config.get("version", "1.0"),
                    "llm_client": self.llm_client_name,
                    "prompt_template": self.prompt_template_name,
                    "approval_threshold": self.approval_threshold
                }
            )
            
        except Exception as e:
            self.logger.error(f"审核失败: {str(e)}")
            raise AgentProcessingError(f"审核失败: {str(e)}")
    
    def _validate_input(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """验证输入数据"""
        required_fields = ["original_commit", "analysis_result"]
        
        for field in required_fields:
            if field not in data:
                raise ValueError(f"输入数据缺少必需字段: {field}")
        
        # 验证原始提交数据
        commit_data = data["original_commit"]
        commit_required = ["hash", "message"]
        for field in commit_required:
            if field not in commit_data:
                raise ValueError(f"原始提交数据缺少必需字段: {field}")
        
        # 验证分析结果数据
        analysis_data = data["analysis_result"]
        analysis_required = ["summary", "category", "confidence_score"]
        for field in analysis_required:
            if field not in analysis_data:
                raise ValueError(f"分析结果数据缺少必需字段: {field}")
        
        return data
    
    async def _preprocess_review_data(self, review_data: Dict[str, Any]) -> Dict[str, Any]:
        """预处理审核数据"""
        commit_data = review_data["original_commit"]
        analysis_data = review_data["analysis_result"]
        
        return {
            # 原始提交信息
            "commit_hash": commit_data.get("hash", "")[:8],
            "commit_message": commit_data.get("message", "").strip(),
            "files_changed": commit_data.get("files_changed", 0),
            
            # 分析结果信息
            "analysis_summary": analysis_data.get("summary", ""),
            "analysis_category": analysis_data.get("category", ""),
            "analysis_impact_level": analysis_data.get("impact_level", ""),
            "analysis_confidence": analysis_data.get("confidence_score", 0.0),
            "analysis_description": analysis_data.get("description", "")
        }
    
    async def _build_review_prompt(self, data: Dict[str, Any]) -> str:
        """构建审核提示词"""
        try:
            return prompt_manager.format_template(self.prompt_template_name, **data)
        except Exception as e:
            raise ValueError(f"构建审核提示词失败: {str(e)}")
    
    async def _call_llm_with_retry(self, prompt: str) -> Dict[str, Any]:
        """带重试的LLM调用"""
        last_error = None
        
        for attempt in range(self.max_retries):
            try:
                response = await ai_service.complete(
                    prompt=prompt,
                    client_name=self.llm_client_name,
                    max_tokens=self.config.get("max_tokens", 800),
                    temperature=self.config.get("temperature", 0.2)  # 审核需要更稳定的输出
                )
                return response
                
            except Exception as e:
                last_error = e
                self.logger.warning(f"LLM调用失败 (尝试 {attempt + 1}/{self.max_retries}): {str(e)}")
                
                if attempt < self.max_retries - 1:
                    await asyncio.sleep(2 ** attempt)
        
        raise AgentProcessingError(f"LLM调用失败，已重试 {self.max_retries} 次: {str(last_error)}")
    
    async def _postprocess_review_result(self, llm_response: Dict[str, Any], review_data: Dict[str, Any]) -> Dict[str, Any]:
        """后处理审核结果"""
        try:
            content = llm_response.get("content", "")
            
            # 尝试解析JSON结果
            try:
                json_start = content.find('{')
                json_end = content.rfind('}') + 1
                
                if json_start >= 0 and json_end > json_start:
                    json_content = content[json_start:json_end]
                    result = json.loads(json_content)
                else:
                    result = self._create_fallback_review_result(content, review_data)
                
            except json.JSONDecodeError:
                result = self._create_fallback_review_result(content, review_data)
            
            # 标准化审核结果
            return self._normalize_review_result(result)
            
        except Exception as e:
            self.logger.error(f"审核结果后处理失败: {str(e)}")
            return self._create_fallback_review_result("", review_data)
    
    def _create_fallback_review_result(self, content: str, review_data: Dict[str, Any]) -> Dict[str, Any]:
        """创建备用审核结果"""
        # 基于简单规则的审核
        analysis_confidence = review_data.get("analysis_confidence", 0.0)
        
        # 简单的质量评估
        accuracy = min(1.0, analysis_confidence + 0.1)
        completeness = 0.8 if review_data.get("analysis_summary") else 0.5
        consistency = 0.8 if review_data.get("analysis_category") else 0.5
        clarity = 0.7 if len(review_data.get("analysis_summary", "")) > 20 else 0.5
        
        overall_score = (accuracy + completeness + consistency + clarity) / 4
        
        return {
            "approved": overall_score >= self.approval_threshold,
            "overall_score": overall_score,
            "dimension_scores": {
                "accuracy": accuracy,
                "completeness": completeness,
                "consistency": consistency,
                "clarity": clarity
            },
            "feedback": content if content else "自动生成的审核结果",
            "suggestions": ["建议提高分析质量"] if overall_score < self.approval_threshold else []
        }
    
    def _normalize_review_result(self, result: Dict[str, Any]) -> Dict[str, Any]:
        """标准化审核结果"""
        # 确保维度分数存在且有效
        dimension_scores = result.get("dimension_scores", {})
        normalized_dimensions = {}
        
        for dimension in ["accuracy", "completeness", "consistency", "clarity"]:
            score = dimension_scores.get(dimension, 0.5)
            normalized_dimensions[dimension] = max(0.0, min(1.0, float(score)))
        
        # 计算加权总分
        overall_score = sum(
            normalized_dimensions[dim] * self.dimension_weights.get(dim, 0.25)
            for dim in normalized_dimensions
        )
        
        return {
            "approved": bool(result.get("approved", overall_score >= self.approval_threshold)),
            "overall_score": round(overall_score, 3),
            "dimension_scores": normalized_dimensions,
            "feedback": str(result.get("feedback", ""))[:500],  # 限制长度
            "suggestions": list(result.get("suggestions", []))[:5]  # 限制数量
        }
    
    async def _make_final_decision(self, review_result: Dict[str, Any]) -> Dict[str, Any]:
        """生成最终审核决策"""
        approved = review_result["approved"]
        overall_score = review_result["overall_score"]
        
        # 计算置信度
        confidence = self._calculate_review_confidence(review_result)
        
        # 判断是否需要修订
        revision_required = not approved and overall_score >= 0.70
        
        final_decision = {
            "approved": approved,
            "overall_score": overall_score,
            "confidence": confidence,
            "dimension_scores": review_result["dimension_scores"],
            "feedback": review_result["feedback"],
            "suggestions": review_result["suggestions"],
            "revision_required": revision_required,
            "decision_reason": self._generate_decision_reason(review_result)
        }
        
        return final_decision
    
    def _calculate_review_confidence(self, review_result: Dict[str, Any]) -> float:
        """计算审核置信度"""
        overall_score = review_result["overall_score"]
        dimension_scores = review_result["dimension_scores"]
        
        # 基础置信度基于总分
        base_confidence = overall_score
        
        # 根据维度分数的一致性调整
        scores = list(dimension_scores.values())
        if scores:
            score_variance = sum((s - overall_score) ** 2 for s in scores) / len(scores)
            consistency_bonus = max(0, 0.2 - score_variance)
            base_confidence += consistency_bonus
        
        return round(min(1.0, base_confidence), 3)
    
    def _generate_decision_reason(self, review_result: Dict[str, Any]) -> str:
        """生成决策原因"""
        approved = review_result["approved"]
        overall_score = review_result["overall_score"]
        dimension_scores = review_result["dimension_scores"]
        
        if approved:
            return f"分析质量良好，总分 {overall_score:.2f} 达到审核标准"
        else:
            # 找出最低的维度
            lowest_dim = min(dimension_scores.items(), key=lambda x: x[1])
            return f"分析质量不足，总分 {overall_score:.2f}，{lowest_dim[0]} 维度得分较低 ({lowest_dim[1]:.2f})"
    
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
