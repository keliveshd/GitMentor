<template>
  <div class="branch-switcher">
    <!-- 分支显示按钮 -->
    <button @click="toggleDropdown" class="branch-button" :class="{ active: showDropdown }"
      :title="currentBranch ? `当前分支: ${currentBranch}` : '点击选择分支'">
      <span class="branch-icon">🌿</span>
      <span class="branch-name">{{ currentBranch || '未知分支' }}</span>
      <span class="dropdown-arrow" :class="{ rotated: showDropdown }">▼</span>
    </button>

    <!-- 分支下拉菜单 -->
    <div v-if="showDropdown" class="branch-dropdown" @click.stop>
      <div class="dropdown-header">
        <h4>分支管理</h4>
        <div class="header-actions">
          <button @click="fetchRemote" class="action-btn" title="获取远程更新" :disabled="isOperating">
            {{ isOperating && currentOperation === 'fetch' ? '⏳' : '📥' }}
          </button>
          <button @click="pullCurrentBranch" class="action-btn" title="拉取当前分支" :disabled="isOperating">
            {{ isOperating && currentOperation === 'pull' ? '⏳' : '⬇️' }}
          </button>
          <button @click="() => pushCurrentBranch()" class="action-btn" title="推送当前分支" :disabled="isOperating">
            {{ isOperating && currentOperation === 'push' ? '⏳' : '⬆️' }}
          </button>
          <button @click="refreshBranches" class="refresh-btn" title="刷新分支列表" :disabled="isOperating">🔄</button>
        </div>
      </div>

      <!-- 搜索框 -->
      <div class="search-box">
        <input v-model="searchQuery" type="text" placeholder="搜索分支..." class="search-input"
          @keydown.escape="closeDropdown" />
      </div>

      <!-- 本地分支 -->
      <div class="branch-section" v-if="filteredLocalBranches.length > 0">
        <div class="section-title">📂 本地分支</div>
        <div class="branch-list">
          <div v-for="branch in filteredLocalBranches" :key="branch.name" @click="switchBranch(branch.name, false)"
            class="branch-item" :class="{ current: branch.is_current, loading: switchingBranch === branch.name }">
            <span class="branch-icon">{{ branch.is_current ? '✓' : '📝' }}</span>
            <span class="branch-name">{{ branch.name }}</span>
            <span v-if="switchingBranch === branch.name" class="loading-spinner">⏳</span>
          </div>
        </div>
      </div>

      <!-- 远程分支 -->
      <div class="branch-section" v-if="filteredRemoteBranches.length > 0">
        <div class="section-title">🌐 远程分支</div>
        <div class="branch-list">
          <div v-for="branch in filteredRemoteBranches" :key="branch.name" @click="checkoutRemoteBranch(branch.name)"
            class="branch-item remote" :class="{ loading: switchingBranch === branch.name }">
            <span class="branch-icon">📡</span>
            <span class="branch-name">{{ branch.name }}</span>
            <span class="checkout-hint">检出</span>
            <span v-if="switchingBranch === branch.name" class="loading-spinner">⏳</span>
          </div>
        </div>
      </div>

      <!-- 无分支提示 -->
      <div v-if="filteredLocalBranches.length === 0 && filteredRemoteBranches.length === 0" class="no-branches">
        <p>{{ searchQuery ? '未找到匹配的分支' : '暂无分支' }}</p>
      </div>

      <!-- 加载状态 -->
      <div v-if="loading" class="loading-state">
        <p>🔄 正在加载分支列表...</p>
      </div>
    </div>

    <!-- 点击外部关闭下拉菜单 -->
    <div v-if="showDropdown" class="dropdown-overlay" @click="closeDropdown"></div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useToast } from '@/composables/useToast'

// Props
interface Props {
  currentBranch?: string
}

const props = withDefaults(defineProps<Props>(), {
  currentBranch: ''
})

// Emits
const emit = defineEmits<{
  branchChanged: [branchName: string]
}>()

// 响应式数据
const showDropdown = ref(false)
const branches = ref<any[]>([])
const loading = ref(false)
const searchQuery = ref('')
const switchingBranch = ref<string | null>(null)
const isOperating = ref(false)
const currentOperation = ref<string | null>(null)

// Toast
const toast = useToast()

