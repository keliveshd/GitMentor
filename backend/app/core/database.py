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
        task_id TEXT,
        analysis_status TEXT DEFAULT 'pending',
        quality_score REAL,
        reviewer_feedback TEXT,
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

    -- Agent执行记录表
    CREATE TABLE IF NOT EXISTS agent_executions (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        task_id TEXT NOT NULL,
        agent_id TEXT NOT NULL,
        agent_type TEXT NOT NULL,
        input_data TEXT NOT NULL,
        output_data TEXT NOT NULL,
        confidence_score REAL,
        processing_time REAL,
        status TEXT NOT NULL, -- 'success', 'error'
        error_message TEXT,
        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
    );

    -- 质量控制记录表
    CREATE TABLE IF NOT EXISTS quality_control_records (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        task_id TEXT NOT NULL,
        commit_hash TEXT NOT NULL,
        analyzer_execution_id INTEGER,
        reviewer_execution_id INTEGER,
        final_status TEXT NOT NULL, -- 'approved', 'rejected', 'pending', 'error'
        overall_quality_score REAL,
        retry_count INTEGER DEFAULT 0,
        completed_at TIMESTAMP,
        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        FOREIGN KEY (analyzer_execution_id) REFERENCES agent_executions (id),
        FOREIGN KEY (reviewer_execution_id) REFERENCES agent_executions (id)
    );

    -- Agent配置版本表
    CREATE TABLE IF NOT EXISTS agent_config_versions (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        agent_type TEXT NOT NULL,
        version TEXT NOT NULL,
        config_data TEXT NOT NULL,
        is_active BOOLEAN DEFAULT FALSE,
        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        activated_at TIMESTAMP
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
