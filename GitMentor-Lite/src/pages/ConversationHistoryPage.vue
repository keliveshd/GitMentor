<template>
  <div class="conversation-history-page">
    <div class="page-header">
      <h1>对话记录</h1>
      <div class="header-controls">
        <div class="repository-filter">
          <label for="repo-select">仓库筛选：</label>
          <select id="repo-select" v-model="selectedRepository" @change="onRepositoryChange" :disabled="loading">
            <option value="all">全部仓库</option>
            <option v-for="path in repositoryPaths" :key="path" :value="path">
              {{ getRepositoryDisplayName(path) }}
            </option>
          </select>
        </div>
        <div class="header-actions">
          <button @click="refreshHistory" class="refresh-btn" :disabled="loading">
            刷新
          </button>
          <button @click="clearHistory" class="clear-btn" :disabled="loading">
            清空记录
          </button>
        </div>
      </div>
    </div>

    <div class="page-content">
      <!-- 统计信息 -->
      <div class="stats-section">
        <div class="stat-card">
          <div class="stat-value">{{ conversationHistory.length }}</div>
          <div class="stat-label">总对话数</div>
        </div>
        <div class="stat-card">
          <div class="stat-value">{{ successCount }}</div>
          <div class="stat-label">成功次数</div>
        </div>
        <div class="stat-card">
          <div class="stat-value">{{ failureCount }}</div>
          <div class="stat-label">失败次数</div>
        </div>
        <div class="stat-card">
          <div class="stat-value">{{ averageTime }}ms</div>
          <div class="stat-label">平均响应时间</div>
        </div>
      </div>

      <!-- 对话记录列表 -->
      <div class="conversation-list">
        <div v-if="loading" class="loading-state">
          <div class="loading-spinner"></div>
          <p>加载中...</p>
        </div>

        <div v-else-if="conversationHistory.length === 0" class="empty-state">
          <div class="empty-icon">💬</div>
          <h3>暂无对话记录</h3>
          <p>当您使用AI生成提交消息时，对话记录将显示在这里</p>
        </div>

        <div v-else class="conversation-items">
          <!-- 分层提交会话 -->
          <div v-for="session in groupedConversations.sessions" :key="session.sessionId" class="session-group">
            <div class="session-header" @click="toggleSessionExpanded(session.sessionId)">
              <div class="session-info">
                <div class="session-title">
                  <span class="session-icon">🔄</span>
                  <span class="session-label">分层提交会话</span>
                  <span class="session-id">{{ session.sessionId.slice(0, 8) }}...</span>
                </div>
                <div class="session-meta">
                  <span class="session-time">{{ formatTime(session.timestamp) }}</span>
                  <span class="session-repository">{{ getRepositoryDisplayName(session.repository) }}</span>
                  <span class="session-stats">{{ session.fileCount }} 文件 · {{ session.totalProcessingTime }}ms</span>
                </div>
              </div>
              <div class="session-toggle">
                <span class="expand-icon" :class="{ expanded: expandedSessions.has(session.sessionId) }">🔽</span>
              </div>
            </div>

            <!-- 会话详情 -->
            <div v-if="expandedSessions.has(session.sessionId)" class="session-details">
              <div v-for="record in session.records" :key="record.id" class="session-step">
                <div class="step-header">
                  <span class="step-icon">{{ getStepIcon(record.step_info?.step_type || '') }}</span>
                  <span class="step-name">{{ getStepTypeName(record.step_info?.step_type || '') }}</span>
                  <span v-if="record.step_info?.file_path" class="step-file">{{ record.step_info.file_path }}</span>
                  <span class="step-time">{{ record.processing_time_ms }}ms</span>
                  <span class="step-status" :class="record.success ? 'success' : 'error'">
                    {{ record.success ? '成功' : '失败' }}
                  </span>
                </div>

                <!-- 用户提示词显示 -->
                <div class="step-content">
                  <!-- 用户提示词 -->
                  <div v-if="record.request.messages && record.request.messages.length > 0" class="step-messages">
                    <h6 style="margin: 10px 0 5px 0; color: #333; font-size: 13px;">💬 对话内容：</h6>
                    <div v-for="(message, index) in record.request.messages" :key="index" class="step-message"
                      :class="message.role">
                      <div class="message-role">{{ message.role === 'system' ? '🤖 系统提示' : '👤 用户提示' }}</div>
                      <div class="message-content">{{ message.content }}</div>
                    </div>
                  </div>

                  <!-- AI响应 -->
                  <div v-if="record.response" class="step-response">
                    <h6 style="margin: 10px 0 5px 0; color: #333; font-size: 13px;">🤖 AI响应：</h6>
                    <div class="response-content">{{ record.response.content }}</div>
                  </div>
                </div>

                <div v-if="record.error_message" class="step-error">
                  <strong>错误:</strong> {{ record.error_message }}
                </div>
              </div>
            </div>
          </div>

          <!-- 单独的对话记录 -->
          <div v-for="conversation in groupedConversations.singleRecords" :key="conversation.id"
            class="conversation-item" :class="{ error: !conversation.success }">
            <div class="conversation-header">
              <div class="conversation-info">
                <span class="conversation-time">{{ formatTime(conversation.timestamp) }}</span>
                <span class="conversation-template">模板: {{ conversation.template_id }}</span>
                <span v-if="conversation.repository_path" class="conversation-repository">
                  仓库: {{ getRepositoryDisplayName(conversation.repository_path) }}
                </span>
                <span class="conversation-status" :class="conversation.success ? 'success' : 'error'">
                  {{ conversation.success ? '成功' : '失败' }}
                </span>
              </div>
              <div class="conversation-meta">
                <span class="processing-time">{{ conversation.processing_time_ms }}ms</span>
                <button @click="toggleExpanded(conversation.id)" class="expand-btn">
                  {{ expandedItems.has(conversation.id) ? '🔼' : '🔽' }}
                </button>
              </div>
            </div>

            <div v-if="expandedItems.has(conversation.id)" class="conversation-details">
              <!-- 请求信息 -->
              <div class="detail-section">
                <h4>📤 请求信息</h4>
                <div class="request-info">
                  <div class="info-item">
                    <label>模型:</label>
                    <span>{{ conversation.request.model }}</span>
                  </div>
                  <div class="info-item">
                    <label>温度值:</label>
                    <span>{{ conversation.request.temperature }}</span>
                  </div>
                  <div class="info-item">
                    <label>最大Token:</label>
                    <span>{{ conversation.request.max_tokens }}</span>
                  </div>
                </div>
                <!-- 用户提示词和系统提示 -->
                <div class="messages-section">
                  <h5 style="margin: 15px 0 10px 0; color: #333; font-size: 14px; font-weight: 600;">� 对话内容</h5>

                  <template v-if="conversation.request.messages && conversation.request.messages.length > 0">
                    <div v-for="(message, index) in conversation.request.messages" :key="index" class="message-item"
                      :class="message.role">
                      <div class="message-role">{{ message.role === 'system' ? '🤖 系统提示' : '👤 用户提示' }}</div>
                      <div class="message-content">{{ message.content }}</div>
                    </div>
                  </template>

                  <template v-else>
                    <div class="no-messages"
                      style="padding: 15px; background: #f8f9fa; border-radius: 6px; color: #666; font-style: italic;">
                      📝 此对话记录没有保存用户提示词信息
                    </div>
                  </template>
                </div>
              </div>

              <!-- 响应信息 -->
              <div v-if="conversation.success" class="detail-section">
                <h4>📥 响应信息</h4>
                <div class="response-info">
                  <div class="info-item">
                    <label>模型:</label>
                    <span>{{ conversation.response?.model }}</span>
                  </div>
                  <div class="info-item" v-if="conversation.response?.usage">
                    <label>Token使用:</label>
                    <span>{{ conversation.response.usage.total_tokens }}
                      (输入: {{ conversation.response.usage.prompt_tokens }},
                      输出: {{ conversation.response.usage.completion_tokens }})</span>
                  </div>
                  <div class="info-item" v-if="conversation.response?.finish_reason">
                    <label>完成原因:</label>
                    <span>{{ conversation.response.finish_reason }}</span>
                  </div>
                </div>
                <div class="response-content">
                  <label>生成内容:</label>
                  <div class="content-text">{{ conversation.response?.content }}</div>
                </div>
              </div>

              <!-- 错误信息 -->
              <div v-if="!conversation.success && conversation.error_message" class="detail-section error">
                <h4>❌ 错误信息</h4>
                <div class="error-content">{{ conversation.error_message }}</div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

