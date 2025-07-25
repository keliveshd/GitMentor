<template>
  <div class="ai-settings-page">
    <div class="settings-layout">
      <!-- 左侧菜单 -->
      <div class="settings-sidebar">
        <div class="sidebar-header">
          <h2>🤖 AI服务设置</h2>
        </div>
        <nav class="settings-menu">
          <div v-for="item in menuItems" :key="item.key" :class="['menu-item', { active: selectedMenu === item.key }]"
            @click="selectedMenu = item.key">
            <span class="menu-icon">{{ item.icon }}</span>
            <span class="menu-label">{{ item.label }}</span>
          </div>
        </nav>
      </div>

      <!-- 右侧内容区 -->
      <div class="settings-content">
        <div class="content-header">
          <h1>{{ getCurrentMenuTitle() }}</h1>
          <div class="header-actions">
            <button @click="testConnection" class="btn btn-info" :disabled="testing || !canTestConnection()">
              {{ testing ? '测试中...' : '🔗 测试连接' }}
            </button>
            <button @click="saveSettings" class="btn btn-primary" :disabled="saving">
              {{ saving ? '保存中...' : '💾 保存设置' }}
            </button>
          </div>
        </div>

        <div class="content-body">
          <!-- 基础设置 -->
          <div v-if="selectedMenu === 'basic'" class="settings-section">
            <div class="section-card">
              <h3>🌐 语言设置</h3>
              <div class="setting-item">
                <label for="language">提交信息语言</label>
                <select id="language" v-model="settings.base.language" class="setting-select">
                  <option value="Simplified Chinese">简体中文</option>
                  <option value="Traditional Chinese">繁体中文</option>
                  <option value="English">English</option>
                  <option value="Japanese">日本語</option>
                  <option value="Korean">한국어</option>
                </select>
              </div>
            </div>

            <div class="section-card">
              <h3>🤖 AI提供商</h3>
              <div class="setting-item">
                <label for="provider">选择AI提供商</label>
                <select id="provider" v-model="settings.base.provider" class="setting-select"
                  @change="onProviderChange">
                  <option v-for="provider in supportedProviders" :key="provider.id" :value="provider.id">
                    {{ provider.name }}
                  </option>
                </select>
                <p class="setting-description">{{ getCurrentProviderDescription() }}</p>
              </div>
            </div>



            <div class="section-card">
              <h3>🔧 {{ getCurrentProviderName() }} 配置</h3>

              <!-- OpenAI 配置 -->
              <div v-if="settings.base.provider === 'OpenAI'" class="provider-config">
                <div class="setting-item">
                  <label for="openai-api-key">API Key</label>
                  <input id="openai-api-key" type="password" v-model="settings.providers.openai.apiKey"
                    placeholder="sk-..." class="setting-input" />
                </div>
                <div class="setting-item">
                  <label for="openai-base-url">Base URL</label>
                  <input id="openai-base-url" type="text" v-model="settings.providers.openai.baseUrl"
                    placeholder="https://api.openai.com/v1" class="setting-input" />
                </div>
              </div>

              <!-- Ollama 配置 -->
              <div v-if="settings.base.provider === 'Ollama'" class="provider-config">
                <div class="setting-item">
                  <label for="ollama-base-url">服务地址</label>
                  <input id="ollama-base-url" type="text" v-model="settings.providers.ollama.baseUrl"
                    placeholder="http://localhost:11434" class="setting-input" />
                </div>
              </div>

              <!-- Zhipu 配置 -->
              <div v-if="settings.base.provider === 'Zhipu'" class="provider-config">
                <div class="setting-item">
                  <label for="zhipu-api-key">API Key</label>
                  <input id="zhipu-api-key" type="password" v-model="settings.providers.zhipu.apiKey"
                    placeholder="请输入智谱AI API Key" class="setting-input" />
                </div>
              </div>

              <!-- Anthropic 配置 -->
              <div v-if="settings.base.provider === 'Anthropic'" class="provider-config">
                <div class="setting-item">
                  <label for="anthropic-api-key">API Key</label>
                  <input id="anthropic-api-key" type="password" v-model="settings.providers.anthropic.apiKey"
                    placeholder="请输入Anthropic API Key" class="setting-input" />
                </div>
              </div>

              <!-- DashScope 配置 -->
              <div v-if="settings.base.provider === 'DashScope'" class="provider-config">
                <div class="setting-item">
                  <label for="dashscope-api-key">API Key</label>
                  <input id="dashscope-api-key" type="password" v-model="settings.providers.dashscope.apiKey"
                    placeholder="请输入阿里云通义API Key" class="setting-input" />
                </div>
              </div>

              <!-- Doubao 配置 -->
              <div v-if="settings.base.provider === 'Doubao'" class="provider-config">
                <div class="setting-item">
                  <label for="doubao-api-key">API Key</label>
                  <input id="doubao-api-key" type="password" v-model="settings.providers.doubao.apiKey"
                    placeholder="请输入豆包AI API Key" class="setting-input" />
                </div>
              </div>

              <!-- Gemini 配置 -->
              <div v-if="settings.base.provider === 'Gemini'" class="provider-config">
                <div class="setting-item">
                  <label for="gemini-api-key">API Key</label>
                  <input id="gemini-api-key" type="password" v-model="settings.providers.gemini.apiKey"
                    placeholder="请输入Google Gemini API Key" class="setting-input" />
                </div>
              </div>

              <!-- Deepseek 配置 -->
              <div v-if="settings.base.provider === 'Deepseek'" class="provider-config">
                <div class="setting-item">
                  <label for="deepseek-api-key">API Key</label>
                  <input id="deepseek-api-key" type="password" v-model="settings.providers.deepseek.apiKey"
                    placeholder="请输入Deepseek API Key" class="setting-input" />
                </div>
              </div>

              <!-- 通用配置提示 -->
              <div v-if="!hasProviderConfig()" class="provider-config">
                <div class="config-notice">
                  <p>📝 {{ getCurrentProviderName() }} 暂无需要配置的参数</p>
                </div>
              </div>
            </div>

            <div class="section-card">
              <h3>🎯 模型选择</h3>
              <div class="setting-item">
                <label for="model">选择模型</label>
                <select id="model" v-model="settings.base.model" class="setting-select"
                  :disabled="!availableModels.length">
                  <option value="">{{ availableModels.length ? '请选择模型' : '请先配置提供商' }}</option>
                  <option v-for="model in availableModels" :key="model.id" :value="model.id">
                    {{ model.name }}
                  </option>
                </select>
                <button @click="refreshModels" class="btn btn-small btn-secondary" :disabled="refreshingModels">
                  {{ refreshingModels ? '刷新中...' : '🔄 刷新模型列表' }}
                </button>
              </div>
            </div>
          </div>



          <!-- 功能设置 -->
          <div v-if="selectedMenu === 'features'" class="settings-section">
            <div class="section-card">
              <h3>📝 提交信息格式</h3>
              <div class="setting-item checkbox-item">
                <input id="enable-emoji" type="checkbox" v-model="settings.features.enableEmoji"
                  class="setting-checkbox" />
                <label for="enable-emoji">启用Emoji表情</label>
                <p class="setting-description">在提交信息中添加相关的emoji表情</p>
              </div>

              <div class="setting-item checkbox-item">
                <input id="enable-body" type="checkbox" v-model="settings.features.enableBody"
                  class="setting-checkbox" />
                <label for="enable-body">启用详细描述</label>
                <p class="setting-description">生成包含详细描述的提交信息</p>
              </div>

              <div class="setting-item checkbox-item">
                <input id="enable-layered-commit" type="checkbox" v-model="settings.features.enableLayeredCommit"
                  class="setting-checkbox" />
                <label for="enable-layered-commit">启用分层提交</label>
                <p class="setting-description">为每个文件生成单独的变更描述</p>
              </div>
            </div>

            <div class="section-card">
              <h3>🔧 AI生成选项</h3>
              <div class="setting-item checkbox-item">
                <input id="use-recent-commits" type="checkbox" v-model="settings.features.useRecentCommits"
                  class="setting-checkbox" />
                <label for="use-recent-commits">参考最近提交</label>
                <p class="setting-description">使用最近的提交记录作为生成参考</p>
              </div>

              <div class="setting-item checkbox-item">
                <input id="enable-streaming" type="checkbox" v-model="settings.features.enableStreaming"
                  class="setting-checkbox" />
                <label for="enable-streaming">启用流式生成</label>
                <p class="setting-description">实时显示AI生成过程</p>
              </div>
            </div>
          </div>

          <!-- 高级设置 -->
          <div v-if="selectedMenu === 'advanced'" class="settings-section">
            <div class="section-card">
              <h3>🎛️ 模型参数</h3>
              <div class="setting-item">
                <label for="temperature">创造性温度 ({{ settings.advanced.temperature }})</label>
                <input id="temperature" type="range" min="0" max="2" step="0.1" v-model="settings.advanced.temperature"
                  class="setting-range" />
                <p class="setting-description">控制AI生成的创造性程度，值越高越有创意</p>
              </div>

              <div class="setting-item">
                <label for="max-tokens">最大令牌数</label>
                <input id="max-tokens" type="number" v-model="settings.advanced.maxTokens" min="100" max="8192"
                  class="setting-input" />
                <p class="setting-description">限制AI生成内容的最大长度</p>
              </div>
            </div>

            <div class="section-card">
              <h3>⚡ 性能设置</h3>
              <div class="setting-item">
                <label for="timeout">请求超时 (秒)</label>
                <input id="timeout" type="number" v-model="settings.advanced.timeout" min="10" max="300"
                  class="setting-input" />
              </div>

              <div class="setting-item">
                <label for="retry-count">重试次数</label>
                <input id="retry-count" type="number" v-model="settings.advanced.retryCount" min="0" max="5"
                  class="setting-input" />
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'

