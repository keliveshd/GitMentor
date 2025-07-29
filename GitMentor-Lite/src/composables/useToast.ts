/**
 * Toast通知系统的组合式API
 * Author: Evilek
 * Date: 2025-01-29
 */

import { ref, App } from 'vue'
import type { ToastOptions } from '../components/Toast.vue'

// 全局Toast实例引用
const toastInstance = ref<any>(null)

// 设置Toast实例
export const setToastInstance = (instance: any) => {
  toastInstance.value = instance
}

// Toast组合式API
export const useToast = () => {
  const addToast = (options: ToastOptions): string => {
    if (!toastInstance.value) {
      console.warn('Toast instance not found. Make sure Toast component is mounted.')
      return ''
    }
    return toastInstance.value.addToast(options)
  }

  const removeToast = (id: string) => {
    if (toastInstance.value) {
      toastInstance.value.removeToast(id)
    }
  }

  const clearAll = () => {
    if (toastInstance.value) {
      toastInstance.value.clearAll()
    }
  }

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

  return {
    addToast,
    removeToast,
    clearAll,
    success,
    error,
    warning,
    info
  }
}

// Vue插件安装函数
export const ToastPlugin = {
  install(app: App) {
    const toast = useToast()
    
    // 全局属性
    app.config.globalProperties.$toast = toast
    
    // 提供注入
    app.provide('toast', toast)
  }
}

// 默认导出
export default useToast