/**
 * 对话记录页面组件
 * 作者：Evilek
 * 编写日期：2025-01-30
 */

// 接口定义
interface ConversationRecord {
  id: string
  timestamp: string
  template_id: string
  repository_path?: string
  session_id?: string // 新增：会话ID
  session_type?: string // 新增：会话类型
  step_info?: StepInfo // 新增：步骤信息
  request: {
    messages: Array<{ role: string; content: string }>
    model: string
    temperature?: number
    max_tokens?: number
    stream?: boolean
  }
  response?: {
    content: string
    model: string
    usage?: {
      prompt_tokens: number
      completion_tokens: number
      total_tokens: number
    }
    finish_reason?: string
  }
  processing_time_ms: number
  success: boolean
  error_message?: string
}

interface StepInfo {
  step_type: string // "file_analysis" | "final_summary"
  step_index?: number
  total_steps?: number
  file_path?: string
  description?: string
}

// 响应式数据
const conversationHistory = ref<ConversationRecord[]>([])
const loading = ref(false)
const expandedItems = ref<Set<string>>(new Set())
const expandedSessions = ref<Set<string>>(new Set())
const repositoryPaths = ref<string[]>([])
const selectedRepository = ref<string>('all')

// 会话分组接口
interface SessionGroup {
  sessionId: string
  sessionType: string
  timestamp: string
  repository: string
  records: ConversationRecord[]
  totalProcessingTime: number
  fileCount: number
}

