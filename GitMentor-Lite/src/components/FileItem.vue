<template>
  <div class="file-item" :class="{ 'staged': isStaged, 'selected': selected }" @contextmenu="handleContextMenu">
    <!-- 批量选择复选框 -->
    <div v-if="batchMode" class="file-checkbox">
      <input type="checkbox" :checked="selected" @change="toggleSelection" />
    </div>

    <div class="file-info" @click="batchMode ? toggleSelection : undefined">
      <div class="file-status-icon">
        {{ getStatusIcon() }}
      </div>
      <div class="file-details">
        <div class="file-path">
          <span class="file-name" :class="{ 'deleted': isDeleted() }">{{ getFileName() }}</span>
          <span class="relative-path" :class="{ 'deleted': isDeleted() }">{{ getRelativePath() }}</span>
          <span class="file-status-text" :class="{ 'deleted': isDeleted() }">{{ getStatusText() }}</span>
        </div>
      </div>
    </div>

    <div class="file-actions">
      <!-- 暂存/取消暂存按钮 -->
      <button @click="handleToggleStage" class="action-btn stage-btn" :title="isStaged ? '取消暂存' : '暂存'">
        {{ isStaged ? '➖' : '➕' }}
      </button>

      <!-- 回滚按钮 -->
      <button @click="handleRevert" class="action-btn revert-btn" :title="isStaged ? '回滚暂存区更改' : '回滚工作区更改'"
        v-if="canRevert()">
        ↩️
      </button>

      <!-- 查看差异按钮 -->
      <button @click="viewDiff" class="action-btn diff-btn" title="查看差异" v-if="canViewDiff()">
        👁️
      </button>
    </div>


  </div>
</template>

<script setup lang="ts">
// import { computed } from 'vue'

// Props
interface FileStatus {
  path: string
  working_tree_status?: string
  index_status?: string
  selected: boolean
  is_staged: boolean
}

interface Props {
  file: FileStatus
  isStaged: boolean
  batchMode?: boolean
  selected?: boolean
}

const props = defineProps<Props>()

// Emits
const emit = defineEmits<{
  toggleStage: [filePath: string, shouldStage: boolean]
  revert: [filePath: string, isStaged: boolean]
  revertClick: [filePath: string, isStaged: boolean]
  viewDiff: [filePath: string, isStaged: boolean]
  toggleSelect: [filePath: string]
  refresh: []
  contextMenu: [file: any, event: MouseEvent]
}>()



// 获取文件类型图标
const getFileTypeIcon = (filePath: string) => {
  const ext = filePath.split('.').pop()?.toLowerCase()

  switch (ext) {
    case 'js': case 'jsx': return '🟨'
    case 'ts': case 'tsx': return '🔷'
    case 'vue': return '💚'
    case 'html': case 'htm': return '🌐'
    case 'css': case 'scss': case 'sass': case 'less': return '🎨'
    case 'json': return '📋'
    case 'md': case 'markdown': return '📝'
    case 'py': return '🐍'
    case 'java': return '☕'
    case 'cpp': case 'c': case 'h': return '⚙️'
    case 'rs': return '🦀'
    case 'go': return '🐹'
    case 'php': return '🐘'
    case 'rb': return '💎'
    case 'swift': return '🦉'
    case 'kt': return '🟣'
    case 'dart': return '🎯'
    case 'xml': return '📄'
    case 'yml': case 'yaml': return '⚙️'
    case 'toml': return '⚙️'
    case 'sql': return '🗃️'
    case 'sh': case 'bash': return '🐚'
    case 'dockerfile': return '🐳'
    case 'png': case 'jpg': case 'jpeg': case 'gif': case 'svg': case 'webp': return '🖼️'
    case 'pdf': return '📕'
    case 'txt': return '📄'
    case 'lock': return '🔒'
    default: return '📄'
  }
}

// 计算属性
const getStatusIcon = () => {
  return getFileTypeIcon(props.file.path)
}



const getStatusText = () => {
  if (props.isStaged) {
    switch (props.file.index_status) {
      case 'Modified': return '已修改'
      case 'Added': return '新增'
      case 'Deleted': return '已删除'
      case 'Renamed': return '重命名'
      case 'Copied': return '复制'
      default: return '未知'
    }
  } else {
    switch (props.file.working_tree_status) {
      case 'Modified': return '已修改'
      case 'Added': return '新增'
      case 'Deleted': return '已删除'
      case 'Untracked': return '未跟踪'
      case 'Conflicted': return '冲突'
      default: return '未知'
    }
  }
}

const canRevert = () => {
  // 未跟踪文件不能回滚，只能删除
  return props.file.working_tree_status !== 'Untracked'
}

const canViewDiff = () => {
  // 未跟踪文件和已删除文件不能查看差异
  return props.file.working_tree_status !== 'Untracked' &&
    props.file.working_tree_status !== 'Deleted' &&
    props.file.index_status !== 'Deleted'
}

