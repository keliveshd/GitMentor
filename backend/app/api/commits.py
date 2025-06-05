"""
提交分析API
"""

from typing import List, Optional
from fastapi import APIRouter, HTTPException, Depends, Query
from sqlalchemy.orm import Session
from pydantic import BaseModel

from app.core.database import get_db
from app.models.repository import Repository
from app.services.git_service import GitService

router = APIRouter()

class CommitResponse(BaseModel):
    hash: str
    message: str
    author_name: str
    author_email: str
    commit_date: str
    files_changed: int
    insertions: int
    deletions: int

@router.get("/repositories/{repo_id}/commits", response_model=List[CommitResponse])
async def get_commits(
    repo_id: int,
    page: int = Query(1, ge=1),
    page_size: int = Query(20, ge=1, le=100),
    db: Session = Depends(get_db)
):
    """获取仓库提交历史"""
    repo = db.query(Repository).filter(Repository.id == repo_id).first()
    if not repo:
        raise HTTPException(status_code=404, detail="仓库不存在")
    
    try:
        git_service = GitService(repo.path)
        commits = git_service.get_commits(page=page, page_size=page_size)
        
        return [
            CommitResponse(
                hash=commit["hash"],
                message=commit["message"],
                author_name=commit["author_name"],
                author_email=commit["author_email"],
                commit_date=commit["commit_date"],
                files_changed=commit["files_changed"],
                insertions=commit["insertions"],
                deletions=commit["deletions"]
            )
            for commit in commits
        ]
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"获取提交历史失败: {str(e)}")

@router.get("/commits/stats")
async def get_commits_stats(repo_id: Optional[int] = Query(None), db: Session = Depends(get_db)):
    """获取提交统计信息"""
    if repo_id:
        repo = db.query(Repository).filter(Repository.id == repo_id).first()
        if not repo:
            raise HTTPException(status_code=404, detail="仓库不存在")
        
        try:
            git_service = GitService(repo.path)
            stats = git_service.get_repository_stats()
            return stats
        except Exception as e:
            raise HTTPException(status_code=500, detail=f"获取统计信息失败: {str(e)}")
    else:
        # 返回所有仓库的统计信息
        repositories = db.query(Repository).all()
        total_commits = sum(repo.total_commits for repo in repositories)
        
        return {
            "total_repositories": len(repositories),
            "total_commits": total_commits,
            "analyzed_repositories": len([repo for repo in repositories if repo.last_analyzed])
        }

@router.get("/repositories/{repo_id}/commits/{commit_hash}")
async def get_commit_details(repo_id: int, commit_hash: str, db: Session = Depends(get_db)):
    """获取提交详细信息"""
    repo = db.query(Repository).filter(Repository.id == repo_id).first()
    if not repo:
        raise HTTPException(status_code=404, detail="仓库不存在")

    try:
        git_service = GitService(repo.path)
        commit_details = git_service.get_commit_details(commit_hash)

        if not commit_details:
            raise HTTPException(status_code=404, detail="提交不存在")

        return commit_details
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"获取提交详情失败: {str(e)}")

@router.get("/repositories/{repo_id}/branches")
async def get_branches(repo_id: int, db: Session = Depends(get_db)):
    """获取仓库分支信息"""
    repo = db.query(Repository).filter(Repository.id == repo_id).first()
    if not repo:
        raise HTTPException(status_code=404, detail="仓库不存在")

    try:
        git_service = GitService(repo.path)
        branches = git_service.get_branches()
        return {"branches": branches}
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"获取分支信息失败: {str(e)}")

@router.get("/repositories/{repo_id}/contributors")
async def get_contributors(repo_id: int, db: Session = Depends(get_db)):
    """获取仓库贡献者信息"""
    repo = db.query(Repository).filter(Repository.id == repo_id).first()
    if not repo:
        raise HTTPException(status_code=404, detail="仓库不存在")

    try:
        git_service = GitService(repo.path)
        contributors = git_service.get_contributors()
        return {"contributors": contributors}
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"获取贡献者信息失败: {str(e)}")

@router.get("/repositories/{repo_id}/files/{file_path:path}/history")
async def get_file_history(
    repo_id: int,
    file_path: str,
    max_count: int = Query(50, ge=1, le=200),
    db: Session = Depends(get_db)
):
    """获取文件变更历史"""
    repo = db.query(Repository).filter(Repository.id == repo_id).first()
    if not repo:
        raise HTTPException(status_code=404, detail="仓库不存在")

    try:
        git_service = GitService(repo.path)
        history = git_service.get_file_history(file_path, max_count)
        return {"file_path": file_path, "history": history}
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"获取文件历史失败: {str(e)}")
