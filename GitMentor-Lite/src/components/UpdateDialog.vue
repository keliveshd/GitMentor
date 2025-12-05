<template>
  <div v-if="visible" class="update-dialog-overlay" @click.self="closeDialog">
    <div class="update-dialog">
      <!-- 对话框头部 -->
      <div class="dialog-header">
        <h3 class="dialog-title">
          <span class="title-icon">🔄</span>
          {{ dialogTitle }}
        </h3>
        <button @click="closeDialog" class="close-btn" :disabled="isDownloading || isInstalling">
          ✕
        </button>
      </div>

      <!-- 对话框内容 -->
      <div class="dialog-content">
        <!-- 检查更新状态 -->
        <div v-if="status === 'checking'" class="status-section">
          <div class="loading-container">
            <div class="loading-spinner-modern"></div>
            <div class="loading-text">
              <h3 class="status-title">检查更新中</h3>
              <p class="status-description">正在从GitHub获取最新版本信息...</p>
            </div>
          </div>
        </div>

        <!-- 无更新状态 -->
        <div v-else-if="status === 'no-update'" class="status-section">
          <div class="status-container success">
            <div class="status-icon-modern success">
              <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7"></path>
              </svg>
            </div>
            <div class="status-text">
              <h3 class="status-title">已是最新版本</h3>
              <p class="status-description">当前版本 {{ currentVersion }} 是最新的</p>
            </div>
          </div>
        </div>

        <!-- 有更新可用 -->
        <div v-else-if="status === 'update-available'" class="update-info">
          <div class="version-info">
            <div class="version-row">
              <span class="version-label">当前版本：</span>
              <span class="version-current">{{ currentVersion }}</span>
            </div>
            <div class="version-row">
              <span class="version-label">最新版本：</span>
              <span class="version-latest">{{ latestVersion }}</span>
            </div>
          </div>

          <!-- 更新日志 -->
          <div v-if="releaseNotes" class="release-notes">
            <h4>更新内容：</h4>
            <div class="notes-content" v-html="formatReleaseNotes(releaseNotes)"></div>
          </div>

          <!-- 发布时间 -->
          <div v-if="publishedAt" class="publish-info">
            <span class="publish-label">发布时间：</span>
            <span class="publish-date">{{ formatDate(publishedAt) }}</span>
          </div>
        </div>

        <!-- 下载进度 -->
        <div v-else-if="status === 'downloading'" class="download-section">
          <div class="download-info">
            <p>正在下载更新包...</p>
            <div class="progress-bar">
              <div class="progress-fill" :style="{ width: downloadProgress + '%' }"></div>
            </div>
            <div class="progress-text">
              {{ downloadProgress.toFixed(1) }}%
              ({{ formatBytes(downloadedBytes) }} / {{ formatBytes(totalBytes) }})
            </div>
          </div>
        </div>

        <!-- 安装状态 -->
        <div v-else-if="status === 'installing'" class="status-section">
          <div class="loading-spinner">⚙️</div>
          <p>正在安装更新，请稍候...</p>
          <p class="install-note">安装完成后应用将自动重启</p>
        </div>

        <!-- 错误状态 -->
        <div v-else-if="status === 'error'" class="status-section">
          <div class="status-container error">
            <div class="status-icon-modern error">
              <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path>
              </svg>
            </div>
            <div class="status-text">
              <h3 class="status-title">更新检查失败</h3>
              <p class="status-description">{{ errorMessage }}</p>
            </div>
          </div>

          <div class="error-actions-modern">
            <button @click="retryCheck" class="btn-modern btn-secondary">
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                  d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15">
                </path>
              </svg>
              重试检查
            </button>
            <button @click="openDownloadPage" class="btn-modern btn-primary" :disabled="isOpeningBrowser">
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                  d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14"></path>
              </svg>
              {{ isOpeningBrowser ? '打开中...' : '手动下载' }}
            </button>
          </div>

          <div class="info-card">
            <div class="info-card-header">
              <svg class="w-5 h-5 text-blue-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                  d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path>
              </svg>
              <span class="info-card-title">手动下载说明</span>
            </div>
            <p class="info-card-description">
              如果自动更新失败，您可以访问GitHub Releases页面手动下载最新版本
            </p>
            <div class="info-card-url">
              <code>https://github.com/keliveshd/GitMentor/releases</code>
            </div>
          </div>
        </div>
      </div>

      <!-- 对话框按钮 -->
      <div class="dialog-actions-modern">
        <button v-if="status === 'update-available'" @click="startDownload" class="btn-modern btn-primary btn-large"
          :disabled="!downloadUrl">
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
              d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4"></path>
          </svg>
          立即更新
        </button>

        <button v-if="status === 'downloading'" @click="cancelDownload" class="btn-modern btn-destructive">
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path>
          </svg>
          取消下载
        </button>

        <button v-if="['no-update', 'error'].includes(status)" @click="closeDialog" class="btn-modern btn-secondary">
          关闭
        </button>

        <button v-if="status === 'update-available'" @click="closeDialog" class="btn-modern btn-ghost">
          稍后更新
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

