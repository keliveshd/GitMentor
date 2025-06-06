/**
 * Agent管理API
 */

import request from './request'

export const agentApi = {
  /**
   * 获取所有Agent列表
   */
  getAgents() {
    return request({
      url: '/api/agents',
      method: 'get'
    })
  },

  /**
   * 获取单个Agent状态
   */
  getAgent(agentId) {
    return request({
      url: `/api/agents/${agentId}`,
      method: 'get'
    })
  },

  /**
   * 创建新的Agent
   */
  createAgent(data) {
    return request({
      url: '/api/agents/create',
      method: 'post',
      data
    })
  },

  /**
   * 更新Agent配置
   */
  updateAgentConfig(agentId, config) {
    return request({
      url: `/api/agents/${agentId}/config`,
      method: 'post',
      data: config
    })
  },

  /**
   * 删除Agent
   */
  removeAgent(agentId) {
    return request({
      url: `/api/agents/${agentId}`,
      method: 'delete'
    })
  },

  /**
   * 检查Agent健康状态
   */
  checkHealth() {
    return request({
      url: '/api/agents/health',
      method: 'get'
    })
  },

  /**
   * 分析提交
   */
  analyzeCommit(commitData) {
    return request({
      url: '/api/agents/analyze',
      method: 'post',
      data: commitData
    })
  },

  /**
   * 获取系统指标
   */
  getMetrics() {
    return request({
      url: '/api/agents/metrics',
      method: 'get'
    })
  },

  /**
   * 获取可用的LLM客户端
   */
  getLLMClients() {
    return request({
      url: '/api/agents/llm-clients',
      method: 'get'
    })
  },

  /**
   * 测试LLM连接
   */
  testLLMConnection(clientName = null) {
    return request({
      url: '/api/agents/llm-clients/test',
      method: 'post',
      data: clientName ? { client_name: clientName } : {}
    })
  }
}

export default agentApi