// 计算属性
const filteredLocalBranches = computed(() => {
  return branches.value
    .filter(branch => !branch.is_remote)
    .filter(branch =>
      searchQuery.value === '' ||
      branch.name.toLowerCase().includes(searchQuery.value.toLowerCase())
    )
})

const filteredRemoteBranches = computed(() => {
  return branches.value
    .filter(branch => branch.is_remote)
    .filter(branch =>
      searchQuery.value === '' ||
      branch.name.toLowerCase().includes(searchQuery.value.toLowerCase())
    )
})

// 方法
const toggleDropdown = () => {
  showDropdown.value = !showDropdown.value
  if (showDropdown.value) {
    loadBranches()
  }
}

const closeDropdown = () => {
  showDropdown.value = false
  searchQuery.value = ''
}

const loadBranches = async () => {
  try {
    loading.value = true
    const result = await invoke('get_branches') as any[]
    branches.value = result
  } catch (error) {
    console.error('Failed to load branches:', error)
    toast.error(`获取分支列表失败: ${error}`, '分支操作失败')
  } finally {
    loading.value = false
  }
}

const refreshBranches = async () => {
  await loadBranches()
  toast.success('分支列表已刷新', '刷新成功')
}

const switchBranch = async (branchName: string, isRemote: boolean) => {
  if (switchingBranch.value) return // 防止重复点击

  try {
    switchingBranch.value = branchName
    const result = await invoke('checkout_branch', {
      branchName,
      isRemote
    }) as any

    if (result.success) {
      toast.success(result.message, '分支切换成功')
      emit('branchChanged', branchName)
      closeDropdown()
    } else {
      toast.error(result.message || '分支切换失败', '操作失败')
    }
  } catch (error) {
    console.error('Failed to switch branch:', error)
    toast.error(`分支切换失败: ${error}`, '操作失败')
  } finally {
    switchingBranch.value = null
  }
}

const checkoutRemoteBranch = async (branchName: string) => {
  await switchBranch(branchName, true)
}

// Git 操作方法
const fetchRemote = async () => {
  if (isOperating.value) return

  try {
    isOperating.value = true
    currentOperation.value = 'fetch'

    const result = await invoke('fetch_remote', {
      remoteName: null
    }) as any

    if (result.success) {
      toast.success(result.message, '获取成功')
      // 刷新分支列表以显示最新的远程分支
      await loadBranches()
    } else {
      toast.error(result.message || '获取远程更新失败', '操作失败')
    }
  } catch (error) {
    console.error('Failed to fetch remote:', error)
    toast.error(`获取远程更新失败: ${error}`, '操作失败')
  } finally {
    isOperating.value = false
    currentOperation.value = null
  }
}

const pullCurrentBranch = async () => {
  if (isOperating.value) return

  try {
    isOperating.value = true
    currentOperation.value = 'pull'

    const result = await invoke('pull_current_branch') as any

    if (result.success) {
      toast.success(result.message, '拉取成功')
      emit('branchChanged', props.currentBranch || 'unknown')
    } else {
      toast.error(result.message || '拉取失败', '操作失败')
    }
  } catch (error) {
    console.error('Failed to pull:', error)
    toast.error(`拉取失败: ${error}`, '操作失败')
  } finally {
    isOperating.value = false
    currentOperation.value = null
  }
}

const pushCurrentBranch = async (force = false) => {
  if (isOperating.value) return

  try {
    isOperating.value = true
    currentOperation.value = 'push'

    const result = await invoke('push_current_branch', {
      force
    }) as any

    if (result.success) {
      toast.success(result.message, '推送成功')
    } else {
      toast.error(result.message || '推送失败', '操作失败')
    }
  } catch (error) {
    console.error('Failed to push:', error)
    const errorMsg = String(error)

    // 检查是否需要强制推送
    if (errorMsg.includes('rejected') || errorMsg.includes('non-fast-forward')) {
      const confirmed = confirm('推送被拒绝，可能需要强制推送。是否强制推送？\n警告：强制推送可能会覆盖远程更改！')
      if (confirmed) {
        await pushCurrentBranch(true)
        return
      }
    }

    toast.error(`推送失败: ${error}`, '操作失败')
  } finally {
    isOperating.value = false
    currentOperation.value = null
  }
}

// 键盘事件处理
const handleKeydown = (event: KeyboardEvent) => {
  if (event.key === 'Escape' && showDropdown.value) {
    closeDropdown()
  }
}

// 生命周期
onMounted(() => {
  document.addEventListener('keydown', handleKeydown)
})

