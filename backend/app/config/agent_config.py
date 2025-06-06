"""
Agent配置管理器
"""

import os
import yaml
import json
import logging
from typing import Dict, Any, List, Optional
from pathlib import Path
from datetime import datetime

class AgentConfigManager:
    """Agent配置管理器"""
    
    def __init__(self, config_dir: str = "config/agents"):
        self.config_dir = Path(config_dir)
        self.logger = logging.getLogger("agent_config_manager")
        self.configs: Dict[str, Dict[str, Any]] = {}
        self.config_versions: Dict[str, str] = {}
        self._ensure_config_dir()
        self._load_all_configs()
    
    def _ensure_config_dir(self):
        """确保配置目录存在"""
        self.config_dir.mkdir(parents=True, exist_ok=True)
        
        # 创建默认的Agent类型目录
        for agent_type in ["analyzer", "reviewer"]:
            agent_dir = self.config_dir / agent_type
            agent_dir.mkdir(exist_ok=True)
            
            # 创建prompts子目录
            prompts_dir = agent_dir / "prompts"
            prompts_dir.mkdir(exist_ok=True)
    
    def _load_all_configs(self):
        """加载所有配置"""
        try:
            for agent_type_dir in self.config_dir.iterdir():
                if agent_type_dir.is_dir():
                    agent_type = agent_type_dir.name
                    config_file = agent_type_dir / "config.yaml"
                    
                    if config_file.exists():
                        self._load_agent_config(agent_type, config_file)
                    else:
                        # 创建默认配置
                        self._create_default_config(agent_type)
            
            self.logger.info(f"加载了 {len(self.configs)} 个Agent配置")
            
        except Exception as e:
            self.logger.error(f"加载配置失败: {str(e)}")
    
    def _load_agent_config(self, agent_type: str, config_file: Path):
        """加载单个Agent配置"""
        try:
            with open(config_file, 'r', encoding='utf-8') as f:
                config = yaml.safe_load(f)
            
            self.configs[agent_type] = config
            self.config_versions[agent_type] = config.get("version", "1.0")
            
            self.logger.info(f"加载Agent配置: {agent_type} v{self.config_versions[agent_type]}")
            
        except Exception as e:
            self.logger.error(f"加载Agent配置失败 {agent_type}: {str(e)}")
    
    def _create_default_config(self, agent_type: str):
        """创建默认配置"""
        if agent_type == "analyzer":
            default_config = {
                "name": "Git Commit Analyzer",
                "version": "1.0",
                "description": "分析Git提交记录并生成结构化总结",
                "llm_client": "openai",
                "prompt_template": "git_commit_analyzer",
                "max_tokens": 1000,
                "temperature": 0.3,
                "max_retries": 3,
                "timeout": 30,
                "capabilities": [
                    "commit_analysis",
                    "semantic_understanding",
                    "change_categorization",
                    "impact_assessment"
                ]
            }
        elif agent_type == "reviewer":
            default_config = {
                "name": "Quality Reviewer",
                "version": "1.0", 
                "description": "审核分析结果的质量并提供反馈",
                "llm_client": "openai",
                "prompt_template": "quality_reviewer",
                "max_tokens": 800,
                "temperature": 0.2,
                "max_retries": 3,
                "approval_threshold": 0.85,
                "dimension_weights": {
                    "accuracy": 0.3,
                    "completeness": 0.25,
                    "consistency": 0.25,
                    "clarity": 0.2
                },
                "capabilities": [
                    "quality_assessment",
                    "accuracy_evaluation",
                    "completeness_check",
                    "consistency_validation"
                ]
            }
        else:
            default_config = {
                "name": f"Default {agent_type.title()} Agent",
                "version": "1.0",
                "description": f"默认的{agent_type}配置"
            }
        
        # 保存默认配置
        config_file = self.config_dir / agent_type / "config.yaml"
        try:
            with open(config_file, 'w', encoding='utf-8') as f:
                yaml.dump(default_config, f, ensure_ascii=False, indent=2)
            
            self.configs[agent_type] = default_config
            self.config_versions[agent_type] = default_config.get("version", "1.0")
            
            self.logger.info(f"创建默认配置: {agent_type}")
            
        except Exception as e:
            self.logger.error(f"创建默认配置失败 {agent_type}: {str(e)}")
    
    def get_config(self, agent_type: str) -> Optional[Dict[str, Any]]:
        """获取Agent配置"""
        return self.configs.get(agent_type)
    
    def update_config(self, agent_type: str, new_config: Dict[str, Any], save: bool = True) -> bool:
        """更新Agent配置"""
        try:
            # 验证配置
            from .config_validator import ConfigValidator
            validator = ConfigValidator()
            
            if not validator.validate_agent_config(agent_type, new_config):
                raise ValueError("配置验证失败")
            
            # 备份当前配置
            old_config = self.configs.get(agent_type, {}).copy()
            
            # 更新版本号
            old_version = new_config.get("version", "1.0")
            version_parts = old_version.split(".")
            if len(version_parts) >= 2:
                minor_version = int(version_parts[1]) + 1
                new_config["version"] = f"{version_parts[0]}.{minor_version}"
            else:
                new_config["version"] = "1.1"
            
            # 更新配置
            self.configs[agent_type] = new_config
            self.config_versions[agent_type] = new_config["version"]
            
            if save:
                self._save_config(agent_type, new_config)
            
            self.logger.info(f"更新Agent配置: {agent_type} v{new_config['version']}")
            return True
            
        except Exception as e:
            # 回滚配置
            if agent_type in self.configs:
                self.configs[agent_type] = old_config
            
            self.logger.error(f"更新Agent配置失败 {agent_type}: {str(e)}")
            return False
    
    def _save_config(self, agent_type: str, config: Dict[str, Any]):
        """保存配置到文件"""
        config_file = self.config_dir / agent_type / "config.yaml"
        
        # 备份旧配置
        if config_file.exists():
            backup_file = config_file.with_suffix(f".backup.{datetime.now().strftime('%Y%m%d_%H%M%S')}.yaml")
            config_file.rename(backup_file)
        
        # 保存新配置
        with open(config_file, 'w', encoding='utf-8') as f:
            yaml.dump(config, f, ensure_ascii=False, indent=2)
    
    def list_agent_types(self) -> List[str]:
        """列出所有Agent类型"""
        return list(self.configs.keys())
    
    def get_config_version(self, agent_type: str) -> Optional[str]:
        """获取配置版本"""
        return self.config_versions.get(agent_type)
    
    def reload_config(self, agent_type: str) -> bool:
        """重新加载配置"""
        try:
            config_file = self.config_dir / agent_type / "config.yaml"
            if config_file.exists():
                self._load_agent_config(agent_type, config_file)
                return True
            else:
                self.logger.warning(f"配置文件不存在: {agent_type}")
                return False
                
        except Exception as e:
            self.logger.error(f"重新加载配置失败 {agent_type}: {str(e)}")
            return False
    
    def export_config(self, agent_type: str, format: str = "yaml") -> Optional[str]:
        """导出配置"""
        config = self.get_config(agent_type)
        if not config:
            return None
        
        try:
            if format.lower() == "json":
                return json.dumps(config, indent=2, ensure_ascii=False)
            else:
                return yaml.dump(config, ensure_ascii=False, indent=2)
                
        except Exception as e:
            self.logger.error(f"导出配置失败 {agent_type}: {str(e)}")
            return None
    
    def import_config(self, agent_type: str, config_data: str, format: str = "yaml") -> bool:
        """导入配置"""
        try:
            if format.lower() == "json":
                config = json.loads(config_data)
            else:
                config = yaml.safe_load(config_data)
            
            return self.update_config(agent_type, config)
            
        except Exception as e:
            self.logger.error(f"导入配置失败 {agent_type}: {str(e)}")
            return False
    
    def get_config_history(self, agent_type: str) -> List[Dict[str, Any]]:
        """获取配置历史"""
        history = []
        agent_dir = self.config_dir / agent_type
        
        if not agent_dir.exists():
            return history
        
        # 查找备份文件
        for backup_file in agent_dir.glob("config.backup.*.yaml"):
            try:
                with open(backup_file, 'r', encoding='utf-8') as f:
                    config = yaml.safe_load(f)
                
                # 从文件名提取时间戳
                timestamp_str = backup_file.stem.split('.')[-1]
                timestamp = datetime.strptime(timestamp_str, '%Y%m%d_%H%M%S')
                
                history.append({
                    "version": config.get("version", "unknown"),
                    "timestamp": timestamp.isoformat(),
                    "file": str(backup_file),
                    "config": config
                })
                
            except Exception as e:
                self.logger.warning(f"读取备份配置失败 {backup_file}: {str(e)}")
        
        # 按时间戳排序
        history.sort(key=lambda x: x["timestamp"], reverse=True)
        return history

# 全局配置管理器实例
agent_config_manager = AgentConfigManager()
