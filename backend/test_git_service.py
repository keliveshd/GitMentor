#!/usr/bin/env python3
"""
Git服务测试脚本
"""

import sys
import os
sys.path.append(os.path.dirname(os.path.abspath(__file__)))

from app.services.git_service import GitService

def test_git_service():
    """测试Git服务功能"""
    # 使用当前GitMentor仓库进行测试
    repo_path = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    
    print(f"测试仓库路径: {repo_path}")
    
    try:
        # 初始化Git服务
        git_service = GitService(repo_path)
        print("✅ Git服务初始化成功")
        
        # 测试获取提交数量
        commits_count = git_service.get_commits_count()
        print(f"✅ 提交总数: {commits_count}")
        
        # 测试获取提交历史
        commits = git_service.get_commits(page=1, page_size=5)
        print(f"✅ 获取到 {len(commits)} 个提交记录")
        
        if commits:
            print("最新提交信息:")
            latest_commit = commits[0]
            print(f"  - Hash: {latest_commit['hash'][:8]}")
            print(f"  - 消息: {latest_commit['message'][:50]}...")
            print(f"  - 作者: {latest_commit['author_name']}")
            print(f"  - 日期: {latest_commit['commit_date']}")
        
        # 测试获取仓库统计
        stats = git_service.get_repository_stats()
        print(f"✅ 仓库统计信息:")
        print(f"  - 总提交数: {stats['total_commits']}")
        print(f"  - 贡献者数: {stats['contributors']}")
        print(f"  - 总文件数: {stats['total_files']}")
        
        # 测试获取贡献者
        contributors = git_service.get_contributors()
        print(f"✅ 获取到 {len(contributors)} 个贡献者")
        
        if contributors:
            print("主要贡献者:")
            for i, contributor in enumerate(contributors[:3]):
                print(f"  {i+1}. {contributor['name']} ({contributor['commits']} 提交)")
        
        # 测试获取分支信息
        branches = git_service.get_branches()
        print(f"✅ 获取到 {len(branches)} 个分支")
        
        if branches:
            print("分支信息:")
            for branch in branches[:5]:
                print(f"  - {branch['name']} ({branch['type']})")
        
        print("\n🎉 所有测试通过！Git集成模块工作正常。")
        
    except Exception as e:
        print(f"❌ 测试失败: {e}")
        return False
    
    return True

if __name__ == "__main__":
    test_git_service()
