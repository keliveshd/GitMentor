"""
Markdown文件生成器
"""

import json
import logging
from typing import Dict, Any, Optional
from datetime import datetime
from pathlib import Path

class MarkdownGenerator:
    """Markdown文件生成器"""
    
    def __init__(self):
        self.logger = logging.getLogger("markdown_generator")
    
    def generate_commit_analysis_report(self, 
                                      commit_data: Dict[str, Any],
                                      analysis_result: Dict[str, Any],
                                      review_result: Dict[str, Any],
                                      task_id: str) -> str:
        """生成提交分析报告"""
        
        # 提取数据
        commit_hash = commit_data.get("hash", "unknown")
        commit_message = commit_data.get("message", "")
        author_name = commit_data.get("author_name", "")
        author_email = commit_data.get("author_email", "")
        commit_date = commit_data.get("commit_date", "")
        files_changed = commit_data.get("files_changed", 0)
        
        # 分析结果
        summary = analysis_result.get("summary", "")
        category = analysis_result.get("category", "")
        impact_level = analysis_result.get("impact_level", "")
        confidence_score = analysis_result.get("confidence_score", 0.0)
        description = analysis_result.get("description", "")
        
        # 审核结果
        approved = review_result.get("approved", False)
        overall_score = review_result.get("overall_score", 0.0)
        dimension_scores = review_result.get("dimension_scores", {})
        feedback = review_result.get("feedback", "")
        suggestions = review_result.get("suggestions", [])
        
        # 生成状态图标
        status_icon = "✅" if approved else "❌"
        
        # 生成Markdown内容
        markdown_content = f"""# 提交分析报告
## {commit_hash[:8]}

### 基本信息
- **提交哈希**: `{commit_hash}`
- **作者**: {author_name} <{author_email}>
- **时间**: {commit_date}
- **文件变更**: {files_changed} 个文件
- **任务ID**: `{task_id}`

### 提交消息
```
{commit_message}
```

---

## Analyzer Agent 分析结果

### 分析摘要
**{summary}**

### 详细信息
- **变更类型**: `{category}`
- **影响级别**: `{impact_level}`
- **置信度**: {confidence_score:.2f}

### 详细描述
{description}

---

## Reviewer Agent 审核结果

### 审核状态
{status_icon} **{"通过" if approved else "未通过"}**

### 质量评分
- **综合评分**: {overall_score:.3f}

#### 维度评分
"""
        
        # 添加维度评分
        for dimension, score in dimension_scores.items():
            dimension_name = {
                "accuracy": "准确性",
                "completeness": "完整性", 
                "consistency": "一致性",
                "clarity": "清晰度"
            }.get(dimension, dimension)
            
            markdown_content += f"- **{dimension_name}**: {score:.3f}\n"
        
        # 添加审核意见
        markdown_content += f"""
### 审核意见
{feedback}
"""
        
        # 添加改进建议
        if suggestions:
            markdown_content += "\n### 改进建议\n"
            for i, suggestion in enumerate(suggestions, 1):
                markdown_content += f"{i}. {suggestion}\n"
        
        # 添加元数据
        current_time = datetime.now().isoformat()
        metadata = {
            "task_id": task_id,
            "processing_pipeline": "analyzer -> reviewer",
            "final_status": "approved" if approved else "rejected",
            "storage_timestamp": current_time,
            "data_version": "2.0"
        }
        
        markdown_content += f"""
---

## 元数据
```json
{json.dumps(metadata, indent=2, ensure_ascii=False)}
```

---
*报告生成时间: {current_time}*
"""
        
        return markdown_content
    
    def generate_daily_summary(self, 
                             date: str,
                             commits: list,
                             repository_name: str,
                             contributor_email: str) -> str:
        """生成日度总结"""
        
        total_commits = len(commits)
        approved_commits = sum(1 for c in commits if c.get("approved", False))
        approval_rate = (approved_commits / total_commits * 100) if total_commits > 0 else 0
        
        # 统计变更类型
        categories = {}
        for commit in commits:
            category = commit.get("category", "unknown")
            categories[category] = categories.get(category, 0) + 1
        
        # 计算平均质量分数
        quality_scores = [c.get("quality_score", 0) for c in commits if c.get("quality_score")]
        avg_quality = sum(quality_scores) / len(quality_scores) if quality_scores else 0
        
        markdown_content = f"""# 日度工作总结
## {date}

### 基本信息
- **仓库**: {repository_name}
- **贡献者**: {contributor_email}
- **日期**: {date}

### 统计概览
- **总提交数**: {total_commits}
- **通过审核**: {approved_commits}
- **审核通过率**: {approval_rate:.1f}%
- **平均质量分**: {avg_quality:.3f}

### 变更类型分布
"""
        
        for category, count in categories.items():
            percentage = (count / total_commits * 100) if total_commits > 0 else 0
            markdown_content += f"- **{category}**: {count} ({percentage:.1f}%)\n"
        
        markdown_content += "\n### 提交详情\n"
        
        for commit in commits:
            commit_hash = commit.get("hash", "")[:8]
            commit_message = commit.get("message", "")[:100]
            status = "✅" if commit.get("approved", False) else "❌"
            quality_score = commit.get("quality_score", 0)
            
            markdown_content += f"""
#### {status} {commit_hash}
- **消息**: {commit_message}
- **质量分**: {quality_score:.3f}
- **类型**: {commit.get("category", "unknown")}
"""
        
        return markdown_content
    
    def generate_monthly_summary(self,
                                year: int,
                                month: int,
                                daily_summaries: list,
                                repository_name: str,
                                contributor_email: str) -> str:
        """生成月度总结"""
        
        total_commits = sum(day.get("total_commits", 0) for day in daily_summaries)
        total_approved = sum(day.get("approved_commits", 0) for day in daily_summaries)
        
        approval_rate = (total_approved / total_commits * 100) if total_commits > 0 else 0
        
        # 计算月度平均质量分
        all_quality_scores = []
        for day in daily_summaries:
            if day.get("avg_quality"):
                all_quality_scores.append(day["avg_quality"])
        
        monthly_avg_quality = sum(all_quality_scores) / len(all_quality_scores) if all_quality_scores else 0
        
        markdown_content = f"""# 月度工作总结
## {year}年{month}月

### 基本信息
- **仓库**: {repository_name}
- **贡献者**: {contributor_email}
- **统计周期**: {year}年{month}月

### 月度统计
- **总提交数**: {total_commits}
- **通过审核**: {total_approved}
- **审核通过率**: {approval_rate:.1f}%
- **月度平均质量分**: {monthly_avg_quality:.3f}
- **活跃天数**: {len(daily_summaries)}

### 日度趋势
"""
        
        for day_summary in daily_summaries:
            date = day_summary.get("date", "")
            commits = day_summary.get("total_commits", 0)
            quality = day_summary.get("avg_quality", 0)
            
            markdown_content += f"- **{date}**: {commits} 提交, 质量分 {quality:.2f}\n"
        
        return markdown_content
    
    def generate_contributor_profile(self,
                                   contributor_email: str,
                                   contributor_name: str,
                                   statistics: Dict[str, Any]) -> str:
        """生成贡献者档案"""
        
        total_commits = statistics.get("total_commits", 0)
        total_approved = statistics.get("total_approved", 0)
        avg_quality = statistics.get("avg_quality", 0)
        favorite_categories = statistics.get("favorite_categories", [])
        
        markdown_content = f"""# 贡献者档案
## {contributor_name}

### 基本信息
- **姓名**: {contributor_name}
- **邮箱**: {contributor_email}
- **档案更新**: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}

### 贡献统计
- **总提交数**: {total_commits}
- **通过审核**: {total_approved}
- **审核通过率**: {(total_approved/total_commits*100):.1f}% (如果有提交)
- **平均质量分**: {avg_quality:.3f}

### 编程偏好
"""
        
        if favorite_categories:
            markdown_content += "#### 常用变更类型\n"
            for category, count in favorite_categories[:5]:  # 显示前5个
                markdown_content += f"- **{category}**: {count} 次\n"
        
        # 添加质量趋势图表占位符
        markdown_content += """
### 质量趋势
*此处可以添加质量趋势图表*

### 技能评估
*基于提交历史的技能评估将在后续版本中添加*
"""
        
        return markdown_content
    
    def generate_repository_overview(self,
                                   repository_name: str,
                                   statistics: Dict[str, Any]) -> str:
        """生成仓库概览"""
        
        total_commits = statistics.get("total_commits", 0)
        total_contributors = statistics.get("total_contributors", 0)
        avg_quality = statistics.get("avg_quality", 0)
        top_contributors = statistics.get("top_contributors", [])
        
        markdown_content = f"""# 仓库概览
## {repository_name}

### 基本统计
- **总提交数**: {total_commits}
- **贡献者数量**: {total_contributors}
- **平均质量分**: {avg_quality:.3f}
- **最后更新**: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}

### 顶级贡献者
"""
        
        for contributor in top_contributors[:10]:  # 显示前10个贡献者
            name = contributor.get("name", "")
            commits = contributor.get("commits", 0)
            quality = contributor.get("avg_quality", 0)
            
            markdown_content += f"- **{name}**: {commits} 提交, 平均质量 {quality:.2f}\n"
        
        markdown_content += """
### 质量分析
*详细的质量分析报告将在analytics目录中生成*

### 团队协作
*团队协作分析将在后续版本中添加*
"""
        
        return markdown_content
