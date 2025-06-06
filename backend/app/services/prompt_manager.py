"""
Prompt模板管理器
"""

import os
import yaml
import json
import logging
from typing import Dict, Any, Optional, List
from pathlib import Path
from string import Template

class PromptTemplate:
    """Prompt模板类"""
    
    def __init__(self, name: str, template: str, variables: List[str], metadata: Dict[str, Any] = None):
        self.name = name
        self.template = Template(template)
        self.variables = variables
        self.metadata = metadata or {}
        self.raw_template = template
    
    def format(self, **kwargs) -> str:
        """格式化模板"""
        try:
            # 检查必需变量
            missing_vars = set(self.variables) - set(kwargs.keys())
            if missing_vars:
                raise ValueError(f"缺少必需变量: {missing_vars}")
            
            return self.template.safe_substitute(**kwargs)
        except Exception as e:
            raise ValueError(f"模板格式化失败: {str(e)}")
    
    def validate_variables(self, variables: Dict[str, Any]) -> bool:
        """验证变量"""
        required_vars = set(self.variables)
        provided_vars = set(variables.keys())
        return required_vars.issubset(provided_vars)

class PromptManager:
    """Prompt管理器"""
    
    def __init__(self, config_dir: str = "config/prompts"):
        self.config_dir = Path(config_dir)
        self.templates: Dict[str, PromptTemplate] = {}
        self.logger = logging.getLogger("prompt_manager")
        self._load_templates()
    
    def _load_templates(self):
        """加载所有模板"""
        try:
            if not self.config_dir.exists():
                self.logger.warning(f"Prompt配置目录不存在: {self.config_dir}")
                self._create_default_templates()
                return
            
            # 递归加载所有yaml和json文件
            for file_path in self.config_dir.rglob("*.yaml"):
                self._load_template_file(file_path)
            
            for file_path in self.config_dir.rglob("*.yml"):
                self._load_template_file(file_path)
            
            for file_path in self.config_dir.rglob("*.json"):
                self._load_template_file(file_path)
            
            self.logger.info(f"加载了 {len(self.templates)} 个Prompt模板")
            
        except Exception as e:
            self.logger.error(f"加载Prompt模板失败: {str(e)}")
            self._create_default_templates()
    
    def _load_template_file(self, file_path: Path):
        """加载单个模板文件"""
        try:
            with open(file_path, 'r', encoding='utf-8') as f:
                if file_path.suffix in ['.yaml', '.yml']:
                    data = yaml.safe_load(f)
                else:
                    data = json.load(f)
            
            # 支持单个模板或多个模板
            if isinstance(data, dict):
                if 'templates' in data:
                    # 多个模板
                    for template_data in data['templates']:
                        self._create_template_from_data(template_data)
                else:
                    # 单个模板
                    self._create_template_from_data(data)
            
        except Exception as e:
            self.logger.error(f"加载模板文件失败 {file_path}: {str(e)}")
    
    def _create_template_from_data(self, data: Dict[str, Any]):
        """从数据创建模板"""
        try:
            name = data.get('name')
            template = data.get('template')
            variables = data.get('variables', [])
            metadata = data.get('metadata', {})
            
            if not name or not template:
                self.logger.warning("模板缺少必需字段: name 或 template")
                return
            
            prompt_template = PromptTemplate(name, template, variables, metadata)
            self.templates[name] = prompt_template
            
        except Exception as e:
            self.logger.error(f"创建模板失败: {str(e)}")
    
    def _create_default_templates(self):
        """创建默认模板"""
        # 创建配置目录
        self.config_dir.mkdir(parents=True, exist_ok=True)
        
        # 默认分析器模板
        analyzer_template = {
            "name": "git_commit_analyzer",
            "template": """你是一个专业的Git提交分析专家。请分析以下提交记录：

提交哈希: $commit_hash
提交信息: $commit_message
作者: $author_name <$author_email>
提交时间: $commit_date
文件变更: $files_changed 个文件
代码行数: +$insertions -$deletions

请生成结构化的分析结果，包括：
1. 简洁的总结（不超过100字）
2. 变更类型分类（feature/bugfix/refactor/docs/style/test/chore）
3. 影响范围评估（low/medium/high）
4. 置信度评分（0.0-1.0）

请以JSON格式返回结果：
{
  "summary": "简洁的总结",
  "category": "变更类型",
  "impact_level": "影响级别",
  "confidence_score": 0.85,
  "description": "详细描述"
}""",
            "variables": [
                "commit_hash", "commit_message", "author_name", 
                "author_email", "commit_date", "files_changed", 
                "insertions", "deletions"
            ],
            "metadata": {
                "version": "1.0",
                "agent_type": "analyzer"
            }
        }
        
        # 默认审核器模板
        reviewer_template = {
            "name": "quality_reviewer",
            "template": """你是一个严格的代码质量审核专家。请审核以下Git提交分析结果的质量：

原始提交信息:
- 哈希: $commit_hash
- 消息: $commit_message
- 文件变更: $files_changed 个文件

分析结果:
- 总结: $analysis_summary
- 类型: $analysis_category
- 影响级别: $analysis_impact_level
- 置信度: $analysis_confidence

请从以下维度评估分析质量（0.0-1.0评分）：
1. 准确性：分析结果是否准确反映提交内容
2. 完整性：是否包含所有重要信息
3. 一致性：分类和描述是否逻辑一致
4. 清晰度：描述是否清晰易懂

请以JSON格式返回审核结果：
{
  "approved": true/false,
  "overall_score": 0.92,
  "dimension_scores": {
    "accuracy": 0.95,
    "completeness": 0.90,
    "consistency": 0.93,
    "clarity": 0.90
  },
  "feedback": "审核意见",
  "suggestions": ["改进建议1", "改进建议2"]
}""",
            "variables": [
                "commit_hash", "commit_message", "files_changed",
                "analysis_summary", "analysis_category", 
                "analysis_impact_level", "analysis_confidence"
            ],
            "metadata": {
                "version": "1.0",
                "agent_type": "reviewer"
            }
        }
        
        # 保存默认模板
        analyzer_file = self.config_dir / "analyzer_templates.yaml"
        reviewer_file = self.config_dir / "reviewer_templates.yaml"
        
        with open(analyzer_file, 'w', encoding='utf-8') as f:
            yaml.dump({"templates": [analyzer_template]}, f, ensure_ascii=False, indent=2)
        
        with open(reviewer_file, 'w', encoding='utf-8') as f:
            yaml.dump({"templates": [reviewer_template]}, f, ensure_ascii=False, indent=2)
        
        # 加载默认模板
        self._create_template_from_data(analyzer_template)
        self._create_template_from_data(reviewer_template)
        
        self.logger.info("创建了默认Prompt模板")
    
    def get_template(self, name: str) -> Optional[PromptTemplate]:
        """获取模板"""
        return self.templates.get(name)
    
    def format_template(self, name: str, **kwargs) -> str:
        """格式化模板"""
        template = self.get_template(name)
        if not template:
            raise ValueError(f"模板不存在: {name}")
        
        return template.format(**kwargs)
    
    def list_templates(self) -> List[str]:
        """列出所有模板名称"""
        return list(self.templates.keys())
    
    def get_templates_by_agent_type(self, agent_type: str) -> List[str]:
        """根据Agent类型获取模板"""
        result = []
        for name, template in self.templates.items():
            if template.metadata.get("agent_type") == agent_type:
                result.append(name)
        return result
    
    def reload_templates(self):
        """重新加载所有模板"""
        self.templates.clear()
        self._load_templates()
    
    def validate_template_variables(self, name: str, variables: Dict[str, Any]) -> bool:
        """验证模板变量"""
        template = self.get_template(name)
        if not template:
            return False
        
        return template.validate_variables(variables)

# 全局Prompt管理器实例
prompt_manager = PromptManager()
