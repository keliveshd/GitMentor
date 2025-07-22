<template>
  <div class="enhanced-diff-viewer">
    <!-- 头部工具栏 -->
    <div class="diff-header">
      <div class="file-info">
        <h3>{{ diffData?.file_path || '文件差异' }}</h3>
        <div class="file-meta" v-if="diffData">
          <span v-if="diffData.is_new_file" class="badge new">新文件</span>
          <span v-if="diffData.is_deleted_file" class="badge deleted">已删除</span>
          <span v-if="diffData.is_binary" class="badge binary">二进制文件</span>
          <span v-if="diffData.file_language" class="language">{{ diffData.file_language }}</span>
          <span class="stats" v-if="diffStats">
            <span class="additions">+{{ diffStats.additions }}</span>
            <span class="deletions">-{{ diffStats.deletions }}</span>
          </span>
        </div>
      </div>

      <div class="diff-controls">
        <!-- 差异导航 -->
        <div class="diff-navigation" v-if="diffFile && !diffData?.is_binary">
          <button @click="goToPreviousDiff" class="control-btn nav-btn" :disabled="currentDiffIndex <= 0"
            title="上一个差异 (Alt+↑)">
            ↑
          </button>
          <span class="diff-counter" v-if="totalDiffs > 0">
            {{ currentDiffIndex + 1 }} / {{ totalDiffs }}
          </span>
          <button @click="goToNextDiff" class="control-btn nav-btn" :disabled="currentDiffIndex >= totalDiffs - 1"
            title="下一个差异 (Alt+↓)">
            ↓
          </button>
        </div>

        <!-- 视图控制 -->
        <div class="view-controls">
          <button @click="toggleMode" class="control-btn" :title="isUnified ? '切换到并排视图' : '切换到统一视图'">
            {{ isUnified ? '📄' : '📋' }}
          </button>
          <button @click="toggleWrap" class="control-btn" :title="wrapLines ? '禁用换行' : '启用换行'">
            {{ wrapLines ? '📏' : '📐' }}
          </button>
          <button @click="toggleIgnoreWhitespace" class="control-btn"
            :title="ignoreWhitespace ? '显示空白字符差异' : '忽略空白字符差异'">
            {{ ignoreWhitespace ? '🔍' : '👁️' }}
          </button>
          <button @click="toggleWhitespace" class="control-btn" :title="showWhitespace ? '隐藏空白字符' : '显示空白字符'">
            {{ showWhitespace ? '⚪' : '⚫' }}
          </button>
          <button @click="toggleSyntaxHighlight" class="control-btn" :title="syntaxHighlight ? '禁用语法高亮' : '启用语法高亮'">
            {{ syntaxHighlight ? '🎨' : '🔤' }}
          </button>
        </div>

        <!-- 操作按钮 -->
        <div class="action-controls">
          <button @click="copyDiff" class="control-btn" title="复制差异">
            📋
          </button>
          <button @click="downloadDiff" class="control-btn" title="下载差异文件">
            💾
          </button>
          <button @click="closeViewer" class="control-btn close-btn" title="关闭 (Esc)">
            ✕
          </button>
        </div>
      </div>
    </div>

    <!-- 差异内容 -->
    <div class="diff-content" v-if="diffFile && !diffData?.is_binary">
      <DiffView :diffFile="diffFile" :diff-view-mode="diffMode" :diff-view-theme="theme"
        :diff-view-highlight="syntaxHighlight" :diff-view-wrap="wrapLines" :diff-view-font-size="fontSize"
        :diff-view-add-widget="false" @error="handleDiffViewError" />
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
        <div class="binary-info" v-if="diffData">
          <p>文件路径: {{ diffData.file_path }}</p>
          <p>文件大小: {{ formatFileSize(diffData.old_content?.length || 0) }} → {{
            formatFileSize(diffData.new_content?.length || 0) }}</p>
        </div>
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
import { ref, computed, onMounted, onUnmounted, nextTick } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { DiffView, DiffModeEnum } from '@git-diff-view/vue'
import { DiffFile, generateDiffFile } from '@git-diff-view/file'
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

/**
 * 差异统计信息
 * 作者：Evilek
 * 编写日期：2025-07-22
 */
