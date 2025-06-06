"""
双重存储管理器
"""

import json
import sqlite3
import logging
from typing import Dict, Any, Optional, List
from datetime import datetime
from .markdown_generator import MarkdownGenerator
from .file_organizer import FileOrganizer

class StorageManager:
    """双重存储管理器"""
    
    def __init__(self, db_path: str = "data/gitmentor.db"):
        self.db_path = db_path
        self.markdown_generator = MarkdownGenerator()
        self.file_organizer = FileOrganizer()
        self.logger = logging.getLogger("storage_manager")
    
    async def store_approved_analysis(self,
                                    commit_data: Dict[str, Any],
                                    analysis_result: Any,
                                    review_result: Any) -> bool:
        """存储通过审核的分析结果"""
        try:
            # 1. 存储到SQLite数据库
            db_success = await self._store_to_database(commit_data, analysis_result, review_result)
            
            # 2. 生成并存储Markdown文件
            md_success = await self._store_to_markdown(commit_data, analysis_result, review_result)
            
            if db_success and md_success:
                self.logger.info(f"双重存储成功: {commit_data.get('hash', '')[:8]}")
                return True
            else:
                self.logger.warning(f"部分存储失败: DB={db_success}, MD={md_success}")
                return False
                
        except Exception as e:
            self.logger.error(f"存储失败: {str(e)}")
            return False
    
    async def store_rejected_analysis(self,
                                    commit_data: Dict[str, Any],
                                    analysis_result: Any,
                                    review_result: Any) -> bool:
        """存储未通过审核的分析结果"""
        try:
            # 只存储到数据库，不生成Markdown文件
            return await self._store_to_database(commit_data, analysis_result, review_result, approved=False)
            
        except Exception as e:
            self.logger.error(f"存储拒绝结果失败: {str(e)}")
            return False
    
    async def _store_to_database(self,
                               commit_data: Dict[str, Any],
                               analysis_result: Any,
                               review_result: Any,
                               approved: bool = True) -> bool:
        """存储到SQLite数据库"""
        try:
            # 提取数据
            commit_hash = commit_data.get("hash", "")
            task_id = getattr(analysis_result, 'task_id', '') if hasattr(analysis_result, 'task_id') else ''
            
            # 分析结果数据
            analysis_data = analysis_result.result if hasattr(analysis_result, 'result') else analysis_result
            review_data = review_result.result if hasattr(review_result, 'result') else review_result
            
            # 连接数据库
            conn = sqlite3.connect(self.db_path)
            cursor = conn.cursor()
            
            try:
                # 1. 存储Agent执行记录
                analyzer_execution_id = await self._store_agent_execution(
                    cursor, task_id, "analyzer", analysis_result
                )
                
                reviewer_execution_id = await self._store_agent_execution(
                    cursor, task_id, "reviewer", review_result
                )
                
                # 2. 存储质量控制记录
                await self._store_quality_control_record(
                    cursor, task_id, commit_hash, analyzer_execution_id, 
                    reviewer_execution_id, approved, review_data
                )
                
                # 3. 更新commits表
                await self._update_commit_record(
                    cursor, commit_hash, task_id, approved, review_data
                )
                
                # 提交事务
                conn.commit()
                return True
                
            except Exception as e:
                conn.rollback()
                raise e
            finally:
                conn.close()
                
        except Exception as e:
            self.logger.error(f"数据库存储失败: {str(e)}")
            return False
    
    async def _store_agent_execution(self,
                                   cursor: sqlite3.Cursor,
                                   task_id: str,
                                   agent_type: str,
                                   agent_output: Any) -> int:
        """存储Agent执行记录"""
        
        # 提取数据
        if hasattr(agent_output, 'result'):
            output_data = agent_output.result
            confidence_score = agent_output.confidence
            processing_time = agent_output.processing_time
            status = agent_output.status
            error_message = agent_output.error_message
        else:
            output_data = agent_output
            confidence_score = 0.8
            processing_time = 0.0
            status = "success"
            error_message = None
        
        # 插入记录
        cursor.execute("""
            INSERT INTO agent_executions (
                task_id, agent_id, agent_type, input_data, output_data,
                confidence_score, processing_time, status, error_message
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        """, (
            task_id,
            f"default_{agent_type}",
            agent_type,
            json.dumps({}),  # 输入数据
            json.dumps(output_data, ensure_ascii=False),
            confidence_score,
            processing_time,
            status,
            error_message
        ))
        
        return cursor.lastrowid
    
    async def _store_quality_control_record(self,
                                          cursor: sqlite3.Cursor,
                                          task_id: str,
                                          commit_hash: str,
                                          analyzer_execution_id: int,
                                          reviewer_execution_id: int,
                                          approved: bool,
                                          review_data: Dict[str, Any]):
        """存储质量控制记录"""
        
        final_status = "approved" if approved else "rejected"
        overall_quality_score = review_data.get("overall_score", 0.0)
        
        cursor.execute("""
            INSERT INTO quality_control_records (
                task_id, commit_hash, analyzer_execution_id, reviewer_execution_id,
                final_status, overall_quality_score, retry_count, completed_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        """, (
            task_id,
            commit_hash,
            analyzer_execution_id,
            reviewer_execution_id,
            final_status,
            overall_quality_score,
            0,  # retry_count
            datetime.now().isoformat()
        ))
    
    async def _update_commit_record(self,
                                  cursor: sqlite3.Cursor,
                                  commit_hash: str,
                                  task_id: str,
                                  approved: bool,
                                  review_data: Dict[str, Any]):
        """更新commits表记录"""
        
        analysis_status = "approved" if approved else "rejected"
        quality_score = review_data.get("overall_score", 0.0)
        reviewer_feedback = review_data.get("feedback", "")
        
        cursor.execute("""
            UPDATE commits 
            SET task_id = ?, analysis_status = ?, quality_score = ?, reviewer_feedback = ?
            WHERE hash = ?
        """, (
            task_id,
            analysis_status,
            quality_score,
            reviewer_feedback,
            commit_hash
        ))
    
    async def _store_to_markdown(self,
                               commit_data: Dict[str, Any],
                               analysis_result: Any,
                               review_result: Any) -> bool:
        """存储到Markdown文件"""
        try:
            # 提取数据
            repository_name = commit_data.get("repository_name", "unknown-repo")
            contributor_email = commit_data.get("author_email", "unknown@example.com")
            commit_date = commit_data.get("commit_date", datetime.now().isoformat())
            commit_hash = commit_data.get("hash", "")
            task_id = getattr(analysis_result, 'task_id', '') if hasattr(analysis_result, 'task_id') else ''
            
            # 分析和审核结果
            analysis_data = analysis_result.result if hasattr(analysis_result, 'result') else analysis_result
            review_data = review_result.result if hasattr(review_result, 'result') else review_result
            
            # 1. 生成提交分析报告
            commit_report = self.markdown_generator.generate_commit_analysis_report(
                commit_data, analysis_data, review_data, task_id
            )
            
            # 2. 获取文件路径并保存
            commit_file_path = self.file_organizer.get_commit_file_path(
                repository_name, contributor_email, commit_date, commit_hash
            )
            
            with open(commit_file_path, 'w', encoding='utf-8') as f:
                f.write(commit_report)
            
            # 3. 更新日度总结（异步任务）
            await self._update_daily_summary(repository_name, contributor_email, commit_date, commit_data, analysis_data, review_data)
            
            # 4. 更新贡献者档案（异步任务）
            await self._update_contributor_profile(repository_name, contributor_email)
            
            # 5. 更新仓库概览（异步任务）
            await self._update_repository_overview(repository_name)
            
            return True
            
        except Exception as e:
            self.logger.error(f"Markdown存储失败: {str(e)}")
            return False
    
    async def _update_daily_summary(self,
                                  repository_name: str,
                                  contributor_email: str,
                                  commit_date: str,
                                  commit_data: Dict[str, Any],
                                  analysis_data: Dict[str, Any],
                                  review_data: Dict[str, Any]):
        """更新日度总结"""
        try:
            # 解析日期
            date_str = commit_date.split('T')[0]  # 获取日期部分
            
            # 获取当天的所有提交（从数据库查询）
            daily_commits = await self._get_daily_commits(repository_name, contributor_email, date_str)
            
            # 添加当前提交
            current_commit = {
                "hash": commit_data.get("hash", ""),
                "message": commit_data.get("message", ""),
                "category": analysis_data.get("category", ""),
                "approved": review_data.get("approved", False),
                "quality_score": review_data.get("overall_score", 0.0)
            }
            daily_commits.append(current_commit)
            
            # 生成日度总结
            daily_summary = self.markdown_generator.generate_daily_summary(
                date_str, daily_commits, repository_name, contributor_email
            )
            
            # 保存文件
            summary_path = self.file_organizer.get_daily_summary_path(
                repository_name, contributor_email, date_str
            )
            
            with open(summary_path, 'w', encoding='utf-8') as f:
                f.write(daily_summary)
                
        except Exception as e:
            self.logger.warning(f"更新日度总结失败: {str(e)}")
    
    async def _update_contributor_profile(self, repository_name: str, contributor_email: str):
        """更新贡献者档案"""
        try:
            # 从数据库获取贡献者统计
            statistics = await self._get_contributor_statistics(repository_name, contributor_email)
            
            # 生成档案
            profile = self.markdown_generator.generate_contributor_profile(
                contributor_email, statistics.get("name", contributor_email), statistics
            )
            
            # 保存文件
            profile_path = self.file_organizer.get_contributor_profile_path(
                repository_name, contributor_email
            )
            
            with open(profile_path, 'w', encoding='utf-8') as f:
                f.write(profile)
                
        except Exception as e:
            self.logger.warning(f"更新贡献者档案失败: {str(e)}")
    
    async def _update_repository_overview(self, repository_name: str):
        """更新仓库概览"""
        try:
            # 从数据库获取仓库统计
            statistics = await self._get_repository_statistics(repository_name)
            
            # 生成概览
            overview = self.markdown_generator.generate_repository_overview(
                repository_name, statistics
            )
            
            # 保存文件
            overview_path = self.file_organizer.get_repository_overview_path(repository_name)
            
            with open(overview_path, 'w', encoding='utf-8') as f:
                f.write(overview)
                
        except Exception as e:
            self.logger.warning(f"更新仓库概览失败: {str(e)}")
    
    async def _get_daily_commits(self, repository_name: str, contributor_email: str, date: str) -> List[Dict[str, Any]]:
        """获取指定日期的提交列表"""
        # 这里应该从数据库查询，简化实现返回空列表
        return []
    
    async def _get_contributor_statistics(self, repository_name: str, contributor_email: str) -> Dict[str, Any]:
        """获取贡献者统计信息"""
        # 这里应该从数据库查询，简化实现返回默认值
        return {
            "name": contributor_email.split('@')[0],
            "total_commits": 0,
            "total_approved": 0,
            "avg_quality": 0.0,
            "favorite_categories": []
        }
    
    async def _get_repository_statistics(self, repository_name: str) -> Dict[str, Any]:
        """获取仓库统计信息"""
        # 这里应该从数据库查询，简化实现返回默认值
        return {
            "total_commits": 0,
            "total_contributors": 0,
            "avg_quality": 0.0,
            "top_contributors": []
        }

# 全局存储管理器实例
storage_manager = StorageManager()
