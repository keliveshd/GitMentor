<template>
  <Teleport to="body">
    <div v-if="visible" class="confirm-overlay" @click="handleOverlayClick">
      <div class="confirm-dialog" @click.stop>
        <div class="confirm-header">
          <div class="confirm-icon">
            <span v-if="type === 'danger'">⚠️</span>
            <span v-else-if="type === 'warning'">🔔</span>
            <span v-else-if="type === 'info'">ℹ️</span>
            <span v-else>❓</span>
          </div>
          <h3 class="confirm-title">{{ title }}</h3>
        </div>

        <div class="confirm-content">
          <p class="confirm-message">{{ message }}</p>
          <div v-if="details" class="confirm-details">
            <div class="details-toggle" @click="showDetails = !showDetails">
              <span>{{ showDetails ? '隐藏详情' : '显示详情' }}</span>
              <span class="toggle-arrow" :class="{ 'open': showDetails }">▼</span>
            </div>
            <div v-if="showDetails" class="details-content">
              <pre>{{ details }}</pre>
            </div>
          </div>
        </div>

        <div class="confirm-actions">
          <button @click="handleCancel" class="confirm-btn cancel-btn" :disabled="loading">
            {{ cancelText }}
          </button>
          <button @click="handleConfirm" :class="['confirm-btn', 'confirm-btn-primary', `confirm-btn-${type}`]"
            :disabled="loading">
            <span v-if="loading" class="btn-spinner"></span>
            {{ loading ? '处理中...' : confirmText }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import type { ConfirmOptions } from '@/types/confirm'

interface Props {
  visible: boolean
  options: ConfirmOptions
}

const props = defineProps<Props>()

const emit = defineEmits<{
  confirm: []
  cancel: []
  close: []
}>()

// 响应式数据
const loading = ref(false)
const showDetails = ref(false)

// 计算属性
const type = computed(() => props.options.type || 'question')
const title = computed(() => props.options.title)
const message = computed(() => props.options.message)
const details = computed(() => props.options.details)
const confirmText = computed(() => props.options.confirmText || '确认')
const cancelText = computed(() => props.options.cancelText || '取消')
const allowOutsideClick = computed(() => props.options.allowOutsideClick !== false)

// 方法
const handleConfirm = async () => {
  loading.value = true
  try {
    emit('confirm')
  } finally {
    loading.value = false
  }
}

const handleCancel = () => {
  if (!loading.value) {
    emit('cancel')
  }
}

const handleOverlayClick = () => {
  if (allowOutsideClick.value && !loading.value) {
    emit('close')
  }
}

// 键盘事件处理
const handleKeydown = (event: KeyboardEvent) => {
  if (!props.visible) return

  if (event.key === 'Escape' && !loading.value) {
    event.preventDefault()
    emit('cancel')
  } else if (event.key === 'Enter' && !loading.value) {
    event.preventDefault()
    handleConfirm()
  }
}

onMounted(() => {
  document.addEventListener('keydown', handleKeydown)
})

onUnmounted(() => {
  document.removeEventListener('keydown', handleKeydown)
})
</script>

<style scoped>
.confirm-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  z-index: 10000;
  display: flex;
  align-items: center;
  justify-content: center;
  animation: fadeIn 0.2s ease;
}

@keyframes fadeIn {
  from {
    opacity: 0;
  }

  to {
    opacity: 1;
  }
}

.confirm-dialog {
  background: white;
  border-radius: 12px;
  box-shadow: 0 10px 30px rgba(0, 0, 0, 0.3);
  min-width: 400px;
  max-width: 600px;
  max-height: 80vh;
  overflow: hidden;
  animation: slideIn 0.3s ease;
}

@keyframes slideIn {
  from {
    opacity: 0;
    transform: translateY(-20px) scale(0.95);
  }

  to {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
}

.confirm-header {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 20px 24px 16px;
  border-bottom: 1px solid #e5e7eb;
}

.confirm-icon {
  font-size: 24px;
  flex-shrink: 0;
}

.confirm-title {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
  color: #1f2937;
}

.confirm-content {
  padding: 16px 24px;
}

.confirm-message {
  margin: 0 0 16px 0;
  font-size: 14px;
  line-height: 1.6;
  color: #4b5563;
}

.confirm-details {
  margin-top: 16px;
}

.details-toggle {
  display: flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
  font-size: 13px;
  color: #6b7280;
  padding: 8px 0;
  border-top: 1px solid #e5e7eb;
}

.details-toggle:hover {
  color: #374151;
}

.toggle-arrow {
  transition: transform 0.2s ease;
}

.toggle-arrow.open {
  transform: rotate(180deg);
}

.details-content {
  margin-top: 8px;
  padding: 12px;
  background: #f9fafb;
  border: 1px solid #e5e7eb;
  border-radius: 6px;
  font-size: 12px;
  max-height: 200px;
  overflow-y: auto;
}

.details-content pre {
  margin: 0;
  white-space: pre-wrap;
  word-wrap: break-word;
  color: #374151;
}

.confirm-actions {
  display: flex;
  gap: 12px;
  padding: 16px 24px 20px;
  justify-content: flex-end;
  background: #f9fafb;
}

.confirm-btn {
  padding: 10px 20px;
  border: none;
  border-radius: 6px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 80px;
  justify-content: center;
}

.confirm-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.cancel-btn {
  background: #f3f4f6;
  color: #374151;
  border: 1px solid #d1d5db;
}

.cancel-btn:hover:not(:disabled) {
  background: #e5e7eb;
}

.confirm-btn-primary {
  color: white;
}

.confirm-btn-danger {
  background: #ef4444;
}

.confirm-btn-danger:hover:not(:disabled) {
  background: #dc2626;
}

.confirm-btn-warning {
  background: #f59e0b;
}

.confirm-btn-warning:hover:not(:disabled) {
  background: #d97706;
}

.confirm-btn-info {
  background: #3b82f6;
}

.confirm-btn-info:hover:not(:disabled) {
  background: #2563eb;
}

.confirm-btn-question {
  background: #6b7280;
}

.confirm-btn-question:hover:not(:disabled) {
  background: #4b5563;
}

.btn-spinner {
  width: 14px;
  height: 14px;
  border: 2px solid rgba(255, 255, 255, 0.3);
  border-top: 2px solid white;
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

/* 深色主题支持 */
@media (prefers-color-scheme: dark) {
  .confirm-dialog {
    background: #1f2937;
    color: #f9fafb;
  }

  .confirm-header {
    border-color: #374151;
  }

  .confirm-title {
    color: #f9fafb;
  }

  .confirm-message {
    color: #d1d5db;
  }

  .details-toggle {
    color: #9ca3af;
    border-color: #374151;
  }

  .details-toggle:hover {
    color: #d1d5db;
  }

  .details-content {
    background: #374151;
    border-color: #4b5563;
  }

  .details-content pre {
    color: #e5e7eb;
  }

  .confirm-actions {
    background: #374151;
  }

  .cancel-btn {
    background: #4b5563;
    color: #e5e7eb;
    border-color: #6b7280;
  }

  .cancel-btn:hover:not(:disabled) {
    background: #6b7280;
  }
}

/* 响应式设计 */
@media (max-width: 640px) {
  .confirm-dialog {
    min-width: auto;
    margin: 20px;
    max-width: calc(100vw - 40px);
  }

  .confirm-actions {
    flex-direction: column-reverse;
  }

  .confirm-btn {
    width: 100%;
  }
}
</style>