onUnmounted(() => {
  document.removeEventListener('keydown', handleKeydown)
})
</script>

<style scoped>
.branch-switcher {
  position: relative;
  display: inline-block;
}

.branch-button {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 8px;
  background: transparent;
  border: 1px solid var(--border-color, #e1e5e9);
  border-radius: 4px;
  cursor: pointer;
  font-size: 13px;
  color: var(--text-color, #24292f);
  transition: all 0.2s ease;
}

.branch-button:hover {
  background: var(--hover-bg, #f6f8fa);
  border-color: var(--border-hover, #d0d7de);
}

.branch-button.active {
  background: var(--active-bg, #dbeafe);
  border-color: var(--primary-color, #0969da);
}

.branch-icon {
  font-size: 12px;
}

.branch-name {
  max-width: 120px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.dropdown-arrow {
  font-size: 10px;
  transition: transform 0.2s ease;
}

.dropdown-arrow.rotated {
  transform: rotate(180deg);
}

.branch-dropdown {
  position: absolute;
  top: 100%;
  left: 0;
  right: 0;
  min-width: 280px;
  max-height: 400px;
  background: white;
  border: 1px solid var(--border-color, #e1e5e9);
  border-radius: 6px;
  box-shadow: 0 8px 24px rgba(140, 149, 159, 0.2);
  z-index: 1000;
  overflow: hidden;
}

.dropdown-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  z-index: 999;
}

.dropdown-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  border-bottom: 1px solid var(--border-color, #e1e5e9);
  background: var(--header-bg, #f6f8fa);
}

.dropdown-header h4 {
  margin: 0;
  font-size: 14px;
  font-weight: 600;
  color: var(--text-color, #24292f);
}

.header-actions {
  display: flex;
  gap: 4px;
  align-items: center;
}

.action-btn,
.refresh-btn {
  background: none;
  border: none;
  cursor: pointer;
  padding: 6px 8px;
  border-radius: 4px;
  font-size: 12px;
  transition: all 0.2s ease;
  min-width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.action-btn:hover,
.refresh-btn:hover {
  background: var(--hover-bg, #e1e5e9);
}

.action-btn:disabled,
.refresh-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.action-btn:disabled:hover,
.refresh-btn:disabled:hover {
  background: none;
}

.search-box {
  padding: 12px 16px;
  border-bottom: 1px solid var(--border-color, #e1e5e9);
}

.search-input {
  width: 100%;
  padding: 6px 8px;
  border: 1px solid var(--border-color, #e1e5e9);
  border-radius: 4px;
  font-size: 13px;
  outline: none;
}

.search-input:focus {
  border-color: var(--primary-color, #0969da);
  box-shadow: 0 0 0 2px rgba(9, 105, 218, 0.1);
}

.branch-section {
  max-height: 200px;
  overflow-y: auto;
}

.section-title {
  padding: 8px 16px;
  font-size: 12px;
  font-weight: 600;
  color: var(--text-secondary, #656d76);
  background: var(--section-bg, #f6f8fa);
  border-bottom: 1px solid var(--border-color, #e1e5e9);
}

.branch-list {
  /* 无额外样式 */
}

.branch-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 16px;
  cursor: pointer;
  font-size: 13px;
  transition: background 0.2s ease;
  position: relative;
}

.branch-item:hover {
  background: var(--hover-bg, #f6f8fa);
}

.branch-item.current {
  background: var(--current-bg, #e6f3ff);
  color: var(--primary-color, #0969da);
  font-weight: 500;
}

.branch-item.remote {
  color: var(--text-secondary, #656d76);
}

.branch-item.loading {
  opacity: 0.6;
  cursor: not-allowed;
}

.branch-item .branch-icon {
  font-size: 12px;
  width: 16px;
  text-align: center;
}

.branch-item .branch-name {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.checkout-hint {
  font-size: 11px;
  color: var(--text-tertiary, #8c959f);
  opacity: 0;
  transition: opacity 0.2s ease;
}

.branch-item:hover .checkout-hint {
  opacity: 1;
}

.loading-spinner {
  font-size: 12px;
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

.no-branches,
.loading-state {
  padding: 20px 16px;
  text-align: center;
  color: var(--text-secondary, #656d76);
  font-size: 13px;
}

.loading-state {
  border-top: 1px solid var(--border-color, #e1e5e9);
}
</style>
