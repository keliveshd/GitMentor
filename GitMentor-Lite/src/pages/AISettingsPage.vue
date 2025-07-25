<template>
  <div class="ai-settings-page">
    <div class="settings-header">
      <h1>🤖 AI服务设置</h1>
      <p class="settings-description">配置您的AI服务参数和偏好设置</p>
    </div>

    <div class="settings-content">
      <div class="settings-section">
        <h2>🔧 基础配置</h2>
        <div class="setting-item">
          <label for="api-endpoint">API端点</label>
          <input 
            id="api-endpoint" 
            type="text" 
            v-model="settings.apiEndpoint" 
            placeholder="请输入API端点地址"
            class="setting-input"
          />
        </div>
        
        <div class="setting-item">
          <label for="api-key">API密钥</label>
          <input 
            id="api-key" 
            type="password" 
            v-model="settings.apiKey" 
            placeholder="请输入API密钥"
            class="setting-input"
          />
        </div>
        
        <div class="setting-item">
          <label for="model-name">模型名称</label>
          <select id="model-name" v-model="settings.modelName" class="setting-select">
            <option value="">请选择模型</option>
            <option value="gpt-3.5-turbo">GPT-3.5 Turbo</option>
            <option value="gpt-4">GPT-4</option>
            <option value="claude-3">Claude-3</option>
            <option value="custom">自定义模型</option>
          </select>
        </div>
      </div>

      <div class="settings-section">
        <h2>⚙️ 高级设置</h2>
        <div class="setting-item">
          <label for="temperature">创造性温度</label>
          <div class="range-container">
            <input 
              id="temperature" 
              type="range" 
              min="0" 
              max="2" 
              step="0.1" 
              v-model="settings.temperature"
              class="setting-range"
            />
            <span class="range-value">{{ settings.temperature }}</span>
          </div>
        </div>
        
        <div class="setting-item">
          <label for="max-tokens">最大令牌数</label>
          <input 
            id="max-tokens" 
            type="number" 
            v-model="settings.maxTokens" 
            min="1" 
            max="4096"
            class="setting-input"
          />
        </div>
        
        <div class="setting-item checkbox-item">
          <input 
            id="auto-commit" 
            type="checkbox" 
            v-model="settings.autoCommit"
            class="setting-checkbox"
          />
          <label for="auto-commit">自动生成提交消息</label>
        </div>
        
        <div class="setting-item checkbox-item">
          <input 
            id="enable-suggestions" 
            type="checkbox" 
            v-model="settings.enableSuggestions"
            class="setting-checkbox"
          />
          <label for="enable-suggestions">启用代码建议</label>
        </div>
      </div>

      <div class="settings-section">
        <h2>🔍 TODO功能</h2>
        <div class="todo-notice">
          <p>📝 以下功能正在开发中，敬请期待：</p>
          <ul>
            <li>AI模型性能测试</li>
            <li>自定义提示词模板</li>
            <li>代码审查规则配置</li>
            <li>多语言支持设置</li>
            <li>使用统计和分析</li>
          </ul>
        </div>
      </div>
    </div>

    <div class="settings-footer">
      <button @click="resetSettings" class="btn btn-secondary">重置设置</button>
      <button @click="testConnection" class="btn btn-info" :disabled="testing">
        {{ testing ? '测试中...' : '测试连接' }}
      </button>
      <button @click="saveSettings" class="btn btn-primary" :disabled="saving">
        {{ saving ? '保存中...' : '保存设置' }}
      </button>
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

// 设置数据结构
interface AISettings {
  apiEndpoint: string
  apiKey: string
  modelName: string
  temperature: number
  maxTokens: number
  autoCommit: boolean
  enableSuggestions: boolean
}

// 响应式数据
const settings = ref<AISettings>({
  apiEndpoint: '',
  apiKey: '',
  modelName: '',
  temperature: 0.7,
  maxTokens: 2048,
  autoCommit: true,
  enableSuggestions: true
})

const saving = ref(false)
const testing = ref(false)

// 方法
const loadSettings = () => {
  // TODO: 从本地存储或后端加载设置
  const savedSettings = localStorage.getItem('ai-settings')
  if (savedSettings) {
    try {
      const parsed = JSON.parse(savedSettings)
      settings.value = { ...settings.value, ...parsed }
    } catch (error) {
      console.error('加载设置失败:', error)
    }
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
    await new Promise(resolve => setTimeout(resolve, 2000))
    
    alert('连接测试成功！')
  } catch (error) {
    console.error('连接测试失败:', error)
    alert('连接测试失败: ' + error)
  } finally {
    testing.value = false
  }
}

