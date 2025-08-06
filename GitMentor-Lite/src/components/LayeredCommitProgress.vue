<template>
  <div v-if="visible" class="layered-progress-overlay">
    <div class="layered-progress-modal">
      <div class="progress-header">
        <h3>智能分层提交</h3>
        <div class="session-info">
          <span class="session-id">会话: {{ sessionId.slice(0, 8) }}...</span>
        </div>
      </div>

      <div class="progress-content">
        <!-- 总体进度 -->
        <div class="overall-progress">
          <div class="progress-label">
            <span>总体进度</span>
            <span class="progress-fraction">{{ currentStep }}/{{ totalSteps }}</span>
          </div>
          <div class="progress-bar">
            <div class="progress-fill" :style="{ width: overallProgressPercent + '%' }"></div>
          </div>
        </div>

        <!-- 当前状态 -->
        <div class="current-status">
          <div class="status-icon">
            <div v-if="isProcessing" class="spinner"></div>
            <div v-else class="check-icon">✓</div>
          </div>
          <div class="status-text">{{ currentStatus }}</div>
        </div>



        <!-- 已完成的文件摘要 -->
        <div v-if="fileSummaries.length > 0" class="completed-files">
          <div class="completed-header">
            <span>已完成分析 ({{ fileSummaries.length }})</span>
            <button @click="toggleSummaries" class="toggle-btn">
              {{ showSummaries ? '收起' : '展开' }}
            </button>
          </div>
          <div v-if="showSummaries" class="summaries-list">
            <div v-for="summary in fileSummaries" :key="summary.file_path" class="summary-item">
              <div class="summary-file">{{ getFileName(summary.file_path) }}</div>
              <div class="summary-text">{{ summary.summary }}</div>
              <div class="summary-meta">{{ summary.tokens_used }} tokens</div>
            </div>
          </div>
        </div>

        <!-- 操作按钮 -->
        <div class="progress-actions">
          <button v-if="canCancel" @click="$emit('cancel')" class="cancel-btn" :disabled="!isProcessing">
            取消
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'

/**
 * 分层提交进度组件
 * 作者：Evilek
 * 编写日期：2025-08-04
 */

interface FileSummary {
  file_path: string
  summary: string
  tokens_used: number
}

interface Props {
  visible: boolean
  sessionId: string
  currentStep: number
  totalSteps: number
  currentStatus: string
  currentFile?: string
  fileSummaries: FileSummary[]
  canCancel?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  canCancel: true
})

defineEmits<{
  cancel: []
}>()

const showSummaries = ref(false)

const overallProgressPercent = computed(() => {
  if (props.totalSteps === 0) return 0
  return Math.round((props.currentStep / props.totalSteps) * 100)
})

const isProcessing = computed(() => {
  return props.currentStep < props.totalSteps
})

const toggleSummaries = () => {
  showSummaries.value = !showSummaries.value
}

const getFileName = (filePath: string) => {
  const parts = filePath.split(/[/\\]/)
  return parts[parts.length - 1]
}
</script>

<style scoped>
.layered-progress-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.7);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.layered-progress-modal {
  background: white;
  border-radius: 12px;
  padding: 24px;
  min-width: 500px;
  max-width: 700px;
  max-height: 80vh;
  overflow-y: auto;
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.3);
}

.progress-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
  padding-bottom: 12px;
  border-bottom: 1px solid #e5e7eb;
}

.progress-header h3 {
  margin: 0;
  color: #1f2937;
  font-size: 18px;
  font-weight: 600;
}

.session-info {
  font-size: 12px;
  color: #6b7280;
}

.session-id {
  background: #f3f4f6;
  padding: 2px 8px;
  border-radius: 4px;
  font-family: monospace;
}

.overall-progress {
  margin-bottom: 20px;
}

.progress-label {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
  font-size: 14px;
  font-weight: 500;
  color: #374151;
}

.progress-fraction {
  color: #6b7280;
  font-size: 13px;
}

.progress-bar {
  height: 8px;
  background: #e5e7eb;
  border-radius: 4px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: linear-gradient(90deg, #3b82f6, #1d4ed8);
  border-radius: 4px;
  transition: width 0.3s ease;
}

.current-status {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 16px;
  padding: 12px;
  background: #f8fafc;
  border-radius: 8px;
}

.status-icon {
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.spinner {
  width: 16px;
  height: 16px;
  border: 2px solid #e5e7eb;
  border-top: 2px solid #3b82f6;
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

.check-icon {
  color: #10b981;
  font-size: 16px;
  font-weight: bold;
}

.status-text {
  font-size: 14px;
  color: #374151;
  font-weight: 500;
}



.completed-files {
  margin-bottom: 20px;
}

.completed-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.completed-header span {
  font-size: 14px;
  font-weight: 500;
  color: #374151;
}

.toggle-btn {
  background: none;
  border: none;
  color: #3b82f6;
  cursor: pointer;
  font-size: 12px;
  padding: 4px 8px;
  border-radius: 4px;
  transition: background-color 0.2s;
}

.toggle-btn:hover {
  background: #eff6ff;
}

.summaries-list {
  max-height: 200px;
  overflow-y: auto;
  border: 1px solid #e5e7eb;
  border-radius: 6px;
}

.summary-item {
  padding: 8px 12px;
  border-bottom: 1px solid #f3f4f6;
}

.summary-item:last-child {
  border-bottom: none;
}

.summary-file {
  font-size: 12px;
  color: #3b82f6;
  font-weight: 500;
  margin-bottom: 4px;
}

.summary-text {
  font-size: 13px;
  color: #374151;
  line-height: 1.4;
  margin-bottom: 4px;
}

.summary-meta {
  font-size: 11px;
  color: #9ca3af;
}

.progress-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 20px;
  padding-top: 16px;
  border-top: 1px solid #e5e7eb;
}

.cancel-btn {
  padding: 8px 16px;
  background: #f3f4f6;
  color: #374151;
  border: 1px solid #d1d5db;
  border-radius: 6px;
  cursor: pointer;
  font-size: 13px;
  transition: all 0.2s;
}

.cancel-btn:hover:not(:disabled) {
  background: #e5e7eb;
}

.cancel-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

@keyframes spin {
  0% {
    transform: rotate(0deg);
  }

  100% {
    transform: rotate(360deg);
  }
}
</style>
