/**
 * 仓库配置API
 */

import request from './request'

export const repositoryConfigApi = {
  /**
   * 获取所有仓库配置
   */
  getRepositories() {
    return request({
      url: '/api/repository-configs',
      method: 'get'
    })
  },

  /**
   * 获取单个仓库配置
   */
  getRepository(repoName) {
    return request({
      url: `/api/repository-configs/${repoName}`,
      method: 'get'
    })
  },

  /**
   * 创建仓库配置
   */
  createRepository(data) {
    return request({
      url: '/api/repository-configs',
      method: 'post',
      data
    })
  },

  /**
   * 更新仓库配置
   */
  updateRepository(repoName, data) {
    return request({
      url: `/api/repository-configs/${repoName}`,
      method: 'put',
      data
    })
  },

  /**
   * 删除仓库配置
   */
  deleteRepository(repoName) {
    return request({
      url: `/api/repository-configs/${repoName}`,
      method: 'delete'
    })
  },

  /**
   * 启用仓库
   */
  enableRepository(repoName) {
    return request({
      url: `/api/repository-configs/${repoName}/enable`,
      method: 'post'
    })
  },

  /**
   * 禁用仓库
   */
  disableRepository(repoName) {
    return request({
      url: `/api/repository-configs/${repoName}/disable`,
      method: 'post'
    })
  },

  /**
   * 获取用户映射
   */
  getUserMapping(repoName) {
    return request({
      url: `/api/repository-configs/${repoName}/user-mapping`,
      method: 'get'
    })
  },

  /**
   * 更新用户映射
   */
  updateUserMapping(repoName, userMapping) {
    return request({
      url: `/api/repository-configs/${repoName}/user-mapping`,
      method: 'put',
      data: { user_mapping: userMapping }
    })
  },

  /**
   * 获取仓库的Agent配置
   */
  getAgents(repoName) {
    return request({
      url: `/api/repository-configs/${repoName}/agents`,
      method: 'get'
    })
  },

  /**
   * 更新仓库的Agent配置
   */
  updateAgents(repoName, agents) {
    return request({
      url: `/api/repository-configs/${repoName}/agents`,
      method: 'put',
      data: { agents }
    })
  },

  /**
   * 验证仓库路径
   */
  validatePath(repoName, path) {
    return request({
      url: `/api/repository-configs/${repoName}/validate-path`,
      method: 'post',
      data: { path }
    })
  },

  /**
   * 获取启用的仓库列表
   */
  getEnabledRepositories() {
    return request({
      url: '/api/repository-configs/enabled',
      method: 'get'
    })
  },

  /**
   * 获取可用的Agent类型
   */
  getAgentTypes() {
    return request({
      url: '/api/agent-types',
      method: 'get'
    })
  }
}

export default repositoryConfigApi
