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
          <div class="repo-info">
            <span class="repo-url">{{ appInfo.repository_url || 'https://github.com/keliveshd/GitMentor' }}</span>
            <button @click="openRepository" class="open-repo-btn" :disabled="isOpening">
              {{ isOpening ? '打开中...' : '🔗 访问仓库' }}
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
      <div class="dialog-footer">
        <button @click="closeDialog" class="close-button">关闭</button>
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

.repo-info {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
}

.repo-url {
  color: #666;
  font-family: monospace;
  font-size: 14px;
  flex: 1;
  min-width: 200px;
}

.open-repo-btn {
  background: #667eea;
  color: white;
  border: none;
  padding: 6px 12px;
  border-radius: 4px;
  cursor: pointer;
  font-size: 12px;
  transition: background-color 0.2s;
}

.open-repo-btn:hover:not(:disabled) {
  background: #5a6fd8;
}

.open-repo-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.dialog-footer {
  padding: 15px 20px;
  border-top: 1px solid #eee;
  text-align: right;
}

.close-button {
  background: #667eea;
  color: white;
  border: none;
  padding: 8px 20px;
  border-radius: 4px;
  cursor: pointer;
  font-size: 14px;
  transition: background-color 0.2s;
}

.close-button:hover {
  background: #5a6fd8;
}
</style>
