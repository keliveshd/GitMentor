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
        <button @click="toggleIgnoreWhitespace" class="control-btn" :title="ignoreWhitespace ? '显示空白字符差异' : '忽略空白字符差异'">
          {{ ignoreWhitespace ? '🔍' : '👁️' }}
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
      <!-- 调试信息 -->
      <div class="debug-info"
        style="padding: 10px; background: #f0f8ff; border: 1px solid #ccc; margin-bottom: 10px; font-size: 12px;">
        <details>
          <summary><strong>🔍 DiffView调试信息</strong></summary>
          <div style="margin-top: 8px;">
            <p><strong>使用方法:</strong> {{ diffFile ? 'diffFile模式' : 'data模式' }}</p>
            <p><strong>忽略空白字符:</strong> {{ ignoreWhitespace ? '是' : '否' }}</p>
            <div v-if="diffFile">
              <p><strong>DiffFile对象:</strong> 已生成</p>
            </div>
            <div v-else-if="diffViewData">
              <p><strong>diffViewData结构:</strong></p>
              <pre style="background: #f5f5f5; padding: 8px; border-radius: 4px; overflow-x: auto;">{{ JSON.stringify({
                oldFile: diffViewData.oldFile ? {
                  fileName: diffViewData.oldFile.fileName,
                  contentLength: diffViewData.oldFile.content?.length || 0,
                  fileLang: diffViewData.oldFile.fileLang
                } : null,
                newFile: diffViewData.newFile ? {
                  fileName: diffViewData.newFile.fileName,
                  contentLength: diffViewData.newFile.content?.length || 0,
                  fileLang: diffViewData.newFile.fileLang
                } : null,
                hunksCount: diffViewData.hunks?.length || 0,
                hunksPreview: diffViewData.hunks?.slice(0, 5) || []
              }, null, 2) }}</pre>
            </div>
          </div>
        </details>
      </div>

      <!-- 优先使用diffFile模式 -->
      <DiffView v-if="diffFile" :diffFile="diffFile" :diff-view-mode="diffMode" :diff-view-theme="'light'"
        :diff-view-highlight="true" :diff-view-wrap="wrapLines" :diff-view-font-size="14" :diff-view-add-widget="false"
        @error="handleDiffViewError" />

      <!-- 备用data模式 -->
      <DiffView v-else-if="diffViewData" :data="diffViewData" :diff-view-mode="diffMode" :diff-view-theme="'light'"
        :diff-view-highlight="true" :diff-view-wrap="wrapLines" :diff-view-font-size="14" :diff-view-add-widget="false"
        @error="handleDiffViewError" />

      <!-- 无法生成差异 -->
      <div v-else class="diff-generation-error" style="padding: 20px; text-align: center; color: #d73a49;">
        <p>⚠️ 无法生成差异显示</p>
        <p>请检查数据格式或查看控制台错误信息</p>
      </div>
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
import { generateDiffFile } from '@git-diff-view/file'
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
const ignoreWhitespace = ref(true) // 默认启用忽略空白字符
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
 * 标准化文本内容，处理换行符和空白字符
 * 作者：Evilek
 * 编写日期：2025-07-22
 */
const normalizeContent = (content: string): string => {
  if (!ignoreWhitespace.value) {
    return content
  }

  return content
    // 统一换行符为 \n
    .replace(/\r\n/g, '\n')
    .replace(/\r/g, '\n')
    // 移除行尾空白字符
    .replace(/[ \t]+$/gm, '')
    // 移除文件末尾的多余空行
    .replace(/\n+$/, '\n')
}

/**
 * 使用@git-diff-view/file库生成DiffFile对象
 * 作者：Evilek
 * 编写日期：2025-07-22
 */