// 计算属性
const successCount = computed(() =>
  conversationHistory.value.filter(c => c.success).length
)

const failureCount = computed(() =>
  conversationHistory.value.filter(c => !c.success).length
)

const averageTime = computed(() => {
  if (conversationHistory.value.length === 0) return 0
  const total = conversationHistory.value.reduce((sum, c) => sum + c.processing_time_ms, 0)
  return Math.round(total / conversationHistory.value.length)
})

// 方法
const formatTime = (timestamp: string) => {
  return new Date(timestamp).toLocaleString('zh-CN')
}

const toggleExpanded = (id: string) => {
  if (expandedItems.value.has(id)) {
    expandedItems.value.delete(id)
  } else {
    expandedItems.value.add(id)
  }
}

// 切换会话展开状态
const toggleSessionExpanded = (sessionId: string) => {
  if (expandedSessions.value.has(sessionId)) {
    expandedSessions.value.delete(sessionId)
  } else {
    expandedSessions.value.add(sessionId)
  }
}

// 获取步骤类型的显示名称
const getStepTypeName = (stepType: string) => {
  switch (stepType) {
    case 'file_analysis':
      return '文件分析'
    case 'final_summary':
      return '最终总结'
    default:
      return stepType
  }
}

// 获取步骤图标
const getStepIcon = (stepType: string) => {
  switch (stepType) {
    case 'file_analysis':
      return '📄'
    case 'final_summary':
      return '📝'
    default:
      return '🔧'
  }
}

const refreshHistory = async () => {
  await loadConversationHistory()
}

const reloadPage = () => {
  window.location.reload()
}

const clearHistory = async () => {
  if (!confirm('确定要清空所有对话记录吗？此操作不可恢复。')) return

  try {
    loading.value = true
    await invoke('clear_conversation_history')
    conversationHistory.value = []
  } catch (error) {
    console.error('清空对话记录失败:', error)
    alert('清空对话记录失败: ' + error)
  } finally {
    loading.value = false
  }
}

