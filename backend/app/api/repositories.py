"""
仓库管理API
"""

import os
from pathlib import Path
from typing import List
from fastapi import APIRouter, HTTPException, Depends
from sqlalchemy.orm import Session
from pydantic import BaseModel
import git

from app.core.database import get_db
from app.models.repository import Repository
from app.services.git_service import GitService

router = APIRouter()

class RepositoryCreate(BaseModel):
    path: str
    name: str

class RepositoryResponse(BaseModel):
    id: int
    path: str
    name: str
    remote_url: str = None
    created_at: str
    last_analyzed: str = None
    total_commits: int = 0

@router.get("/repositories", response_model=List[RepositoryResponse])
async def get_repositories(db: Session = Depends(get_db)):
    """获取所有仓库"""
    repositories = db.query(Repository).all()
    return [
        RepositoryResponse(
            id=repo.id,
            path=repo.path,
            name=repo.name,
            remote_url=repo.remote_url,
            created_at=repo.created_at.isoformat() if repo.created_at else "",
            last_analyzed=repo.last_analyzed.isoformat() if repo.last_analyzed else None,
            total_commits=repo.total_commits
        )
        for repo in repositories
    ]

@router.post("/repositories", response_model=RepositoryResponse)
async def create_repository(repo_data: RepositoryCreate, db: Session = Depends(get_db)):
    """添加新仓库"""
    # 验证路径是否存在
    if not os.path.exists(repo_data.path):
        raise HTTPException(status_code=400, detail="指定路径不存在")
    
    # 验证是否为Git仓库
    try:
        git_repo = git.Repo(repo_data.path)
        remote_url = None
        if git_repo.remotes:
            remote_url = git_repo.remotes.origin.url
    except git.exc.InvalidGitRepositoryError:
        raise HTTPException(status_code=400, detail="指定路径不是有效的Git仓库")
    
    # 检查仓库是否已存在
    existing = db.query(Repository).filter(Repository.path == repo_data.path).first()
    if existing:
        raise HTTPException(status_code=400, detail="仓库已存在")
    
    # 创建仓库记录
    db_repo = Repository(
        path=repo_data.path,
        name=repo_data.name,
        remote_url=remote_url
    )
    
    db.add(db_repo)
    db.commit()
    db.refresh(db_repo)
    
    return RepositoryResponse(
        id=db_repo.id,
        path=db_repo.path,
        name=db_repo.name,
        remote_url=db_repo.remote_url,
        created_at=db_repo.created_at.isoformat(),
        last_analyzed=None,
        total_commits=0
    )

@router.delete("/repositories/{repo_id}")
async def delete_repository(repo_id: int, db: Session = Depends(get_db)):
    """删除仓库"""
    repo = db.query(Repository).filter(Repository.id == repo_id).first()
    if not repo:
        raise HTTPException(status_code=404, detail="仓库不存在")
    
    db.delete(repo)
    db.commit()
    
    return {"message": "仓库删除成功"}

@router.post("/repositories/{repo_id}/analyze")
async def analyze_repository(repo_id: int, db: Session = Depends(get_db)):
    """分析仓库"""
    repo = db.query(Repository).filter(Repository.id == repo_id).first()
    if not repo:
        raise HTTPException(status_code=404, detail="仓库不存在")
    
    try:
        git_service = GitService(repo.path)
        commits_count = git_service.get_commits_count()
        
        # 更新仓库信息
        repo.total_commits = commits_count
        from datetime import datetime
        repo.last_analyzed = datetime.now()
        
        db.commit()
        
        return {
            "message": "仓库分析完成",
            "total_commits": commits_count
        }
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"分析失败: {str(e)}")