// 获取文件名
const getFileName = () => {
  const parts = props.file.path.split('/')
  return parts[parts.length - 1]
}

// 获取相对路径（不包含文件名，使用反斜杠）
const getRelativePath = () => {
  const parts = props.file.path.split('/')
  if (parts.length <= 1) return ''
  return parts.slice(0, -1).join('\\')
}

// 判断文件是否被删除
const isDeleted = () => {
  return props.file.working_tree_status === 'Deleted' || props.file.index_status === 'Deleted'
}

// 判断文件是否为未跟踪文件
// const isUntracked = () => {
//   return props.file.working_tree_status === 'Untracked'
// }

// 方法
const handleToggleStage = () => {
  emit('toggleStage', props.file.path, !props.isStaged)
}

const handleRevert = () => {
  // 通过 emits 通知父组件处理确认逻辑，避免使用原生 confirm() 对话框
  emit('revert-click', props.file.path, props.isStaged)
}

const viewDiff = () => {
  emit('viewDiff', props.file.path, props.isStaged)
}

// 批量选择相关方法
const toggleSelection = (event?: Event) => {
  // 阻止事件冒泡，避免触发父元素的点击事件
  if (event) {
    event.stopPropagation()
    event.preventDefault()
  }
  console.log('toggleSelection called for:', props.file.path, 'current selected:', props.selected)
  emit('toggleSelect', props.file.path)
}

// 右键菜单处理方法
const handleContextMenu = (event: MouseEvent) => {
  emit('contextMenu', props.file, event)
}
</script>

<style scoped>
.file-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 2px 6px;
  margin: 1px 0;
  border: 1px solid #e1e4e8;
  border-radius: 4px;
  background-color: #ffffff;
  transition: all 0.2s ease;
  cursor: pointer;
  min-height: 22px;
}

.file-item.selected {
  background-color: #e3f2fd;
  border-color: #2196f3;
}

.file-checkbox {
  margin-right: 8px;
  display: flex;
  align-items: center;
}

.file-checkbox input[type="checkbox"] {
  width: 16px;
  height: 16px;
  cursor: pointer;
}

.file-item:hover {
  background-color: #f6f8fa;
  border-color: #d0d7de;
}

.file-item.staged {
  background-color: #f0f9ff;
  border-color: #0969da;
}

.file-info {
  display: flex;
  align-items: center;
  flex: 1;
  min-width: 0;
}

.file-status-icon {
  font-size: 12px;
  margin-right: 4px;
  flex-shrink: 0;
}

.file-details {
  flex: 1;
  min-width: 0;
}

.file-path {
  display: flex;
  align-items: center;
  gap: 6px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.file-name {
  font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
  font-size: 12px;
  font-weight: 500;
  color: #24292f;
  flex-shrink: 0;
}

.file-name.deleted {
  text-decoration: line-through;
  opacity: 0.7;
}

.relative-path {
  font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
  font-size: 10px;
  font-weight: 400;
  color: #656d76;
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
}

.relative-path.deleted {
  text-decoration: line-through;
  opacity: 0.7;
}

.file-status-text {
  font-size: 9px;
  color: #656d76;
  background: #f6f8fa;
  padding: 1px 3px;
  border-radius: 2px;
  flex-shrink: 0;
}

.file-status-text.deleted {
  text-decoration: line-through;
  opacity: 0.7;
}

.file-actions {
  display: flex;
  gap: 4px;
  opacity: 0;
  transition: opacity 0.2s ease;
}

.file-item:hover .file-actions {
  opacity: 1;
}

.action-btn {
  padding: 4px 6px;
  border: none;
  border-radius: 4px;
  background-color: transparent;
  cursor: pointer;
  font-size: 12px;
  transition: background-color 0.2s ease;
}

.action-btn:hover {
  background-color: #f3f4f6;
}

.stage-btn:hover {
  background-color: #dbeafe;
}

.revert-btn:hover {
  background-color: #fef2f2;
}

.diff-btn:hover {
  background-color: #f0fdf4;
}

/* 深色主题支持 */
@media (prefers-color-scheme: dark) {
  .file-item {
    background-color: #21262d;
    border-color: #30363d;
    color: #e6edf3;
  }

  .file-item:hover {
    background-color: #262c36;
    border-color: #444c56;
  }

  .file-item.staged {
    background-color: #0d1117;
    border-color: #1f6feb;
  }

  .file-name {
    color: #e6edf3;
  }

  .file-name.deleted {
    color: #8b949e;
  }

  .relative-path {
    color: #8b949e;
  }

  .relative-path.deleted {
    color: #6e7681;
  }

  .file-status-text {
    color: #8b949e;
    background: #30363d;
  }

  .file-status-text.deleted {
    color: #6e7681;
  }

  .action-btn:hover {
    background-color: #30363d;
  }

  .stage-btn:hover {
    background-color: #1f2937;
  }

  .revert-btn:hover {
    background-color: #2d1b1b;
  }

  .diff-btn:hover {
    background-color: #1b2d1b;
  }
}
</style>