/**
 * GitMentor 更新对话框组件
 * 作者：Evilek
 * 编写日期：2025-01-18
 */

// Props
interface Props {
  visible: boolean
}

const props = defineProps<Props>()

// Emits
const emit = defineEmits<{
  close: []
  updateStarted: []
  updateCompleted: []
}>()

// 状态管理
const status = ref<'checking' | 'no-update' | 'update-available' | 'downloading' | 'installing' | 'error'>('checking')
const currentVersion = ref('')
const latestVersion = ref('')
const releaseNotes = ref('')
const publishedAt = ref('')
const downloadUrl = ref('')
const errorMessage = ref('')

// 下载进度
const downloadProgress = ref(0)
const downloadedBytes = ref(0)
const totalBytes = ref(0)
const isDownloading = ref(false)
const isInstalling = ref(false)
const isOpeningBrowser = ref(false)

// 计算属性
const dialogTitle = computed(() => {
  switch (status.value) {
    case 'checking': return '检查更新'
    case 'no-update': return '已是最新版本'
    case 'update-available': return '发现新版本'
    case 'downloading': return '下载更新'
    case 'installing': return '安装更新'
    case 'error': return '更新失败'
    default: return '检查更新'
  }
})

// 方法
const checkForUpdates = async () => {
  console.log('🔄 [UpdateDialog] ========== checkForUpdates 函数开始 ==========')

  try {
    status.value = 'checking'
    console.log('🔄 [UpdateDialog] 设置状态为 checking')

    // 添加超时处理
    const timeoutPromise = new Promise((_, reject) => {
      setTimeout(() => reject(new Error('检查更新超时（60秒）')), 60000)
    })

    // 获取当前版本
    console.log('📋 [UpdateDialog] 开始获取当前版本...')
    const getCurrentVersionPromise = invoke('get_current_version')
    currentVersion.value = await Promise.race([getCurrentVersionPromise, timeoutPromise]) as string
    console.log('📋 [UpdateDialog] 当前版本:', currentVersion.value)

    // 先测试网络连接
    console.log('🌐 [UpdateDialog] 开始测试网络连接...')
    try {
      const testNetworkPromise = invoke('test_network_connection')
      const networkResult = await Promise.race([testNetworkPromise, timeoutPromise])
      console.log('🌐 [UpdateDialog] 网络连接测试原始结果:', networkResult)

      if (typeof networkResult === 'boolean') {
        console.log('🌐 [UpdateDialog] 网络连接测试结果 (boolean):', networkResult)
        if (!networkResult) {
          throw new Error('网络连接测试失败，请检查网络连接或防火墙设置')
        }
        console.log('✅ [UpdateDialog] 网络连接正常')
      } else {
        console.log('🌐 [UpdateDialog] 网络连接测试返回非布尔值:', typeof networkResult, networkResult)
        // 如果返回的不是布尔值，可能是错误信息
        throw new Error(`网络连接异常: ${networkResult}`)
      }
    } catch (networkError) {
      console.error('❌ [UpdateDialog] 网络连接失败:', networkError)
      console.error('❌ [UpdateDialog] 错误类型:', typeof networkError)
      console.error('❌ [UpdateDialog] 错误详情:', networkError)
      throw new Error(`网络连接失败: ${networkError}`)
    }

    // 检查更新
    console.log('🌐 [UpdateDialog] 开始调用 GitHub API 检查更新...')
    const checkUpdatesPromise = invoke('check_for_updates')
    const updateInfo = await Promise.race([checkUpdatesPromise, timeoutPromise]) as any
    console.log('📦 [UpdateDialog] 更新信息:', updateInfo)

    if (updateInfo.has_update) {
      status.value = 'update-available'
      latestVersion.value = updateInfo.latest
      releaseNotes.value = updateInfo.release_notes || ''
      publishedAt.value = updateInfo.published_at || ''
      downloadUrl.value = updateInfo.download_url || ''
      console.log('✅ [UpdateDialog] 发现新版本:', latestVersion.value)
    } else {
      status.value = 'no-update'
      console.log('✅ [UpdateDialog] 已是最新版本')
    }
  } catch (error) {
    status.value = 'error'
    errorMessage.value = error as string
    console.error('❌ [UpdateDialog] 检查更新失败:', error)
  }
}