const loadConversationHistory = async () => {
  try {
    loading.value = true
    let history: ConversationRecord[]

    if (selectedRepository.value === 'all') {
      history = await invoke('get_conversation_history') as ConversationRecord[]
    } else {
      history = await invoke('get_conversation_history_by_repository', {
        repositoryPath: selectedRepository.value
      }) as ConversationRecord[]
    }

    conversationHistory.value = history.sort((a, b) =>
      new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime()
    )
  } catch (error) {
    console.error('加载对话记录失败:', error)
    conversationHistory.value = []
  } finally {
    loading.value = false
  }
}

// 加载仓库路径列表
const loadRepositoryPaths = async () => {
  try {
    const paths = await invoke('get_repository_paths') as string[]
    repositoryPaths.value = paths
  } catch (error) {
    console.error('加载仓库路径失败:', error)
    repositoryPaths.value = []
  }
}

// 获取仓库显示名称
const getRepositoryDisplayName = (path: string) => {
  const parts = path.split(/[/\\]/)
  return parts[parts.length - 1] || path
}

// 仓库选择变更处理
const onRepositoryChange = () => {
  loadConversationHistory()
}

// 计算属性：分组后的对话记录
const groupedConversations = computed(() => {
  const sessions = new Map<string, SessionGroup>()
  const singleRecords: ConversationRecord[] = []

  conversationHistory.value.forEach(record => {
    if (record.session_id && record.session_type === 'layered') {
      // 分层提交记录，按会话分组
      if (!sessions.has(record.session_id)) {
        sessions.set(record.session_id, {
          sessionId: record.session_id,
          sessionType: record.session_type,
          timestamp: record.timestamp,
          repository: record.repository_path || '未知仓库',
          records: [],
          totalProcessingTime: 0,
          fileCount: 0
        })
      }

      const session = sessions.get(record.session_id)!
      session.records.push(record)
      session.totalProcessingTime += record.processing_time_ms

      // 统计文件数量
      if (record.step_info?.step_type === 'file_analysis') {
        session.fileCount++
      }
    } else {
      // 单独的对话记录
      singleRecords.push(record)
    }
  })

  return {
    sessions: Array.from(sessions.values()).sort((a, b) =>
      new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime()
    ),
    singleRecords: singleRecords.sort((a, b) =>
      new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime()
    )
  }
})

// 生命周期
onMounted(async () => {
  await loadRepositoryPaths()
  await loadConversationHistory()
})
</script>

<style scoped>
.conversation-history-page {
  height: 100vh;
  background: #f5f5f5;
  display: flex;
  flex-direction: column;
}

.page-header {
  background: white;
  padding: 20px 30px;
  border-bottom: 1px solid #e0e0e0;
}

.header-controls {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 16px;
}

.repository-filter {
  display: flex;
  align-items: center;
  gap: 8px;
}

.repository-filter label {
  font-weight: 500;
  color: #4a5568;
  font-size: 14px;
}

.repository-filter select {
  padding: 6px 12px;
  border: 1px solid #d1d5db;
  border-radius: 6px;
  background: white;
  font-size: 14px;
  min-width: 200px;
}

.page-header h1 {
  margin: 0;
  font-size: 24px;
  color: #333;
}

.header-actions {
  display: flex;
  gap: 10px;
}

.refresh-btn,
.clear-btn {
  padding: 8px 16px;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-size: 14px;
  transition: background 0.2s;
}

.refresh-btn {
  background: #2196f3;
  color: white;
}

.refresh-btn:hover:not(:disabled) {
  background: #1976d2;
}

.clear-btn {
  background: #f44336;
  color: white;
}

.clear-btn:hover:not(:disabled) {
  background: #d32f2f;
}

.refresh-btn:disabled,
.clear-btn:disabled {
  background: #ccc;
  cursor: not-allowed;
}

