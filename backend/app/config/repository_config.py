"""
仓库配置管理器
"""

import os
import yaml
import logging
from typing import Dict, Any, List, Optional
from pathlib import Path

class RepositoryConfigManager:
    """仓库配置管理器"""
    
    def __init__(self, config_file: str = "config/repositories.yaml"):
        self.config_file = Path(config_file)
        self.logger = logging.getLogger("repository_config_manager")
        self.repositories: Dict[str, Dict[str, Any]] = {}
        self._ensure_config_file()
        self._load_repositories()
    
    def _ensure_config_file(self):
        """确保配置文件存在"""
        self.config_file.parent.mkdir(parents=True, exist_ok=True)
        
        if not self.config_file.exists():
            self._create_default_config()
    
    def _create_default_config(self):
        """创建默认配置"""
        default_config = {
            "repositories": [
                {
                    "name": "example-repo",
                    "path": "/path/to/your/repository",
                    "type": "git",
                    "enabled": False,
                    "agents": {
                        "analyzer": "default_analyzer",
                        "reviewer": "default_reviewer"
                    },
                    "user_mapping": {
                        "user@example.com": {
                            "display_name": "Example User",
                            "aliases": ["user@company.com"],
                            "team": "Development Team",
                            "role": "Developer"
                        }
                    },
                    "analysis_settings": {
                        "auto_analysis": True,
                        "batch_size": 10,
                        "retry_limit": 3,
                        "quality_threshold": 0.85
                    }
                }
            ]
        }
        
        try:
            with open(self.config_file, 'w', encoding='utf-8') as f:
                yaml.dump(default_config, f, ensure_ascii=False, indent=2)
            
            self.logger.info("创建默认仓库配置文件")
            
        except Exception as e:
            self.logger.error(f"创建默认配置失败: {str(e)}")
    
    def _load_repositories(self):
        """加载仓库配置"""
        try:
            with open(self.config_file, 'r', encoding='utf-8') as f:
                config = yaml.safe_load(f)
            
            repositories = config.get("repositories", [])
            
            for repo_config in repositories:
                repo_name = repo_config.get("name")
                if repo_name:
                    self.repositories[repo_name] = repo_config
            
            self.logger.info(f"加载了 {len(self.repositories)} 个仓库配置")
            
        except Exception as e:
            self.logger.error(f"加载仓库配置失败: {str(e)}")
    
    def get_repository(self, name: str) -> Optional[Dict[str, Any]]:
        """获取仓库配置"""
        return self.repositories.get(name)
    
    def list_repositories(self) -> List[str]:
        """列出所有仓库名称"""
        return list(self.repositories.keys())
    
    def get_enabled_repositories(self) -> List[Dict[str, Any]]:
        """获取启用的仓库列表"""
        return [
            repo for repo in self.repositories.values()
            if repo.get("enabled", False)
        ]
    
    def add_repository(self, repo_config: Dict[str, Any]) -> bool:
        """添加仓库配置"""
        try:
            # 验证配置
            from .config_validator import ConfigValidator
            validator = ConfigValidator()
            
            if not validator.validate_repository_config(repo_config):
                raise ValueError("仓库配置验证失败")
            
            repo_name = repo_config["name"]
            
            # 检查是否已存在
            if repo_name in self.repositories:
                raise ValueError(f"仓库已存在: {repo_name}")
            
            # 添加配置
            self.repositories[repo_name] = repo_config
            
            # 保存到文件
            self._save_repositories()
            
            self.logger.info(f"添加仓库配置: {repo_name}")
            return True
            
        except Exception as e:
            self.logger.error(f"添加仓库配置失败: {str(e)}")
            return False
    
    def update_repository(self, name: str, repo_config: Dict[str, Any]) -> bool:
        """更新仓库配置"""
        try:
            if name not in self.repositories:
                raise ValueError(f"仓库不存在: {name}")
            
            # 验证配置
            from .config_validator import ConfigValidator
            validator = ConfigValidator()
            
            if not validator.validate_repository_config(repo_config):
                raise ValueError("仓库配置验证失败")
            
            # 更新配置
            self.repositories[name] = repo_config
            
            # 保存到文件
            self._save_repositories()
            
            self.logger.info(f"更新仓库配置: {name}")
            return True
            
        except Exception as e:
            self.logger.error(f"更新仓库配置失败: {str(e)}")
            return False
    
    def remove_repository(self, name: str) -> bool:
        """移除仓库配置"""
        try:
            if name not in self.repositories:
                raise ValueError(f"仓库不存在: {name}")
            
            del self.repositories[name]
            
            # 保存到文件
            self._save_repositories()
            
            self.logger.info(f"移除仓库配置: {name}")
            return True
            
        except Exception as e:
            self.logger.error(f"移除仓库配置失败: {str(e)}")
            return False
    
    def enable_repository(self, name: str) -> bool:
        """启用仓库"""
        return self._set_repository_enabled(name, True)
    
    def disable_repository(self, name: str) -> bool:
        """禁用仓库"""
        return self._set_repository_enabled(name, False)
    
    def _set_repository_enabled(self, name: str, enabled: bool) -> bool:
        """设置仓库启用状态"""
        try:
            if name not in self.repositories:
                raise ValueError(f"仓库不存在: {name}")
            
            self.repositories[name]["enabled"] = enabled
            
            # 保存到文件
            self._save_repositories()
            
            status = "启用" if enabled else "禁用"
            self.logger.info(f"{status}仓库: {name}")
            return True
            
        except Exception as e:
            self.logger.error(f"设置仓库状态失败: {str(e)}")
            return False
    
    def get_user_mapping(self, repo_name: str) -> Dict[str, Any]:
        """获取仓库的用户映射"""
        repo = self.get_repository(repo_name)
        if repo:
            return repo.get("user_mapping", {})
        return {}
    
    def update_user_mapping(self, repo_name: str, user_mapping: Dict[str, Any]) -> bool:
        """更新用户映射"""
        try:
            if repo_name not in self.repositories:
                raise ValueError(f"仓库不存在: {repo_name}")
            
            self.repositories[repo_name]["user_mapping"] = user_mapping
            
            # 保存到文件
            self._save_repositories()
            
            self.logger.info(f"更新用户映射: {repo_name}")
            return True
            
        except Exception as e:
            self.logger.error(f"更新用户映射失败: {str(e)}")
            return False
    
    def get_agent_config(self, repo_name: str) -> Dict[str, str]:
        """获取仓库的Agent配置"""
        repo = self.get_repository(repo_name)
        if repo:
            return repo.get("agents", {})
        return {}
    
    def update_agent_config(self, repo_name: str, agent_config: Dict[str, str]) -> bool:
        """更新Agent配置"""
        try:
            if repo_name not in self.repositories:
                raise ValueError(f"仓库不存在: {repo_name}")
            
            self.repositories[repo_name]["agents"] = agent_config
            
            # 保存到文件
            self._save_repositories()
            
            self.logger.info(f"更新Agent配置: {repo_name}")
            return True
            
        except Exception as e:
            self.logger.error(f"更新Agent配置失败: {str(e)}")
            return False
    
    def _save_repositories(self):
        """保存仓库配置到文件"""
        try:
            # 构建配置结构
            config = {
                "repositories": list(self.repositories.values())
            }
            
            # 备份现有文件
            if self.config_file.exists():
                backup_file = self.config_file.with_suffix('.backup.yaml')
                self.config_file.rename(backup_file)
            
            # 保存新配置
            with open(self.config_file, 'w', encoding='utf-8') as f:
                yaml.dump(config, f, ensure_ascii=False, indent=2)
            
        except Exception as e:
            self.logger.error(f"保存仓库配置失败: {str(e)}")
            raise
    
    def reload_repositories(self):
        """重新加载仓库配置"""
        self.repositories.clear()
        self._load_repositories()
    
    def get_repository_by_path(self, path: str) -> Optional[Dict[str, Any]]:
        """根据路径查找仓库"""
        for repo in self.repositories.values():
            if repo.get("path") == path:
                return repo
        return None
    
    def validate_repository_path(self, path: str) -> bool:
        """验证仓库路径"""
        try:
            repo_path = Path(path)
            
            # 检查路径是否存在
            if not repo_path.exists():
                return False
            
            # 检查是否是Git仓库
            git_dir = repo_path / ".git"
            if git_dir.exists():
                return True
            
            # 检查是否是有效的目录
            return repo_path.is_dir()
            
        except Exception:
            return False

# 全局仓库配置管理器实例
repository_config_manager = RepositoryConfigManager()