/**
 * AI设置页面组件
 * 作者：Evilek
 * 编写日期：2025-07-25
 */

// 接口定义
interface AIProvider {
  id: string
  name: string
  description: string
  requiresApiKey?: boolean
  requiresBaseUrl?: boolean
}

interface AIModel {
  id: string
  name: string
  maxTokens?: number
}

interface AISettings {
  base: {
    language: string
    provider: string
    model: string
  }
  providers: {
    openai: {
      apiKey: string
      baseUrl: string
    }
    ollama: {
      baseUrl: string
    }
    zhipu: {
      apiKey: string
    }
    anthropic: {
      apiKey: string
    }
    dashscope: {
      apiKey: string
    }
    doubao: {
      apiKey: string
    }
    gemini: {
      apiKey: string
    }
    deepseek: {
      apiKey: string
    }
  }
  features: {
    enableEmoji: boolean
    enableBody: boolean
    enableLayeredCommit: boolean
    useRecentCommits: boolean
    enableStreaming: boolean
  }
  advanced: {
    temperature: number
    maxTokens: number
    timeout: number
    retryCount: number
  }
}

// 响应式数据
const selectedMenu = ref('basic')
const saving = ref(false)
const testing = ref(false)
const refreshingModels = ref(false)
const availableModels = ref<AIModel[]>([])

