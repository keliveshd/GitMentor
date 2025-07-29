/**
 * 确认对话框组合式API
 * Author: Evilek
 * Date: 2025-01-29
 */

import { ref, App } from 'vue'
import type { ConfirmOptions } from '@/components/ConfirmDialog.vue'

// 全局确认对话框状态
const visible = ref(false)
const options = ref<ConfirmOptions>({
  title: '',
  message: ''
})
let resolvePromise: ((value: boolean) => void) | null = null

// 确认对话框实例
export const useConfirm = () => {
  const show = (confirmOptions: ConfirmOptions): Promise<boolean> => {
    return new Promise((resolve) => {
      options.value = {
        type: 'question',
        confirmText: '确认',
        cancelText: '取消',
        allowOutsideClick: true,
        ...confirmOptions
      }
      visible.value = true
      resolvePromise = resolve
    })
  }

  const confirm = () => {
    visible.value = false
    if (resolvePromise) {
      resolvePromise(true)
      resolvePromise = null
    }
  }

  const cancel = () => {
    visible.value = false
    if (resolvePromise) {
      resolvePromise(false)
      resolvePromise = null
    }
  }

  const close = () => {
    cancel() // 关闭等同于取消
  }

  return {
    visible,
    options,
    show,
    confirm,
    cancel,
    close
  }
}

// 全局确认对话框实例
const globalConfirm = useConfirm()

// 便捷方法
export const confirm = {
  // 危险操作确认
  danger: (title: string, message: string, details?: string) => {
    return globalConfirm.show({
      type: 'danger',
      title,
      message,
      details,
      confirmText: '确认删除',
      cancelText: '取消'
    })
  },

  // 警告确认
  warning: (title: string, message: string, details?: string) => {
    return globalConfirm.show({
      type: 'warning',
      title,
      message,
      details,
      confirmText: '继续',
      cancelText: '取消'
    })
  },

  // 信息确认
  info: (title: string, message: string, details?: string) => {
    return globalConfirm.show({
      type: 'info',
      title,
      message,
      details,
      confirmText: '确定',
      cancelText: '取消'
    })
  },

  // 通用确认
  ask: (title: string, message: string, confirmText = '确认', cancelText = '取消') => {
    return globalConfirm.show({
      type: 'question',
      title,
      message,
      confirmText,
      cancelText
    })
  },

  // 自定义确认
  custom: (options: ConfirmOptions) => {
    return globalConfirm.show(options)
  }
}

// Vue插件安装
export const ConfirmPlugin = {
  install(app: App) {
    // 全局属性
    app.config.globalProperties.$confirm = confirm
    
    // 提供全局确认实例
    app.provide('confirm', globalConfirm)
  }
}

// 导出全局实例供组件使用
export { globalConfirm }

// TypeScript声明增强
declare module '@vue/runtime-core' {
  interface ComponentCustomProperties {
    $confirm: typeof confirm
  }
}
