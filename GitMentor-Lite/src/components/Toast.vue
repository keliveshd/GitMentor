<template>
  <Teleport to="body">
    <div class="toast-container">
      <TransitionGroup name="toast" tag="div">
        <div
          v-for="toast in toasts"
          :key="toast.id"
          :class="['toast', `toast-${toast.type}`]"
          @click="removeToast(toast.id)"
        >
          <div class="toast-icon">
            <span v-if="toast.type === 'success'">✅</span>
            <span v-else-if="toast.type === 'error'">❌</span>
            <span v-else-if="toast.type === 'warning'">⚠️</span>
            <span v-else-if="toast.type === 'info'">ℹ️</span>
            <span v-else>📢</span>
          </div>
          <div class="toast-content">
            <div class="toast-title" v-if="toast.title">{{ toast.title }}</div>
            <div class="toast-message">{{ toast.message }}</div>
          </div>
          <button class="toast-close" @click.stop="removeToast(toast.id)">×</button>
        </div>
      </TransitionGroup>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'

export interface ToastOptions {
  type?: 'success' | 'error' | 'warning' | 'info'
  title?: string
  message: string
  duration?: number
  persistent?: boolean
}

interface Toast extends Required<Omit<ToastOptions, 'persistent'>> {
  id: string
  persistent: boolean
}

// 响应式数据
const toasts = ref<Toast[]>([])

// 生成唯一ID
const generateId = () => Math.random().toString(36).substr(2, 9)

// 添加Toast
const addToast = (options: ToastOptions): string => {
  const toast: Toast = {
    id: generateId(),
    type: options.type || 'info',
    title: options.title || '',
    message: options.message,
    duration: options.duration || 4000,
    persistent: options.persistent || false
  }

  toasts.value.push(toast)

  // 如果不是持久化Toast，设置自动移除
  if (!toast.persistent) {
    setTimeout(() => {
      removeToast(toast.id)
    }, toast.duration)
  }

  return toast.id
}

// 移除Toast
const removeToast = (id: string) => {
  const index = toasts.value.findIndex(toast => toast.id === id)
  if (index > -1) {
    toasts.value.splice(index, 1)
  }
}

// 清空所有Toast
const clearAll = () => {
  toasts.value = []
}

// 便捷方法
const success = (message: string, title?: string, duration?: number) => {
  return addToast({ type: 'success', message, title, duration })
}

const error = (message: string, title?: string, persistent = false) => {
  return addToast({ type: 'error', message, title, persistent })
}

const warning = (message: string, title?: string, duration?: number) => {
  return addToast({ type: 'warning', message, title, duration })
}

const info = (message: string, title?: string, duration?: number) => {
  return addToast({ type: 'info', message, title, duration })
}

// 暴露方法给父组件
defineExpose({
  addToast,
  removeToast,
  clearAll,
  success,
  error,
  warning,
  info
})

// 全局键盘事件处理
const handleKeydown = (event: KeyboardEvent) => {
  if (event.key === 'Escape') {
    clearAll()
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
.toast-container {
  position: fixed;
  top: 20px;
  right: 20px;
  z-index: 1000005;
  pointer-events: none;
}

.toast {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  min-width: 300px;
  max-width: 500px;
  padding: 16px;
  margin-bottom: 12px;
  background: white;
  border-radius: 8px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  border-left: 4px solid;
  pointer-events: auto;
  cursor: pointer;
  transition: all 0.3s ease;
}

.toast:hover {
  transform: translateX(-4px);
  box-shadow: 0 6px 20px rgba(0, 0, 0, 0.2);
}

.toast-success {
  border-left-color: #10b981;
  background: linear-gradient(135deg, #ecfdf5 0%, #f0fdf4 100%);
}

.toast-error {
  border-left-color: #ef4444;
  background: linear-gradient(135deg, #fef2f2 0%, #fef1f1 100%);
}

.toast-warning {
  border-left-color: #f59e0b;
  background: linear-gradient(135deg, #fffbeb 0%, #fefce8 100%);
}

.toast-info {
  border-left-color: #3b82f6;
  background: linear-gradient(135deg, #eff6ff 0%, #f0f9ff 100%);
}

.toast-icon {
  font-size: 20px;
  flex-shrink: 0;
  margin-top: 2px;
}

.toast-content {
  flex: 1;
  min-width: 0;
}

.toast-title {
  font-weight: 600;
  font-size: 14px;
  color: #1f2937;
  margin-bottom: 4px;
  line-height: 1.4;
}

.toast-message {
  font-size: 13px;
  color: #6b7280;
  line-height: 1.5;
  word-wrap: break-word;
}

.toast-close {
  background: none;
  border: none;
  font-size: 18px;
  color: #9ca3af;
  cursor: pointer;
  padding: 0;
  width: 20px;
  height: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 50%;
  transition: all 0.2s ease;
  flex-shrink: 0;
}

.toast-close:hover {
  background: rgba(0, 0, 0, 0.1);
  color: #374151;
}

/* 动画效果 */
.toast-enter-active,
.toast-leave-active {
  transition: all 0.3s ease;
}

.toast-enter-from {
  opacity: 0;
  transform: translateX(100%);
}

.toast-leave-to {
  opacity: 0;
  transform: translateX(100%);
}

.toast-move {
  transition: transform 0.3s ease;
}

/* 深色主题支持 */
@media (prefers-color-scheme: dark) {
  .toast {
    background: #374151;
    color: #f9fafb;
  }

  .toast-success {
    background: linear-gradient(135deg, #064e3b 0%, #065f46 100%);
  }

  .toast-error {
    background: linear-gradient(135deg, #7f1d1d 0%, #991b1b 100%);
  }

  .toast-warning {
    background: linear-gradient(135deg, #78350f 0%, #92400e 100%);
  }

  .toast-info {
    background: linear-gradient(135deg, #1e3a8a 0%, #1e40af 100%);
  }

  .toast-title {
    color: #f9fafb;
  }

  .toast-message {
    color: #d1d5db;
  }

  .toast-close {
    color: #9ca3af;
  }

  .toast-close:hover {
    background: rgba(255, 255, 255, 0.1);
    color: #f3f4f6;
  }
}

/* 响应式设计 */
@media (max-width: 640px) {
  .toast-container {
    top: 10px;
    right: 10px;
    left: 10px;
  }

  .toast {
    min-width: auto;
    max-width: none;
  }
}
</style>