const resetSettings = () => {
  if (confirm('确定要重置所有设置吗？此操作不可撤销。')) {
    settings.value = {
      apiEndpoint: '',
      apiKey: '',
      modelName: '',
      temperature: 0.7,
      maxTokens: 2048,
      autoCommit: true,
      enableSuggestions: true
    }
  }
}

// 生命周期
onMounted(() => {
  loadSettings()
})
</script>

<style scoped>
.ai-settings-page {
  max-width: 800px;
  margin: 0 auto;
  padding: 20px;
  background: #ffffff;
  min-height: 100vh;
}

.settings-header {
  text-align: center;
  margin-bottom: 30px;
  padding-bottom: 20px;
  border-bottom: 2px solid #f0f0f0;
}

.settings-header h1 {
  color: #333;
  margin: 0 0 10px 0;
  font-size: 28px;
}

.settings-description {
  color: #666;
  margin: 0;
  font-size: 16px;
}

.settings-content {
  display: flex;
  flex-direction: column;
  gap: 30px;
}

.settings-section {
  background: #f8f9fa;
  padding: 20px;
  border-radius: 8px;
  border: 1px solid #e9ecef;
}

.settings-section h2 {
  color: #333;
  margin: 0 0 20px 0;
  font-size: 20px;
  border-bottom: 1px solid #dee2e6;
  padding-bottom: 10px;
}

.setting-item {
  margin-bottom: 20px;
}

.setting-item label {
  display: block;
  margin-bottom: 8px;
  font-weight: 600;
  color: #333;
}

.setting-input,
.setting-select {
  width: 100%;
  padding: 10px 12px;
  border: 1px solid #ddd;
  border-radius: 6px;
  font-size: 14px;
  transition: border-color 0.3s ease;
}

.setting-input:focus,
.setting-select:focus {
  outline: none;
  border-color: #667eea;
  box-shadow: 0 0 0 3px rgba(102, 126, 234, 0.1);
}

.range-container {
  display: flex;
  align-items: center;
  gap: 12px;
}

.setting-range {
  flex: 1;
}

.range-value {
  min-width: 40px;
  text-align: center;
  font-weight: 600;
  color: #667eea;
}

.checkbox-item {
  display: flex;
  align-items: center;
  gap: 10px;
}

.checkbox-item label {
  margin: 0;
  cursor: pointer;
}

.setting-checkbox {
  width: auto;
  margin: 0;
}

.todo-notice {
  background: #fff3cd;
  border: 1px solid #ffeaa7;
  border-radius: 6px;
  padding: 15px;
  color: #856404;
}

.todo-notice p {
  margin: 0 0 10px 0;
  font-weight: 600;
}

.todo-notice ul {
  margin: 0;
  padding-left: 20px;
}

.todo-notice li {
  margin-bottom: 5px;
}

.settings-footer {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  margin-top: 30px;
  padding-top: 20px;
  border-top: 2px solid #f0f0f0;
}

.btn {
  padding: 10px 20px;
  border: none;
  border-radius: 6px;
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.3s ease;
}

.btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.btn-primary {
  background: #667eea;
  color: white;
}

.btn-primary:hover:not(:disabled) {
  background: #5a67d8;
  transform: translateY(-1px);
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

/* 深色主题支持 */
@media (prefers-color-scheme: dark) {
  .ai-settings-page {
    background: #1a202c;
    color: #e2e8f0;
  }

  .settings-section {
    background: #2d3748;
    border-color: #4a5568;
  }

  .settings-header h1,
  .settings-section h2,
  .setting-item label {
    color: #e2e8f0;
  }

  .settings-description {
    color: #a0aec0;
  }

  .setting-input,
  .setting-select {
    background: #2d3748;
    border-color: #4a5568;
    color: #e2e8f0;
  }

  .setting-input:focus,
  .setting-select:focus {
    border-color: #667eea;
  }

  .todo-notice {
    background: #2d3748;
    border-color: #4a5568;
    color: #e2e8f0;
  }
}
</style>
