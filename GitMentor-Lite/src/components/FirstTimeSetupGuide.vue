<template>
  <div class="setup-overlay">
    <div class="setup-modal">
      <div class="setup-header">
        <h2>🚀 欢迎使用 GitMentor</h2>
        <p>首次使用需要配置AI服务，让我们开始设置吧！</p>
      </div>

      <div class="setup-content">
        <!-- 步骤指示器与箭头导航 - Author: Evilek, Date: 2025-01-09 -->
        <div class="step-indicator-wrapper">
          <button v-if="currentStep > 1" @click="prevStep" class="nav-btn nav-prev">
            <span class="nav-icon">←</span>
            <span class="nav-text">上一步</span>
          </button>

          <div class="step-indicator">
            <div class="step" :class="{ active: currentStep >= 1, completed: currentStep > 1 }">
              <span class="step-number">1</span>
              <span class="step-title">选择AI提供商</span>
            </div>
            <div class="step" :class="{ active: currentStep >= 2, completed: currentStep > 2 }">
              <span class="step-number">2</span>
              <span class="step-title">配置API密钥</span>
            </div>
            <div class="step" :class="{ active: currentStep >= 3, completed: currentStep > 3 }">
              <span class="step-number">3</span>
              <span class="step-title">测试连接</span>
            </div>
          </div>

          <button v-if="currentStep < 3" @click="nextStep"
            :disabled="(currentStep === 1 && !selectedProvider) || (currentStep === 2 && !isConfigValid)"
            class="nav-btn nav-next">
            <span class="nav-text">下一步</span>
            <span class="nav-icon">→</span>
          </button>
          <button v-else-if="currentStep === 3 && testResult?.success" @click="completeSetup"
            class="nav-btn nav-complete">
            <span class="nav-text">完成设置</span>
            <span class="nav-icon">✓</span>
          </button>
          <button v-else-if="currentStep === 3" @click="testConnection" class="nav-btn nav-retry" :disabled="testing">
            <span class="nav-text">重新测试</span>
            <span class="nav-icon">↻</span>
          </button>
        </div>

        <!-- 步骤内容容器，添加动画过渡 - Author: Evilek, Date: 2025-01-09 -->
        <div class="step-content-wrapper">
          <!-- 步骤1：选择AI提供商 -->
          <Transition name="step-slide" mode="out-in">
            <div v-if="currentStep === 1" key="step1" class="step-content">
              <div class="step-header">
                <h3>选择AI提供商</h3>
                <p>请选择您要使用的AI服务提供商：</p>
              </div>

              <div class="provider-grid">
                <div v-for="provider in popularProviders" :key="provider.id" class="provider-card"
                  :class="{ selected: selectedProvider === provider.id }" @click="selectProvider(provider.id)">
                  <div class="provider-icon">
                    <img :src="provider.icon" :alt="provider.name" />
                  </div>
                  <div class="provider-name">{{ provider.name }}</div>
                  <div class="provider-desc">{{ provider.description }}</div>
                  <div class="provider-price">{{ provider.price }}</div>
                </div>
              </div>
            </div>
          </Transition>

          <!-- 步骤2：配置API密钥 -->
          <Transition name="step-slide" mode="out-in">
            <div v-if="currentStep === 2" key="step2" class="step-content">
              <div class="step-header">
                <h3>配置API密钥</h3>
                <p>请输入 {{ getProviderName(selectedProvider) }} 的API密钥：</p>
              </div>

              <div class="config-form">
                <div v-if="selectedProvider === 'OpenAI'" class="form-group">
                  <label>API密钥：</label>
                  <input v-model="apiConfig.openai.api_key" type="password" placeholder="sk-..." class="config-input" />
                  <label>Base URL（可选）：</label>
                  <input v-model="apiConfig.openai.base_url" type="text" placeholder="https://api.openai.com/v1"
                    class="config-input" />
                </div>

                <div v-else-if="selectedProvider === 'Ollama'" class="form-group">
                  <label>服务地址：</label>
                  <input v-model="apiConfig.ollama.base_url" type="text" placeholder="http://localhost:11434"
                    class="config-input" />
                  <p class="config-note">💡 Ollama是本地AI服务，无需API密钥</p>
                </div>

                <div v-else-if="selectedProvider === 'Anthropic'" class="form-group">
                  <label>API密钥：</label>
                  <input v-model="apiConfig.anthropic.api_key" type="password" placeholder="sk-ant-..."
                    class="config-input" />
                </div>

                <div v-else-if="selectedProvider === 'Zhipu'" class="form-group">
                  <label>API密钥：</label>
                  <input v-model="apiConfig.zhipu.api_key" type="password" placeholder="..." class="config-input" />
                </div>

                <div v-else-if="selectedProvider === 'Deepseek'" class="form-group">
                  <label>API密钥：</label>
                  <input v-model="apiConfig.deepseek.api_key" type="password" placeholder="sk-..."
                    class="config-input" />
                </div>

                <div v-else-if="selectedProvider === 'Gemini'" class="form-group">
                  <label>API密钥：</label>
                  <input v-model="apiConfig.gemini.api_key" type="password" placeholder="AIza..."
                    class="config-input" />
                </div>

                <div v-else-if="selectedProvider === 'DashScope'" class="form-group">
                  <label>API密钥：</label>
                  <input v-model="apiConfig.dashscope.api_key" type="password" placeholder="sk-..."
                    class="config-input" />
                </div>

                <div v-else-if="selectedProvider === 'Doubao'" class="form-group">
                  <label>API密钥：</label>
                  <input v-model="apiConfig.doubao.api_key" type="password" placeholder="..." class="config-input" />
                </div>

                <div v-else-if="selectedProvider === 'Siliconflow'" class="form-group">
                  <label>API密钥：</label>
                  <input v-model="apiConfig.siliconflow.api_key" type="password" placeholder="sk-..."
                    class="config-input" />
                </div>

                <div v-else-if="selectedProvider === 'OpenRouter'" class="form-group">
                  <label>API密钥：</label>
                  <input v-model="apiConfig.openrouter.api_key" type="password" placeholder="sk-or-..."
                    class="config-input" />
                </div>

                <div v-else-if="selectedProvider === 'Together'" class="form-group">
                  <label>API密钥：</label>
                  <input v-model="apiConfig.together.api_key" type="password" placeholder="..." class="config-input" />
                </div>

                <div v-else-if="selectedProvider === 'Groq'" class="form-group">
                  <label>API密钥：</label>
                  <input v-model="apiConfig.groq.api_key" type="password" placeholder="gsk_..." class="config-input" />
                </div>
              </div>
            </div>
          </Transition>

          <!-- 步骤3：测试连接 -->
          <Transition name="step-slide" mode="out-in">
            <div v-if="currentStep === 3" key="step3" class="step-content">
              <div class="step-header">
                <h3>测试连接</h3>
                <p>正在测试AI服务连接...</p>
              </div>

              <div class="test-status">
                <div v-if="testing" class="testing">
                  <div class="spinner"></div>
                  <span>正在测试连接...</span>
                </div>
                <div v-else-if="testResult" class="test-result" :class="testResult.success ? 'success' : 'error'">
                  <span class="result-icon">{{ testResult.success ? '✅' : '❌' }}</span>
                  <span>{{ testResult.message }}</span>
                </div>
              </div>
            </div>
          </Transition>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

