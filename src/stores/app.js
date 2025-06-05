import { defineStore } from 'pinia'
import { ref, reactive } from 'vue'

export const useAppStore = defineStore('app', () => {
  // 应用状态
  const isLoading = ref(false)
  const currentRepository = ref(null)
  const repositories = ref([])
  
  // 应用配置
  const config = reactive({
    theme: 'light',
    language: 'zh-CN',
    autoSave: true,
    apiEndpoint: 'http://localhost:8000'
  })

  // 初始化应用
  const initialize = async () => {
    try {
      isLoading.value = true
      // 加载配置
      await loadConfig()
      // 加载仓库列表
      await loadRepositories()
    } catch (error) {
      console.error('应用初始化失败:', error)
    } finally {
      isLoading.value = false
    }
  }

  // 加载配置
  const loadConfig = async () => {
    try {
      // 从本地存储或API加载配置
      const savedConfig = localStorage.getItem('gitmentor-config')
      if (savedConfig) {
        Object.assign(config, JSON.parse(savedConfig))
      }
    } catch (error) {
      console.error('加载配置失败:', error)
    }
  }

  // 保存配置
  const saveConfig = () => {
    try {
      localStorage.setItem('gitmentor-config', JSON.stringify(config))
    } catch (error) {
      console.error('保存配置失败:', error)
    }
  }

  // 加载仓库列表
  const loadRepositories = async () => {
    try {
      // 这里将来会调用后端API
      const savedRepos = localStorage.getItem('gitmentor-repositories')
      if (savedRepos) {
        repositories.value = JSON.parse(savedRepos)
      }
    } catch (error) {
      console.error('加载仓库列表失败:', error)
    }
  }

  // 添加仓库
  const addRepository = (repo) => {
    repositories.value.push(repo)
    localStorage.setItem('gitmentor-repositories', JSON.stringify(repositories.value))
  }

  // 删除仓库
  const removeRepository = (id) => {
    const index = repositories.value.findIndex(repo => repo.id === id)
    if (index > -1) {
      repositories.value.splice(index, 1)
      localStorage.setItem('gitmentor-repositories', JSON.stringify(repositories.value))
    }
  }

  // 设置当前仓库
  const setCurrentRepository = (repo) => {
    currentRepository.value = repo
  }

  return {
    // 状态
    isLoading,
    currentRepository,
    repositories,
    config,
    
    // 方法
    initialize,
    loadConfig,
    saveConfig,
    loadRepositories,
    addRepository,
    removeRepository,
    setCurrentRepository
  }
})
