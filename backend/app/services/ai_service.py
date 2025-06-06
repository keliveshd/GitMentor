"""
AI服务集成模块
"""

import asyncio
import json
import logging
from typing import Dict, Any, Optional, List
from enum import Enum
import httpx

class LLMProvider(Enum):
    """LLM提供商枚举"""
    OPENAI = "openai"
    ANTHROPIC = "anthropic"
    LOCAL = "local"

class AIServiceError(Exception):
    """AI服务异常"""
    pass

class LLMClient:
    """LLM客户端基类"""
    
    def __init__(self, provider: LLMProvider, config: Dict[str, Any]):
        self.provider = provider
        self.config = config
        self.logger = logging.getLogger(f"llm.{provider.value}")
        self._client = None
        self._initialize()
    
    def _initialize(self):
        """初始化客户端"""
        try:
            if self.provider == LLMProvider.OPENAI:
                self._initialize_openai()
            elif self.provider == LLMProvider.ANTHROPIC:
                self._initialize_anthropic()
            elif self.provider == LLMProvider.LOCAL:
                self._initialize_local()
            else:
                raise AIServiceError(f"不支持的LLM提供商: {self.provider}")
        except Exception as e:
            self.logger.error(f"LLM客户端初始化失败: {str(e)}")
            raise
    
    def _initialize_openai(self):
        """初始化OpenAI客户端"""
        api_key = self.config.get("api_key")
        if not api_key:
            raise AIServiceError("OpenAI API密钥未配置")
        
        self.base_url = self.config.get("base_url", "https://api.openai.com/v1")
        self.model = self.config.get("model", "gpt-3.5-turbo")
        self.headers = {
            "Authorization": f"Bearer {api_key}",
            "Content-Type": "application/json"
        }
    
    def _initialize_anthropic(self):
        """初始化Anthropic客户端"""
        api_key = self.config.get("api_key")
        if not api_key:
            raise AIServiceError("Anthropic API密钥未配置")
        
        self.base_url = self.config.get("base_url", "https://api.anthropic.com/v1")
        self.model = self.config.get("model", "claude-3-sonnet-20240229")
        self.headers = {
            "x-api-key": api_key,
            "Content-Type": "application/json",
            "anthropic-version": "2023-06-01"
        }
    
    def _initialize_local(self):
        """初始化本地LLM客户端"""
        self.base_url = self.config.get("base_url", "http://localhost:11434")
        self.model = self.config.get("model", "llama2")
        self.headers = {"Content-Type": "application/json"}
    
    async def complete(self, prompt: str, **kwargs) -> Dict[str, Any]:
        """
        调用LLM完成文本生成
        
        Args:
            prompt: 输入提示词
            **kwargs: 其他参数
            
        Returns:
            Dict: LLM响应结果
        """
        try:
            if self.provider == LLMProvider.OPENAI:
                return await self._complete_openai(prompt, **kwargs)
            elif self.provider == LLMProvider.ANTHROPIC:
                return await self._complete_anthropic(prompt, **kwargs)
            elif self.provider == LLMProvider.LOCAL:
                return await self._complete_local(prompt, **kwargs)
        except Exception as e:
            self.logger.error(f"LLM调用失败: {str(e)}")
            raise AIServiceError(f"LLM调用失败: {str(e)}")
    
    async def _complete_openai(self, prompt: str, **kwargs) -> Dict[str, Any]:
        """OpenAI API调用"""
        payload = {
            "model": self.model,
            "messages": [{"role": "user", "content": prompt}],
            "max_tokens": kwargs.get("max_tokens", 1000),
            "temperature": kwargs.get("temperature", 0.3),
            "top_p": kwargs.get("top_p", 1.0)
        }
        
        async with httpx.AsyncClient(timeout=30.0) as client:
            response = await client.post(
                f"{self.base_url}/chat/completions",
                headers=self.headers,
                json=payload
            )
            response.raise_for_status()
            
            result = response.json()
            return {
                "content": result["choices"][0]["message"]["content"],
                "usage": result.get("usage", {}),
                "model": result.get("model", self.model)
            }
    
    async def _complete_anthropic(self, prompt: str, **kwargs) -> Dict[str, Any]:
        """Anthropic API调用"""
        payload = {
            "model": self.model,
            "max_tokens": kwargs.get("max_tokens", 1000),
            "temperature": kwargs.get("temperature", 0.3),
            "messages": [{"role": "user", "content": prompt}]
        }
        
        async with httpx.AsyncClient(timeout=30.0) as client:
            response = await client.post(
                f"{self.base_url}/messages",
                headers=self.headers,
                json=payload
            )
            response.raise_for_status()
            
            result = response.json()
            return {
                "content": result["content"][0]["text"],
                "usage": result.get("usage", {}),
                "model": result.get("model", self.model)
            }
    
    async def _complete_local(self, prompt: str, **kwargs) -> Dict[str, Any]:
        """本地LLM API调用（Ollama格式）"""
        payload = {
            "model": self.model,
            "prompt": prompt,
            "stream": False,
            "options": {
                "temperature": kwargs.get("temperature", 0.3),
                "top_p": kwargs.get("top_p", 1.0)
            }
        }
        
        async with httpx.AsyncClient(timeout=60.0) as client:
            response = await client.post(
                f"{self.base_url}/api/generate",
                headers=self.headers,
                json=payload
            )
            response.raise_for_status()
            
            result = response.json()
            return {
                "content": result.get("response", ""),
                "usage": {},
                "model": result.get("model", self.model)
            }
    
    async def test_connection(self) -> bool:
        """测试连接"""
        try:
            test_prompt = "Hello, this is a test message."
            result = await self.complete(test_prompt, max_tokens=10)
            return bool(result.get("content"))
        except Exception as e:
            self.logger.error(f"连接测试失败: {str(e)}")
            return False

class AIService:
    """AI服务管理器"""
    
    def __init__(self):
        self.clients: Dict[str, LLMClient] = {}
        self.default_client: Optional[LLMClient] = None
        self.logger = logging.getLogger("ai_service")
    
    def register_client(self, name: str, provider: LLMProvider, config: Dict[str, Any]):
        """注册LLM客户端"""
        try:
            client = LLMClient(provider, config)
            self.clients[name] = client
            
            # 设置默认客户端
            if self.default_client is None or config.get("default", False):
                self.default_client = client
            
            self.logger.info(f"LLM客户端 {name} 注册成功")
        except Exception as e:
            self.logger.error(f"LLM客户端 {name} 注册失败: {str(e)}")
            raise
    
    def get_client(self, name: Optional[str] = None) -> LLMClient:
        """获取LLM客户端"""
        if name:
            if name not in self.clients:
                raise AIServiceError(f"LLM客户端 {name} 不存在")
            return self.clients[name]
        
        if self.default_client is None:
            raise AIServiceError("没有可用的LLM客户端")
        
        return self.default_client
    
    async def complete(self, prompt: str, client_name: Optional[str] = None, **kwargs) -> Dict[str, Any]:
        """调用LLM完成文本生成"""
        client = self.get_client(client_name)
        return await client.complete(prompt, **kwargs)
    
    async def test_all_connections(self) -> Dict[str, bool]:
        """测试所有客户端连接"""
        results = {}
        for name, client in self.clients.items():
            results[name] = await client.test_connection()
        return results
    
    def get_available_clients(self) -> List[str]:
        """获取可用客户端列表"""
        return list(self.clients.keys())

# 全局AI服务实例
ai_service = AIService()
