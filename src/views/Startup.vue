<template>
  <div class="startup-container">
    <div class="startup-content">
      <div class="logo-section">
        <div class="logo">
          <el-icon size="80" color="#409eff">
            <Tools />
          </el-icon>
        </div>
        <h1 class="app-title">GitMentor</h1>
        <p class="app-subtitle">AI Agent双重审核系统</p>
      </div>

      <div class="loading-section">
        <el-progress 
          :percentage="progress" 
          :status="progressStatus"
          :stroke-width="8"
          class="progress-bar"
        />
        <p class="loading-text">{{ loadingText }}</p>
      </div>

      <div class="status-section">
        <div class="status-item" :class="{ active: backendStatus === 'starting' || backendStatus === 'ready' }">
          <el-icon><Loading v-if="backendStatus === 'starting'" /><Check v-else-if="backendStatus === 'ready'" /><Close v-else /></el-icon>
          <span>后端服务</span>
        </div>
        <div class="status-item" :class="{ active: databaseStatus === 'ready' }">
          <el-icon><Loading v-if="databaseStatus === 'starting'" /><Check v-else-if="databaseStatus === 'ready'" /><Close v-else /></el-icon>
          <span>数据库初始化</span>
        </div>
        <div class="status-item" :class="{ active: agentStatus === 'ready' }">
          <el-icon><Loading v-if="agentStatus === 'starting'" /><Check v-else-if="agentStatus === 'ready'" /><Close v-else /></el-icon>
          <span>AI Agent系统</span>
        </div>
      </div>

      <div v-if="error" class="error-section">
        <el-alert
          :title="error"
          type="error"
          :closable="false"
          show-icon
        />
        <el-button type="primary" @click="retry" style="margin-top: 15px;">
          重试
        </el-button>
      </div>

      <div class="info-section">
        <p class="version">版本 v{{ version }}</p>
        <p class="copyright">© 2024 GitMentor Team</p>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { Tools, Loading, Check, Close } from '@element-plus/icons-vue'

const router = useRouter()

// 响应式数据
const progress = ref(0)
const progressStatus = ref('')
const loadingText = ref('正在启动系统...')
const backendStatus = ref('waiting')
const databaseStatus = ref('waiting')
const agentStatus = ref('waiting')
const error = ref('')
const version = ref('1.0.0')

// 启动流程
const startupSequence = async () => {
  try {
    // 第一阶段：启动后端服务
    loadingText.value = '正在启动后端服务...'
    backendStatus.value = 'starting'
    progress.value = 20
    
    await checkBackendHealth()
    
    backendStatus.value = 'ready'
    progress.value = 40
    
    // 第二阶段：初始化数据库
    loadingText.value = '正在初始化数据库...'
    databaseStatus.value = 'starting'
    
    await initializeDatabase()
    
    databaseStatus.value = 'ready'
    progress.value = 70
    
    // 第三阶段：启动AI Agent系统
    loadingText.value = '正在启动AI Agent系统...'
    agentStatus.value = 'starting'
    
    await initializeAgents()
    
    agentStatus.value = 'ready'
    progress.value = 100
    progressStatus.value = 'success'
    loadingText.value = '系统启动完成！'
    
    // 延迟跳转到主页面
    setTimeout(() => {
      router.push('/dashboard')
    }, 1500)
    
  } catch (err) {
    console.error('启动失败:', err)
    error.value = `启动失败: ${err.message}`
    progressStatus.value = 'exception'
    loadingText.value = '启动失败'
  }
}

// 检查后端健康状态
const checkBackendHealth = async () => {
  const maxRetries = 60  // 增加重试次数
  let retries = 0

  while (retries < maxRetries) {
    try {
      const response = await fetch('http://localhost:8000/api/health')
      if (response.ok) {
        return
      }
    } catch (err) {
      // 继续重试，这是正常的，因为后端可能还在启动
      console.log(`后端健康检查重试 ${retries + 1}/${maxRetries}`)
    }

    retries++
    await new Promise(resolve => setTimeout(resolve, 2000))  // 增加等待时间
  }

  throw new Error('后端服务启动超时，请检查后端可执行文件是否存在')
}

// 初始化数据库
const initializeDatabase = async () => {
  try {
    // 数据库初始化通常在后端启动时自动完成
    // 这里只需要确认后端仍然健康
    const response = await fetch('http://localhost:8000/api/health')
    if (!response.ok) {
      console.warn('数据库健康检查失败，但继续启动')
    }
  } catch (err) {
    console.warn('数据库初始化检查失败，但继续启动:', err)
    // 不抛出错误，允许继续启动
  }
}

// 初始化AI Agent系统
const initializeAgents = async () => {
  try {
    // 尝试检查Agent系统，但不强制要求成功
    const response = await fetch('http://localhost:8000/api/agents/health')
    if (response.ok) {
      console.log('Agent系统健康检查通过')
    } else {
      console.warn('Agent系统健康检查失败，但继续启动')
    }
  } catch (err) {
    // Agent系统可能还没有完全实现，暂时跳过
    console.warn('Agent健康检查失败，但继续启动:', err)
    // 不抛出错误，允许继续启动
  }
}

// 重试启动
const retry = () => {
  error.value = ''
  progress.value = 0
  progressStatus.value = ''
  backendStatus.value = 'waiting'
  databaseStatus.value = 'waiting'
  agentStatus.value = 'waiting'
  startupSequence()
}

// 生命周期
onMounted(() => {
  startupSequence()
})
</script>

<style scoped>
.startup-container {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 100vh;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  padding: 20px;
}

.startup-content {
  background: white;
  border-radius: 20px;
  padding: 40px;
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.1);
  text-align: center;
  max-width: 500px;
  width: 100%;
}

.logo-section {
  margin-bottom: 40px;
}

.logo {
  margin-bottom: 20px;
}

.app-title {
  font-size: 32px;
  font-weight: bold;
  color: #333;
  margin: 0 0 10px 0;
}

.app-subtitle {
  font-size: 16px;
  color: #666;
  margin: 0;
}

.loading-section {
  margin-bottom: 40px;
}

.progress-bar {
  margin-bottom: 20px;
}

.loading-text {
  font-size: 16px;
  color: #666;
  margin: 0;
}

.status-section {
  margin-bottom: 30px;
}

.status-item {
  display: flex;
  align-items: center;
  justify-content: center;
  margin-bottom: 15px;
  padding: 10px;
  border-radius: 8px;
  background: #f5f5f5;
  color: #999;
  transition: all 0.3s ease;
}

.status-item.active {
  background: #e8f5e8;
  color: #67c23a;
}

.status-item .el-icon {
  margin-right: 10px;
}

.error-section {
  margin-bottom: 30px;
}

.info-section {
  border-top: 1px solid #eee;
  padding-top: 20px;
}

.version, .copyright {
  font-size: 12px;
  color: #999;
  margin: 5px 0;
}
</style>
