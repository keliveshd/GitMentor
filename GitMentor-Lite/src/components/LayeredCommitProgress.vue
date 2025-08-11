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

        <!-- AI实时输出显示区域 - Author: Evilek, Date: 2025-01-10 -->
        <div v-if="aiStreamContent || isProcessing" class="ai-stream-section">
          <div class="stream-header">
            <span>⚡ AI实时反馈</span>
            <button @click="toggleStreamExpanded" class="toggle-stream-btn">
              {{ streamExpanded ? '收起' : '展开' }}
            </button>
          </div>
          <div v-if="streamExpanded" class="stream-content">
            <div class="stream-output" ref="streamOutputRef">
              <div v-if="aiStreamContent" class="stream-text">
                <div v-for="(part, index) in parsedAiContent" :key="index">
                  <div v-if="part.type === 'text'" v-html="part.content"></div>
                  <div v-else-if="part.type === 'think'" class="think-section">
                    <div class="think-header" @click="toggleThinkSection(index)">
                      <span class="think-icon">{{ part.expanded ? '▼' : '▶' }}</span>
                      <span class="think-title">🧠 推理过程</span>
                    </div>
                    <div v-if="part.expanded" class="think-content">
                      <pre>{{ part.content }}</pre>
                    </div>
                  </div>
                </div>
              </div>
              <div v-if="isProcessing && !aiStreamContent" class="stream-placeholder">
                <div class="typing-indicator">
                  <span></span>
                  <span></span>
                  <span></span>
                </div>
                <span class="typing-text">AI正在思考中...</span>
              </div>
            </div>
          </div>
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
            <div v-for="summary in fileSummaries" :key="summary.filePath" class="summary-item">
              <div class="summary-file">{{ getFileName(summary.filePath) }}</div>
              <div class="summary-text">{{ summary.summary }}</div>
              <div class="summary-meta">{{ summary.tokensUsed }} tokens</div>
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
import { computed, ref, watch, nextTick } from 'vue'

/**
 * 分层提交进度组件
 * 作者：Evilek
 * 编写日期：2025-08-04
 */

interface FileSummary {
  filePath: string  // 修复：与后端serde重命名保持一致 - Author: Evilek, Date: 2025-01-09
  summary: string
  tokensUsed: number  // 修复：与后端serde重命名保持一致 - Author: Evilek, Date: 2025-01-09
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
  aiStreamContent?: string  // AI实时输出内容 - Author: Evilek, Date: 2025-01-10
}

const props = withDefaults(defineProps<Props>(), {
  canCancel: true
})

defineEmits<{
  cancel: []
}>()

const showSummaries = ref(false)
const streamExpanded = ref(true)  // AI流式输出默认展开 - Author: Evilek, Date: 2025-01-10
const streamOutputRef = ref<HTMLElement | null>(null)  // 流式输出容器引用
const thinkSections = ref<{ [key: number]: boolean }>({})  // 记录每个think区域的展开状态

const overallProgressPercent = computed(() => {
  if (props.totalSteps === 0) return 0
  return Math.round((props.currentStep / props.totalSteps) * 100)
})

const isProcessing = computed(() => {
  return props.currentStep < props.totalSteps
})

// 解析AI内容，分离文本和think标签 - Author: Evilek, Date: 2025-01-10
const parsedAiContent = computed(() => {
  if (!props.aiStreamContent) return []

  const parts: Array<{ type: 'text' | 'think', content: string, expanded?: boolean }> = []
  const content = props.aiStreamContent
  const thinkRegex = /<think>([\s\S]*?)<\/think>/g

  let lastIndex = 0
  let match
  let thinkIndex = 0

  while ((match = thinkRegex.exec(content)) !== null) {
    // 添加think标签前的文本
    if (match.index > lastIndex) {
      const textContent = content.slice(lastIndex, match.index).trim()
      if (textContent) {
        parts.push({
          type: 'text',
          content: textContent.replace(/\n/g, '<br>')
        })
      }
    }

    // 添加think内容，默认展开第一个think区域 - Author: Evilek, Date: 2025-01-10
    parts.push({
      type: 'think',
      content: match[1].trim(),
      expanded: thinkSections.value[thinkIndex] !== undefined ? thinkSections.value[thinkIndex] : (thinkIndex === 0)
    })

    lastIndex = match.index + match[0].length
    thinkIndex++
  }

  // 添加最后剩余的文本
  if (lastIndex < content.length) {
    const textContent = content.slice(lastIndex).trim()
    if (textContent) {
      parts.push({
        type: 'text',
        content: textContent.replace(/\n/g, '<br>')
      })
    }
  }

  return parts
})

const toggleSummaries = () => {
  showSummaries.value = !showSummaries.value
}

const toggleStreamExpanded = () => {
  streamExpanded.value = !streamExpanded.value
}

const toggleThinkSection = (index: number) => {
  // 确保响应式更新 - Author: Evilek, Date: 2025-01-10
  const currentState = thinkSections.value[index] !== undefined ? thinkSections.value[index] : (index === 0)
  thinkSections.value = {
    ...thinkSections.value,
    [index]: !currentState
  }
}