const diffFile = computed(() => {
  if (!diffData.value || !hasValidContent.value) {
    return null
  }

  try {
    console.log('🔧 [DiffViewer] 使用@git-diff-view/file库生成DiffFile')
    console.log('🔧 [DiffViewer] 忽略空白字符:', ignoreWhitespace.value)

    // 根据设置决定是否标准化内容
    const oldContent = normalizeContent(diffData.value.old_content || '')
    const newContent = normalizeContent(diffData.value.new_content || '')

    console.log('📊 [DiffViewer] 内容长度对比:', {
      original: {
        old: diffData.value.old_content?.length || 0,
        new: diffData.value.new_content?.length || 0
      },
      normalized: {
        old: oldContent.length,
        new: newContent.length
      }
    })

    const file = generateDiffFile(
      diffData.value.old_file_name || diffData.value.file_path,
      oldContent,
      diffData.value.new_file_name || diffData.value.file_path,
      newContent,
      diffData.value.file_language || '',
      diffData.value.file_language || ''
    )

    file.initTheme('light')
    file.init()

    if (isUnified.value) {
      file.buildUnifiedDiffLines()
    } else {
      file.buildSplitDiffLines()
    }

    console.log('✅ [DiffViewer] DiffFile生成成功')
    return file
  } catch (error) {
    console.error('❌ [DiffViewer] DiffFile生成失败:', error)
    return null
  }
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
    console.log('🔧 [DiffViewer] 开始转换hunks数据')
    console.log('� [DiffViewer] 忽略空白字符:', ignoreWhitespace.value)
    console.log('�📥 [DiffViewer] 输入的hunks数据:', diffData.value.hunks)

    // 转换后端返回的hunks数据为Git diff字符串格式
    const hunks: string[] = []

    if (!diffData.value.hunks || diffData.value.hunks.length === 0) {
      console.warn('⚠️ [DiffViewer] hunks数组为空或不存在')
      return null
    }

    diffData.value.hunks.forEach((hunk, hunkIndex) => {
      console.log(`🔍 [DiffViewer] 处理Hunk ${hunkIndex + 1}:`, hunk)

      // 添加hunk头
      const hunkHeader = `@@ -${hunk.old_start},${hunk.old_lines} +${hunk.new_start},${hunk.new_lines} @@`
      hunks.push(hunkHeader)
      console.log(`  📝 [DiffViewer] 添加hunk头: ${hunkHeader}`)

      // 添加hunk中的每一行
      if (hunk.lines && hunk.lines.length > 0) {
        hunk.lines.forEach((line, lineIndex) => {
          let prefix = ' ' // 默认为上下文行
          if (line.line_type === 'Delete') {
            prefix = '-'
          } else if (line.line_type === 'Insert') {
            prefix = '+'
          }

          // 确保content不为undefined或null
          let content = line.content ?? ''

          // 如果启用了忽略空白字符，则标准化内容
          if (ignoreWhitespace.value) {
            const originalContent = content
            content = content
              .replace(/\r\n/g, '\n')
              .replace(/\r/g, '\n')
              .replace(/[ \t]+$/, '') // 移除行尾空白

            if (originalContent !== content && lineIndex < 3) {
              console.log(`    🔧 [DiffViewer] 标准化行内容: "${originalContent}" -> "${content}"`)
            }
          }

          // Git diff格式要求：前缀 + 内容
          const diffLine = prefix + content
          hunks.push(diffLine)

          if (lineIndex < 5) { // 显示前5行的详细信息
            console.log(`    📄 [DiffViewer] 行${lineIndex + 1}: ${line.line_type} -> "${diffLine}" (content长度: ${content.length})`)
          }
        })
        console.log(`  ✅ [DiffViewer] Hunk ${hunkIndex + 1} 处理完成，共${hunk.lines.length}行`)
      } else {
        console.warn(`  ⚠️ [DiffViewer] Hunk ${hunkIndex + 1} 没有lines数据`)
      }
    })

    console.log('📤 [DiffViewer] 转换后的Git diff字符串数组:')
    console.log('  总行数:', hunks.length)
    console.log('  前10行:', hunks.slice(0, 10))

    // 检查是否有空行或异常行
    const emptyLines = hunks.filter((line, index) => {
      const isEmpty = line.length <= 1 // 只有前缀字符
      const isOnlyPrefix = line === '+' || line === '-' || line === ' '
      if (isEmpty || isOnlyPrefix) {
        console.warn(`  ⚠️ [DiffViewer] 发现异常行 ${index}: "${line}" (长度: ${line.length})`)
        return true
      }
      return false
    })

    if (emptyLines.length > 0) {
      console.warn(`  ⚠️ [DiffViewer] 总共发现 ${emptyLines.length} 个异常行`)
    }

    // 根据设置决定是否标准化文件内容
    const oldContent = normalizeContent(diffData.value.old_content || '')
    const newContent = normalizeContent(diffData.value.new_content || '')

    const result = {
      oldFile: {
        fileName: diffData.value.old_file_name || diffData.value.file_path,
        content: oldContent,
        fileLang: diffData.value.file_language || ''
      },
      newFile: {
        fileName: diffData.value.new_file_name || diffData.value.file_path,
        content: newContent,
        fileLang: diffData.value.file_language || ''
      },
      hunks
    }

    console.log('🎯 [DiffViewer] 最终传递给DiffView的data对象:', {
      oldFile: {
        fileName: result.oldFile.fileName,
        contentLength: result.oldFile.content.length,
        fileLang: result.oldFile.fileLang
      },
      newFile: {
        fileName: result.newFile.fileName,
        contentLength: result.newFile.content.length,
        fileLang: result.newFile.fileLang
      },
      hunksCount: result.hunks.length
    })

    return result
  } catch (error) {
    console.error('❌ [DiffViewer] 处理diff数据失败:', error)
    console.error('❌ [DiffViewer] 错误堆栈:', error instanceof Error ? error.stack : 'No stack trace')
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

    console.log('🔍 [DiffViewer] 开始加载差异数据')
    console.log('📋 [DiffViewer] 请求参数:', {
      file_path: props.filePath,
      diff_type: props.diffType
    })

    const result = await invoke('get_file_diff', {
      request: {
        file_path: props.filePath,
        diff_type: props.diffType
      }
    }) as FileDiffResult

    console.log('📦 [DiffViewer] 后端返回的原始数据:', result)
    console.log('📊 [DiffViewer] 数据结构分析:', {
      hunks_count: result?.hunks?.length || 0,
      old_content_length: result?.old_content?.length || 0,
      new_content_length: result?.new_content?.length || 0,
      is_binary: result?.is_binary,
      is_new_file: result?.is_new_file,
      is_deleted_file: result?.is_deleted_file,
      file_language: result?.file_language
    })

    if (result?.hunks?.length > 0) {
      console.log('🔍 [DiffViewer] Hunks详细内容:')
      result.hunks.forEach((hunk, index) => {
        console.log(`  Hunk ${index + 1}:`, {
          old_start: hunk.old_start,
          old_lines: hunk.old_lines,
          new_start: hunk.new_start,
          new_lines: hunk.new_lines,
          lines_count: hunk.lines?.length || 0
        })
        if (hunk.lines?.length > 0) {
          console.log(`    前3行内容:`, hunk.lines.slice(0, 3))
        }
      })
    } else {
      console.warn('⚠️ [DiffViewer] 没有找到任何hunks数据')
    }

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

/**
 * 切换视图模式（并排/统一）
 * 作者：Evilek
 * 编写日期：2025-07-22
 */
const toggleMode = () => {
  isUnified.value = !isUnified.value

  // 如果使用diffFile模式，需要重新构建差异行
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
 * 切换折叠模式
 * 作者：Evilek
 * 编写日期：2025-07-22
 */
const toggleCollapse = () => {
  collapseUnchanged.value = !collapseUnchanged.value
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
  console.error('❌ [DiffViewer] DiffView组件渲染错误:', errorEvent)
  error.value = 'DiffView组件渲染失败: ' + (errorEvent?.message || '未知错误')
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
  align-items: center;
  gap: 12px;
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

.no-diff-icon {
  font-size: 48px;
  display: block;
  margin-bottom: 16px;
  opacity: 0.6;
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
