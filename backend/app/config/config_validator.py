"""
配置验证器
"""

import logging
from typing import Dict, Any, List, Optional

class ConfigValidator:
    """配置验证器"""
    
    def __init__(self):
        self.logger = logging.getLogger("config_validator")
    
    def validate_agent_config(self, agent_type: str, config: Dict[str, Any]) -> bool:
        """验证Agent配置"""
        try:
            # 基础字段验证
            required_fields = ["name", "version"]
            for field in required_fields:
                if field not in config:
                    raise ValueError(f"缺少必需字段: {field}")
            
            # 类型特定验证
            if agent_type == "analyzer":
                return self._validate_analyzer_config(config)
            elif agent_type == "reviewer":
                return self._validate_reviewer_config(config)
            else:
                # 通用验证
                return self._validate_generic_config(config)
                
        except Exception as e:
            self.logger.error(f"配置验证失败 {agent_type}: {str(e)}")
            return False
    
    def _validate_analyzer_config(self, config: Dict[str, Any]) -> bool:
        """验证分析器配置"""
        required_fields = [
            "llm_client", "prompt_template", "max_tokens", 
            "temperature", "max_retries", "timeout"
        ]
        
        for field in required_fields:
            if field not in config:
                raise ValueError(f"分析器配置缺少必需字段: {field}")
        
        # 数值范围验证
        if not 0 <= config["temperature"] <= 2:
            raise ValueError("temperature必须在0-2范围内")
        
        if not 1 <= config["max_tokens"] <= 4000:
            raise ValueError("max_tokens必须在1-4000范围内")
        
        if not 1 <= config["max_retries"] <= 10:
            raise ValueError("max_retries必须在1-10范围内")
        
        if not 1 <= config["timeout"] <= 300:
            raise ValueError("timeout必须在1-300秒范围内")
        
        return True
    
    def _validate_reviewer_config(self, config: Dict[str, Any]) -> bool:
        """验证审核器配置"""
        required_fields = [
            "llm_client", "prompt_template", "max_tokens",
            "temperature", "max_retries", "approval_threshold"
        ]
        
        for field in required_fields:
            if field not in config:
                raise ValueError(f"审核器配置缺少必需字段: {field}")
        
        # 数值范围验证
        if not 0 <= config["temperature"] <= 1:
            raise ValueError("审核器temperature必须在0-1范围内")
        
        if not 0.5 <= config["approval_threshold"] <= 1.0:
            raise ValueError("approval_threshold必须在0.5-1.0范围内")
        
        # 维度权重验证
        if "dimension_weights" in config:
            weights = config["dimension_weights"]
            if not isinstance(weights, dict):
                raise ValueError("dimension_weights必须是字典类型")
            
            required_dimensions = ["accuracy", "completeness", "consistency", "clarity"]
            for dim in required_dimensions:
                if dim not in weights:
                    raise ValueError(f"dimension_weights缺少维度: {dim}")
                
                if not 0 <= weights[dim] <= 1:
                    raise ValueError(f"维度权重 {dim} 必须在0-1范围内")
            
            # 权重总和应该接近1.0
            total_weight = sum(weights.values())
            if not 0.8 <= total_weight <= 1.2:
                raise ValueError(f"维度权重总和应该接近1.0，当前为: {total_weight}")
        
        return True
    
    def _validate_generic_config(self, config: Dict[str, Any]) -> bool:
        """验证通用配置"""
        # 版本格式验证
        version = config.get("version", "")
        if not self._validate_version_format(version):
            raise ValueError(f"版本格式无效: {version}")
        
        # 名称验证
        name = config.get("name", "")
        if not name or len(name.strip()) == 0:
            raise ValueError("名称不能为空")
        
        return True
    
    def _validate_version_format(self, version: str) -> bool:
        """验证版本格式"""
        try:
            parts = version.split(".")
            if len(parts) < 2:
                return False
            
            # 检查每个部分都是数字
            for part in parts:
                int(part)
            
            return True
            
        except (ValueError, AttributeError):
            return False
    
    def validate_repository_config(self, config: Dict[str, Any]) -> bool:
        """验证仓库配置"""
        try:
            required_fields = ["name", "path", "type"]
            for field in required_fields:
                if field not in config:
                    raise ValueError(f"仓库配置缺少必需字段: {field}")
            
            # 路径验证
            path = config["path"]
            if not path or not isinstance(path, str):
                raise ValueError("仓库路径无效")
            
            # 类型验证
            valid_types = ["git", "local"]
            if config["type"] not in valid_types:
                raise ValueError(f"仓库类型必须是: {valid_types}")
            
            # Agent配置验证
            if "agents" in config:
                agents = config["agents"]
                if not isinstance(agents, dict):
                    raise ValueError("agents配置必须是字典类型")
                
                for agent_type, agent_config in agents.items():
                    if not isinstance(agent_config, str):
                        raise ValueError(f"Agent配置 {agent_type} 必须是字符串")
            
            # 用户映射验证
            if "user_mapping" in config:
                user_mapping = config["user_mapping"]
                if not isinstance(user_mapping, dict):
                    raise ValueError("user_mapping必须是字典类型")
                
                for email, user_info in user_mapping.items():
                    if not self._validate_email_format(email):
                        raise ValueError(f"邮箱格式无效: {email}")
                    
                    if isinstance(user_info, str):
                        # 简单的字符串映射
                        continue
                    elif isinstance(user_info, dict):
                        # 详细的用户信息
                        if "display_name" not in user_info:
                            raise ValueError(f"用户信息缺少display_name: {email}")
                    else:
                        raise ValueError(f"用户映射格式无效: {email}")
            
            return True
            
        except Exception as e:
            self.logger.error(f"仓库配置验证失败: {str(e)}")
            return False
    
    def _validate_email_format(self, email: str) -> bool:
        """验证邮箱格式"""
        import re
        pattern = r'^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$'
        return bool(re.match(pattern, email))
    
    def validate_llm_client_config(self, config: Dict[str, Any]) -> bool:
        """验证LLM客户端配置"""
        try:
            required_fields = ["provider"]
            for field in required_fields:
                if field not in config:
                    raise ValueError(f"LLM客户端配置缺少必需字段: {field}")
            
            provider = config["provider"]
            
            if provider == "openai":
                if "api_key" not in config:
                    raise ValueError("OpenAI配置缺少api_key")
                if "model" not in config:
                    config["model"] = "gpt-3.5-turbo"  # 设置默认值
                    
            elif provider == "anthropic":
                if "api_key" not in config:
                    raise ValueError("Anthropic配置缺少api_key")
                if "model" not in config:
                    config["model"] = "claude-3-sonnet-20240229"
                    
            elif provider == "local":
                if "base_url" not in config:
                    config["base_url"] = "http://localhost:11434"
                if "model" not in config:
                    config["model"] = "llama2"
            else:
                raise ValueError(f"不支持的LLM提供商: {provider}")
            
            return True
            
        except Exception as e:
            self.logger.error(f"LLM客户端配置验证失败: {str(e)}")
            return False
    
    def get_validation_errors(self, config_type: str, config: Dict[str, Any]) -> List[str]:
        """获取详细的验证错误信息"""
        errors = []
        
        try:
            if config_type == "agent":
                # 这里可以添加更详细的错误收集逻辑
                pass
            elif config_type == "repository":
                # 仓库配置错误收集
                pass
            elif config_type == "llm_client":
                # LLM客户端配置错误收集
                pass
                
        except Exception as e:
            errors.append(str(e))
        
        return errors