/**
 * 首次启动引导组件
 * Author: Evilek
 * Date: 2025-01-09
 */

// 组件事件
const emit = defineEmits<{
  complete: []
}>()

// 响应式数据
const currentStep = ref(1)
const selectedProvider = ref('')
const testing = ref(false)
const testResult = ref<{ success: boolean; message: string } | null>(null)

// API配置
const apiConfig = ref({
  openai: { api_key: '', base_url: 'https://api.openai.com/v1' },
  ollama: { base_url: 'http://localhost:11434' },
  anthropic: { api_key: '' },
  zhipu: { api_key: '' },
  deepseek: { api_key: '' },
  gemini: { api_key: '' },
  dashscope: { api_key: '' },
  doubao: { api_key: '' },
  siliconflow: { api_key: '' },
  openrouter: { api_key: '' },
  together: { api_key: '' },
  groq: { api_key: '' },
})

// 热门AI提供商 - 使用本地静态图标 Author: Evilek, Date: 2025-01-09
const popularProviders = [
  {
    id: 'OpenAI',
    name: 'OpenAI',
    icon: '/src/assets/static-webp/light/openai.webp',
    description: 'GPT-4, GPT-3.5等模型',
    price: '付费服务'
  },
  {
    id: 'Ollama',
    name: 'Ollama',
    icon: '/src/assets/static-webp/light/ollama.webp',
    description: '本地AI模型服务',
    price: '免费本地'
  },
  {
    id: 'Anthropic',
    name: 'Anthropic',
    icon: '/src/assets/static-webp/light/anthropic.webp',
    description: 'Claude系列模型',
    price: '付费服务'
  },
  {
    id: 'Zhipu',
    name: '智谱AI',
    icon: '/src/assets/static-webp/light/zhipu.webp',
    description: 'GLM系列模型',
    price: '付费服务'
  },
  {
    id: 'Deepseek',
    name: 'Deepseek',
    icon: '/src/assets/static-webp/light/deepseek.webp',
    description: 'Deepseek系列模型',
    price: '付费服务'
  },
  {
    id: 'Gemini',
    name: 'Google Gemini',
    icon: '/src/assets/static-webp/light/gemini.webp',
    description: 'Gemini Pro/Ultra模型',
    price: '付费服务'
  },
  {
    id: 'DashScope',
    name: '阿里云DashScope',
    icon: '/src/assets/static-webp/light/qwen.webp',
    description: '通义千问系列模型',
    price: '付费服务'
  },
  {
    id: 'Doubao',
    name: '豆包AI',
    icon: '/src/assets/static-webp/light/doubao.webp',
    description: '字节跳动AI模型',
    price: '付费服务'
  },
  {
    id: 'Siliconflow',
    name: 'SiliconFlow',
    icon: '/src/assets/static-webp/light/siliconcloud.webp',
    description: '硅基流动AI平台',
    price: '付费服务'
  },
  {
    id: 'OpenRouter',
    name: 'OpenRouter',
    icon: '/src/assets/static-webp/light/openrouter.webp',
    description: '多模型聚合平台',
    price: '付费服务'
  },
  {
    id: 'Together',
    name: 'Together AI',
    icon: '/src/assets/static-webp/light/together.webp',
    description: '开源模型托管平台',
    price: '付费服务'
  },
  {
    id: 'Groq',
    name: 'Groq',
    icon: '/src/assets/static-webp/light/groq.webp',
    description: '超高速推理平台',
    price: '付费服务'
  }
]

