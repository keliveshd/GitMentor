<template>
  <div class="git-panel">
    <!-- 仓库信息 -->
    <div class="repo-header">
      <div class="repo-info" v-if="currentRepoPath">
        <span class="repo-name">📂 {{ getRepoName(currentRepoPath) }}</span>
        <span class="branch-info" v-if="gitStatus">
          <span class="branch-name">🌿 {{ gitStatus.branch }}</span>
          <span v-if="gitStatus.ahead > 0" class="ahead">↑{{ gitStatus.ahead }}</span>
          <span v-if="gitStatus.behind > 0" class="behind">↓{{ gitStatus.behind }}</span>
        </span>
      </div>
      <!-- 功能按钮组 -->
      <div class="function-buttons">
        <!-- 全局加载指示器 -->
        <div v-if="loading" class="global-loading">
          <div class="loading-spinner"></div>
          <span class="loading-text">{{ loadingText || '加载中...' }}</span>
          <div class="loading-progress-bar">
            <div class="loading-progress-fill" :style="{ width: loadingProgress + '%' }"></div>
          </div>
        </div>

        <!-- AI服务设置按钮 -->
        <button @click="openAISettings" class="ai-settings-btn" :disabled="loading || !tauriReady" title="AI服务设置">
          🤖 AI设置
        </button>

        <!-- 选择仓库按钮组 -->
        <div class="repo-selector">
          <button @click="openRepository" class="select-repo-btn" :disabled="loading || !tauriReady">
            {{ loading ? '加载中...' : !tauriReady ? '初始化中...' : '选择仓库' }}
          </button>

          <!-- 最近仓库下拉菜单 -->
          <div class="recent-repos-dropdown" v-if="recentRepos.length > 0">
            <button @click="toggleRecentDropdown" class="recent-dropdown-btn" :disabled="loading || !tauriReady"
              title="最近打开的仓库">
              📋
            </button>
            <div v-if="showRecentDropdown" class="recent-dropdown-menu">
              <div class="recent-dropdown-header">
                <span>最近打开的仓库</span>
                <button @click="clearRecentRepos" class="clear-recent-btn" title="清空历史">🗑️</button>
              </div>
              <div class="recent-repo-item" v-for="repo in recentRepos" :key="repo.path"
                @click="openRecentRepo(repo.path)" :class="{ active: repo.path === currentRepoPath }">
                <div class="repo-item-info">
                  <div class="repo-item-name">📂 {{ repo.name }}</div>
                  <div class="repo-item-path">{{ repo.path }}</div>
                  <div class="repo-item-time">{{ getRepoDisplayTime(repo) }}</div>
                </div>
                <button @click.stop="removeRecentRepo(repo.path)" class="remove-repo-btn" title="从历史中移除">×</button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Git状态面板 -->
    <div v-if="gitStatus" class="git-status-panel">
      <!-- 暂存区 -->
      <div class="file-section" v-if="gitStatus.staged_files.length > 0">
        <div class="section-header">
          <h4>📋 暂存的更改 ({{ gitStatus.staged_files.length }})</h4>
          <div class="section-actions">
            <button @click="unstageAll" class="action-btn" title="取消暂存所有">
              ↩️
            </button>
          </div>
        </div>
        <div class="file-list">
          <FileItem v-for="file in gitStatus.staged_files" :key="file.path" :file="file" :is-staged="true"
            @toggle-stage="toggleStage" @revert="revertFile" @viewDiff="openDiffViewer" />
        </div>
      </div>

      <!-- 提交区域 -->
      <div class="commit-section" v-if="gitStatus">
        <div class="commit-input">
          <textarea v-model="commitMessage" placeholder="输入提交消息..." rows="3" class="commit-textarea"
            :disabled="!hasCommittableFiles"></textarea>
          <div class="commit-actions">
            <div class="ai-generate-section">
              <select v-model="selectedTemplate" class="template-select" title="选择提交消息模板风格">
                <option value="standard" title="生成符合常规规范的英文提交消息">标准提交</option>
                <option value="chinese" title="生成简洁明了的中文提交消息">中文提交</option>
                <option value="detailed" title="生成包含详细描述的提交消息">详细提交</option>
                <option value="conventional" title="生成符合约定式提交规范的消息">约定式提交</option>
              </select>
              <button @click="generateCommitMessage" class="generate-btn" :disabled="loading || !hasCommittableFiles"
                title="快捷键: Ctrl+G">
                <span v-if="!isGenerating">🤖 AI生成</span>
                <span v-else>⏳ 生成中...</span>
              </button>
              <button v-if="lastGeneratedMessage && commitMessage === lastGeneratedMessage"
                @click="regenerateCommitMessage" class="regenerate-btn" :disabled="loading || !hasCommittableFiles"
                title="重新生成提交消息">
                🔄 重新生成
              </button>
            </div>
            <button @click="commitChanges" class="commit-btn"
              :disabled="!commitMessage.trim() || loading || !hasCommittableFiles" title="快捷键: Ctrl+Enter">
              ✅ 提交
            </button>
          </div>
          <div v-if="!hasCommittableFiles" class="commit-hint">
            <p>✨ 工作区干净，没有待提交的更改</p>
          </div>
          <div v-else-if="!gitStatus.staged_files.length" class="commit-hint">
            <p>💡 暂存区为空，AI生成和提交将自动暂存所有修改的文件</p>
          </div>
          <div v-if="generationProgress" class="generation-progress">
            <div class="progress-content">
              <div class="progress-text">{{ generationProgress }}</div>
              <div v-if="isGenerating" class="progress-bar">
                <div class="progress-fill"></div>
              </div>
            </div>
          </div>
          <!-- 提交消息预览 -->
          <div v-if="commitMessage && lastGeneratedMessage === commitMessage" class="message-preview">
            <div class="preview-header">
              <span class="preview-label">🤖 AI生成的提交消息</span>
              <div class="preview-actions">
                <button @click="regenerateCommitMessage" class="preview-action-btn" :disabled="loading" title="重新生成">
                  🔄
                </button>
                <button @click="clearCommitMessage" class="preview-action-btn" title="清空消息">
                  🗑️
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- 工作区更改 -->
      <div class="file-section" v-if="gitStatus.unstaged_files.length > 0">
        <div class="section-header">
          <h4>📝 更改 ({{ gitStatus.unstaged_files.length }})</h4>
          <div class="section-actions">
            <button @click="stageAll" class="action-btn" title="暂存所有">
              ➕
            </button>
          </div>
        </div>
        <div class="file-list">
          <FileItem v-for="file in gitStatus.unstaged_files" :key="file.path" :file="file" :is-staged="false"
            @toggle-stage="toggleStage" @revert="revertFile" @viewDiff="openDiffViewer" />
        </div>
      </div>

      <!-- 未跟踪文件 -->
      <div class="file-section" v-if="gitStatus.untracked_files.length > 0">
        <div class="section-header">
          <h4>❓ 未跟踪的文件 ({{ gitStatus.untracked_files.length }})</h4>
          <div class="section-actions">
            <button @click="stageAllUntracked" class="action-btn" title="暂存所有">
              ➕
            </button>
          </div>
        </div>
        <div class="file-list">
          <FileItem v-for="file in gitStatus.untracked_files" :key="file.path" :file="file" :is-staged="false"
            @toggle-stage="toggleStage" @revert="revertFile" @viewDiff="openDiffViewer" />
        </div>
      </div>

      <!-- 冲突文件 -->
      <div class="file-section" v-if="gitStatus.conflicted_files.length > 0">
        <div class="section-header">
          <h4>⚠️ 合并冲突 ({{ gitStatus.conflicted_files.length }})</h4>
        </div>
        <div class="file-list">
          <FileItem v-for="file in gitStatus.conflicted_files" :key="file.path" :file="file" :is-staged="false"
            @toggle-stage="toggleStage" @revert="revertFile" @viewDiff="openDiffViewer" />
        </div>
      </div>

      <!-- 无更改状态 -->
      <div v-if="!gitStatus.has_changes" class="no-changes">
        <p>✨ 工作区干净，没有待提交的更改</p>
      </div>
    </div>

    <!-- 提交历史 -->
    <div class="commit-history" v-if="commitHistory.length > 0">
      <div class="section-header">
        <h4>📜 提交历史</h4>
        <button @click="refreshHistory" class="action-btn">🔄</button>
      </div>
      <div class="history-list">
        <div v-for="commit in commitHistory" :key="commit.hash" class="commit-item">
          <div class="commit-info">
            <div class="commit-message">{{ commit.message }}</div>
            <div class="commit-meta">
              <span class="commit-author">{{ commit.author }}</span>
              <span class="commit-hash">{{ commit.short_hash }}</span>
              <span class="commit-time">{{ formatTime(commit.timestamp) }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 差异查看器已改为独立窗口，此处不再需要模态框 -->

    <!-- Toast通知组件 -->
    <Toast ref="toastRef" />

    <!-- 确认对话框组件 -->
    <ConfirmDialog :visible="globalConfirm.visible.value" :options="globalConfirm.options.value"
      @confirm="globalConfirm.confirm" @cancel="globalConfirm.cancel" @close="globalConfirm.close" />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import FileItem from './FileItem.vue'
import Toast from './Toast.vue'
import ConfirmDialog from './ConfirmDialog.vue'
import WindowManager from '../utils/WindowManager'
import { RecentReposManager, type RecentRepo } from '../utils/RecentRepos'
import { useToast, setToastInstance } from '../composables/useToast'
import { confirm, globalConfirm } from '../composables/useConfirm'

// 响应式数据
const currentRepoPath = ref<string>('')
const gitStatus = ref<any>(null)
const commitMessage = ref('')
const commitHistory = ref<any[]>([])
const loading = ref(false)
const loadingText = ref('')
const loadingProgress = ref(0)
const tauriReady = ref(false)
const selectedTemplate = ref('standard')
const isGenerating = ref(false)
const generationProgress = ref('')
const lastGeneratedMessage = ref('')

// 最近仓库相关状态
const recentRepos = ref<RecentRepo[]>([])
const showRecentDropdown = ref(false)

// Toast通知系统
const toast = useToast()
const toastRef = ref<InstanceType<typeof Toast> | null>(null)

// 计算属性：判断是否有可提交的文件
const hasCommittableFiles = computed(() => {
  if (!gitStatus.value) return false
  return gitStatus.value.staged_files.length > 0 ||
    gitStatus.value.unstaged_files.length > 0 ||
    gitStatus.value.untracked_files.length > 0
})

// 差异查看器已改为独立窗口，不再需要本地状态

// 加载状态管理
const setLoading = (isLoading: boolean, text = '', progress = 0) => {
  loading.value = isLoading
  loadingText.value = text
  loadingProgress.value = progress
}

// 方法
const openRepository = async () => {
  if (!tauriReady.value) {
    toast.warning('应用正在初始化，请稍后再试', '请稍候')
    return
  }

  try {
    setLoading(true, '正在打开文件夹选择器...', 10)

    const selectedPath = await invoke('open_folder_dialog') as string | null
    if (selectedPath) {
      setLoading(true, '正在加载仓库信息...', 50)
      await openRepoByPath(selectedPath)
    }
    // 如果 selectedPath 为 null，说明用户取消了选择或选择的不是有效的Git仓库
    // 这种情况下不需要显示错误消息，因为后端已经处理了
  } catch (error) {
    console.error('Failed to open repository:', error)
    toast.error('打开仓库失败: ' + error, '操作失败')
  } finally {
    setLoading(false)
  }
}

// 通过路径打开仓库的通用方法
const openRepoByPath = async (path: string) => {
  setLoading(true, '正在选择仓库...', 20)
  currentRepoPath.value = path

  setLoading(true, '正在初始化仓库...', 40)
  await invoke('select_repository', { path })

  setLoading(true, '正在获取Git状态...', 60)
  await refreshGitStatus()

  setLoading(true, '正在加载提交历史...', 80)
  await refreshHistory()

  setLoading(true, '正在保存配置...', 90)
  // 保存到最近仓库列表
  RecentReposManager.addRecentRepo(path)
  loadRecentRepos()

  // 关闭下拉菜单
  showRecentDropdown.value = false

  setLoading(true, '完成', 100)
  setTimeout(() => setLoading(false), 500)
}

const refreshGitStatus = async () => {
  try {
    const status = await invoke('get_git_status')
    gitStatus.value = status
  } catch (error) {
    console.error('Failed to get git status:', error)
  }
}

const refreshHistory = async () => {
  try {
    const history = await invoke('get_commit_history', { limit: 10 }) as any[]
    commitHistory.value = history
  } catch (error) {
    console.error('Failed to get commit history:', error)
  }
}

const toggleStage = async (filePath: string, shouldStage: boolean) => {
  try {
    await invoke('stage_files', {
      request: {
        file_paths: [filePath],
        stage: shouldStage
      }
    })
    await refreshGitStatus()
  } catch (error) {
    console.error('Failed to toggle stage:', error)
    toast.error('暂存操作失败: ' + error, '操作失败')
  }
}

const stageAll = async () => {
  if (!gitStatus.value?.unstaged_files.length) return

  try {
    const filePaths = gitStatus.value.unstaged_files.map((f: any) => f.path)
    await invoke('stage_files', {
      request: { file_paths: filePaths, stage: true }
    })
    await refreshGitStatus()
  } catch (error) {
    console.error('Failed to stage all:', error)
    toast.error('暂存所有文件失败: ' + error, '操作失败')
  }
}

const unstageAll = async () => {
  if (!gitStatus.value?.staged_files.length) return

  try {
    const filePaths = gitStatus.value.staged_files.map((f: any) => f.path)
    await invoke('stage_files', {
      request: { file_paths: filePaths, stage: false }
    })
    await refreshGitStatus()
  } catch (error) {
    console.error('Failed to unstage all:', error)
    toast.error('取消暂存所有文件失败: ' + error, '操作失败')
  }
}

const stageAllUntracked = async () => {
  if (!gitStatus.value?.untracked_files.length) return

  try {
    const filePaths = gitStatus.value.untracked_files.map((f: any) => f.path)
    await invoke('stage_files', {
      request: { file_paths: filePaths, stage: true }
    })
    await refreshGitStatus()
  } catch (error) {
    console.error('Failed to stage untracked files:', error)
    toast.error('暂存未跟踪文件失败: ' + error, '操作失败')
  }
}

// 防抖生成函数
let generateTimeout: number | null = null

const generateCommitMessage = async () => {
  if (!hasCommittableFiles.value) return

  // 防抖处理
  if (generateTimeout) {
    clearTimeout(generateTimeout)
  }

  generateTimeout = setTimeout(async () => {
    try {
      isGenerating.value = true
      loading.value = true
      generationProgress.value = '正在分析代码变更...'

      // 如果暂存区为空，先暂存所有修改的文件
      if (!gitStatus.value.staged_files.length) {
        generationProgress.value = '暂存区为空，正在自动暂存所有修改的文件...'

        // 暂存所有未暂存的文件
        if (gitStatus.value.unstaged_files.length > 0) {
          const unstagedPaths = gitStatus.value.unstaged_files.map((f: any) => f.path)
          await invoke('stage_files', {
            request: { file_paths: unstagedPaths, stage: true }
          })
        }

        // 暂存所有未跟踪的文件
        if (gitStatus.value.untracked_files.length > 0) {
          const untrackedPaths = gitStatus.value.untracked_files.map((f: any) => f.path)
          await invoke('stage_files', {
            request: { file_paths: untrackedPaths, stage: true }
          })
        }

        // 刷新Git状态
        await refreshGitStatus()
      }

      const filePaths = gitStatus.value.staged_files.map((f: any) => f.path)

      // 获取暂存文件的差异摘要
      generationProgress.value = '正在获取差异信息...'
      const diffContent = await invoke('get_staged_diff_summary') as string

      // 使用模板生成提交消息
      generationProgress.value = '正在生成提交消息...'
      const result = await invoke('generate_commit_with_template', {
        templateId: selectedTemplate.value,
        diff: diffContent,
        stagedFiles: filePaths,
        branchName: gitStatus.value.branch
      }) as string

      commitMessage.value = result
      lastGeneratedMessage.value = result
      generationProgress.value = '生成完成！'

      // 短暂显示完成状态
      setTimeout(() => {
        generationProgress.value = ''
      }, 1000)

    } catch (error) {
      console.error('Failed to generate commit message:', error)
      console.log('生成提交消息失败: ' + error)
      generationProgress.value = '生成失败，请重试'
      setTimeout(() => {
        generationProgress.value = ''
      }, 2000)
    } finally {
      isGenerating.value = false
      loading.value = false
    }
  }, 300) // 300ms防抖
}

// 重新生成提交消息
const regenerateCommitMessage = async () => {
  // 清空当前消息，然后重新生成
  commitMessage.value = ''
  lastGeneratedMessage.value = ''
  await generateCommitMessage()
}

// 清空提交消息
const clearCommitMessage = () => {
  commitMessage.value = ''
  lastGeneratedMessage.value = ''
}

const commitChanges = async () => {
  if (!commitMessage.value.trim() || !hasCommittableFiles.value) return

  try {
    setLoading(true, '准备提交...', 10)

    // 如果暂存区为空，先暂存所有修改的文件
    if (!gitStatus.value.staged_files.length) {
      setLoading(true, '正在暂存文件...', 30)

      // 暂存所有未暂存的文件
      if (gitStatus.value.unstaged_files.length > 0) {
        const unstagedPaths = gitStatus.value.unstaged_files.map((f: any) => f.path)
        await invoke('stage_files', {
          request: { file_paths: unstagedPaths, stage: true }
        })
      }

      // 暂存所有未跟踪的文件
      if (gitStatus.value.untracked_files.length > 0) {
        const untrackedPaths = gitStatus.value.untracked_files.map((f: any) => f.path)
        await invoke('stage_files', {
          request: { file_paths: untrackedPaths, stage: true }
        })
      }

      setLoading(true, '正在刷新状态...', 50)
      // 刷新Git状态
      await refreshGitStatus()
    }

    setLoading(true, '正在提交更改...', 70)
    await invoke('commit_changes', {
      request: {
        message: commitMessage.value,
        selected_files: [],
        additional_context: null,
        amend: false
      }
    })

    setLoading(true, '正在更新状态...', 90)
    commitMessage.value = ''
    await refreshGitStatus()
    await refreshHistory()

    setLoading(true, '提交完成！', 100)
    toast.success('提交成功！', '操作完成')
    setTimeout(() => setLoading(false), 1000)
  } catch (error) {
    console.error('Failed to commit:', error)
    toast.error('提交失败: ' + error, '操作失败')
    setLoading(false)
  }
}

const revertFile = async (filePath: string, isStaged: boolean) => {
  const fileName = filePath.split(/[/\\]/).pop() || filePath
  const revertType = isStaged ? '暂存区' : '工作区'

  const confirmed = await confirm.danger(
    '回滚文件',
    `确定要回滚${revertType}中的文件 "${fileName}" 吗？`,
    '此操作将丢失该文件的所有未提交更改，且无法撤销。'
  )

  if (!confirmed) return

  try {
    setLoading(true, `正在回滚${revertType}文件...`, 50)
    await invoke('revert_files', {
      request: {
        file_paths: [filePath],
        revert_type: isStaged ? 'Staged' : 'WorkingTree'
      }
    })

    setLoading(true, '正在刷新状态...', 80)
    await refreshGitStatus()

    setLoading(true, '回滚完成', 100)
    toast.success(`${revertType}文件 ${fileName} 已回滚`, '操作完成')
    setTimeout(() => setLoading(false), 1000)
  } catch (error) {
    console.error('Failed to revert file:', error)
    toast.error('回滚文件失败: ' + error, '操作失败')
    setLoading(false)
  }
}

// 工具函数
const getRepoName = (path: string) => {
  return path.split(/[/\\]/).pop() || path
}

const formatTime = (timestamp: number) => {
  return new Date(timestamp * 1000).toLocaleString()
}

// 最近仓库相关方法
const loadRecentRepos = () => {
  recentRepos.value = RecentReposManager.getRecentRepos()
}

const toggleRecentDropdown = () => {
  showRecentDropdown.value = !showRecentDropdown.value
}

const openRecentRepo = async (path: string) => {
  if (!tauriReady.value || loading.value) return

  try {
    loading.value = true
    await openRepoByPath(path)
  } catch (error) {
    console.error('Failed to open recent repository:', error)
    alert('打开仓库失败: ' + error)
  } finally {
    loading.value = false
  }
}

const removeRecentRepo = async (path: string) => {
  const repoName = path.split(/[/\\]/).pop() || path
  const confirmed = await confirm.warning(
    '移除仓库记录',
    `确定要从历史记录中移除 "${repoName}" 吗？`
  )

  if (confirmed) {
    RecentReposManager.removeRecentRepo(path)
    loadRecentRepos()
    toast.success('已从历史记录中移除', '操作完成')
  }
}

const clearRecentRepos = async () => {
  const confirmed = await confirm.warning(
    '清空历史记录',
    '确定要清空所有最近打开的仓库记录吗？此操作无法撤销。'
  )

  if (confirmed) {
    RecentReposManager.clearRecentRepos()
    loadRecentRepos()
    showRecentDropdown.value = false
    toast.success('历史记录已清空', '操作完成')
  }
}

const getRepoDisplayTime = (repo: RecentRepo) => {
  return RecentReposManager.getDisplayText(repo)
}

// 自动加载上次打开的仓库
const autoLoadLastRepo = async () => {
  const lastRepoPath = RecentReposManager.getLastOpenedRepo()
  if (lastRepoPath && tauriReady.value) {
    try {
      // 验证路径是否仍然有效
      await invoke('select_repository', { path: lastRepoPath })
      currentRepoPath.value = lastRepoPath
      await refreshGitStatus()
      await refreshHistory()
      console.log('自动加载上次仓库:', lastRepoPath)
    } catch (error) {
      console.warn('自动加载上次仓库失败:', error)
      // 如果加载失败，从最近列表中移除该路径
      RecentReposManager.removeRecentRepo(lastRepoPath)
      loadRecentRepos()
    }
  }
}

// AI服务设置方法
// 作者：Evilek
// 编写日期：2025-07-25
const openAISettings = async () => {
  try {
    console.log('🤖 [GitPanel] 打开AI服务设置窗口')

    // 使用WindowManager打开AI设置窗口
    await WindowManager.openAISettings()
    console.log('✅ [GitPanel] 已打开AI服务设置窗口')
  } catch (error) {
    console.error('❌ [GitPanel] 打开AI服务设置窗口失败:', error)
    alert(`打开AI服务设置失败: ${error instanceof Error ? error.message : '未知错误'}`)
  }
}

// 差异查看器方法
const openDiffViewer = async (filePath: string, isStaged?: boolean) => {
  try {
    console.log(`🔍 [GitPanel] 打开差异查看器: ${filePath}, isStaged: ${isStaged}`)

    // 根据文件状态和用户点击的区域确定差异类型
    const stagedFile = gitStatus.value?.staged_files?.find((f: any) => f.path === filePath)
    const unstagedFile = gitStatus.value?.unstaged_files?.find((f: any) => f.path === filePath)

    let currentDiffType: 'WorkingTree' | 'Staged' | 'HeadToWorking' = 'HeadToWorking'

    // 如果明确指定了isStaged参数，优先使用
    if (isStaged !== undefined) {
      if (isStaged && stagedFile) {
        // 用户点击的是暂存区的文件，显示暂存区与HEAD的差异
        currentDiffType = 'Staged'
      } else if (!isStaged && unstagedFile) {
        // 用户点击的是工作区的文件，显示工作区与暂存区的差异
        currentDiffType = 'WorkingTree'
      } else {
        // 默认显示工作区与HEAD的差异
        currentDiffType = 'HeadToWorking'
      }
    } else {
      // 兼容旧的逻辑（没有isStaged参数时）
      if (stagedFile) {
        currentDiffType = 'Staged'
      } else if (unstagedFile) {
        currentDiffType = 'WorkingTree'
      } else {
        currentDiffType = 'HeadToWorking'
      }
    }

    console.log(`📋 [GitPanel] 差异类型: ${currentDiffType}`)

    // 使用WindowManager打开新窗口
    await WindowManager.openDiffViewer(filePath, currentDiffType)
    console.log(`✅ [GitPanel] 已打开差异查看器窗口: ${filePath}`)
  } catch (error) {
    console.error('❌ [GitPanel] 打开差异查看器失败:', error)
    // 可以在这里添加用户友好的错误提示
    toast.error(`打开差异查看器失败: ${error instanceof Error ? error.message : '未知错误'}`, '操作失败')
  }
}

// 快捷键处理
const handleKeydown = (event: KeyboardEvent) => {
  if (event.ctrlKey && event.key === 'g') {
    event.preventDefault()
    generateCommitMessage()
  } else if (event.ctrlKey && event.key === 'Enter') {
    event.preventDefault()
    if (commitMessage.value.trim() && hasCommittableFiles.value) {
      commitChanges()
    }
  } else if (event.key === 'Escape' && isGenerating.value) {
    // 取消生成
    if (generateTimeout) {
      clearTimeout(generateTimeout)
      generateTimeout = null
      isGenerating.value = false
      loading.value = false
      generationProgress.value = '已取消生成'
      setTimeout(() => {
        generationProgress.value = ''
      }, 1000)
    }
  }
}

// 生命周期
onMounted(async () => {
  // 初始化Toast实例
  if (toastRef.value) {
    setToastInstance(toastRef.value)
  }

  // 等待 Tauri 初始化
  try {
    // 测试 invoke 函数是否可用
    await new Promise(resolve => setTimeout(resolve, 100)) // 等待100ms
    if (typeof invoke === 'function') {
      tauriReady.value = true
      console.log('Tauri API 已就绪')

      // 加载最近仓库列表
      loadRecentRepos()

      // 自动加载上次打开的仓库
      await autoLoadLastRepo()
    } else {
      console.error('Tauri API 未正确加载')
    }
  } catch (error) {
    console.error('Tauri 初始化失败:', error)
  }

  // 添加快捷键监听
  document.addEventListener('keydown', handleKeydown)
})

// 清理
onUnmounted(() => {
  document.removeEventListener('keydown', handleKeydown)
  if (generateTimeout) {
    clearTimeout(generateTimeout)
  }
})
</script>

<style scoped>
.git-panel {
  display: flex;
  flex-direction: column;
  gap: 12px;
  max-height: 90vh;
  overflow-y: auto;
}

/* 仓库头部 */
.repo-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 12px;
  background: #f8fafc;
  border: 1px solid #e2e8f0;
  border-radius: 6px;
  min-height: 40px;
}

