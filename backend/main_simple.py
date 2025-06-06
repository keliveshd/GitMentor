#!/usr/bin/env python3
"""
GitMentor Simple Backend
最简化的后端实现，避免复杂依赖
"""

import json
import sqlite3
import time
from http.server import HTTPServer, BaseHTTPRequestHandler
from urllib.parse import urlparse, parse_qs
import threading
import os
from pathlib import Path

class GitMentorHandler(BaseHTTPRequestHandler):
    """简单的HTTP请求处理器"""
    
    def do_GET(self):
        """处理GET请求"""
        parsed_path = urlparse(self.path)
        path = parsed_path.path
        
        # 设置CORS头
        self.send_response(200)
        self.send_header('Content-type', 'application/json')
        self.send_header('Access-Control-Allow-Origin', '*')
        self.send_header('Access-Control-Allow-Methods', 'GET, POST, PUT, DELETE, OPTIONS')
        self.send_header('Access-Control-Allow-Headers', 'Content-Type')
        self.end_headers()
        
        # 路由处理
        if path == '/api/health':
            self.handle_health()
        elif path == '/api/agents/health':
            self.handle_agents_health()
        elif path.startswith('/api/'):
            self.handle_api_default()
        else:
            self.handle_not_found()
    
    def do_POST(self):
        """处理POST请求"""
        self.do_GET()  # 简化处理，都当作GET
    
    def do_PUT(self):
        """处理PUT请求"""
        self.do_GET()  # 简化处理，都当作GET
    
    def do_DELETE(self):
        """处理DELETE请求"""
        self.do_GET()  # 简化处理，都当作GET
    
    def do_OPTIONS(self):
        """处理OPTIONS请求（CORS预检）"""
        self.send_response(200)
        self.send_header('Access-Control-Allow-Origin', '*')
        self.send_header('Access-Control-Allow-Methods', 'GET, POST, PUT, DELETE, OPTIONS')
        self.send_header('Access-Control-Allow-Headers', 'Content-Type')
        self.end_headers()
    
    def handle_health(self):
        """健康检查端点"""
        response = {
            "status": "healthy",
            "timestamp": time.time(),
            "version": "1.0.0",
            "message": "GitMentor Backend is running"
        }
        self.wfile.write(json.dumps(response).encode())
    
    def handle_agents_health(self):
        """Agent健康检查端点"""
        response = {
            "status": "healthy",
            "agents": {
                "analyzer": {"status": "ready", "version": "1.0.0"},
                "reviewer": {"status": "ready", "version": "1.0.0"}
            },
            "timestamp": time.time()
        }
        self.wfile.write(json.dumps(response).encode())
    
    def handle_api_default(self):
        """默认API响应"""
        response = {
            "status": "ok",
            "message": "GitMentor API endpoint",
            "timestamp": time.time(),
            "data": []
        }
        self.wfile.write(json.dumps(response).encode())
    
    def handle_not_found(self):
        """404处理"""
        self.send_response(404)
        self.send_header('Content-type', 'application/json')
        self.send_header('Access-Control-Allow-Origin', '*')
        self.end_headers()
        
        response = {
            "status": "error",
            "message": "Endpoint not found",
            "timestamp": time.time()
        }
        self.wfile.write(json.dumps(response).encode())
    
    def log_message(self, format, *args):
        """自定义日志格式"""
        print(f"[{time.strftime('%Y-%m-%d %H:%M:%S')}] {format % args}")

def init_database():
    """初始化SQLite数据库"""
    try:
        # 创建数据目录
        data_dir = Path("data")
        data_dir.mkdir(exist_ok=True)
        
        # 创建数据库
        db_path = data_dir / "gitmentor.db"
        conn = sqlite3.connect(str(db_path))
        
        # 创建基本表结构
        cursor = conn.cursor()
        
        # 创建分析任务表
        cursor.execute('''
            CREATE TABLE IF NOT EXISTS analysis_tasks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                task_id TEXT UNIQUE,
                repository TEXT,
                commit_hash TEXT,
                status TEXT DEFAULT 'pending',
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )
        ''')
        
        # 创建配置表
        cursor.execute('''
            CREATE TABLE IF NOT EXISTS configurations (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                key TEXT UNIQUE,
                value TEXT,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )
        ''')
        
        conn.commit()
        conn.close()
        
        print("✅ 数据库初始化成功")
        return True
        
    except Exception as e:
        print(f"❌ 数据库初始化失败: {e}")
        return False

def start_server():
    """启动HTTP服务器"""
    try:
        # 初始化数据库
        init_database()
        
        # 启动HTTP服务器
        server_address = ('localhost', 8000)
        httpd = HTTPServer(server_address, GitMentorHandler)
        
        print("🚀 GitMentor Backend Server Starting...")
        print(f"📡 Server running on http://{server_address[0]}:{server_address[1]}")
        print("📋 Health check: http://localhost:8000/api/health")
        print("🤖 Agent health: http://localhost:8000/api/agents/health")
        print("⏹️  Press Ctrl+C to stop")
        
        # 启动服务器
        httpd.serve_forever()
        
    except KeyboardInterrupt:
        print("\n🛑 Server stopped by user")
    except Exception as e:
        print(f"❌ Server error: {e}")
    finally:
        try:
            httpd.shutdown()
        except:
            pass

if __name__ == "__main__":
    start_server()
