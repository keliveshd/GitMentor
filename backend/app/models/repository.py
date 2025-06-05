"""
仓库数据模型
"""

from sqlalchemy import Column, Integer, String, DateTime, Text
from sqlalchemy.sql import func
from app.core.database import Base

class Repository(Base):
    __tablename__ = "repositories"
    
    id = Column(Integer, primary_key=True, index=True)
    path = Column(String, unique=True, nullable=False, index=True)
    name = Column(String, nullable=False)
    remote_url = Column(String)
    created_at = Column(DateTime, server_default=func.now())
    last_analyzed = Column(DateTime)
    total_commits = Column(Integer, default=0)
