"""
缓存管理器
"""

import time
import json
import logging
import hashlib
from typing import Dict, Any, Optional, List
from threading import Lock
from dataclasses import dataclass, asdict

@dataclass
class CacheItem:
    """缓存项"""
    key: str
    value: Any
    created_at: float
    expires_at: Optional[float] = None
    access_count: int = 0
    last_accessed: float = None
    
    def __post_init__(self):
        if self.last_accessed is None:
            self.last_accessed = self.created_at
    
    def is_expired(self) -> bool:
        """检查是否过期"""
        if self.expires_at is None:
            return False
        return time.time() > self.expires_at
    
    def access(self):
        """记录访问"""
        self.access_count += 1
        self.last_accessed = time.time()

class CacheManager:
    """内存缓存管理器"""
    
    def __init__(self, max_size: int = 1000, default_ttl: int = 3600):
        self.max_size = max_size
        self.default_ttl = default_ttl
        self.cache: Dict[str, CacheItem] = {}
        self.lock = Lock()
        self.logger = logging.getLogger("cache_manager")
        
        # 统计信息
        self.stats = {
            "hits": 0,
            "misses": 0,
            "evictions": 0,
            "expired": 0
        }
    
    def _generate_key(self, prefix: str, data: Any) -> str:
        """生成缓存键"""
        if isinstance(data, dict):
            # 对字典进行排序以确保一致性
            sorted_data = json.dumps(data, sort_keys=True)
        else:
            sorted_data = str(data)
        
        hash_obj = hashlib.md5(sorted_data.encode())
        return f"{prefix}:{hash_obj.hexdigest()}"
    
    def get(self, key: str) -> Optional[Any]:
        """获取缓存值"""
        with self.lock:
            if key not in self.cache:
                self.stats["misses"] += 1
                return None
            
            item = self.cache[key]
            
            # 检查是否过期
            if item.is_expired():
                del self.cache[key]
                self.stats["expired"] += 1
                self.stats["misses"] += 1
                return None
            
            # 记录访问
            item.access()
            self.stats["hits"] += 1
            return item.value
    
    def set(self, key: str, value: Any, ttl: Optional[int] = None) -> bool:
        """设置缓存值"""
        with self.lock:
            current_time = time.time()
            expires_at = None
            
            if ttl is not None:
                expires_at = current_time + ttl
            elif self.default_ttl > 0:
                expires_at = current_time + self.default_ttl
            
            # 如果缓存已满，执行LRU淘汰
            if len(self.cache) >= self.max_size and key not in self.cache:
                self._evict_lru()
            
            item = CacheItem(
                key=key,
                value=value,
                created_at=current_time,
                expires_at=expires_at
            )
            
            self.cache[key] = item
            return True
    
    def delete(self, key: str) -> bool:
        """删除缓存项"""
        with self.lock:
            if key in self.cache:
                del self.cache[key]
                return True
            return False
    
    def clear(self):
        """清空缓存"""
        with self.lock:
            self.cache.clear()
            self.logger.info("缓存已清空")
    
    def _evict_lru(self):
        """淘汰最近最少使用的项"""
        if not self.cache:
            return
        
        # 找到最近最少访问的项
        lru_key = min(self.cache.keys(), key=lambda k: self.cache[k].last_accessed)
        del self.cache[lru_key]
        self.stats["evictions"] += 1
        self.logger.debug(f"淘汰缓存项: {lru_key}")
    
    def cleanup_expired(self) -> int:
        """清理过期项"""
        with self.lock:
            current_time = time.time()
            expired_keys = []
            
            for key, item in self.cache.items():
                if item.is_expired():
                    expired_keys.append(key)
            
            for key in expired_keys:
                del self.cache[key]
                self.stats["expired"] += 1
            
            if expired_keys:
                self.logger.info(f"清理了 {len(expired_keys)} 个过期缓存项")
            
            return len(expired_keys)
    
    def get_stats(self) -> Dict[str, Any]:
        """获取缓存统计信息"""
        with self.lock:
            total_requests = self.stats["hits"] + self.stats["misses"]
            hit_rate = (self.stats["hits"] / total_requests * 100) if total_requests > 0 else 0
            
            return {
                "cache_size": len(self.cache),
                "max_size": self.max_size,
                "hit_rate": round(hit_rate, 2),
                "total_requests": total_requests,
                **self.stats
            }
    
    def get_cache_info(self) -> List[Dict[str, Any]]:
        """获取缓存项信息"""
        with self.lock:
            items = []
            current_time = time.time()
            
            for key, item in self.cache.items():
                items.append({
                    "key": key,
                    "size": len(str(item.value)),
                    "created_at": item.created_at,
                    "expires_at": item.expires_at,
                    "ttl": item.expires_at - current_time if item.expires_at else None,
                    "access_count": item.access_count,
                    "last_accessed": item.last_accessed,
                    "is_expired": item.is_expired()
                })
            
            # 按访问次数排序
            items.sort(key=lambda x: x["access_count"], reverse=True)
            return items

class AnalysisCache:
    """分析结果缓存"""
    
    def __init__(self, cache_manager: CacheManager):
        self.cache = cache_manager
        self.logger = logging.getLogger("analysis_cache")
    
    def get_commit_analysis(self, commit_hash: str, agent_config_version: str) -> Optional[Dict[str, Any]]:
        """获取提交分析缓存"""
        key = self.cache._generate_key("commit_analysis", {
            "commit_hash": commit_hash,
            "config_version": agent_config_version
        })
        
        result = self.cache.get(key)
        if result:
            self.logger.debug(f"命中分析缓存: {commit_hash[:8]}")
        
        return result
    
    def set_commit_analysis(self, commit_hash: str, agent_config_version: str, 
                          analysis_result: Dict[str, Any], ttl: int = 7200) -> bool:
        """设置提交分析缓存"""
        key = self.cache._generate_key("commit_analysis", {
            "commit_hash": commit_hash,
            "config_version": agent_config_version
        })
        
        success = self.cache.set(key, analysis_result, ttl)
        if success:
            self.logger.debug(f"缓存分析结果: {commit_hash[:8]}")
        
        return success
    
    def get_quality_review(self, analysis_hash: str, reviewer_config_version: str) -> Optional[Dict[str, Any]]:
        """获取质量审核缓存"""
        key = self.cache._generate_key("quality_review", {
            "analysis_hash": analysis_hash,
            "config_version": reviewer_config_version
        })
        
        result = self.cache.get(key)
        if result:
            self.logger.debug(f"命中审核缓存: {analysis_hash[:8]}")
        
        return result
    
    def set_quality_review(self, analysis_hash: str, reviewer_config_version: str,
                         review_result: Dict[str, Any], ttl: int = 7200) -> bool:
        """设置质量审核缓存"""
        key = self.cache._generate_key("quality_review", {
            "analysis_hash": analysis_hash,
            "config_version": reviewer_config_version
        })
        
        success = self.cache.set(key, review_result, ttl)
        if success:
            self.logger.debug(f"缓存审核结果: {analysis_hash[:8]}")
        
        return success
    
    def invalidate_commit(self, commit_hash: str):
        """使提交相关的缓存失效"""
        # 这里可以实现更复杂的失效逻辑
        # 例如，删除所有与该提交相关的缓存项
        pass

# 全局缓存管理器实例
cache_manager = CacheManager(max_size=2000, default_ttl=3600)
analysis_cache = AnalysisCache(cache_manager)