// 菜单项配置
const menuItems = ref([
  { key: 'basic', icon: '🏠', label: '基础设置' },
  { key: 'features', icon: '⚙️', label: '功能设置' },
  { key: 'advanced', icon: '🎛️', label: '高级选项' }
])

// 支持的AI提供商列表
const supportedProviders = ref<AIProvider[]>([
  {
    id: 'OpenAI',
    name: 'OpenAI',
    description: '支持GPT-3.5、GPT-4等模型',
    requiresApiKey: true,
    requiresBaseUrl: true
  },
  {
    id: 'Ollama',
    name: 'Ollama',
    description: '本地部署的开源模型服务',
    requiresBaseUrl: true
  },
  {
    id: 'Zhipu',
    name: '智谱AI',
    description: '智谱AI GLM系列模型',
    requiresApiKey: true
  },
  {
    id: 'Anthropic',
    name: 'Anthropic',
    description: 'Claude系列模型',
    requiresApiKey: true
  },
  {
    id: 'DashScope',
    name: '阿里云通义',
    description: '阿里云通义千问模型',
    requiresApiKey: true
  },
  {
    id: 'Doubao',
    name: '豆包AI',
    description: '字节跳动豆包AI模型',
    requiresApiKey: true
  },
  {
    id: 'Gemini',
    name: 'Google Gemini',
    description: 'Google Gemini模型',
    requiresApiKey: true
  },
  {
    id: 'Deepseek',
    name: 'Deepseek',
    description: 'Deepseek AI模型',
    requiresApiKey: true
  }
])

// 默认设置
const defaultSettings: AISettings = {
  base: {
    language: 'Simplified Chinese',
    provider: 'OpenAI',
    model: ''
  },
  providers: {
    openai: {
      apiKey: '',
      baseUrl: 'https://api.openai.com/v1'
    },
    ollama: {
      baseUrl: 'http://localhost:11434'
    },
    zhipu: {
      apiKey: ''
    },
    anthropic: {
      apiKey: ''
    },
    dashscope: {
      apiKey: ''
    },
    doubao: {
      apiKey: ''
    },
    gemini: {
      apiKey: ''
    },
    deepseek: {
      apiKey: ''
    }
  },
  features: {
    enableEmoji: true,
    enableBody: true,
    enableLayeredCommit: false,
    useRecentCommits: true,
    enableStreaming: true
  },
  advanced: {
    temperature: 0.7,
    maxTokens: 2048,
    timeout: 60,
    retryCount: 3
  }
}

