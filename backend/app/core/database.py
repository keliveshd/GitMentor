"""
数据库配置和初始化
"""

import sqlite3
from pathlib import Path
from sqlalchemy import create_engine, MetaData
from sqlalchemy.ext.declarative import declarative_base
from sqlalchemy.orm import sessionmaker

# 数据库文件路径
DB_PATH = Path(__file__).parent.parent.parent / "data" / "gitmentor.db"
DB_PATH.parent.mkdir(exist_ok=True)

# SQLAlchemy配置
SQLALCHEMY_DATABASE_URL = f"sqlite:///{DB_PATH}"
engine = create_engine(SQLALCHEMY_DATABASE_URL, connect_args={"check_same_thread": False})
SessionLocal = sessionmaker(autocommit=False, autoflush=False, bind=engine)
Base = declarative_base()

def get_db():
    """获取数据库会话"""
    db = SessionLocal()
    try:
        yield db
    finally:
        db.close()

def init_db():
    """初始化数据库表"""
    # 创建所有表
    Base.metadata.create_all(bind=engine)
    
    # 执行初始化SQL
    init_sql = """
    -- 仓库信息表
    CREATE TABLE IF NOT EXISTS repositories (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        path TEXT NOT NULL UNIQUE,
        name TEXT NOT NULL,
        remote_url TEXT,
        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        last_analyzed TIMESTAMP,
        total_commits INTEGER DEFAULT 0
    );

    -- 提交信息表
    CREATE TABLE IF NOT EXISTS commits (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        repo_id INTEGER NOT NULL,
        hash TEXT NOT NULL,
        author_name TEXT NOT NULL,
        author_email TEXT NOT NULL,
        commit_date TIMESTAMP NOT NULL,
        message TEXT NOT NULL,
        files_changed INTEGER DEFAULT 0,
        insertions INTEGER DEFAULT 0,
        deletions INTEGER DEFAULT 0,
        ai_analysis TEXT,
        category TEXT,
        FOREIGN KEY (repo_id) REFERENCES repositories (id),
        UNIQUE(repo_id, hash)
    );

    -- 文件变更表
    CREATE TABLE IF NOT EXISTS file_changes (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        commit_id INTEGER NOT NULL,
        file_path TEXT NOT NULL,
        change_type TEXT NOT NULL, -- 'A', 'M', 'D', 'R'
        insertions INTEGER DEFAULT 0,
        deletions INTEGER DEFAULT 0,
        FOREIGN KEY (commit_id) REFERENCES commits (id)
    );

    -- AI分析结果表
    CREATE TABLE IF NOT EXISTS ai_analyses (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        commit_id INTEGER NOT NULL,
        analysis_type TEXT NOT NULL, -- 'summary', 'quality', 'efficiency'
        result TEXT NOT NULL,
        confidence_score REAL,
        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        FOREIGN KEY (commit_id) REFERENCES commits (id)
    );

    -- 配置表
    CREATE TABLE IF NOT EXISTS settings (
        key TEXT PRIMARY KEY,
        value TEXT NOT NULL,
        updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
    );
    """
    
    with sqlite3.connect(DB_PATH) as conn:
        conn.executescript(init_sql)
        conn.commit()
    
    print(f"数据库初始化完成: {DB_PATH}")
