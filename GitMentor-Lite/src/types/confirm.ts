/**
 * 确认对话框类型定义
 * Author: Evilek
 * Date: 2025-01-29
 */

export interface ConfirmOptions {
  type?: 'danger' | 'warning' | 'info' | 'question'
  title: string
  message: string
  details?: string
  confirmText?: string
  cancelText?: string
  showCancel?: boolean
  showClose?: boolean
  confirmButtonClass?: string
  cancelButtonClass?: string
  width?: string
  maxWidth?: string
  zIndex?: number
  closeOnClickOutside?: boolean
  closeOnEsc?: boolean
  allowOutsideClick?: boolean
  beforeConfirm?: () => boolean | Promise<boolean>
  beforeCancel?: () => boolean | Promise<boolean>
}