.page-content {
  flex: 1;
  padding: 20px 30px;
  overflow-y: auto;
}

/* 统计信息样式 */
.stats-section {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 20px;
  margin-bottom: 30px;
}

.stat-card {
  background: white;
  padding: 20px;
  border-radius: 8px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  text-align: center;
}

.stat-value {
  font-size: 32px;
  font-weight: bold;
  color: #2196f3;
  margin-bottom: 8px;
}

.stat-label {
  font-size: 14px;
  color: #666;
}

/* 对话记录列表样式 */
.conversation-list {
  background: white;
  border-radius: 8px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  overflow: hidden;
}

.loading-state,
.empty-state {
  text-align: center;
  padding: 60px 20px;
  color: #666;
}

.loading-spinner {
  width: 40px;
  height: 40px;
  border: 4px solid #f3f3f3;
  border-top: 4px solid #2196f3;
  border-radius: 50%;
  animation: spin 1s linear infinite;
  margin: 0 auto 20px;
}

@keyframes spin {
  0% {
    transform: rotate(0deg);
  }

  100% {
    transform: rotate(360deg);
  }
}

.empty-icon {
  font-size: 48px;
  margin-bottom: 20px;
}

.empty-state h3 {
  margin: 0 0 10px 0;
  font-size: 18px;
}

.empty-state p {
  margin: 0;
  font-size: 14px;
}

.conversation-items {
  max-height: 600px;
  overflow-y: auto;
}

.conversation-item {
  border-bottom: 1px solid #e0e0e0;
  transition: background 0.2s;
}

.conversation-item:hover {
  background: #f8f9fa;
}

.conversation-item.error {
  border-left: 4px solid #f44336;
}

.conversation-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 15px 20px;
  cursor: pointer;
}

.conversation-info {
  display: flex;
  align-items: center;
  gap: 15px;
}

.conversation-time {
  font-size: 14px;
  color: #666;
}

.conversation-template {
  font-size: 14px;
  background: #e3f2fd;
  color: #1976d2;
  padding: 2px 8px;
  border-radius: 12px;
}

.conversation-repository {
  font-size: 14px;
  background: #dbeafe;
  color: #1e40af;
  padding: 2px 8px;
  border-radius: 12px;
}

/* 分层提交会话样式 */
.session-group {
  margin-bottom: 20px;
  border: 1px solid #e5e7eb;
  border-radius: 8px;
  overflow: hidden;
  background: white;
}

.session-header {
  padding: 16px;
  background: #f8fafc;
  border-bottom: 1px solid #e5e7eb;
  cursor: pointer;
  transition: background-color 0.2s;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.session-header:hover {
  background: #f1f5f9;
}

.session-title {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 4px;
}

.session-icon {
  font-size: 16px;
}

.session-label {
  font-weight: 600;
  color: #374151;
  font-size: 16px;
}

.session-id {
  background: #e5e7eb;
  color: #6b7280;
  padding: 2px 6px;
  border-radius: 4px;
  font-size: 12px;
  font-family: monospace;
}

.session-meta {
  display: flex;
  align-items: center;
  gap: 12px;
  font-size: 13px;
  color: #6b7280;
}

.session-time {
  font-weight: 500;
}

.session-repository {
  background: #dbeafe;
  color: #1e40af;
  padding: 2px 6px;
  border-radius: 4px;
}

.session-stats {
  background: #f3f4f6;
  padding: 2px 6px;
  border-radius: 4px;
}

.session-toggle {
  display: flex;
  align-items: center;
}

.expand-icon {
  font-size: 14px;
  color: #6b7280;
  transition: transform 0.2s;
}

.expand-icon.expanded {
  transform: rotate(180deg);
}

.session-details {
  padding: 0;
}

.session-step {
  border-bottom: 1px solid #f3f4f6;
  padding: 12px 16px;
}

.session-step:last-child {
  border-bottom: none;
}

.step-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
  flex-wrap: wrap;
}

