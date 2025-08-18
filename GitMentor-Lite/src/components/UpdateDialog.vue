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
          <div class="loading-spinner">🔄</div>
          <p>正在检查更新...</p>
        </div>

        <!-- 无更新状态 -->
        <div v-else-if="status === 'no-update'" class="status-section">
          <div class="success-icon">✅</div>
          <p>您已使用最新版本 {{ currentVersion }}</p>
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
        <div v-else-if="status === 'error'" class="error-section">
          <div class="error-icon">❌</div>
          <p class="error-message">{{ errorMessage }}</p>
        </div>
      </div>

      <!-- 对话框按钮 -->
      <div class="dialog-actions">
        <button 
          v-if="status === 'update-available'" 
          @click="startDownload" 
          class="primary-btn"
          :disabled="!downloadUrl"
        >
          立即更新
        </button>
        
        <button 
          v-if="status === 'downloading'" 
          @click="cancelDownload" 
          class="secondary-btn"
        >
          取消下载
        </button>
        
        <button 
          v-if="['no-update', 'error'].includes(status)" 
          @click="closeDialog" 
          class="secondary-btn"
        >
          关闭
        </button>
        
        <button 
          v-if="status === 'update-available'" 
          @click="closeDialog" 
          class="secondary-btn"
        >
          稍后更新
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
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
  try {
    status.value = 'checking'
    
    // 获取当前版本
    currentVersion.value = await invoke('get_current_version')
    
    // 检查更新
    const updateInfo = await invoke('check_for_updates') as any
    
    if (updateInfo.has_update) {
      status.value = 'update-available'
      latestVersion.value = updateInfo.latest
      releaseNotes.value = updateInfo.release_notes || ''
      publishedAt.value = updateInfo.published_at || ''
      downloadUrl.value = updateInfo.download_url || ''
    } else {
      status.value = 'no-update'
    }
  } catch (error) {
    status.value = 'error'
    errorMessage.value = error as string
  }
}

const startDownload = async () => {
  if (!downloadUrl.value) {
    errorMessage.value = '下载链接不可用'
    status.value = 'error'
    return
  }

  try {
    status.value = 'downloading'
    isDownloading.value = true
    downloadProgress.value = 0
    downloadedBytes.value = 0
    totalBytes.value = 0

    emit('updateStarted')

    // 开始下载
    const installerPath = await invoke('download_update', {
      downloadUrl: downloadUrl.value
    }) as string

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

// 生命周期
onMounted(async () => {
  if (props.visible) {
    await checkForUpdates()
  }

  // 监听下载进度事件
  const unlisten = await listen('download-progress', (event: any) => {
    const { downloaded, total, percentage } = event.payload
    downloadedBytes.value = downloaded
    totalBytes.value = total
    downloadProgress.value = percentage
  })

  onUnmounted(() => {
    unlisten()
  })
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
}

.status-section {
  text-align: center;
  padding: 20px 0;
}

.loading-spinner {
  font-size: 24px;
  margin-bottom: 12px;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.success-icon, .error-icon {
  font-size: 24px;
  margin-bottom: 12px;
}

.update-info {
  space-y: 16px;
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
</style>