// 计算属性
const isConfigValid = computed(() => {
  if (!selectedProvider.value) return false

  switch (selectedProvider.value) {
    case 'OpenAI':
      return apiConfig.value.openai.api_key.trim() !== ''
    case 'Ollama':
      return apiConfig.value.ollama.base_url.trim() !== ''
    case 'Anthropic':
      return apiConfig.value.anthropic.api_key.trim() !== ''
    case 'Zhipu':
      return apiConfig.value.zhipu.api_key.trim() !== ''
    case 'Deepseek':
      return apiConfig.value.deepseek.api_key.trim() !== ''
    case 'Gemini':
      return apiConfig.value.gemini.api_key.trim() !== ''
    case 'DashScope':
      return apiConfig.value.dashscope.api_key.trim() !== ''
    case 'Doubao':
      return apiConfig.value.doubao.api_key.trim() !== ''
    case 'Siliconflow':
      return apiConfig.value.siliconflow.api_key.trim() !== ''
    case 'OpenRouter':
      return apiConfig.value.openrouter.api_key.trim() !== ''
    case 'Together':
      return apiConfig.value.together.api_key.trim() !== ''
    case 'Groq':
      return apiConfig.value.groq.api_key.trim() !== ''
    default:
      return false
  }
})

// 方法
const getProviderName = (providerId: string) => {
  const provider = popularProviders.find(p => p.id === providerId)
  return provider?.name || providerId
}

/**
 * 选择AI提供商并自动跳转到下一步
 * Author: Evilek, Date: 2025-01-09
 */
const selectProvider = (providerId: string) => {
  selectedProvider.value = providerId
  // 自动跳转到下一步
  nextStep()
}