const startDownload = async () => {
  console.log('📥 [UpdateDialog] startDownload 开始执行')
  console.log('📥 [UpdateDialog] downloadUrl:', downloadUrl.value)

  if (!downloadUrl.value) {
    errorMessage.value = '下载链接不可用'
    status.value = 'error'
    return
  }

  try {
    console.log('📥 [UpdateDialog] 设置下载状态')
    status.value = 'downloading'
    isDownloading.value = true
    downloadProgress.value = 0
    downloadedBytes.value = 0
    totalBytes.value = 0

    console.log('📥 [UpdateDialog] 发送 updateStarted 事件')
    emit('updateStarted')

    // 开始下载
    console.log('📥 [UpdateDialog] 调用 download_update 命令')
    console.log('📥 [UpdateDialog] 下载参数:', { downloadUrl: downloadUrl.value })

    const installerPath = await invoke('download_update', {
      downloadUrl: downloadUrl.value
    }).catch(error => {
      console.error('📥 [UpdateDialog] download_update 命令失败:', error)
      throw error
    }) as string

    console.log('📥 [UpdateDialog] 下载完成，安装包路径:', installerPath)

    // 下载完成，开始安装
    status.value = 'installing'
    isDownloading.value = false
    isInstalling.value = true

    await invoke('install_update', {
      installerPath
    })

    // 安装完成
    emit('updateCompleted')
    closeDialog()

  } catch (error) {
    status.value = 'error'
    errorMessage.value = error as string
    isDownloading.value = false
    isInstalling.value = false
  }
}

const cancelDownload = () => {
  // TODO: 实现下载取消逻辑
  isDownloading.value = false
  status.value = 'update-available'
}

const retryCheck = async () => {
  console.log('🔄 [UpdateDialog] 用户点击重试按钮')
  await checkForUpdates()
}

const openDownloadPage = async () => {
  if (isOpeningBrowser.value) return

  try {
    isOpeningBrowser.value = true
    const downloadPageUrl = 'https://github.com/keliveshd/GitMentor/releases'

    console.log('🔗 [UpdateDialog] 打开下载页面:', downloadPageUrl)
    await invoke('open_browser_url', { url: downloadPageUrl })
    console.log('✅ [UpdateDialog] 成功打开下载页面')
  } catch (error) {
    console.error('❌ [UpdateDialog] 打开下载页面失败:', error)
    alert(`打开下载页面失败: ${error}`)
  } finally {
    isOpeningBrowser.value = false
  }
}

const closeDialog = () => {
  if (isDownloading.value || isInstalling.value) {
    return // 下载或安装过程中不允许关闭
  }
  emit('close')
}

// 格式化函数
const formatReleaseNotes = (notes: string) => {
  // 简单的 Markdown 转换
  return notes
    .replace(/\n/g, '<br>')
    .replace(/\*\*(.*?)\*\*/g, '<strong>$1</strong>')
    .replace(/\*(.*?)\*/g, '<em>$1</em>')
    .replace(/`(.*?)`/g, '<code>$1</code>')
}

