"""
文件组织器
"""

import os
import logging
from typing import Dict, Any, Optional
from pathlib import Path
from datetime import datetime

class FileOrganizer:
    """文件组织器"""
    
    def __init__(self, workspace_root: str = "workspace"):
        self.workspace_root = Path(workspace_root)
        self.logger = logging.getLogger("file_organizer")
        self._ensure_workspace()
    
    def _ensure_workspace(self):
        """确保工作空间目录存在"""
        self.workspace_root.mkdir(parents=True, exist_ok=True)
        self.logger.info(f"工作空间根目录: {self.workspace_root.absolute()}")
    
    def get_commit_file_path(self, 
                           repository_name: str,
                           contributor_email: str,
                           commit_date: str,
                           commit_hash: str) -> Path:
        """获取提交文件路径"""
        
        # 解析日期
        try:
            date_obj = datetime.fromisoformat(commit_date.replace('Z', '+00:00'))
        except:
            date_obj = datetime.now()
        
        year = date_obj.year
        month = f"{date_obj.month:02d}"
        
        # 构建路径: /workspace/{仓库名}/{贡献者}/{年份}/{月份}/commits/
        commit_dir = (self.workspace_root / 
                     repository_name / 
                     self._sanitize_email(contributor_email) / 
                     str(year) / 
                     month / 
                     "commits")
        
        # 确保目录存在
        commit_dir.mkdir(parents=True, exist_ok=True)
        
        # 返回文件路径
        return commit_dir / f"commit_{commit_hash[:8]}.md"
    
    def get_daily_summary_path(self,
                             repository_name: str,
                             contributor_email: str,
                             date: str) -> Path:
        """获取日度总结文件路径"""
        
        # 解析日期
        try:
            date_obj = datetime.strptime(date, '%Y-%m-%d')
        except:
            date_obj = datetime.now()
        
        year = date_obj.year
        month = f"{date_obj.month:02d}"
        
        # 构建路径
        summary_dir = (self.workspace_root / 
                      repository_name / 
                      self._sanitize_email(contributor_email) / 
                      str(year) / 
                      month / 
                      "daily_summaries")
        
        # 确保目录存在
        summary_dir.mkdir(parents=True, exist_ok=True)
        
        return summary_dir / f"{date}.md"
    
    def get_monthly_summary_path(self,
                               repository_name: str,
                               contributor_email: str,
                               year: int,
                               month: int) -> Path:
        """获取月度总结文件路径"""
        
        # 构建路径
        summary_dir = (self.workspace_root / 
                      repository_name / 
                      self._sanitize_email(contributor_email) / 
                      str(year))
        
        # 确保目录存在
        summary_dir.mkdir(parents=True, exist_ok=True)
        
        return summary_dir / f"monthly_summary_{year}-{month:02d}.md"
    
    def get_contributor_profile_path(self,
                                   repository_name: str,
                                   contributor_email: str) -> Path:
        """获取贡献者档案文件路径"""
        
        # 构建路径
        contributor_dir = (self.workspace_root / 
                          repository_name / 
                          self._sanitize_email(contributor_email))
        
        # 确保目录存在
        contributor_dir.mkdir(parents=True, exist_ok=True)
        
        return contributor_dir / "contributor_profile.md"
    
    def get_repository_overview_path(self, repository_name: str) -> Path:
        """获取仓库概览文件路径"""
        
        # 构建路径
        repo_dir = self.workspace_root / repository_name
        
        # 确保目录存在
        repo_dir.mkdir(parents=True, exist_ok=True)
        
        return repo_dir / "repository_overview.md"
    
    def get_analytics_path(self, repository_name: str, analytics_type: str) -> Path:
        """获取分析报告文件路径"""
        
        # 构建路径
        analytics_dir = self.workspace_root / repository_name / "analytics"
        
        # 确保目录存在
        analytics_dir.mkdir(parents=True, exist_ok=True)
        
        return analytics_dir / f"{analytics_type}.md"
    
    def _sanitize_email(self, email: str) -> str:
        """清理邮箱地址用作目录名"""
        # 替换不适合作为目录名的字符
        sanitized = email.replace("@", "_at_").replace(".", "_")
        # 移除其他特殊字符
        sanitized = "".join(c for c in sanitized if c.isalnum() or c in "_-")
        return sanitized
    
    def create_directory_structure(self, repository_name: str) -> bool:
        """为仓库创建完整的目录结构"""
        try:
            repo_dir = self.workspace_root / repository_name
            
            # 创建主要目录
            directories = [
                repo_dir,
                repo_dir / "analytics",
                repo_dir / "team_analytics"
            ]
            
            for directory in directories:
                directory.mkdir(parents=True, exist_ok=True)
            
            # 创建README文件
            readme_path = repo_dir / "README.md"
            if not readme_path.exists():
                readme_content = f"""# {repository_name} 分析报告

这个目录包含了 {repository_name} 仓库的AI分析报告。

## 目录结构

- `repository_overview.md` - 仓库概览
- `analytics/` - 详细分析报告
  - `quality_trends.md` - 质量趋势分析
  - `team_insights.md` - 团队洞察报告
- `team_analytics/` - 团队协作分析
- `[contributor_email]/` - 各贡献者的个人报告
  - `contributor_profile.md` - 贡献者档案
  - `[year]/` - 年度报告
    - `[month]/` - 月度报告
      - `commits/` - 提交分析报告
      - `daily_summaries/` - 日度总结
      - `monthly_summary_[year]-[month].md` - 月度总结

## 报告生成

所有报告都由GitMentor AI Agent双重审核系统自动生成。

生成时间: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}
"""
                
                with open(readme_path, 'w', encoding='utf-8') as f:
                    f.write(readme_content)
            
            self.logger.info(f"创建仓库目录结构: {repository_name}")
            return True
            
        except Exception as e:
            self.logger.error(f"创建目录结构失败 {repository_name}: {str(e)}")
            return False
    
    def cleanup_old_files(self, repository_name: str, days_to_keep: int = 90) -> int:
        """清理旧文件"""
        try:
            repo_dir = self.workspace_root / repository_name
            if not repo_dir.exists():
                return 0
            
            cutoff_time = datetime.now().timestamp() - (days_to_keep * 24 * 3600)
            deleted_count = 0
            
            # 递归查找并删除旧文件
            for file_path in repo_dir.rglob("*.md"):
                if file_path.stat().st_mtime < cutoff_time:
                    # 跳过重要文件
                    if file_path.name in ["repository_overview.md", "contributor_profile.md", "README.md"]:
                        continue
                    
                    try:
                        file_path.unlink()
                        deleted_count += 1
                    except Exception as e:
                        self.logger.warning(f"删除文件失败 {file_path}: {str(e)}")
            
            self.logger.info(f"清理了 {deleted_count} 个旧文件")
            return deleted_count
            
        except Exception as e:
            self.logger.error(f"清理旧文件失败: {str(e)}")
            return 0
    
    def get_file_statistics(self, repository_name: str) -> Dict[str, Any]:
        """获取文件统计信息"""
        try:
            repo_dir = self.workspace_root / repository_name
            if not repo_dir.exists():
                return {}
            
            stats = {
                "total_files": 0,
                "total_size": 0,
                "file_types": {},
                "contributors": set(),
                "date_range": {"earliest": None, "latest": None}
            }
            
            for file_path in repo_dir.rglob("*.md"):
                stats["total_files"] += 1
                stats["total_size"] += file_path.stat().st_size
                
                # 统计文件类型
                if "commit_" in file_path.name:
                    file_type = "commit_reports"
                elif "daily_summary" in file_path.name:
                    file_type = "daily_summaries"
                elif "monthly_summary" in file_path.name:
                    file_type = "monthly_summaries"
                elif file_path.name == "contributor_profile.md":
                    file_type = "contributor_profiles"
                else:
                    file_type = "other"
                
                stats["file_types"][file_type] = stats["file_types"].get(file_type, 0) + 1
                
                # 提取贡献者信息
                path_parts = file_path.parts
                if len(path_parts) > 2:
                    contributor = path_parts[-4] if "commits" in str(file_path) else path_parts[-3]
                    if "_at_" in contributor:
                        stats["contributors"].add(contributor)
                
                # 更新日期范围
                file_time = datetime.fromtimestamp(file_path.stat().st_mtime)
                if stats["date_range"]["earliest"] is None or file_time < stats["date_range"]["earliest"]:
                    stats["date_range"]["earliest"] = file_time
                if stats["date_range"]["latest"] is None or file_time > stats["date_range"]["latest"]:
                    stats["date_range"]["latest"] = file_time
            
            # 转换集合为列表
            stats["contributors"] = list(stats["contributors"])
            
            # 转换日期为字符串
            if stats["date_range"]["earliest"]:
                stats["date_range"]["earliest"] = stats["date_range"]["earliest"].isoformat()
            if stats["date_range"]["latest"]:
                stats["date_range"]["latest"] = stats["date_range"]["latest"].isoformat()
            
            return stats
            
        except Exception as e:
            self.logger.error(f"获取文件统计失败: {str(e)}")
            return {}

# 全局文件组织器实例
file_organizer = FileOrganizer()
