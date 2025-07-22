<template>
  <div class="diff-viewer-debug">
    <h2>差异查看器调试工具</h2>
    
    <!-- 控制面板 -->
    <div class="debug-controls">
      <div class="input-group">
        <label>文件路径:</label>
        <input v-model="filePath" placeholder="例如: backend/pom.xml" />
      </div>
      
      <div class="input-group">
        <label>差异类型:</label>
        <select v-model="diffType">
          <option value="WorkingTree">工作区 vs 暂存区</option>
          <option value="Staged">暂存区 vs HEAD</option>
          <option value="HeadToWorking">HEAD vs 工作区</option>
        </select>
      </div>
      
      <div class="input-group">
        <label>组件类型:</label>
        <select v-model="componentType">
          <option value="basic">基础DiffViewer</option>
          <option value="enhanced">增强EnhancedDiffViewer</option>
        </select>
      </div>
      
      <button @click="loadDiff" :disabled="!filePath">加载差异</button>
      <button @click="clearLogs">清空日志</button>
    </div>

    <!-- 状态显示 -->
    <div class="status-panel">
      <div class="status-item">
        <strong>加载状态:</strong> 
        <span :class="{ loading: isLoading, success: !isLoading && !error, error: error }">
          {{ isLoading ? '加载中...' : error ? '错误' : '就绪' }}
        </span>
      </div>
      
      <div class="status-item" v-if="diffData">
        <strong>数据状态:</strong>
        <ul>
          <li>文件路径: {{ diffData.file_path }}</li>
          <li>是否二进制: {{ diffData.is_binary ? '是' : '否' }}</li>
          <li>是否新文件: {{ diffData.is_new_file ? '是' : '否' }}</li>
          <li>是否删除文件: {{ diffData.is_deleted_file ? '是' : '否' }}</li>
          <li>文件语言: {{ diffData.file_language || '未知' }}</li>
          <li>Hunks数量: {{ diffData.hunks?.length || 0 }}</li>
          <li>旧文件内容长度: {{ diffData.old_content?.length || 0 }}</li>
          <li>新文件内容长度: {{ diffData.new_content?.length || 0 }}</li>
        </ul>
      </div>
    </div>

    <!-- 错误信息 -->
    <div v-if="error" class="error-panel">
      <h3>错误信息</h3>
      <pre>{{ error }}</pre>
    </div>

    <!-- 差异查看器 -->
    <div v-if="showDiffViewer" class="diff-container">
      <div class="diff-header">
        <h3>差异显示 ({{ componentType === 'basic' ? '基础版本' : '增强版本' }})</h3>
        <button @click="closeDiffViewer">关闭</button>
      </div>
      
      <!-- 基础版本 -->
      <DiffViewer
        v-if="componentType === 'basic'"
        :filePath="filePath"
        :diffType="diffType"
        @close="closeDiffViewer"
      />
      
      <!-- 增强版本 -->
      <EnhancedDiffViewer
        v-if="componentType === 'enhanced'"
        :filePath="filePath"
        :diffType="diffType"
        @close="closeDiffViewer"
      />
    </div>

    <!-- 调试日志 -->
    <div class="debug-logs">
      <h3>调试日志</h3>
      <div class="log-container" ref="logContainer">
        <div v-for="(log, index) in logs" :key="index" :class="['log-entry', log.level]">
          <span class="log-time">{{ log.time }}</span>
          <span class="log-level">{{ log.level.toUpperCase() }}</span>
          <span class="log-message">{{ log.message }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import DiffViewer from './DiffViewer.vue'
import EnhancedDiffViewer from './EnhancedDiffViewer.vue'

/**
 * 差异查看器调试工具
 * 作者：Evilek
 * 编写日期：2025-07-22
 */

interface LogEntry {
  time: string
  level: 'info' | 'warn' | 'error'
  message: string
}

// 响应式数据
const filePath = ref('backend/pom.xml')
const diffType = ref<'WorkingTree' | 'Staged' | 'HeadToWorking'>('WorkingTree')
const componentType = ref<'basic' | 'enhanced'>('basic')
const isLoading = ref(false)
const error = ref<string | null>(null)
const diffData = ref<any>(null)
const showDiffViewer = ref(false)
const logs = ref<LogEntry[]>([])
const logContainer = ref<HTMLElement>()

// 原始console方法
const originalConsole = {
  log: console.log,
  warn: console.warn,
  error: console.error
}

// 添加日志
const addLog = (level: 'info' | 'warn' | 'error', message: string) => {
  const time = new Date().toLocaleTimeString()
  logs.value.push({ time, level, message })
  
  // 自动滚动到底部
  setTimeout(() => {
    if (logContainer.value) {
      logContainer.value.scrollTop = logContainer.value.scrollHeight
    }
  }, 100)
}

// 拦截console输出
const interceptConsole = () => {
  console.log = (...args) => {
    const message = args.map(arg => 
      typeof arg === 'object' ? JSON.stringify(arg, null, 2) : String(arg)
    ).join(' ')
    addLog('info', message)
    originalConsole.log(...args)
  }
  
  console.warn = (...args) => {
    const message = args.map(arg => 
      typeof arg === 'object' ? JSON.stringify(arg, null, 2) : String(arg)
    ).join(' ')
    addLog('warn', message)
    originalConsole.warn(...args)
  }
  
  console.error = (...args) => {
    const message = args.map(arg => 
      typeof arg === 'object' ? JSON.stringify(arg, null, 2) : String(arg)
    ).join(' ')
    addLog('error', message)
    originalConsole.error(...args)
  }
}

// 恢复console
const restoreConsole = () => {
  console.log = originalConsole.log
  console.warn = originalConsole.warn
  console.error = originalConsole.error
}

// 加载差异数据
const loadDiff = async () => {
  try {
    isLoading.value = true
    error.value = null
    diffData.value = null
    
    addLog('info', `开始加载差异数据: ${filePath.value} (${diffType.value})`)
    
    const result = await invoke('get_file_diff', {
      request: {
        file_path: filePath.value,
        diff_type: diffType.value
      }
    })
    
    diffData.value = result
    showDiffViewer.value = true
    
    addLog('info', '差异数据加载成功')
    addLog('info', `数据概览: ${JSON.stringify({
      hunks: result?.hunks?.length || 0,
      is_binary: result?.is_binary,
      old_content_length: result?.old_content?.length || 0,
      new_content_length: result?.new_content?.length || 0
    }, null, 2)}`)
    
  } catch (err) {
    error.value = err instanceof Error ? err.message : '未知错误'
    addLog('error', `加载失败: ${error.value}`)
  } finally {
    isLoading.value = false
  }
}

// 关闭差异查看器
const closeDiffViewer = () => {
  showDiffViewer.value = false
  addLog('info', '差异查看器已关闭')
}

// 清空日志
const clearLogs = () => {
  logs.value = []
}

// 生命周期
onMounted(() => {
  interceptConsole()
  addLog('info', '调试工具已启动')
})

onUnmounted(() => {
  restoreConsole()
})
</script>

<style scoped>
.diff-viewer-debug {
  padding: 20px;
  max-width: 1400px;
  margin: 0 auto;
}

.debug-controls {
  display: flex;
  gap: 16px;
  align-items: end;
  margin-bottom: 20px;
  padding: 16px;
  background: #f6f8fa;
  border-radius: 8px;
  flex-wrap: wrap;
}

.input-group {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.input-group label {
  font-weight: 500;
  font-size: 14px;
}

.input-group input,
.input-group select {
  padding: 8px 12px;
  border: 1px solid #d1d5da;
  border-radius: 4px;
  font-size: 14px;
  min-width: 200px;
}

button {
  padding: 8px 16px;
  background: #0366d6;
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 14px;
  transition: background 0.2s;
}

button:hover:not(:disabled) {
  background: #0256cc;
}

button:disabled {
  background: #94a3b8;
  cursor: not-allowed;
}

.status-panel {
  margin-bottom: 20px;
  padding: 16px;
  border: 1px solid #d1d5da;
  border-radius: 8px;
}

.status-item {
  margin-bottom: 12px;
}

.status-item:last-child {
  margin-bottom: 0;
}

.status-item ul {
  margin: 8px 0 0 20px;
  font-size: 14px;
}

.status-item li {
  margin-bottom: 4px;
}

.loading {
  color: #0366d6;
}

.success {
  color: #28a745;
}

.error {
  color: #d73a49;
}

.error-panel {
  margin-bottom: 20px;
  padding: 16px;
  background: #ffeaea;
  border: 1px solid #d73a49;
  border-radius: 8px;
}

.error-panel pre {
  margin: 8px 0 0 0;
  font-size: 14px;
  white-space: pre-wrap;
}

.diff-container {
  margin-bottom: 20px;
  border: 1px solid #d1d5da;
  border-radius: 8px;
  overflow: hidden;
}

.diff-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  background: #f6f8fa;
  border-bottom: 1px solid #d1d5da;
}

.diff-header h3 {
  margin: 0;
  font-size: 16px;
}

.debug-logs {
  margin-top: 20px;
}

.log-container {
  max-height: 400px;
  overflow-y: auto;
  border: 1px solid #d1d5da;
  border-radius: 4px;
  background: #f8f9fa;
  font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
  font-size: 12px;
}

.log-entry {
  padding: 4px 8px;
  border-bottom: 1px solid #e1e4e8;
  display: flex;
  gap: 8px;
}

.log-entry:last-child {
  border-bottom: none;
}

.log-entry.info {
  background: #ffffff;
}

.log-entry.warn {
  background: #fff8e1;
}

.log-entry.error {
  background: #ffeaea;
}

.log-time {
  color: #586069;
  min-width: 80px;
}

.log-level {
  font-weight: bold;
  min-width: 50px;
}

.log-level.INFO {
  color: #0366d6;
}

.log-level.WARN {
  color: #f66a0a;
}

.log-level.ERROR {
  color: #d73a49;
}

.log-message {
  flex: 1;
  white-space: pre-wrap;
}

/* 响应式设计 */
@media (max-width: 768px) {
  .debug-controls {
    flex-direction: column;
    align-items: stretch;
  }
  
  .input-group input,
  .input-group select {
    min-width: auto;
  }
}
</style>
