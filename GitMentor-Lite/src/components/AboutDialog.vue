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
        <!-- 应用描述 -->
        <div class="section">
          <h3>📝 应用简介</h3>
          <p>{{ appInfo.description || 'AI驱动的Git提交信息生成工具，让代码提交更规范、更高效。' }}</p>
        </div>

        <!-- 作者信息 -->
        <div class="section">
          <h3>👨‍💻 开发者</h3>
          <p>{{ appInfo.authors || 'Evilek' }}</p>
        </div>

        <!-- 技术栈 -->
        <div class="section">
          <h3>🛠️ 技术栈</h3>
          <div class="tech-stack">
            <span class="tech-tag">Tauri v2</span>
            <span class="tech-tag">Vue 3</span>
            <span class="tech-tag">TypeScript</span>
            <span class="tech-tag">Rust</span>
            <span class="tech-tag">Git2</span>
          </div>
        </div>

        <!-- Git仓库 -->
        <div class="section">
          <h3>📦 源代码</h3>
          <div class="repo-card">
            <div class="repo-card-content">
              <div class="repo-icon">
                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                    d="M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4"></path>
                </svg>
              </div>
              <div class="repo-details">
                <p class="repo-url">{{ appInfo.repository_url || 'https://github.com/keliveshd/GitMentor' }}</p>
                <p class="repo-description">在GitHub上查看源代码、提交问题或贡献代码</p>
              </div>
            </div>
            <button @click="openRepository" class="btn-modern btn-primary" :disabled="isOpening">
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                  d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14"></path>
              </svg>
              {{ isOpening ? '打开中...' : '访问仓库' }}
            </button>
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

.tech-stack {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.tech-tag {
  background: #f0f0f0;
  padding: 4px 8px;
  border-radius: 4px;
  font-size: 12px;
  color: #666;
}

/* 现代化仓库卡片 */
.repo-card {
  background: linear-gradient(135deg, #f8fafc 0%, #f1f5f9 100%);
  border: 1px solid #e2e8f0;
  border-radius: 12px;
  padding: 16px;
}

.repo-card-content {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  margin-bottom: 16px;
}

.repo-icon {
  width: 40px;
  height: 40px;
  background: linear-gradient(135deg, #3b82f6 0%, #2563eb 100%);
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  flex-shrink: 0;
}

.repo-details {
  flex: 1;
}

.repo-url {
  font-family: 'SFMono-Regular', Consolas, 'Liberation Mono', Menlo, monospace;
  font-size: 13px;
  color: #475569;
  margin: 0 0 4px 0;
  word-break: break-all;
}

.repo-description {
  font-size: 12px;
  color: #64748b;
  margin: 0;
  line-height: 1.4;
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

.btn-modern svg {
  width: 16px;
  height: 16px;
  flex-shrink: 0;
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

.h-4 {
  height: 1rem;
}

.h-5 {
  height: 1.25rem;
}
</style>
