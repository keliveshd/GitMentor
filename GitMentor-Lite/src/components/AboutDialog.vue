<template>
  <div v-if="visible" class="dialog-overlay" @click="closeDialog">
    <div class="about-dialog" @click.stop>
      <!-- 对话框头部 -->
      <div class="dialog-header">
        <div class="header-content">
          <div class="app-icon">🚀</div>
          <div class="app-title">
            <h2>{{ appInfo.name || 'GitMentor Lite' }}</h2>
            <p class="version">版本 {{ appInfo.version || '0.1.3' }}</p>
          </div>
        </div>
        <button @click="closeDialog" class="close-btn">✕</button>
      </div>

      <!-- 对话框内容 -->
      <div class="dialog-content">
        <!-- 作者信息 -->
        <div class="section">
          <h3>👨‍💻 开发者</h3>
          <p>{{ appInfo.authors || 'Evilek' }}</p>
        </div>

        <!-- Git仓库 -->
        <div class="section">
          <h3>📦 源代码</h3>
          <div class="repo-link-card" @click="openRepository" :class="{ disabled: isOpening }">
            <div class="repo-link-content">
              <div class="repo-link-icon">
                <svg viewBox="0 0 24 24" fill="currentColor">
                  <path
                    d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z" />
                </svg>
              </div>
              <div class="repo-link-info">
                <div class="repo-link-title">GitHub 仓库</div>
                <div class="repo-link-url">{{ (appInfo.repository_url ||
                  'https://github.com/keliveshd/GitMentor').replace('https://', '') }}</div>
              </div>
              <div class="repo-link-arrow">
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                    d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14"></path>
                </svg>
              </div>
            </div>
            <div v-if="isOpening" class="repo-link-loading">
              <div class="loading-spinner-small"></div>
            </div>
          </div>
        </div>

        <!-- 许可证 -->
        <div class="section">
          <h3>📄 许可证</h3>
          <p>{{ appInfo.license || 'GPL-3.0' }}</p>
        </div>

        <!-- 更新信息 -->
        <div class="section">
          <h3>🔄 更新</h3>
          <p>如需获取最新版本，请访问上述Git仓库的Releases页面。</p>
        </div>
      </div>

      <!-- 对话框底部 -->
      <div class="dialog-footer-modern">
        <button @click="closeDialog" class="btn-modern btn-secondary">
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path>
          </svg>
          关闭
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

/**
 * 关于对话框组件
 * 作者：Evilek
 * 编写日期：2025-08-21
 */

// Props
interface Props {
  visible: boolean
}

const props = defineProps<Props>()

// Emits
const emit = defineEmits<{
  close: []
}>()

// 响应式数据
const appInfo = ref({
  name: '',
  version: '',
  description: '',
  authors: '',
  repository_url: '',
  license: ''
})

const isOpening = ref(false)

// 方法
const closeDialog = () => {
  emit('close')
}

const openRepository = async () => {
  if (isOpening.value) return

  try {
    isOpening.value = true
    const repoUrl = appInfo.value.repository_url || 'https://github.com/keliveshd/GitMentor'

    await invoke('open_browser_url', { url: repoUrl })
    console.log('✅ [AboutDialog] 成功打开仓库链接:', repoUrl)
  } catch (error) {
    console.error('❌ [AboutDialog] 打开仓库链接失败:', error)
    alert(`打开仓库链接失败: ${error}`)
  } finally {
    isOpening.value = false
  }
}

const loadAppInfo = async () => {
  try {
    const info = await invoke('get_app_info') as any
    appInfo.value = info
    console.log('✅ [AboutDialog] 应用信息加载成功:', info)
  } catch (error) {
    console.error('❌ [AboutDialog] 加载应用信息失败:', error)
    // 使用默认值
  }
}

// 生命周期
onMounted(() => {
  loadAppInfo()
})
</script>

