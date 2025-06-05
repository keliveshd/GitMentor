"""
Git操作服务
"""

import git
from datetime import datetime
from typing import List, Dict, Any
from pathlib import Path

class GitService:
    def __init__(self, repo_path: str):
        """初始化Git服务"""
        self.repo_path = Path(repo_path)
        try:
            self.repo = git.Repo(repo_path)
        except git.exc.InvalidGitRepositoryError:
            raise ValueError(f"无效的Git仓库路径: {repo_path}")
    
    def get_commits_count(self) -> int:
        """获取提交总数"""
        try:
            return len(list(self.repo.iter_commits()))
        except Exception as e:
            print(f"获取提交数失败: {e}")
            return 0
    
    def get_commits(self, page: int = 1, page_size: int = 20) -> List[Dict[str, Any]]:
        """获取提交历史"""
        try:
            # 计算跳过的提交数
            skip = (page - 1) * page_size
            
            commits = []
            for i, commit in enumerate(self.repo.iter_commits()):
                if i < skip:
                    continue
                if len(commits) >= page_size:
                    break
                
                # 获取提交统计信息
                stats = commit.stats
                
                commit_data = {
                    "hash": commit.hexsha,
                    "message": commit.message.strip(),
                    "author_name": commit.author.name,
                    "author_email": commit.author.email,
                    "commit_date": commit.committed_datetime.isoformat(),
                    "files_changed": len(stats.files),
                    "insertions": stats.total["insertions"],
                    "deletions": stats.total["deletions"]
                }
                commits.append(commit_data)
            
            return commits
        except Exception as e:
            print(f"获取提交历史失败: {e}")
            return []
    
    def get_repository_stats(self) -> Dict[str, Any]:
        """获取仓库统计信息"""
        try:
            commits = list(self.repo.iter_commits())
            
            if not commits:
                return {
                    "total_commits": 0,
                    "contributors": 0,
                    "first_commit": None,
                    "last_commit": None,
                    "total_files": 0
                }
            
            # 统计贡献者
            contributors = set()
            total_insertions = 0
            total_deletions = 0
            
            for commit in commits:
                contributors.add(commit.author.email)
                stats = commit.stats
                total_insertions += stats.total["insertions"]
                total_deletions += stats.total["deletions"]
            
            # 获取文件数量
            try:
                total_files = len(list(self.repo.tree().traverse()))
            except:
                total_files = 0
            
            return {
                "total_commits": len(commits),
                "contributors": len(contributors),
                "first_commit": commits[-1].committed_datetime.isoformat(),
                "last_commit": commits[0].committed_datetime.isoformat(),
                "total_files": total_files,
                "total_insertions": total_insertions,
                "total_deletions": total_deletions
            }
        except Exception as e:
            print(f"获取仓库统计失败: {e}")
            return {
                "total_commits": 0,
                "contributors": 0,
                "first_commit": None,
                "last_commit": None,
                "total_files": 0,
                "total_insertions": 0,
                "total_deletions": 0
            }
    
    def get_contributors(self) -> List[Dict[str, Any]]:
        """获取贡献者列表"""
        try:
            contributors = {}
            
            for commit in self.repo.iter_commits():
                email = commit.author.email
                name = commit.author.name
                
                if email not in contributors:
                    contributors[email] = {
                        "name": name,
                        "email": email,
                        "commits": 0,
                        "insertions": 0,
                        "deletions": 0,
                        "first_commit": commit.committed_datetime,
                        "last_commit": commit.committed_datetime
                    }
                
                contributor = contributors[email]
                contributor["commits"] += 1
                
                stats = commit.stats
                contributor["insertions"] += stats.total["insertions"]
                contributor["deletions"] += stats.total["deletions"]
                
                # 更新时间范围
                if commit.committed_datetime < contributor["first_commit"]:
                    contributor["first_commit"] = commit.committed_datetime
                if commit.committed_datetime > contributor["last_commit"]:
                    contributor["last_commit"] = commit.committed_datetime
            
            # 转换为列表并格式化日期
            result = []
            for contributor in contributors.values():
                contributor["first_commit"] = contributor["first_commit"].isoformat()
                contributor["last_commit"] = contributor["last_commit"].isoformat()
                result.append(contributor)
            
            # 按提交数排序
            result.sort(key=lambda x: x["commits"], reverse=True)
            return result
            
        except Exception as e:
            print(f"获取贡献者列表失败: {e}")
            return []