interface DiffStats {
  additions: number
  deletions: number
  changes: number
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
const diffFile = ref<DiffFile | null>(null)
const loading = ref(false)
const error = ref<string | null>(null)
const isUnified = ref(false)
const wrapLines = ref(false)
const showWhitespace = ref(false)
const ignoreWhitespace = ref(true) // 默认启用忽略空白字符
const syntaxHighlight = ref(true)
const currentDiffIndex = ref(0)
const fontSize = ref(14)
const theme = ref<'light' | 'dark'>('light')

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
 * 计算差异统计信息
 * 作者：Evilek
 * 编写日期：2025-07-22
 */
const diffStats = computed((): DiffStats | null => {
  if (!diffData.value?.hunks) return null

  let additions = 0
  let deletions = 0
  let changes = 0

  diffData.value.hunks.forEach(hunk => {
    hunk.lines.forEach(line => {
      if (line.line_type === 'Insert') {
        additions++
      } else if (line.line_type === 'Delete') {
        deletions++
      }
    })
    changes++
  })

  return { additions, deletions, changes }
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

    // 使用@git-diff-view/file库生成DiffFile对象
    if (result && !result.is_binary && hasValidContent.value) {
      await nextTick()

      // 根据设置决定是否标准化内容
      let oldContent = result.old_content || ''
      let newContent = result.new_content || ''

      if (ignoreWhitespace.value) {
        console.log('🔧 [EnhancedDiffViewer] 标准化文件内容以忽略空白字符差异')

        const normalizeContent = (content: string): string => {
          return content
            // 统一换行符为 \n
            .replace(/\r\n/g, '\n')
            .replace(/\r/g, '\n')
            // 移除行尾空白字符
            .replace(/[ \t]+$/gm, '')
            // 移除文件末尾的多余空行
            .replace(/\n+$/, '\n')
        }

        const originalOldLength = oldContent.length
        const originalNewLength = newContent.length

        oldContent = normalizeContent(oldContent)
        newContent = normalizeContent(newContent)

        console.log('📊 [EnhancedDiffViewer] 内容标准化结果:', {
          old: { original: originalOldLength, normalized: oldContent.length },
          new: { original: originalNewLength, normalized: newContent.length }
        })
      }

      const file = generateDiffFile(
        result.old_file_name || result.file_path,
        oldContent,
        result.new_file_name || result.file_path,
        newContent,
        result.file_language || '',
        result.file_language || ''
      )

      file.initTheme(theme.value)
      file.init()

      if (isUnified.value) {
        file.buildUnifiedDiffLines()
      } else {
        file.buildSplitDiffLines()
      }

      diffFile.value = file
    }
  } catch (err) {
    error.value = err instanceof Error ? err.message : '未知错误'
    console.error('❌ [EnhancedDiffViewer] 加载差异失败:', err)
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
    scrollToDiff(currentDiffIndex.value)
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
    scrollToDiff(currentDiffIndex.value)
  }
}

/**
 * 滚动到指定差异位置
 * 作者：Evilek
 * 编写日期：2025-07-22
 */
const scrollToDiff = (index: number) => {
  // TODO: 实现滚动到对应差异位置的逻辑
  console.log('滚动到差异:', index)
}

/**
 * 切换视图模式（并排/统一）
 * 作者：Evilek
 * 编写日期：2025-07-22
 */
const toggleMode = async () => {
  isUnified.value = !isUnified.value

  if (diffFile.value) {
    if (isUnified.value) {
      diffFile.value.buildUnifiedDiffLines()
    } else {
      diffFile.value.buildSplitDiffLines()
    }
  }
}

/**
 * 切换换行模式
 * 作者：Evilek
 * 编写日期：2025-07-22
 */
const toggleWrap = () => {
  wrapLines.value = !wrapLines.value
}

/**
 * 切换忽略空白字符模式
 * 作者：Evilek
 * 编写日期：2025-07-22
 */
const toggleIgnoreWhitespace = () => {
  ignoreWhitespace.value = !ignoreWhitespace.value
}

/**
 * 切换空白字符显示
 * 作者：Evilek
 * 编写日期：2025-07-22
 */
const toggleWhitespace = () => {
  showWhitespace.value = !showWhitespace.value
}

/**
 * 切换语法高亮
 * 作者：Evilek
 * 编写日期：2025-07-22
 */
const toggleSyntaxHighlight = () => {
  syntaxHighlight.value = !syntaxHighlight.value
}

/**
 * 复制差异内容
 * 作者：Evilek
 * 编写日期：2025-07-22
 */
const copyDiff = async () => {
  if (!diffData.value) return

  try {
    // 生成Git diff格式的文本
    let diffText = `diff --git a/${diffData.value.file_path} b/${diffData.value.file_path}\n`

    if (diffData.value.is_new_file) {
      diffText += 'new file mode 100644\n'
    } else if (diffData.value.is_deleted_file) {
      diffText += 'deleted file mode 100644\n'
    }

    diffText += `--- a/${diffData.value.old_file_name || diffData.value.file_path}\n`
    diffText += `+++ b/${diffData.value.new_file_name || diffData.value.file_path}\n`

    // 添加hunks
    diffData.value.hunks.forEach(hunk => {
      diffText += `@@ -${hunk.old_start},${hunk.old_lines} +${hunk.new_start},${hunk.new_lines} @@\n`
      hunk.lines.forEach(line => {
        let prefix = ' '
        if (line.line_type === 'Delete') prefix = '-'
        else if (line.line_type === 'Insert') prefix = '+'
        diffText += prefix + line.content + '\n'
      })
    })

    await navigator.clipboard.writeText(diffText)
    console.log('差异内容已复制到剪贴板')
  } catch (err) {
    console.error('复制失败:', err)
  }
}

/**
 * 下载差异文件
 * 作者：Evilek
 * 编写日期：2025-07-22
 */
const downloadDiff = () => {
  if (!diffData.value) return

  try {
    // 生成Git diff格式的文本
    let diffText = `diff --git a/${diffData.value.file_path} b/${diffData.value.file_path}\n`

    if (diffData.value.is_new_file) {
      diffText += 'new file mode 100644\n'
    } else if (diffData.value.is_deleted_file) {
      diffText += 'deleted file mode 100644\n'
    }

    diffText += `--- a/${diffData.value.old_file_name || diffData.value.file_path}\n`
    diffText += `+++ b/${diffData.value.new_file_name || diffData.value.file_path}\n`

    // 添加hunks
    diffData.value.hunks.forEach(hunk => {
      diffText += `@@ -${hunk.old_start},${hunk.old_lines} +${hunk.new_start},${hunk.new_lines} @@\n`
      hunk.lines.forEach(line => {
        let prefix = ' '
        if (line.line_type === 'Delete') prefix = '-'
        else if (line.line_type === 'Insert') prefix = '+'
        diffText += prefix + line.content + '\n'
      })
    })

    // 创建下载链接
    const blob = new Blob([diffText], { type: 'text/plain' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = `${diffData.value.file_path.replace(/[/\\]/g, '_')}.diff`
    document.body.appendChild(a)
    a.click()
    document.body.removeChild(a)
    URL.revokeObjectURL(url)
  } catch (err) {
    console.error('下载失败:', err)
  }
}

/**
 * 格式化文件大小
 * 作者：Evilek
 * 编写日期：2025-07-22
 */
const formatFileSize = (bytes: number): string => {
  if (bytes === 0) return '0 B'
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
}

/**
 * 关闭差异查看器
 * 作者：Evilek
 * 编写日期：2025-07-22
 */
const closeViewer = () => {
  emit('close')
}

/**
 * 重试加载差异
 * 作者：Evilek
 * 编写日期：2025-07-22
 */
const retry = () => {
  loadDiff()
}

/**
 * 处理DiffView组件错误
 * 作者：Evilek
 * 编写日期：2025-07-22
 */
const handleDiffViewError = (errorEvent: any) => {
  console.error('❌ [EnhancedDiffViewer] DiffView组件渲染错误:', errorEvent)
  error.value = 'DiffView组件渲染失败: ' + (errorEvent?.message || '未知错误')
}

/**
 * 处理键盘快捷键
 * 作者：Evilek
 * 编写日期：2025-07-22
 */
const handleKeydown = (event: KeyboardEvent) => {
  if (event.altKey && event.key === 'ArrowUp') {
    event.preventDefault()
    goToPreviousDiff()
  } else if (event.altKey && event.key === 'ArrowDown') {
    event.preventDefault()
    goToNextDiff()
  } else if (event.key === 'Escape') {
    event.preventDefault()
    closeViewer()
  }
}

// 生命周期
onMounted(() => {
  loadDiff()
  document.addEventListener('keydown', handleKeydown)
})

onUnmounted(() => {
  document.removeEventListener('keydown', handleKeydown)
})
</script>

<style scoped>
.enhanced-diff-viewer {
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
  flex-wrap: wrap;
  gap: 12px;
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
  flex-wrap: wrap;
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

.stats {
  display: flex;
  gap: 8px;
  font-size: 12px;
  font-weight: 500;
}

.additions {
  color: #28a745;
}

.deletions {
  color: #d73a49;
}

.diff-controls {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-wrap: wrap;
}

.diff-navigation {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 8px;
  background: #f6f8fa;
  border: 1px solid #d1d5da;
  border-radius: 6px;
}

.diff-counter {
  font-size: 12px;
  color: #586069;
  font-weight: 500;
  min-width: 40px;
  text-align: center;
}

.nav-btn {
  padding: 4px 8px !important;
  font-size: 12px !important;
  min-width: 24px;
}

.nav-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
  background: #f6f8fa !important;
}

.view-controls,
.action-controls {
  display: flex;
  gap: 6px;
}

.control-btn {
  padding: 6px 10px;
  border: 1px solid #d1d5da;
  border-radius: 4px;
  background: white;
  cursor: pointer;
  font-size: 14px;
  transition: all 0.2s;
  white-space: nowrap;
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

.no-diff,
.binary-notice,
.loading,
.error {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 40px;
}

.no-diff-content,
.notice-content,
.loading-content,
.error-content {
  text-align: center;
  max-width: 400px;
}

.no-diff-icon,
.notice-icon,
.loading-spinner,
.error-icon {
  font-size: 48px;
  display: block;
  margin-bottom: 16px;
  opacity: 0.6;
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

.binary-info {
  margin-top: 16px;
  padding: 12px;
  background: #f6f8fa;
  border-radius: 6px;
  font-size: 12px;
}

.binary-info p {
  margin: 4px 0;
  color: #586069;
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

/* 响应式设计 */
@media (max-width: 768px) {
  .diff-header {
    flex-direction: column;
    align-items: stretch;
  }

  .diff-controls {
    justify-content: center;
  }

  .view-controls,
  .action-controls {
    flex-wrap: wrap;
    justify-content: center;
  }
}
</style>