/* 功能按钮组样式 */
.function-buttons {
  display: flex;
  gap: 12px;
  align-items: center;
  flex-wrap: wrap;
}

/* 全局加载指示器 */
.global-loading {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  background: linear-gradient(135deg, #e3f2fd 0%, #f0f9ff 100%);
  border: 1px solid #2196f3;
  border-radius: 6px;
  font-size: 12px;
  color: #1976d2;
  min-width: 200px;
}

.loading-spinner {
  width: 16px;
  height: 16px;
  border: 2px solid #bbdefb;
  border-top: 2px solid #2196f3;
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  0% {
    transform: rotate(0deg);
  }

  100% {
    transform: rotate(360deg);
  }
}

.loading-text {
  font-weight: 500;
  white-space: nowrap;
}

.loading-progress-bar {
  flex: 1;
  height: 4px;
  background: #bbdefb;
  border-radius: 2px;
  overflow: hidden;
}

.loading-progress-fill {
  height: 100%;
  background: linear-gradient(90deg, #2196f3, #1976d2);
  border-radius: 2px;
  transition: width 0.3s ease;
}

/* AI设置按钮样式 */
.ai-settings-btn {
  padding: 8px 16px;
  background: linear-gradient(135deg, #ff6b6b 0%, #ee5a24 100%);
  color: white;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-size: 14px;
  font-weight: 600;
  transition: all 0.3s ease;
  white-space: nowrap;
}

.ai-settings-btn:hover:not(:disabled) {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(255, 107, 107, 0.4);
}

.ai-settings-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

/* 仓库选择器样式 */
.repo-selector {
  display: flex;
  gap: 8px;
  align-items: center;
}

.select-repo-btn {
  padding: 8px 16px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-size: 14px;
  transition: all 0.3s ease;
}

.select-repo-btn:hover:not(:disabled) {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(102, 126, 234, 0.4);
}

.select-repo-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

/* 最近仓库下拉菜单样式 */
.recent-repos-dropdown {
  position: relative;
}

.recent-dropdown-btn {
  padding: 8px 12px;
  background: rgba(255, 255, 255, 0.9);
  border: 1px solid #ddd;
  border-radius: 6px;
  cursor: pointer;
  font-size: 16px;
  transition: all 0.3s ease;
}

.recent-dropdown-btn:hover:not(:disabled) {
  background: rgba(255, 255, 255, 1);
  transform: translateY(-1px);
}

.recent-dropdown-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.recent-dropdown-menu {
  position: absolute;
  top: 100%;
  right: 0;
  background: white;
  border: 1px solid #ddd;
  border-radius: 8px;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.15);
  z-index: 1000;
  min-width: 350px;
  max-height: 400px;
  overflow-y: auto;
  margin-top: 4px;
}

.recent-dropdown-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  border-bottom: 1px solid #eee;
  background: #f8f9fa;
  border-radius: 8px 8px 0 0;
  font-weight: 600;
  color: #333;
}

.clear-recent-btn {
  background: none;
  border: none;
  cursor: pointer;
  font-size: 16px;
  padding: 4px;
  border-radius: 4px;
  transition: background-color 0.2s;
}

.clear-recent-btn:hover {
  background: rgba(255, 0, 0, 0.1);
}

.recent-repo-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  cursor: pointer;
  border-bottom: 1px solid #f0f0f0;
  transition: background-color 0.2s;
}

