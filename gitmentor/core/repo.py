"""
Git仓库访问和分析模块

提供Git仓库的基本访问和分析功能
"""

import os
import logging
from typing import Dict, List, Optional, Any
from dataclasses import dataclass, field
from datetime import datetime

# 尝试导入PyDriller，如果不可用则提供错误信息
try:
    from pydriller import Repository, Commit as PyDrillerCommit
    PYDRILLER_AVAILABLE = True
except ImportError:
    PYDRILLER_AVAILABLE = False

logger = logging.getLogger("gitmentor.core.repo")

@dataclass
class CommitInfo:
    """提交信息数据类"""
    hash: str
    message: str
    author_name: str
    author_email: str
    date: datetime
    insertions: int = 0
    deletions: int = 0
    files_changed: int = 0
    is_merge: bool = False
    branches: List[str] = field(default_factory=list)
    
    @classmethod
    def from_pydriller(cls, commit: 'PyDrillerCommit') -> 'CommitInfo':
        """从PyDriller的Commit对象创建CommitInfo"""
        return cls(
            hash=commit.hash,
            message=commit.msg,
            author_name=commit.author.name,
            author_email=commit.author.email,
            date=commit.author_date,
            insertions=commit.insertions,
            deletions=commit.deletions,
            files_changed=len(commit.modified_files),
            is_merge=commit.merge,
            branches=commit.branches
        )

@dataclass
class ContributorInfo:
    """贡献者信息数据类"""
    name: str
    email: str
    commits: int = 0
    insertions: int = 0
    deletions: int = 0
    files_changed: int = 0
    first_commit: Optional[datetime] = None
    last_commit: Optional[datetime] = None

@dataclass
class RepositoryInfo:
    """仓库信息数据类"""
    path: str
    name: str
    total_commits: int = 0
    total_contributors: int = 0
    contributors: Dict[str, ContributorInfo] = field(default_factory=dict)
    
    @property
    def is_valid(self) -> bool:
        """检查仓库路径是否有效"""
        return os.path.exists(os.path.join(self.path, '.git'))

class GitRepository:
    """Git仓库访问和分析类"""
    
    def __init__(self, path: str):
        """
        初始化Git仓库分析器
        
        Args:
            path: Git仓库的本地路径
        """
        self.path = os.path.abspath(path)
        self.name = os.path.basename(self.path)
        self._check_dependencies()
    
    def _check_dependencies(self) -> None:
        """检查必要的依赖是否可用"""
        if not PYDRILLER_AVAILABLE:
            logger.error("PyDriller库不可用，请安装: pip install pydriller")
            raise ImportError("缺少必要的依赖: PyDriller")
    
    def is_valid_repository(self) -> bool:
        """检查路径是否为有效的Git仓库"""
        return os.path.exists(os.path.join(self.path, '.git'))
    
    def get_repository_info(self, limit: Optional[int] = None) -> RepositoryInfo:
        """
        获取仓库基本信息
        
        Args:
            limit: 可选的提交数量限制，用于快速分析
            
        Returns:
            包含仓库信息的RepositoryInfo对象
        """
        if not self.is_valid_repository():
            raise ValueError(f"无效的Git仓库路径: {self.path}")
        
        repo_info = RepositoryInfo(path=self.path, name=self.name)
        contributors = {}
        
        try:
            # 使用PyDriller分析仓库
            repo = Repository(self.path)
            
            for i, commit in enumerate(repo.traverse_commits()):
                if limit and i >= limit:
                    break
                
                # 更新提交计数
                repo_info.total_commits += 1
                
                # 获取作者信息
                author_email = commit.author.email
                
                # 更新或创建贡献者信息
                if author_email not in contributors:
                    contributors[author_email] = ContributorInfo(
                        name=commit.author.name,
                        email=author_email,
                        first_commit=commit.author_date,
                        last_commit=commit.author_date
                    )
                    repo_info.total_contributors += 1
                
                contributor = contributors[author_email]
                contributor.commits += 1
                contributor.insertions += commit.insertions
                contributor.deletions += commit.deletions
                contributor.files_changed += len(commit.modified_files)
                
                # 更新首次和最近提交时间
                if not contributor.first_commit or commit.author_date < contributor.first_commit:
                    contributor.first_commit = commit.author_date
                if not contributor.last_commit or commit.author_date > contributor.last_commit:
                    contributor.last_commit = commit.author_date
            
            repo_info.contributors = contributors
            
        except Exception as e:
            logger.error(f"分析仓库时出错: {str(e)}")
            raise
        
        return repo_info
    
    def get_commits(self, limit: Optional[int] = None) -> List[CommitInfo]:
        """
        获取仓库的提交信息
        
        Args:
            limit: 可选的提交数量限制
            
        Returns:
            包含提交信息的CommitInfo对象列表
        """
        if not self.is_valid_repository():
            raise ValueError(f"无效的Git仓库路径: {self.path}")
        
        commits = []
        
        try:
            # 使用PyDriller分析仓库
            repo = Repository(self.path)
            
            for i, commit in enumerate(repo.traverse_commits()):
                if limit and i >= limit:
                    break
                
                commit_info = CommitInfo.from_pydriller(commit)
                commits.append(commit_info)
                
        except Exception as e:
            logger.error(f"获取提交信息时出错: {str(e)}")
            raise
        
        return commits 