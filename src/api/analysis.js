/**
 * 分析结果API
 */

import request from './request'

export const analysisApi = {
  /**
   * 获取分析结果列表
   */
  getResults(params = {}) {
    return request({
      url: '/api/analysis/results',
      method: 'get',
      params
    })
  },

  /**
   * 获取单个分析结果
   */
  getResult(taskId) {
    return request({
      url: `/api/analysis/results/${taskId}`,
      method: 'get'
    })
  },

  /**
   * 重试分析任务
   */
  retryAnalysis(taskId) {
    return request({
      url: `/api/analysis/results/${taskId}/retry`,
      method: 'post'
    })
  },

  /**
   * 获取处理统计信息
   */
  getStatistics() {
    return request({
      url: '/api/analysis/statistics',
      method: 'get'
    })
  },

  /**
   * 获取质量趋势数据
   */
  getQualityTrends(hours = 24) {
    return request({
      url: '/api/analysis/quality-trends',
      method: 'get',
      params: { hours }
    })
  },

  /**
   * 根据提交哈希获取分析结果
   */
  getCommitAnalysis(commitHash) {
    return request({
      url: `/api/analysis/commit/${commitHash}`,
      method: 'get'
    })
  },

  /**
   * 清理旧的分析结果
   */
  cleanupOldResults(maxAgeHours = 24) {
    return request({
      url: '/api/analysis/results',
      method: 'delete',
      params: { max_age_hours: maxAgeHours }
    })
  },

  /**
   * 获取分析仪表板数据
   */
  getDashboard() {
    return request({
      url: '/api/analysis/dashboard',
      method: 'get'
    })
  }
}

export default analysisApi