.step-icon {
  font-size: 14px;
}

.step-name {
  font-weight: 500;
  color: #374151;
}

.step-file {
  background: #f3f4f6;
  color: #6b7280;
  padding: 2px 6px;
  border-radius: 4px;
  font-size: 12px;
  font-family: monospace;
}

.step-time {
  color: #9ca3af;
  font-size: 12px;
}

.step-status {
  padding: 2px 6px;
  border-radius: 4px;
  font-size: 12px;
  font-weight: 500;
}

.step-status.success {
  background: #d1fae5;
  color: #065f46;
}

.step-status.error {
  background: #fee2e2;
  color: #991b1b;
}

.step-content {
  margin-left: 22px;
}

.response-content {
  background: #f9fafb;
  border: 1px solid #e5e7eb;
  border-radius: 6px;
  padding: 12px;
  font-size: 14px;
  line-height: 1.5;
  color: #374151;
  white-space: pre-wrap;
}

.step-error {
  margin-left: 22px;
  background: #fef2f2;
  border: 1px solid #fecaca;
  border-radius: 6px;
  padding: 8px 12px;
  color: #991b1b;
  font-size: 13px;
}

.conversation-status.success {
  color: #4caf50;
}

.conversation-status.error {
  color: #f44336;
}

.conversation-meta {
  display: flex;
  align-items: center;
  gap: 10px;
}

.processing-time {
  font-size: 12px;
  color: #999;
}

.expand-btn {
  background: none;
  border: none;
  cursor: pointer;
  font-size: 16px;
  padding: 4px;
}

.conversation-details {
  padding: 0 20px 20px;
  border-top: 1px solid #f0f0f0;
}

.detail-section {
  margin-bottom: 20px;
}

.detail-section h4 {
  margin: 0 0 10px 0;
  font-size: 16px;
  color: #333;
}

.detail-section.error h4 {
  color: #f44336;
}

.request-info,
.response-info {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 10px;
  margin-bottom: 15px;
}

.info-item {
  display: flex;
  gap: 8px;
}

.info-item label {
  font-weight: 500;
  color: #666;
  min-width: 80px;
}

.messages-section {
  margin-top: 15px;
}

.message-item {
  margin-bottom: 15px;
  padding: 10px;
  border-radius: 6px;
  border-left: 4px solid #ddd;
}

.message-item.system {
  background: #f8f9fa;
  border-left-color: #2196f3;
}

.message-item.user {
  background: #e8f5e8;
  border-left-color: #4caf50;
}

.message-role {
  font-weight: 500;
  margin-bottom: 5px;
  font-size: 14px;
}

.message-content {
  white-space: pre-wrap;
  line-height: 1.4;
  font-size: 14px;
}

/* 分层会话中的消息样式 */
.step-messages {
  margin: 10px 0;
}

.step-message {
  margin-bottom: 10px;
  padding: 8px;
  border-radius: 4px;
  border-left: 3px solid #ddd;
  font-size: 13px;
}

.step-message.system {
  background: #f8f9fa;
  border-left-color: #2196f3;
}

.step-message.user {
  background: #e8f5e8;
  border-left-color: #4caf50;
}

.step-message .message-role {
  font-weight: 500;
  margin-bottom: 4px;
  font-size: 12px;
}

.step-message .message-content {
  font-size: 12px;
  line-height: 1.3;
}

.step-response {
  margin: 10px 0;
}

.response-content {
  margin-top: 15px;
}

.response-content label {
  font-weight: 500;
  color: #666;
  display: block;
  margin-bottom: 8px;
}

.content-text {
  background: #f8f9fa;
  padding: 12px;
  border-radius: 6px;
  white-space: pre-wrap;
  line-height: 1.4;
  font-size: 14px;
  border-left: 4px solid #2196f3;
}

.error-content {
  background: #ffebee;
  color: #c62828;
  padding: 12px;
  border-radius: 6px;
  border-left: 4px solid #f44336;
  font-size: 14px;
}
</style>