<style scoped>
.dialog-overlay {
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

.about-dialog {
  background: white;
  border-radius: 12px;
  width: 500px;
  max-width: 90vw;
  max-height: 80vh;
  overflow: hidden;
  box-shadow: 0 10px 30px rgba(0, 0, 0, 0.3);
}

.dialog-header {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  padding: 20px;
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.header-content {
  display: flex;
  align-items: center;
  gap: 15px;
}

.app-icon {
  font-size: 48px;
}

.app-title h2 {
  margin: 0;
  font-size: 24px;
  font-weight: 600;
}

.version {
  margin: 5px 0 0 0;
  opacity: 0.9;
  font-size: 14px;
}

.close-btn {
  background: none;
  border: none;
  color: white;
  font-size: 20px;
  cursor: pointer;
  padding: 5px;
  border-radius: 4px;
  transition: background-color 0.2s;
}

.close-btn:hover {
  background: rgba(255, 255, 255, 0.2);
}

.dialog-content {
  padding: 20px;
  max-height: 400px;
  overflow-y: auto;
}

.section {
  margin-bottom: 20px;
}

.section h3 {
  margin: 0 0 10px 0;
  font-size: 16px;
  color: #333;
}

.section p {
  margin: 0;
  color: #666;
  line-height: 1.5;
}



/* 现代化仓库链接卡片 */
.repo-link-card {
  background: linear-gradient(135deg, #f8fafc 0%, #f1f5f9 100%);
  border: 1px solid #e2e8f0;
  border-radius: 12px;
  padding: 0;
  cursor: pointer;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  position: relative;
  overflow: hidden;
}

.repo-link-card:hover:not(.disabled) {
  background: linear-gradient(135deg, #f1f5f9 0%, #e2e8f0 100%);
  border-color: #cbd5e1;
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}

.repo-link-card.disabled {
  opacity: 0.7;
  cursor: not-allowed;
}

.repo-link-content {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 14px 16px;
}

.repo-link-icon {
  width: 24px;
  height: 24px;
  color: #1f2937;
  flex-shrink: 0;
}

.repo-link-icon svg {
  width: 100%;
  height: 100%;
}

.repo-link-info {
  flex: 1;
  min-width: 0;
}

.repo-link-title {
  font-size: 13px;
  font-weight: 600;
  color: #1f2937;
  margin-bottom: 2px;
}

.repo-link-url {
  font-family: 'SFMono-Regular', Consolas, 'Liberation Mono', Menlo, monospace;
  font-size: 11px;
  color: #6b7280;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.repo-link-arrow {
  color: #9ca3af;
  flex-shrink: 0;
  transition: all 0.2s ease;
}

.repo-link-card:hover:not(.disabled) .repo-link-arrow {
  color: #6b7280;
  transform: translateX(2px);
}

.repo-link-loading {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(248, 250, 252, 0.9);
  display: flex;
  align-items: center;
  justify-content: center;
}

.loading-spinner-small {
  width: 16px;
  height: 16px;
  border: 2px solid #e5e7eb;
  border-top: 2px solid #3b82f6;
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from {
    transform: rotate(0deg);
  }

  to {
    transform: rotate(360deg);
  }
}

/* 现代化对话框底部 */
.dialog-footer-modern {
  padding: 20px 24px;
  border-top: 1px solid #e2e8f0;
  background: #f8fafc;
  display: flex;
  justify-content: flex-end;
}

/* 现代化按钮样式 - 复用UpdateDialog的样式 */
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

.btn-modern.btn-small {
  padding: 6px 12px;
  font-size: 12px;
}

.btn-modern svg {
  width: 16px;
  height: 16px;
  flex-shrink: 0;
}

/* SVG图标样式 */
.w-3 {
  width: 0.75rem;
  height: 0.75rem;
}

.w-4 {
  width: 1rem;
  height: 1rem;
}

.w-5 {
  width: 1.25rem;
  height: 1.25rem;
}

.h-3 {
  height: 0.75rem;
}

.h-4 {
  height: 1rem;
}

.h-5 {
  height: 1.25rem;
}
</style>