const formatDate = (dateString: string) => {
  return new Date(dateString).toLocaleString('zh-CN')
}

const formatBytes = (bytes: number) => {
  if (bytes === 0) return '0 B'
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
}

// 下载进度监听器
const unlisten = ref<null | (() => void)>(null)

// 监听 visible 属性变化
watch(() => props.visible, async (newVisible, oldVisible) => {
  console.log('🔄 [UpdateDialog] visible 属性变化:', { oldVisible, newVisible })
  if (newVisible && !oldVisible) {
    console.log('🔄 [UpdateDialog] 对话框从隐藏变为显示，开始检查更新')
    await checkForUpdates()
  }
})

// 生命周期
onMounted(async () => {
  console.log('🔄 [UpdateDialog] 组件已挂载, visible:', props.visible)

  if (props.visible) {
    console.log('🔄 [UpdateDialog] 对话框可见，开始检查更新')
    await checkForUpdates()
  } else {
    console.log('🔄 [UpdateDialog] 对话框不可见，跳过检查更新')
  }

  // 设置下载进度监听器
  console.log('🔄 [UpdateDialog] 设置下载进度监听器')
  unlisten.value = await listen('download-progress', (event: any) => {
    const { downloaded, total, percentage } = event.payload
    console.log('📥 [UpdateDialog] 下载进度:', { downloaded, total, percentage })
    downloadedBytes.value = downloaded
    totalBytes.value = total
    downloadProgress.value = percentage
  })
})

onUnmounted(() => {
  console.log('🔄 [UpdateDialog] 组件卸载，清理监听器')
  if (unlisten.value) {
    unlisten.value()
    unlisten.value = null
  }
})
</script>

<style scoped>
.update-dialog-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.update-dialog {
  background: white;
  border-radius: 8px;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.15);
  width: 500px;
  max-width: 90vw;
  max-height: 80vh;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.dialog-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid #e1e5e9;
  background: #f6f8fa;
}

.dialog-title {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
  display: flex;
  align-items: center;
  gap: 8px;
}

.title-icon {
  font-size: 18px;
}

.close-btn {
  background: none;
  border: none;
  font-size: 18px;
  cursor: pointer;
  padding: 4px;
  border-radius: 4px;
  color: #656d76;
}

.close-btn:hover:not(:disabled) {
  background: #e1e5e9;
}

.close-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.dialog-content {
  padding: 20px;
  flex: 1;
  overflow-y: auto;
  min-height: 200px;
  /* 设置最小高度避免滚动条抖动 */
}

/* 现代化状态区域样式 */
.status-section {
  padding: 24px 0;
}

