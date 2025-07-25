/**
 * 最近打开仓库管理工具类
 * 作者：Evilek
 * 编写日期：2025-01-25
 */

export interface RecentRepo {
  path: string
  name: string
  lastOpened: number
}

export class RecentReposManager {
  private static readonly STORAGE_KEY = 'gitmentor_recent_repos'
  private static readonly MAX_RECENT_COUNT = 10

  /**
   * 获取最近打开的仓库列表
   */
  static getRecentRepos(): RecentRepo[] {
    try {
      const stored = localStorage.getItem(this.STORAGE_KEY)
      if (!stored) return []
      
      const repos: RecentRepo[] = JSON.parse(stored)
      // 按最后打开时间排序（最新的在前）
      return repos.sort((a, b) => b.lastOpened - a.lastOpened)
    } catch (error) {
      console.error('获取最近仓库列表失败:', error)
      return []
    }
  }

  /**
   * 添加或更新最近打开的仓库
   */
  static addRecentRepo(path: string): void {
    try {
      const name = this.extractRepoName(path)
      const newRepo: RecentRepo = {
        path,
        name,
        lastOpened: Date.now()
      }

      let repos = this.getRecentRepos()
      
      // 移除已存在的相同路径
      repos = repos.filter(repo => repo.path !== path)
      
      // 添加到开头
      repos.unshift(newRepo)
      
      // 限制数量
      if (repos.length > this.MAX_RECENT_COUNT) {
        repos = repos.slice(0, this.MAX_RECENT_COUNT)
      }

      localStorage.setItem(this.STORAGE_KEY, JSON.stringify(repos))
    } catch (error) {
      console.error('保存最近仓库失败:', error)
    }
  }

  /**
   * 获取上次打开的仓库路径
   */
  static getLastOpenedRepo(): string | null {
    const repos = this.getRecentRepos()
    return repos.length > 0 ? repos[0].path : null
  }

  /**
   * 移除指定路径的仓库
   */
  static removeRecentRepo(path: string): void {
    try {
      let repos = this.getRecentRepos()
      repos = repos.filter(repo => repo.path !== path)
      localStorage.setItem(this.STORAGE_KEY, JSON.stringify(repos))
    } catch (error) {
      console.error('移除最近仓库失败:', error)
    }
  }

  /**
   * 清空所有最近仓库
   */
  static clearRecentRepos(): void {
    try {
      localStorage.removeItem(this.STORAGE_KEY)
    } catch (error) {
      console.error('清空最近仓库失败:', error)
    }
  }

  /**
   * 从路径中提取仓库名称
   */
  private static extractRepoName(path: string): string {
    return path.split(/[/\\]/).pop() || path
  }

  /**
   * 验证路径是否仍然存在（可选的验证功能）
   */
  static async validateRepoPath(path: string): Promise<boolean> {
    // 这里可以添加路径验证逻辑
    // 由于是前端代码，无法直接访问文件系统
    // 可以通过 Tauri 命令来验证，但为了简单起见暂时返回 true
    return true
  }

  /**
   * 获取格式化的显示文本
   */
  static getDisplayText(repo: RecentRepo): string {
    const date = new Date(repo.lastOpened)
    const timeStr = date.toLocaleString('zh-CN', {
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit'
    })
    return `${repo.name} (${timeStr})`
  }
}