const settings = ref<AISettings>(JSON.parse(JSON.stringify(defaultSettings)))

// 计算属性
const getCurrentMenuTitle = () => {
  const item = menuItems.value.find(item => item.key === selectedMenu.value)
  return item ? item.label : '设置'
}

const getCurrentProviderName = () => {
  const provider = supportedProviders.value.find(p => p.id === settings.value.base.provider)
  return provider ? provider.name : settings.value.base.provider
}

const getCurrentProviderDescription = () => {
  const provider = supportedProviders.value.find(p => p.id === settings.value.base.provider)
  return provider ? provider.description : ''
}

const hasProviderConfig = () => {
  const provider = supportedProviders.value.find(p => p.id === settings.value.base.provider)
  return provider && (provider.requiresApiKey || provider.requiresBaseUrl)
}

const canTestConnection = () => {
  const provider = settings.value.base.provider
  switch (provider) {
    case 'OpenAI':
      return settings.value.providers.openai.apiKey.trim() !== ''
    case 'Ollama':
      return settings.value.providers.ollama.baseUrl.trim() !== ''
    case 'Zhipu':
      return settings.value.providers.zhipu.apiKey.trim() !== ''
    case 'Anthropic':
      return settings.value.providers.anthropic.apiKey.trim() !== ''
    default:
      return false
  }
}

// 方法
const loadSettings = () => {
  try {
    const savedSettings = localStorage.getItem('ai-settings')
    if (savedSettings) {
      const parsed = JSON.parse(savedSettings)
      settings.value = { ...defaultSettings, ...parsed }
    }
  } catch (error) {
    console.error('加载设置失败:', error)
  }
}

const saveSettings = async () => {
  try {
    saving.value = true

    // TODO: 保存到后端或本地存储
    localStorage.setItem('ai-settings', JSON.stringify(settings.value))

    // 模拟保存延迟
    await new Promise(resolve => setTimeout(resolve, 1000))

    alert('设置保存成功！')
  } catch (error) {
    console.error('保存设置失败:', error)
    alert('保存设置失败: ' + error)
  } finally {
    saving.value = false
  }
}

const testConnection = async () => {
  try {
    testing.value = true

    // TODO: 实现实际的连接测试
    const provider = settings.value.base.provider
    console.log(`测试 ${provider} 连接...`)

    // 模拟连接测试
    await new Promise(resolve => setTimeout(resolve, 2000))

    alert(`${getCurrentProviderName()} 连接测试成功！`)
  } catch (error) {
    console.error('连接测试失败:', error)
    alert('连接测试失败: ' + error)
  } finally {
    testing.value = false
  }
}

const refreshModels = async () => {
  try {
    refreshingModels.value = true

    // TODO: 根据当前提供商获取可用模型列表
    const provider = settings.value.base.provider
    console.log(`刷新 ${provider} 模型列表...`)

    // 模拟获取模型列表
    await new Promise(resolve => setTimeout(resolve, 1500))

    // 根据提供商返回不同的模型列表
    switch (provider) {
      case 'OpenAI':
        availableModels.value = [
          { id: 'gpt-3.5-turbo', name: 'GPT-3.5 Turbo', maxTokens: 4096 },
          { id: 'gpt-4', name: 'GPT-4', maxTokens: 8192 },
          { id: 'gpt-4-turbo', name: 'GPT-4 Turbo', maxTokens: 128000 }
        ]
        break
      case 'Anthropic':
        availableModels.value = [
          { id: 'claude-3-haiku', name: 'Claude 3 Haiku', maxTokens: 200000 },
          { id: 'claude-3-sonnet', name: 'Claude 3 Sonnet', maxTokens: 200000 },
          { id: 'claude-3-opus', name: 'Claude 3 Opus', maxTokens: 200000 }
        ]
        break
      case 'Zhipu':
        availableModels.value = [
          { id: 'glm-4', name: 'GLM-4', maxTokens: 128000 },
          { id: 'glm-3-turbo', name: 'GLM-3 Turbo', maxTokens: 128000 }
        ]
        break
      default:
        availableModels.value = []
    }

    console.log(`获取到 ${availableModels.value.length} 个模型`)
  } catch (error) {
    console.error('刷新模型列表失败:', error)
    alert('刷新模型列表失败: ' + error)
  } finally {
    refreshingModels.value = false
  }
}

const onProviderChange = () => {
  // 切换提供商时清空模型选择和模型列表
  settings.value.base.model = ''
  availableModels.value = []

  // 自动刷新模型列表
  if (canTestConnection()) {
    refreshModels()
  }
}

