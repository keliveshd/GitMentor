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
        <!-- 差异导航 -->
        <div class="diff-navigation" v-if="diffData && !diffData.is_binary">
          <button @click="goToPreviousDiff" class="control-btn nav-btn" :disabled="currentDiffIndex <= 0" title="上一个差异">
            ↑
          </button>
          <span class="diff-counter" v-if="totalDiffs > 0">
            {{ currentDiffIndex + 1 }} / {{ totalDiffs }}
          </span>
          <button @click="goToNextDiff" class="control-btn nav-btn" :disabled="currentDiffIndex >= totalDiffs - 1"
            title="下一个差异">
            ↓
          </button>
        </div>

        <!-- 视图控制 -->
        <button @click="toggleMode" class="control-btn" :title="isUnified ? '切换到并排视图' : '切换到统一视图'">
          {{ isUnified ? '📄' : '📋' }}
        </button>
        <button @click="toggleWrap" class="control-btn" :title="wrapLines ? '禁用换行' : '启用换行'">
          {{ wrapLines ? '📏' : '📐' }}
        </button>
        <button @click="toggleCollapse" class="control-btn" :title="collapseUnchanged ? '展开相同代码' : '折叠相同代码'">
          {{ collapseUnchanged ? '📖' : '📕' }}
        </button>
        <button @click="closeViewer" class="control-btn close-btn" title="关闭">
          ✕
        </button>
      </div>
    </div>



    <!-- 差异内容 -->
    <div class="diff-content" v-if="diffData && !diffData.is_binary && hasValidContent">
      <DiffView :data="diffViewData" :diff-view-mode="diffMode" :diff-view-theme="'light'" :diff-view-highlight="true"
        :diff-view-wrap="wrapLines" :diff-view-font-size="14" :diff-view-add-widget="false"
        @error="handleDiffViewError" />
    </div>

    <!-- 无差异内容提示 -->
    <div v-else-if="diffData && !diffData.is_binary && !hasValidContent" class="no-diff">
      <div class="no-diff-content">
        <span class="no-diff-icon">📄</span>
        <h4>没有差异</h4>
        <p>此文件没有检测到任何更改</p>
      </div>
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

// 类型定义
/**
 * 文件差异结果类型
 * 作者：Evilek
 * 编写日期：2025-07-22
 */
interface FileDiffResult {
  file_path: string
  old_content?: string
  new_content?: string
  old_file_name?: string
  new_file_name?: string
  file_language?: string
  hunks: Array<{
    old_start: number
    old_lines: number
    new_start: number
    new_lines: number
    lines: Array<{
      line_type: 'Context' | 'Delete' | 'Insert'
      content: string
      old_line_number?: number
      new_line_number?: number
    }>
  }>
  is_binary: boolean
  is_new_file: boolean
  is_deleted_file: boolean
}

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
const diffData = ref<FileDiffResult | null>(null)
const loading = ref(false)
const error = ref<string | null>(null)
const isUnified = ref(false)
const wrapLines = ref(false)
const collapseUnchanged = ref(false)
const currentDiffIndex = ref(0)

// 计算属性
const diffMode = computed(() =>
  isUnified.value ? DiffModeEnum.Unified : DiffModeEnum.Split
)

/**
 * 检查是否有有效的差异内容
 * 作者：Evilek
 * 编写日期：2025-07-22
 */
const hasValidContent = computed(() => {
  if (!diffData.value) return false

  // 检查是否有内容差异
  const hasContentDiff = diffData.value.old_content !== diffData.value.new_content

  // 检查是否有hunks
  const hasHunks = diffData.value.hunks && diffData.value.hunks.length > 0

  return hasContentDiff || hasHunks
})

/**
 * 计算总差异数量
 * 作者：Evilek
 * 编写日期：2025-07-22
 */
const totalDiffs = computed(() => {
  if (!diffData.value?.hunks) return 0
  return diffData.value.hunks.length
})

/**
 * 转换数据为DiffView组件所需格式
 * 作者：Evilek
 * 编写日期：2025-07-22
 */
const diffViewData = computed(() => {
  if (!diffData.value || !hasValidContent.value) {
    return null
  }

  try {
    // 直接使用文件内容，让@git-diff-view/vue库自动计算差异
    const result = {
      oldFile: {
        fileName: diffData.value.old_file_name || diffData.value.file_path,
        content: diffData.value.old_content || '',
        fileLang: diffData.value.file_language || ''
      },
      newFile: {
        fileName: diffData.value.new_file_name || diffData.value.file_path,
        content: diffData.value.new_content || '',
        fileLang: diffData.value.file_language || ''
      },
      hunks: [] // 让库自动生成hunks
    }

    return result
  } catch (error) {
    console.error('❌ [DiffViewer] 处理diff数据失败:', error)
    return null
  }
})

// 方法
/**
 * 加载文件差异数据
 * 作者：Evilek
 * 编写日期：2025-07-22
 */
const loadDiff = async () => {
  try {
    loading.value = true
    error.value = null

    const result = await invoke('get_file_diff', {
      request: {
        file_path: props.filePath,
        diff_type: props.diffType
      }
    }) as FileDiffResult

    diffData.value = result
  } catch (err) {
    error.value = err instanceof Error ? err.message : '未知错误'
    console.error('❌ [DiffViewer] 加载差异失败:', err)
  } finally {
    loading.value = false
  }
}

/**
 * 跳转到上一个差异
 * 作者：Evilek
 * 编写日期：2025-07-22
 */
const goToPreviousDiff = () => {
  if (currentDiffIndex.value > 0) {
    currentDiffIndex.value--
    // TODO: 实现滚动到对应差异位置的逻辑
  }
}

/**
 * 跳转到下一个差异
 * 作者：Evilek
 * 编写日期：2025-07-22
 */
const goToNextDiff = () => {
  if (currentDiffIndex.value < totalDiffs.value - 1) {
    currentDiffIndex.value++
    // TODO: 实现滚动到对应差异位置的逻辑
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

const handleDiffViewError = (error: any) => {
  console.error('❌ [DiffViewer] DiffView组件渲染错误:', error)
  error.value = 'DiffView组件渲染失败: ' + (error?.message || '未知错误')
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
  from {
    transform: rotate(0deg);
  }

  to {
    transform: rotate(360deg);
  }
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