.recent-repo-item:hover {
  background: #f8f9fa;
}

.recent-repo-item.active {
  background: rgba(102, 126, 234, 0.1);
  border-left: 3px solid #667eea;
}

.recent-repo-item:last-child {
  border-bottom: none;
}

.repo-item-info {
  flex: 1;
  min-width: 0;
}

.repo-item-name {
  font-weight: 600;
  color: #333;
  margin-bottom: 4px;
}

.repo-item-path {
  font-size: 12px;
  color: #666;
  margin-bottom: 2px;
  word-break: break-all;
}

.repo-item-time {
  font-size: 11px;
  color: #999;
}

.remove-repo-btn {
  background: none;
  border: none;
  cursor: pointer;
  font-size: 18px;
  color: #999;
  padding: 4px 8px;
  border-radius: 4px;
  transition: all 0.2s;
  margin-left: 8px;
}

.remove-repo-btn:hover {
  background: rgba(255, 0, 0, 0.1);
  color: #ff4444;
}

.repo-info {
  display: flex;
  align-items: center;
  gap: 12px;
  flex: 1;
  min-width: 0;
}

.repo-name {
  color: #1a202c;
  font-size: 14px;
  font-weight: 600;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.branch-info {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-shrink: 0;
}

.branch-name {
  font-weight: 500;
  color: #2d3748;
  font-size: 12px;
}

.ahead {
  background: #48bb78;
  color: white;
  padding: 2px 6px;
  border-radius: 12px;
  font-size: 11px;
  font-weight: 600;
}

.behind {
  background: #ed8936;
  color: white;
  padding: 2px 6px;
  border-radius: 12px;
  font-size: 11px;
  font-weight: 600;
}



/* Git状态面板 */
.git-status-panel {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.file-section {
  border: 1px solid #e2e8f0;
  border-radius: 8px;
  overflow: hidden;
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  background: #f7fafc;
  border-bottom: 1px solid #e2e8f0;
}

.section-header h4 {
  margin: 0;
  font-size: 14px;
  font-weight: 600;
  color: #2d3748;
}

.section-actions {
  display: flex;
  gap: 4px;
}

.action-btn {
  padding: 4px 8px;
  background: transparent;
  border: 1px solid #cbd5e0;
  border-radius: 4px;
  cursor: pointer;
  font-size: 12px;
  transition: all 0.2s ease;
}

.action-btn:hover {
  background: #edf2f7;
  border-color: #a0aec0;
}

.file-list {
  padding: 4px;
  background: white;
}

/* 提交区域 */
.commit-section {
  padding: 12px;
  background: #f7fafc;
  border: 1px solid #e2e8f0;
  border-radius: 6px;
}

.commit-input {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.commit-textarea {
  width: 100%;
  padding: 12px;
  border: 2px solid #e2e8f0;
  border-radius: 6px;
  font-family: inherit;
  font-size: 14px;
  resize: vertical;
  min-height: 80px;
  transition: border-color 0.2s ease;
}

.commit-textarea:focus {
  outline: none;
  border-color: #667eea;
}

.commit-actions {
  display: flex;
  gap: 8px;
  align-items: center;
}

.ai-generate-section {
  display: flex;
  gap: 8px;
  align-items: center;
}

.template-select {
  padding: 6px 8px;
  border: 1px solid #ddd;
  border-radius: 4px;
  background: white;
  font-size: 12px;
  min-width: 100px;
}

.template-select:focus {
  outline: none;
  border-color: #007acc;
}

.generation-progress {
  margin-top: 8px;
  padding: 12px;
  background: #e3f2fd;
  border: 1px solid #2196f3;
  border-radius: 6px;
  font-size: 12px;
  color: #1976d2;
}

.progress-content {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.progress-text {
  font-weight: 500;
}

.progress-bar {
  height: 4px;
  background: #bbdefb;
  border-radius: 2px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: linear-gradient(90deg, #2196f3, #1976d2);
  border-radius: 2px;
  animation: progress-fill 2s ease-in-out infinite;
}

@keyframes progress-fill {
  0% {
    width: 0%;
  }

  50% {
    width: 70%;
  }

  100% {
    width: 100%;
  }
}

/* 提交消息预览样式 */
.message-preview {
  margin-top: 8px;
  padding: 8px 12px;
  background: #f0f9ff;
  border: 1px solid #0ea5e9;
  border-radius: 6px;
  font-size: 12px;
}

.preview-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.preview-label {
  color: #0369a1;
  font-weight: 500;
}

.preview-actions {
  display: flex;
  gap: 4px;
}

.preview-action-btn {
  background: none;
  border: none;
  padding: 4px;
  border-radius: 4px;
  cursor: pointer;
  font-size: 12px;
  color: #0369a1;
  transition: background-color 0.2s ease;
}

.preview-action-btn:hover:not(:disabled) {
  background: rgba(3, 105, 161, 0.1);
}

.preview-action-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* 重新生成按钮样式 */
.regenerate-btn {
  padding: 6px 12px;
  background: #f59e0b;
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 12px;
  font-weight: 500;
  transition: all 0.2s ease;
}

.regenerate-btn:hover:not(:disabled) {
  background: #d97706;
  transform: translateY(-1px);
}

.regenerate-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
  transform: none;
}

@keyframes pulse {
  0% {
    opacity: 1;
  }

  50% {
    opacity: 0.7;
  }

  100% {
    opacity: 1;
  }
}

.generate-btn {
  padding: 8px 16px;
  background: #48bb78;
  color: white;
  border: none;
  border-radius: 6px;
  font-weight: 600;
  cursor: pointer;
  transition: background-color 0.2s ease;
}

.generate-btn:hover:not(:disabled) {
  background: #38a169;
}

.generate-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.commit-btn {
  padding: 8px 16px;
  background: #667eea;
  color: white;
  border: none;
  border-radius: 6px;
  font-weight: 600;
  cursor: pointer;
  transition: background-color 0.2s ease;
}

.commit-btn:hover:not(:disabled) {
  background: #5a67d8;
}

.commit-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.commit-hint {
  margin-top: 8px;
  padding: 8px 12px;
  background: #fff3cd;
  border: 1px solid #ffeaa7;
  border-radius: 4px;
  color: #856404;
  font-size: 12px;
  text-align: center;
}

.commit-hint p {
  margin: 0;
}

.commit-textarea:disabled {
  background-color: #f8f9fa;
  color: #6c757d;
  cursor: not-allowed;
}

/* 无更改状态 */
.no-changes {
  text-align: center;
  padding: 40px 20px;
  color: #718096;
}

.no-changes p {
  margin: 0;
  font-size: 16px;
}

/* 提交历史 */
.commit-history {
  border: 1px solid #e2e8f0;
  border-radius: 8px;
  overflow: hidden;
}

.history-list {
  max-height: 300px;
  overflow-y: auto;
  background: white;
}

.commit-item {
  padding: 12px 16px;
  border-bottom: 1px solid #f1f5f9;
  transition: background-color 0.2s ease;
}

.commit-item:hover {
  background: #f8fafc;
}

.commit-item:last-child {
  border-bottom: none;
}

.commit-message {
  font-weight: 500;
  color: #2d3748;
  margin-bottom: 4px;
  line-height: 1.4;
}

.commit-meta {
  display: flex;
  gap: 12px;
  font-size: 12px;
  color: #718096;
}

.commit-author {
  font-weight: 500;
}

.commit-hash {
  font-family: 'Consolas', 'Monaco', monospace;
  background: #edf2f7;
  padding: 2px 4px;
  border-radius: 3px;
}

/* 深色主题支持 */
@media (prefers-color-scheme: dark) {

  .repo-header,
  .file-section,
  .commit-section,
  .commit-history {
    background: #2d3748;
    border-color: #4a5568;
  }

  .section-header {
    background: #1a202c;
    border-color: #4a5568;
  }

  .file-list,
  .history-list {
    background: #2d3748;
  }

  .commit-textarea {
    background: #2d3748;
    border-color: #4a5568;
    color: #e2e8f0;
  }

  .repo-name,
  .section-header h4,
  .commit-message {
    color: #e2e8f0;
  }

  .commit-meta {
    color: #a0aec0;
  }

  .branch-name {
    color: #e2e8f0;
  }

  .commit-item:hover {
    background: #4a5568;
  }

  .commit-hash {
    background: #4a5568;
    color: #e2e8f0;
  }

  .no-changes {
    color: #a0aec0;
  }

  .commit-hint {
    background: #2d3748;
    border-color: #4a5568;
    color: #e2e8f0;
  }

  .commit-textarea:disabled {
    background-color: #2d3748;
    color: #a0aec0;
  }
}

/* 差异查看器弹窗样式 */
.diff-viewer-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  z-index: 1000;
  display: flex;
  align-items: center;
  justify-content: center;
}

.diff-viewer-overlay>* {
  width: 90vw;
  height: 90vh;
  max-width: 1200px;
  max-height: 800px;
  border-radius: 8px;
  overflow: hidden;
  box-shadow: 0 10px 30px rgba(0, 0, 0, 0.3);
}
</style>