.loading-container {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 20px;
  background: linear-gradient(135deg, #f8fafc 0%, #f1f5f9 100%);
  border-radius: 12px;
  border: 1px solid #e2e8f0;
}

.loading-spinner-modern {
  width: 24px;
  height: 24px;
  border: 3px solid #e2e8f0;
  border-top: 3px solid #3b82f6;
  border-radius: 50%;
  animation: spin 1s linear infinite;
  flex-shrink: 0;
}

@keyframes spin {
  from {
    transform: rotate(0deg);
  }

  to {
    transform: rotate(360deg);
  }
}

.loading-text {
  flex: 1;
}

.status-container {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 20px;
  border-radius: 12px;
  border: 1px solid;
}

.status-container.success {
  background: linear-gradient(135deg, #f0fdf4 0%, #dcfce7 100%);
  border-color: #bbf7d0;
}

.status-container.error {
  background: linear-gradient(135deg, #fef2f2 0%, #fee2e2 100%);
  border-color: #fecaca;
}

.status-icon-modern {
  width: 40px;
  height: 40px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.status-icon-modern.success {
  background: #22c55e;
  color: white;
}

.status-icon-modern.error {
  background: #ef4444;
  color: white;
}

.status-text {
  flex: 1;
}

.status-title {
  font-size: 16px;
  font-weight: 600;
  color: #1f2937;
  margin: 0 0 4px 0;
}

.status-description {
  font-size: 14px;
  color: #6b7280;
  margin: 0;
  line-height: 1.5;
}

.update-info {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.version-info {
  background: #f6f8fa;
  padding: 16px;
  border-radius: 6px;
  margin-bottom: 16px;
}

.version-row {
  display: flex;
  justify-content: space-between;
  margin-bottom: 8px;
}

.version-row:last-child {
  margin-bottom: 0;
}

.version-label {
  font-weight: 500;
  color: #656d76;
}

.version-current {
  color: #0969da;
  font-family: monospace;
}

.version-latest {
  color: #1a7f37;
  font-weight: 600;
  font-family: monospace;
}

.release-notes {
  margin-bottom: 16px;
}

.release-notes h4 {
  margin: 0 0 8px 0;
  font-size: 14px;
  font-weight: 600;
  color: #24292f;
}

.notes-content {
  background: #f6f8fa;
  padding: 12px;
  border-radius: 6px;
  font-size: 13px;
  line-height: 1.5;
  max-height: 150px;
  overflow-y: auto;
}

.publish-info {
  font-size: 13px;
  color: #656d76;
}

.publish-label {
  font-weight: 500;
}

.download-section {
  text-align: center;
}

.download-info p {
  margin-bottom: 16px;
}

.progress-bar {
  width: 100%;
  height: 8px;
  background: #e1e5e9;
  border-radius: 4px;
  overflow: hidden;
  margin-bottom: 8px;
}

.progress-fill {
  height: 100%;
  background: linear-gradient(90deg, #0969da, #1a7f37);
  transition: width 0.3s ease;
}

.progress-text {
  font-size: 13px;
  color: #656d76;
}

.install-note {
  font-size: 13px;
  color: #656d76;
  margin-top: 8px;
}

.error-section {
  text-align: center;
}

.error-message {
  color: #d1242f;
  margin-top: 8px;
}

.dialog-actions {
  padding: 16px 20px;
  border-top: 1px solid #e1e5e9;
  display: flex;
  gap: 12px;
  justify-content: flex-end;
}

.primary-btn {
  background: #1f883d;
  color: white;
  border: none;
  padding: 8px 16px;
  border-radius: 6px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: background 0.2s;
}

.primary-btn:hover:not(:disabled) {
  background: #1a7f37;
}

.primary-btn:disabled {
  background: #94a3b8;
  cursor: not-allowed;
}

.secondary-btn {
  background: #f6f8fa;
  color: #24292f;
  border: 1px solid #d0d7de;
  padding: 8px 16px;
  border-radius: 6px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
}

.secondary-btn:hover {
  background: #e1e5e9;
  border-color: #afb8c1;
}

.retry-btn {
  background: #0969da;
  color: white;
  border: none;
  padding: 6px 12px;
  border-radius: 4px;
  font-size: 12px;
  cursor: pointer;
  margin-top: 12px;
  transition: background 0.2s;
}

.retry-btn:hover {
  background: #0860ca;
}

.error-actions {
  display: flex;
  gap: 12px;
  justify-content: center;
  margin-top: 12px;
}

.download-page-btn {
  background: #6f42c1;
  color: white;
  border: none;
  padding: 6px 12px;
  border-radius: 4px;
  font-size: 12px;
  cursor: pointer;
  transition: background 0.2s;
}

.download-page-btn:hover:not(:disabled) {
  background: #5a32a3;
}

.download-page-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.manual-download-info {
  margin-top: 16px;
  padding: 12px;
  background: #f8f9fa;
  border-radius: 6px;
  border-left: 4px solid #0969da;
}

.info-text {
  margin: 0 0 8px 0;
  font-size: 13px;
  color: #656d76;
}

.repo-url {
  margin: 0;
  font-family: 'SFMono-Regular', Consolas, 'Liberation Mono', Menlo, monospace;
  font-size: 12px;
  color: #0969da;
  background: white;
  padding: 4px 8px;
  border-radius: 4px;
  border: 1px solid #d0d7de;
}

/* 现代化按钮样式 */
.btn-modern {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 10px 16px;
  border-radius: 8px;
  font-size: 14px;
  font-weight: 500;
  border: 1px solid transparent;
  cursor: pointer;
  transition: all 0.2s ease;
  text-decoration: none;
  outline: none;
  position: relative;
  overflow: hidden;
}

.btn-modern:focus-visible {
  outline: 2px solid #3b82f6;
  outline-offset: 2px;
}

.btn-modern:disabled {
  opacity: 0.5;
  cursor: not-allowed;
  pointer-events: none;
}

.btn-modern.btn-primary {
  background: linear-gradient(135deg, #3b82f6 0%, #2563eb 100%);
  color: white;
  border-color: #2563eb;
  box-shadow: 0 1px 3px rgba(59, 130, 246, 0.3);
}

.btn-modern.btn-primary:hover:not(:disabled) {
  background: linear-gradient(135deg, #2563eb 0%, #1d4ed8 100%);
  box-shadow: 0 4px 12px rgba(59, 130, 246, 0.4);
  transform: translateY(-1px);
}

.btn-modern.btn-secondary {
  background: #f8fafc;
  color: #475569;
  border-color: #e2e8f0;
}

.btn-modern.btn-secondary:hover:not(:disabled) {
  background: #f1f5f9;
  border-color: #cbd5e1;
}

.btn-modern.btn-ghost {
  background: transparent;
  color: #64748b;
  border-color: transparent;
}

.btn-modern.btn-ghost:hover:not(:disabled) {
  background: #f8fafc;
  color: #475569;
}

.btn-modern.btn-destructive {
  background: linear-gradient(135deg, #ef4444 0%, #dc2626 100%);
  color: white;
  border-color: #dc2626;
  box-shadow: 0 1px 3px rgba(239, 68, 68, 0.3);
}

.btn-modern.btn-destructive:hover:not(:disabled) {
  background: linear-gradient(135deg, #dc2626 0%, #b91c1c 100%);
  box-shadow: 0 4px 12px rgba(239, 68, 68, 0.4);
  transform: translateY(-1px);
}

.btn-modern.btn-large {
  padding: 12px 20px;
  font-size: 16px;
}

.btn-modern svg {
  width: 16px;
  height: 16px;
  flex-shrink: 0;
}

/* 现代化对话框动作区域 */
.dialog-actions-modern {
  padding: 20px 24px;
  border-top: 1px solid #e2e8f0;
  background: #f8fafc;
  display: flex;
  gap: 12px;
  justify-content: flex-end;
  align-items: center;
}

/* 现代化错误动作区域 */
.error-actions-modern {
  display: flex;
  gap: 12px;
  justify-content: center;
  margin: 20px 0;
}

/* 现代化信息卡片 */
.info-card {
  margin-top: 20px;
  padding: 16px;
  background: linear-gradient(135deg, #f0f9ff 0%, #e0f2fe 100%);
  border: 1px solid #bae6fd;
  border-radius: 12px;
}

.info-card-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
}

.info-card-title {
  font-size: 14px;
  font-weight: 600;
  color: #0369a1;
}

.info-card-description {
  font-size: 13px;
  color: #0c4a6e;
  margin: 0 0 12px 0;
  line-height: 1.5;
}

.info-card-url {
  background: white;
  padding: 8px 12px;
  border-radius: 6px;
  border: 1px solid #bae6fd;
}

.info-card-url code {
  font-family: 'SFMono-Regular', Consolas, 'Liberation Mono', Menlo, monospace;
  font-size: 12px;
  color: #0369a1;
  background: none;
  padding: 0;
}

/* SVG图标样式 */
.w-4 {
  width: 1rem;
  height: 1rem;
}

.w-5 {
  width: 1.25rem;
  height: 1.25rem;
}

.w-6 {
  width: 1.5rem;
  height: 1.5rem;
}

.h-4 {
  height: 1rem;
}

.h-5 {
  height: 1.25rem;
}

.h-6 {
  height: 1.5rem;
}

.text-blue-500 {
  color: #3b82f6;
}
</style>