// 生命周期
onMounted(() => {
  loadSettings()

  // 如果已配置提供商，自动加载模型列表
  if (canTestConnection()) {
    refreshModels()
  }
})
</script>

<style scoped>
.ai-settings-page {
  height: 100vh;
  background: #f8f9fa;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
}

.settings-layout {
  display: flex;
  height: 100%;
}

/* 左侧边栏 */
.settings-sidebar {
  width: 280px;
  background: #ffffff;
  border-right: 1px solid #e9ecef;
  display: flex;
  flex-direction: column;
}

.sidebar-header {
  padding: 20px;
  border-bottom: 1px solid #e9ecef;
  background: #f8f9fa;
}

.sidebar-header h2 {
  margin: 0;
  font-size: 18px;
  color: #333;
  font-weight: 600;
}

.settings-menu {
  flex: 1;
  padding: 16px 0;
}

.menu-item {
  display: flex;
  align-items: center;
  padding: 12px 20px;
  cursor: pointer;
  transition: all 0.2s ease;
  border-left: 3px solid transparent;
}

.menu-item:hover {
  background: #f8f9fa;
}

.menu-item.active {
  background: #e3f2fd;
  border-left-color: #2196f3;
  color: #1976d2;
}

.menu-icon {
  font-size: 16px;
  margin-right: 12px;
  width: 20px;
  text-align: center;
}

.menu-label {
  font-size: 14px;
  font-weight: 500;
}

/* 右侧内容区 */
.settings-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.content-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px 24px;
  background: #ffffff;
  border-bottom: 1px solid #e9ecef;
}

.content-header h1 {
  margin: 0;
  font-size: 24px;
  color: #333;
  font-weight: 600;
}

.header-actions {
  display: flex;
  gap: 12px;
}

.content-body {
  flex: 1;
  padding: 24px;
  overflow-y: auto;
}

/* 设置区域 */
.settings-section {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.section-card {
  background: #ffffff;
  border: 1px solid #e9ecef;
  border-radius: 8px;
  padding: 24px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
}

.section-card h3 {
  margin: 0 0 20px 0;
  font-size: 18px;
  color: #333;
  font-weight: 600;
  border-bottom: 1px solid #f0f0f0;
  padding-bottom: 12px;
}

/* 设置项 */
.setting-item {
  margin-bottom: 20px;
}

.setting-item:last-child {
  margin-bottom: 0;
}

.setting-item label {
  display: block;
  margin-bottom: 8px;
  font-weight: 600;
  color: #333;
  font-size: 14px;
}

.setting-input,
.setting-select {
  width: 100%;
  padding: 10px 12px;
  border: 1px solid #ddd;
  border-radius: 6px;
  font-size: 14px;
  transition: border-color 0.3s ease, box-shadow 0.3s ease;
  background: #ffffff;
}

.setting-input:focus,
.setting-select:focus {
  outline: none;
  border-color: #2196f3;
  box-shadow: 0 0 0 3px rgba(33, 150, 243, 0.1);
}

.setting-range {
  width: 100%;
  margin: 8px 0;
}

.setting-description {
  margin: 8px 0 0 0;
  font-size: 12px;
  color: #666;
  line-height: 1.4;
}

/* 复选框项 */
.checkbox-item {
  display: flex;
  align-items: flex-start;
  gap: 12px;
}

.checkbox-item label {
  margin: 0;
  cursor: pointer;
  flex: 1;
}

.setting-checkbox {
  width: auto;
  margin: 0;
  margin-top: 2px;
}

/* 按钮样式 */
.btn {
  padding: 10px 16px;
  border: none;
  border-radius: 6px;
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.3s ease;
  display: inline-flex;
  align-items: center;
  gap: 6px;
}

.btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.btn-primary {
  background: #2196f3;
  color: white;
}

.btn-primary:hover:not(:disabled) {
  background: #1976d2;
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(33, 150, 243, 0.3);
}

.btn-secondary {
  background: #6c757d;
  color: white;
}

.btn-secondary:hover:not(:disabled) {
  background: #5a6268;
  transform: translateY(-1px);
}

.btn-info {
  background: #17a2b8;
  color: white;
}

.btn-info:hover:not(:disabled) {
  background: #138496;
  transform: translateY(-1px);
}

.btn-small {
  padding: 6px 12px;
  font-size: 12px;
  margin-top: 8px;
}

/* 提供商配置 */
.provider-config {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.config-notice {
  text-align: center;
  padding: 20px;
  background: #f8f9fa;
  border-radius: 6px;
  color: #666;
}

.config-notice p {
  margin: 0;
  font-size: 14px;
}
</style>