const nextStep = () => {
  if (currentStep.value < 3) {
    currentStep.value++
    if (currentStep.value === 3) {
      testConnection()
    }
  }
}

const prevStep = () => {
  if (currentStep.value > 1) {
    currentStep.value--
    testResult.value = null
  }
}

const testConnection = async () => {
  testing.value = true
  testResult.value = null

  try {
    // 构建配置对象
    const config = {
      base: {
        language: 'Simplified Chinese',
        provider: selectedProvider.value,
        model: getDefaultModel(selectedProvider.value)
      },
      providers: apiConfig.value,
      features: {
        enable_emoji: true,
        enable_body: true,
        enable_layered_commit: true,
        use_recent_commits: true,
        enable_streaming: true
      },
      advanced: {
        temperature: 0.7,
        max_tokens: 2048,
        timeout: 60,
        retry_count: 3
      }
    }

    // 测试AI连接
    await invoke('update_ai_config', { config })
    await invoke('test_ai_connection')

    testResult.value = {
      success: true,
      message: `${getProviderName(selectedProvider.value)} 连接测试成功！`
    }
  } catch (error) {
    testResult.value = {
      success: false,
      message: `连接测试失败: ${error}`
    }
  } finally {
    testing.value = false
  }
}

const getDefaultModel = (provider: string) => {
  switch (provider) {
    case 'OpenAI': return 'gpt-3.5-turbo'
    case 'Ollama': return 'llama2'
    case 'Anthropic': return 'claude-3-sonnet-20240229'
    case 'Zhipu': return 'glm-4'
    case 'Deepseek': return 'deepseek-chat'
    case 'Gemini': return 'gemini-pro'
    case 'DashScope': return 'qwen-turbo'
    case 'Doubao': return 'doubao-pro-4k'
    case 'Siliconflow': return 'deepseek-ai/deepseek-coder-6.7b-instruct'
    case 'OpenRouter': return 'openai/gpt-3.5-turbo'
    case 'Together': return 'meta-llama/Llama-2-7b-chat-hf'
    case 'Groq': return 'llama2-70b-4096'
    default: return ''
  }
}

const completeSetup = () => {
  emit('complete')
}

onMounted(() => {
  // 组件挂载时的初始化逻辑
})
</script>

<style scoped>
.setup-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: linear-gradient(135deg, rgba(102, 126, 234, 0.9) 0%, rgba(118, 75, 162, 0.9) 100%);
  backdrop-filter: blur(8px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 2000;
  animation: fadeIn 0.3s ease-out;
}

@keyframes fadeIn {
  from {
    opacity: 0;
  }

  to {
    opacity: 1;
  }
}

