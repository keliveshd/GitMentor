<template>
  <div class="diff-viewer">
    <!-- 头部工具栏 -->
    <div class="diff-header">
      <div class="file-info">
        <h3>{{ diffData?.file_path || '文件差异' }}</h3>
        <div class="file-meta" v-if="diffData">
          <span v-if="diffData.is_new_file" class="badge new">新文件</span>
          <span v-if="diffData.is_deleted_file" class="badge deleted">已删除</span>
          <span v-if="diffData.is_binary" class="badge binary">二进制文件</span>
          <span v-if="diffData.file_language" class="language">{{ diffData.file_language }}</span>
        </div>
      </div>
      
      <div class="diff-controls">
        <button @click="toggleMode" class="control-btn" :title="isUnified ? '切换到并排视图' : '切换到统一视图'">
          {{ isUnified ? '📄' : '📋' }}
        </button>
        <button @click="toggleWrap" class="control-btn" :title="wrapLines ? '禁用换行' : '启用换行'">
          {{ wrapLines ? '📏' : '📐' }}
        </button>
        <button @click="closeViewer" class="control-btn close-btn" title="关闭">
          ✕
        </button>
      </div>
    </div>

    <!-- 差异内容 -->
    <div class="diff-content" v-if="diffData && !diffData.is_binary && diffViewData">
      <DiffView
        :data="diffViewData"
        :diff-view-mode="diffMode"
        :diff-view-theme="'light'"
        :diff-view-highlight="true"
        :diff-view-wrap="wrapLines"
        :diff-view-font-size="14"
      />
    </div>

    <!-- 二进制文件提示 -->
    <div class="binary-notice" v-else-if="diffData?.is_binary">
      <div class="notice-content">
        <span class="notice-icon">📁</span>
        <h4>二进制文件</h4>
        <p>无法显示二进制文件的差异内容</p>
      </div>
    </div>

    <!-- 加载状态 -->
    <div class="loading" v-else-if="loading">
      <div class="loading-content">
        <span class="loading-spinner">⏳</span>
        <p>正在加载差异...</p>
      </div>
    </div>

    <!-- 错误状态 -->
    <div class="error" v-else-if="error">
      <div class="error-content">
        <span class="error-icon">❌</span>
        <h4>加载失败</h4>
        <p>{{ error }}</p>
        <button @click="retry" class="retry-btn">重试</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { DiffView, DiffModeEnum } from '@git-diff-view/vue'
import '@git-diff-view/vue/styles/diff-view.css'

// Props
interface Props {
  filePath: string
  diffType?: 'WorkingTree' | 'Staged' | 'HeadToWorking'
}

const props = withDefaults(defineProps<Props>(), {
  diffType: 'WorkingTree'
})

// Emits
const emit = defineEmits<{
  close: []
}>()

// 响应式数据
const diffData = ref<any>(null)
const loading = ref(false)
const error = ref<string | null>(null)
const isUnified = ref(false)
const wrapLines = ref(false)

// 计算属性
const diffMode = computed(() => 
  isUnified.value ? DiffModeEnum.Unified : DiffModeEnum.Split
)

const diffViewData = computed(() => {
  if (!diffData.value) return null
  
  return {
    oldFile: {
      fileName: diffData.value.old_file_name,
      content: diffData.value.old_content || '',
      fileLang: diffData.value.file_language
    },
    newFile: {
      fileName: diffData.value.new_file_name,
      content: diffData.value.new_content || '',
      fileLang: diffData.value.file_language
    },
    hunks: diffData.value.diff_hunks || []
  }
})

// 方法
const loadDiff = async () => {
  try {
    loading.value = true
    error.value = null
    
    const result = await invoke('get_file_diff', {
      request: {
        file_path: props.filePath,
        diff_type: props.diffType
      }
    })
    
    diffData.value = result
  } catch (err) {
    error.value = err instanceof Error ? err.message : '未知错误'
    console.error('Failed to load diff:', err)
  } finally {
    loading.value = false
  }
}

const toggleMode = () => {
  isUnified.value = !isUnified.value
}

const toggleWrap = () => {
  wrapLines.value = !wrapLines.value
}

const closeViewer = () => {
  emit('close')
}

const retry = () => {
  loadDiff()
}

// 生命周期
onMounted(() => {
  loadDiff()
})
</script>

<style scoped>
.diff-viewer {
  height: 100vh;
  display: flex;
  flex-direction: column;
  background: #ffffff;
  font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
}

.diff-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  border-bottom: 1px solid #e1e4e8;
  background: #f6f8fa;
  min-height: 60px;
}

.file-info h3 {
  margin: 0 0 4px 0;
  font-size: 16px;
  font-weight: 600;
  color: #24292e;
}

.file-meta {
  display: flex;
  gap: 8px;
  align-items: center;
}

.badge {
  padding: 2px 6px;
  border-radius: 3px;
  font-size: 11px;
  font-weight: 500;
  text-transform: uppercase;
}

.badge.new {
  background: #28a745;
  color: white;
}

.badge.deleted {
  background: #d73a49;
  color: white;
}

.badge.binary {
  background: #6f42c1;
  color: white;
}

.language {
  font-size: 12px;
  color: #586069;
  font-weight: 500;
}

.diff-controls {
  display: flex;
  gap: 8px;
}

.control-btn {
  padding: 6px 10px;
  border: 1px solid #d1d5da;
  border-radius: 4px;
  background: white;
  cursor: pointer;
  font-size: 14px;
  transition: all 0.2s;
}

.control-btn:hover {
  background: #f6f8fa;
  border-color: #c6cbd1;
}

.close-btn {
  color: #d73a49;
  border-color: #d73a49;
}

.close-btn:hover {
  background: #d73a49;
  color: white;
}

.diff-content {
  flex: 1;
  overflow: hidden;
}

.binary-notice,
.loading,
.error {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 40px;
}

.notice-content,
.loading-content,
.error-content {
  text-align: center;
  max-width: 400px;
}

.notice-icon,
.loading-spinner,
.error-icon {
  font-size: 48px;
  display: block;
  margin-bottom: 16px;
}

.loading-spinner {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.notice-content h4,
.error-content h4 {
  margin: 0 0 8px 0;
  font-size: 18px;
  color: #24292e;
}

.notice-content p,
.loading-content p,
.error-content p {
  margin: 0 0 16px 0;
  color: #586069;
  line-height: 1.5;
}

.retry-btn {
  padding: 8px 16px;
  border: 1px solid #0366d6;
  border-radius: 4px;
  background: #0366d6;
  color: white;
  cursor: pointer;
  font-size: 14px;
  transition: background 0.2s;
}

.retry-btn:hover {
  background: #0256cc;
}
</style>