const getFileName = (filePath: string) => {
  const parts = filePath.split(/[/\\]/)
  return parts[parts.length - 1]
}



// 监听AI流式内容变化，自动滚动到底部 - Author: Evilek, Date: 2025-01-10
watch(() => props.aiStreamContent, async () => {
  if (streamExpanded.value && streamOutputRef.value) {
    await nextTick()
    streamOutputRef.value.scrollTop = streamOutputRef.value.scrollHeight
  }
})
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
  min-width: 600px;
  max-width: 900px;
  width: 85vw;
  /* 自适应高度，移除固定的max-height - Author: Evilek, Date: 2025-01-10 */
  min-height: 300px;
  max-height: 90vh;
  /* 只在内容过多时限制最大高度 */
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.3);
  box-sizing: border-box;
  display: flex;
  flex-direction: column;
  /* 让弹窗内容自适应，避免不必要的滚动条 */
  overflow: hidden;
  /* 主容器不滚动 */
}

.progress-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
  padding-bottom: 12px;
  border-bottom: 1px solid #e5e7eb;
  flex-shrink: 0;
  /* 防止被压缩 - Author: Evilek, Date: 2025-01-10 */
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
  flex-shrink: 0;
  /* 防止被压缩 - Author: Evilek, Date: 2025-01-10 */
}

/* AI流式输出区域样式 - Author: Evilek, Date: 2025-01-10 */
.ai-stream-section {
  margin: 20px 0;
  border: 1px solid #e5e7eb;
  border-radius: 8px;
  background: #f9fafb;
  /* 在flex布局中自适应高度 - Author: Evilek, Date: 2025-01-10 */
  flex: 1;
  /* 占用剩余空间 */
  min-height: 150px;
  /* 减少最小高度 */
  display: flex;
  flex-direction: column;
}

.stream-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  background: #f3f4f6;
  border-bottom: 1px solid #e5e7eb;
  border-radius: 8px 8px 0 0;
  font-weight: 500;
  color: #374151;
}

.toggle-stream-btn {
  background: #ffffff;
  border: 1px solid #d1d5db;
  border-radius: 4px;
  padding: 4px 8px;
  font-size: 12px;
  color: #6b7280;
  cursor: pointer;
  transition: all 0.2s;
}

.toggle-stream-btn:hover {
  background: #f9fafb;
  border-color: #9ca3af;
}

.stream-content {
  padding: 16px;
  flex: 1;
  /* 占用剩余空间 - Author: Evilek, Date: 2025-01-10 */
  display: flex;
  flex-direction: column;
}

.stream-output {
  background: #ffffff;
  border: 1px solid #e5e7eb;
  border-radius: 6px;
  padding: 12px;
  /* 自适应高度，移除固定的min/max-height - Author: Evilek, Date: 2025-01-10 */
  flex: 1;
  overflow-y: auto;
  font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
  font-size: 13px;
  line-height: 1.5;
  /* 防止内容变化时的布局跳动 */
  transition: none;
  will-change: auto;
  /* 优化滚动条样式 */
  scrollbar-width: thin;
  scrollbar-color: #cbd5e1 #f1f5f9;
}

.stream-text {
  color: #1f2937;
  white-space: pre-wrap;
  word-break: break-word;
  /* 防止流式输出时的布局跳动 - Author: Evilek, Date: 2025-01-10 */
  min-height: 20px;
  display: block;
}

.stream-placeholder {
  display: flex;
  align-items: center;
  gap: 8px;
  color: #6b7280;
  font-style: italic;
}

.typing-indicator {
  display: flex;
  gap: 4px;
}

.typing-indicator span {
  width: 6px;
  height: 6px;
  background: #9ca3af;
  border-radius: 50%;
  animation: typing 1.4s infinite ease-in-out;
}

.typing-indicator span:nth-child(1) {
  animation-delay: -0.32s;
}

.typing-indicator span:nth-child(2) {
  animation-delay: -0.16s;
}

@keyframes typing {

  0%,
  80%,
  100% {
    transform: scale(0.8);
    opacity: 0.5;
  }

  40% {
    transform: scale(1);
    opacity: 1;
  }
}

.typing-text {
  font-size: 12px;
}

/* Think标签折叠样式 - Author: Evilek, Date: 2025-01-10 */
.think-section {
  margin: 12px 0;
  border: 1px solid #e5e7eb;
  border-radius: 6px;
  background: #f9fafb;
}

.think-header {
  padding: 8px 12px;
  background: #f3f4f6;
  border-bottom: 1px solid #e5e7eb;
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: 8px;
  border-radius: 6px 6px 0 0;
  transition: background-color 0.2s;
}

.think-header:hover {
  background: #e5e7eb;
}

.think-icon {
  font-size: 12px;
  color: #6b7280;
  transition: transform 0.2s;
}

.think-title {
  font-size: 13px;
  font-weight: 500;
  color: #374151;
}

.think-content {
  padding: 12px;
  background: #ffffff;
  border-radius: 0 0 6px 6px;
}

.think-content pre {
  margin: 0;
  font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
  font-size: 12px;
  line-height: 1.4;
  color: #1f2937;
  white-space: pre-wrap;
  word-break: break-word;
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