.setup-modal {
  background: linear-gradient(135deg, #ffffff 0%, #f8fafc 100%);
  border-radius: 20px;
  padding: 40px;
  max-width: 650px;
  width: 90%;
  max-height: 85vh;
  overflow-y: auto;
  box-shadow:
    0 25px 80px rgba(0, 0, 0, 0.15),
    0 0 0 1px rgba(255, 255, 255, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.1);
  animation: slideUp 0.4s cubic-bezier(0.4, 0, 0.2, 1);
}

@keyframes slideUp {
  from {
    opacity: 0;
    transform: translateY(30px) scale(0.95);
  }

  to {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
}

.setup-header {
  text-align: center;
  margin-bottom: 32px;
}

.setup-header h2 {
  color: #2c3e50;
  margin-bottom: 8px;
}

.setup-header p {
  color: #7f8c8d;
  font-size: 14px;
}

/* 步骤指示器与导航按钮样式 - Author: Evilek, Date: 2025-01-09 */
.step-indicator-wrapper {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 24px;
  margin-bottom: 32px;
  padding: 0 16px;
}

/* 导航按钮 - 专业UI设计 */
.nav-btn {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 16px;
  border: 2px solid transparent;
  border-radius: 8px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  box-shadow: 0 4px 12px rgba(102, 126, 234, 0.25);
  min-width: 100px;
  justify-content: center;
  flex-shrink: 0;
}

.nav-btn:hover:not(:disabled) {
  transform: translateY(-2px);
  box-shadow: 0 8px 20px rgba(102, 126, 234, 0.35);
  background: linear-gradient(135deg, #5a6fd8 0%, #6a4190 100%);
}

.nav-btn:active:not(:disabled) {
  transform: translateY(0);
  box-shadow: 0 4px 12px rgba(102, 126, 234, 0.25);
}

.nav-btn:disabled {
  background: linear-gradient(135deg, #e2e8f0 0%, #cbd5e0 100%);
  color: #a0aec0;
  cursor: not-allowed;
  transform: none;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}

/* 完成按钮 - 成功绿色主题 */
.nav-complete {
  background: linear-gradient(135deg, #48bb78 0%, #38a169 100%);
  box-shadow: 0 4px 12px rgba(72, 187, 120, 0.25);
}

.nav-complete:hover {
  background: linear-gradient(135deg, #38a169 0%, #2f855a 100%);
  box-shadow: 0 8px 20px rgba(72, 187, 120, 0.35);
}

/* 重试按钮 - 警告橙色主题 */
.nav-retry {
  background: linear-gradient(135deg, #ed8936 0%, #dd6b20 100%);
  box-shadow: 0 4px 12px rgba(237, 137, 54, 0.25);
}

.nav-retry:hover:not(:disabled) {
  background: linear-gradient(135deg, #dd6b20 0%, #c05621 100%);
  box-shadow: 0 8px 20px rgba(237, 137, 54, 0.35);
}

.nav-icon {
  font-size: 16px;
  font-weight: bold;
}

.nav-text {
  font-size: 14px;
  font-weight: 600;
  letter-spacing: 0.025em;
}

.step-indicator {
  display: flex;
  justify-content: center;
  gap: 20px;
  flex: 1;
  max-width: 400px;
}

.step {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
}

.step-number {
  width: 36px;
  height: 36px;
  border-radius: 50%;
  background: linear-gradient(135deg, #f1f5f9 0%, #e2e8f0 100%);
  color: #64748b;
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: 700;
  font-size: 14px;
  transition: all 0.4s cubic-bezier(0.4, 0, 0.2, 1);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
  border: 2px solid transparent;
}

.step.active .step-number {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  box-shadow: 0 4px 16px rgba(102, 126, 234, 0.3);
  border-color: rgba(255, 255, 255, 0.2);
  transform: scale(1.05);
}

.step.completed .step-number {
  background: linear-gradient(135deg, #48bb78 0%, #38a169 100%);
  color: white;
  box-shadow: 0 4px 16px rgba(72, 187, 120, 0.3);
  border-color: rgba(255, 255, 255, 0.2);
}

.step-title {
  font-size: 13px;
  color: #64748b;
  text-align: center;
  font-weight: 500;
  transition: all 0.3s ease;
}

.step.active .step-title {
  color: #1e293b;
  font-weight: 600;
  transform: translateY(-1px);
}

.step.completed .step-title {
  color: #059669;
  font-weight: 600;
}

.provider-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
  /* 优化：进一步减小最小宽度，避免滚动条 */
  gap: 10px;
  /* 优化：进一步减小间距 */
  margin: 16px 0;
  /* 优化：减小上下边距 */
  max-height: 300px;
  /* 优化：限制最大高度，避免滚动条 */
  overflow: visible;
}

.provider-card {
  border: 2px solid #e2e8f0;
  border-radius: 12px;
  padding: 16px 12px;
  text-align: center;
  cursor: pointer;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  min-height: 130px;
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  background: linear-gradient(135deg, #ffffff 0%, #f8fafc 100%);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.04);
  position: relative;
  overflow: hidden;
}

.provider-card::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 3px;
  background: linear-gradient(90deg, #667eea 0%, #764ba2 100%);
  opacity: 0;
  transition: opacity 0.3s ease;
}

.provider-card:hover {
  border-color: #667eea;
  transform: translateY(-4px);
  box-shadow: 0 8px 24px rgba(102, 126, 234, 0.15);
}

.provider-card:hover::before {
  opacity: 1;
}

.provider-card.selected {
  border-color: #667eea;
  background: linear-gradient(135deg, #f0f4ff 0%, #e0e7ff 100%);
  box-shadow: 0 8px 24px rgba(102, 126, 234, 0.2);
  transform: translateY(-2px);
}

.provider-card.selected::before {
  opacity: 1;
}

.provider-icon {
  width: 32px;
  /* 优化：设置图标容器尺寸 */
  height: 32px;
  margin: 0 auto 8px auto;
  display: flex;
  align-items: center;
  justify-content: center;
}

.provider-icon img {
  width: 24px;
  /* 优化：设置图标实际尺寸 */
  height: 24px;
  object-fit: contain;
}

.provider-name {
  font-weight: 700;
  color: #1e293b;
  margin-bottom: 4px;
  font-size: 15px;
  letter-spacing: 0.025em;
}

.provider-desc {
  font-size: 12px;
  color: #64748b;
  margin-bottom: 8px;
  line-height: 1.4;
  font-weight: 400;
}

.provider-price {
  font-size: 11px;
  color: #059669;
  font-weight: 600;
  padding: 2px 8px;
  background: rgba(5, 150, 105, 0.1);
  border-radius: 12px;
  display: inline-block;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.form-group label {
  font-weight: 500;
  color: #2c3e50;
}

.config-input {
  padding: 12px;
  border: 1px solid #ddd;
  border-radius: 6px;
  font-size: 14px;
}

.config-input:focus {
  outline: none;
  border-color: #3498db;
}

.config-note {
  background: #f8f9fa;
  padding: 12px;
  border-radius: 6px;
  font-size: 13px;
  color: #7f8c8d;
  margin: 0;
}

.test-status {
  text-align: center;
  margin: 24px 0;
}

.testing {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 12px;
  color: #7f8c8d;
}

.spinner {
  width: 20px;
  height: 20px;
  border: 2px solid #ecf0f1;
  border-top: 2px solid #3498db;
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  0% {
    transform: rotate(0deg);
  }

  100% {
    transform: rotate(360deg);
  }
}

.test-result {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 12px;
  border-radius: 6px;
}

.test-result.success {
  background: #d4edda;
  color: #155724;
}

.test-result.error {
  background: #f8d7da;
  color: #721c24;
}

/* 步骤切换动画 - Author: Evilek, Date: 2025-01-09 */
.step-content-wrapper {
  position: relative;
  min-height: 300px;
  /* 优化：恢复原始高度，按钮已移到顶部 */
}

.step-slide-enter-active,
.step-slide-leave-active {
  transition: all 0.3s ease;
}

.step-slide-enter-from {
  opacity: 0;
  transform: translateX(30px);
}

.step-slide-leave-to {
  opacity: 0;
  transform: translateX(-30px);
}

.step-header {
  text-align: center;
  margin-bottom: 24px;
  /* 优化：恢复原始边距 */
}

.step-header h3 {
  color: #2c3e50;
  margin-bottom: 8px;
}

.step-header p {
  color: #7f8c8d;
  margin: 0;
}

.prev-btn,
.next-btn,
.test-btn,
.complete-btn {
  padding: 12px 24px;
  border: none;
  border-radius: 8px;
  /* 优化：更圆润的边角 */
  font-weight: 500;
  cursor: pointer;
  transition: all 0.3s;
  font-size: 14px;
  min-width: 120px;
  /* 优化：统一按钮宽度 */
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
}

.prev-btn {
  background: #f8f9fa;
  color: #6c757d;
  border: 1px solid #dee2e6;
}

.prev-btn:hover {
  background: #e9ecef;
  transform: translateY(-1px);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.next-btn,
.test-btn {
  background: linear-gradient(135deg, #3498db, #2980b9);
  /* 优化：渐变背景 */
  color: white;
  box-shadow: 0 2px 8px rgba(52, 152, 219, 0.3);
}

.next-btn:hover,
.test-btn:hover {
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(52, 152, 219, 0.4);
}

.next-btn:disabled,
.test-btn:disabled {
  background: #bdc3c7;
  cursor: not-allowed;
  transform: none;
  box-shadow: none;
}

.complete-btn {
  background: linear-gradient(135deg, #27ae60, #229954);
  /* 优化：渐变背景 */
  color: white;
  box-shadow: 0 2px 8px rgba(39, 174, 96, 0.3);
}

.complete-btn:hover {
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(39, 174, 96, 0.4);
}
</style>
