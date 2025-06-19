<template>
  <div class="file-item" :class="{ 'staged': isStaged }">
    <div class="file-info" @click="toggleSelection">
      <div class="file-status-icon">
        {{ getStatusIcon() }}
      </div>
      <div class="file-details">
        <div class="file-path">
          <span class="path-text">{{ file.path }}</span>
          <span class="file-status-text">{{ getStatusText() }}</span>
        </div>
      </div>
    </div>
    
    <div class="file-actions">
      <!-- 暂存/取消暂存按钮 -->
      <button
        @click="handleToggleStage"
        class="action-btn stage-btn"
        :title="isStaged ? '取消暂存' : '暂存'"
      >
        {{ isStaged ? '➖' : '➕' }}
      </button>
      
      <!-- 回滚按钮 -->
      <button
        @click="handleRevert"
        class="action-btn revert-btn"
        :title="isStaged ? '回滚暂存区更改' : '回滚工作区更改'"
        v-if="canRevert()"
      >
        ↩️
      </button>
      
      <!-- 查看差异按钮 -->
      <button
        @click="viewDiff"
        class="action-btn diff-btn"
        title="查看差异"
        v-if="canViewDiff()"
      >
        👁️
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">

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
}

const props = defineProps<Props>()

// Emits
const emit = defineEmits<{
  toggleStage: [filePath: string, shouldStage: boolean]
  revert: [filePath: string, isStaged: boolean]
  viewDiff: [filePath: string]
}>()

// 计算属性
const getStatusIcon = () => {
  if (props.isStaged) {
    switch (props.file.index_status) {
      case 'Modified': return '📝'
      case 'Added': return '➕'
      case 'Deleted': return '🗑️'
      case 'Renamed': return '📛'
      case 'Copied': return '📋'
      default: return '📄'
    }
  } else {
    switch (props.file.working_tree_status) {
      case 'Modified': return '📝'
      case 'Added': return '➕'
      case 'Deleted': return '🗑️'
      case 'Untracked': return '❓'
      case 'Conflicted': return '⚠️'
      default: return '📄'
    }
  }
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

// 方法
const toggleSelection = () => {
  // 可以在这里处理文件选择逻辑
}

const handleToggleStage = () => {
  emit('toggleStage', props.file.path, !props.isStaged)
}

const handleRevert = () => {
  if (confirm(`确定要回滚 ${props.file.path} 的更改吗？此操作不可撤销。`)) {
    emit('revert', props.file.path, props.isStaged)
  }
}

const viewDiff = () => {
  emit('viewDiff', props.file.path)
  // 这里可以实现差异查看功能
  alert(`查看 ${props.file.path} 的差异（功能待实现）`)
}
</script>

<style scoped>
.file-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 4px 8px;
  margin: 1px 0;
  border: 1px solid #e1e4e8;
  border-radius: 4px;
  background-color: #ffffff;
  transition: all 0.2s ease;
  cursor: pointer;
  min-height: 28px;
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
  font-size: 14px;
  margin-right: 6px;
  flex-shrink: 0;
}

.file-details {
  flex: 1;
  min-width: 0;
}

.file-path {
  display: flex;
  align-items: center;
  gap: 8px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.path-text {
  font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
  font-size: 13px;
  font-weight: 500;
  color: #24292f;
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
}

.file-status-text {
  font-size: 10px;
  color: #656d76;
  background: #f6f8fa;
  padding: 1px 4px;
  border-radius: 3px;
  flex-shrink: 0;
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

  .path-text {
    color: #e6edf3;
  }

  .file-status-text {
    color: #8b949e;
    background: #30363d;
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
