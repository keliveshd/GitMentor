<!--  --><template>
  <div class="git-panel">
    <!-- 菜单栏 -->
    <div class="menu-bar">
      <div class="menu-left">
        <span class="app-title">GitMentor</span>
      </div>
      <div class="menu-right">
        <div class="menu-dropdown">
          <button @click="toggleMenu" class="menu-btn" title="菜单">
            ☰
          </button>
          <div v-if="showMenu" class="menu-dropdown-content">
            <button @click="openAISettings" class="menu-item" :disabled="loading || !tauriReady">
              🤖 AI服务设置
            </button>
            <button @click="openTemplateConfig" class="menu-item" :disabled="loading || !tauriReady">
              📝 模板配置
            </button>
            <button @click="openConversationHistory" class="menu-item" :disabled="loading || !tauriReady">
              📊 对话记录
            </button>
            <div class="menu-divider"></div>
            <button @click="openDebugSettings" class="menu-item">
              🛠️ 开发设置
            </button>
            <button @click="openAbout" class="menu-item">
              ℹ️ 关于
            </button>
          </div>
        </div>
      </div>
    </div>



    <!-- Tab导航栏 -->
    <!-- Author: Evilek, Date: 2025-01-08 -->
    <div class="tab-navigation">
      <div class="tab-list">
        <button v-for="tab in tabs" :key="tab.id" @click="switchTab(tab.id)"
          :class="['tab-item', { active: activeTab === tab.id }]" :title="tab.name">
          <span class="tab-icon">{{ tab.icon }}</span>
          <span class="tab-name">{{ tab.name }}</span>
        </button>
      </div>
    </div>

    <!-- 加载状态 -->
    <div v-if="loading" class="loading-status">
      <div class="loading-info">
        <div class="loading-spinner"></div>
        <span class="loading-text">{{ loadingText || '加载中...' }}</span>
      </div>
    </div>

    <!-- Tab页内容区域 -->
    <!-- Author: Evilek, Date: 2025-01-08 -->
    <div class="tab-content">
      <!-- 消息生成Tab页 -->
      <div v-show="activeTab === 'message-generation'" class="tab-pane">
        <!-- 仓库信息 -->
        <div class="repo-header">
          <div class="repo-info" v-if="currentRepoPath">
            <span class="repo-name">📂 {{ getRepoName(currentRepoPath) }}</span>
            <span class="branch-info" v-if="gitStatus">
              <span class="branch-name">🌿 {{ gitStatus.branch }}</span>
              <span v-if="gitStatus.ahead > 0" class="ahead">↑{{ gitStatus.ahead }}</span>
              <span v-if="gitStatus.behind > 0" class="behind">↓{{ gitStatus.behind }}</span>
              <span v-if="isRefreshing" class="refresh-indicator" title="正在刷新Git状态">🔄</span>
            </span>
          </div>

          <div class="repo-actions">
            <button @click="openRepository" class="select-repo-btn" :disabled="loading || !tauriReady">
              {{ loading ? '加载中...' : !tauriReady ? '初始化中...' : '选择仓库' }}
            </button>

            <!-- 最近仓库按钮 -->
            <div class="recent-repos-dropdown" v-if="recentRepos.length > 0">
              <button @click="toggleRecentDropdown" class="recent-dropdown-btn" :disabled="loading || !tauriReady"
                title="最近打开的仓库">
                ⋯
              </button>
              <div v-if="showRecentDropdown" class="recent-dropdown-menu">
                <div class="recent-dropdown-header">
                  <span>最近打开的仓库</span>
                  <button @click="clearRecentRepos" class="clear-recent-btn" title="清空历史">清空</button>
                </div>
                <div class="recent-repo-item" v-for="repo in recentRepos" :key="repo.path"
                  @click="openRecentRepo(repo.path)" :class="{ active: repo.path === currentRepoPath }">
                  <div class="repo-item-info">
                    <div class="repo-item-name">{{ repo.name }}</div>
                    <div class="repo-item-path">{{ repo.path }}</div>
                    <div class="repo-item-time">{{ getRepoDisplayTime(repo) }}</div>
                  </div>
                  <button @click.stop="removeRecentRepo(repo.path)" class="remove-repo-btn" title="从历史中移除">×</button>
                </div>
              </div>
            </div>
          </div>
        </div>

        <div class="main-content">
          <!-- 暂存区 -->
          <div class="staged-files" v-if="gitStatus && gitStatus.staged_files.length > 0">
            <div class="section-title">
              <h4>📋 暂存的更改 ({{ gitStatus?.staged_files?.length || 0 }})</h4>
              <div class="section-actions">
                <button @click="toggleBatchMode" class="batch-mode-btn" :class="{ active: batchMode }" title="批量操作模式">
                  {{ batchMode ? '✅ 批量模式' : '☑️ 批量选择' }}
                </button>
                <button @click="unstageAll" class="action-btn" title="取消暂存所有">
                  ↩️
                </button>
              </div>
            </div>

            <!-- 批量操作工具栏 -->
            <div v-if="batchMode && selectedFilesCount > 0" class="batch-toolbar">
              <div class="batch-info">
                <span>已选择 {{ selectedFilesCount }} 个文件</span>
              </div>
              <div class="batch-actions">
                <button v-if="canBatchUnstage" @click="batchUnstageFiles" class="batch-btn unstage-btn"
                  :disabled="loading" title="批量取消暂存选中文件">
                  取消暂存
                </button>
                <button @click="batchRevertFiles" class="batch-btn revert-btn" :disabled="loading" title="批量回滚选中文件">
                  回滚选中
                </button>
                <button @click="selectAllStaged" class="batch-btn select-all-btn" title="全选暂存区文件">
                  全选
                </button>
                <button @click="clearSelection" class="batch-btn clear-btn" title="清空选择">
                  清空
                </button>
              </div>
            </div>

            <div class="file-list">
              <FileItem v-for="file in gitStatus?.staged_files || []" :key="file.path" :file="file" :is-staged="true"
                :batch-mode="batchMode" :selected="selectedFiles.has(file.path)" @toggle-stage="toggleStage"
                @revert="revertFile" @viewDiff="openDiffViewer" @toggle-select="toggleFileSelection" />
            </div>
          </div>

          <!-- 提交区域 -->
          <div class="commit-area" v-if="gitStatus">
            <textarea v-model="commitMessage" placeholder="输入提交消息..." class="commit-input"
              :disabled="!hasCommittableFiles" :style="{ height: commitTextareaHeight + 'px' }"
              @input="adjustTextareaHeight" ref="commitTextarea"></textarea>

            <!-- 推理内容折叠展示区域 - Author: Evilek, Date: 2025-01-10 -->
            <div v-if="reasoningContent" class="reasoning-content-section">
              <div class="reasoning-header" @click="toggleReasoningExpanded">
                <span class="reasoning-icon">🤔</span>
                <span class="reasoning-title">AI推理过程</span>
                <span class="reasoning-toggle">{{ reasoningExpanded ? '▼' : '▶' }}</span>
              </div>
              <div v-if="reasoningExpanded" class="reasoning-content">
                <pre class="reasoning-text">{{ reasoningContent }}</pre>
              </div>
            </div>

            <!-- 优化后的水平布局按钮区域 - 节省垂直空间 -->
            <div class="commit-controls-horizontal">
              <div class="left-controls">
                <select v-model="selectedTemplate" class="template-select" title="选择提交消息模板风格">
                  <option v-for="template in availableTemplates" :key="template.id" :value="template.id"
                    :title="template.description">
                    {{ template.name }}
                  </option>
                </select>
              </div>
              <div class="right-controls">
                <button @click="generateCommitMessage" class="action-btn generate-btn"
                  :disabled="loading || !hasCommittableFiles" title="快捷键: Ctrl+G">
                  <span v-if="!isGenerating">AI生成</span>
                  <span v-else>生成中...</span>
                </button>
                <button @click="commitChanges" class="action-btn commit-btn"
                  :disabled="!commitMessage.trim() || loading || !hasCommittableFiles" title="快捷键: Ctrl+Enter">
                  提交更改
                </button>
              </div>
            </div>
            <div v-if="!hasCommittableFiles" class="commit-hint">
              <p>工作区干净，没有待提交的更改</p>
            </div>
            <div v-else-if="gitStatus && !gitStatus.staged_files.length" class="commit-hint">
              <p>暂存区为空，AI生成和提交将自动暂存所有修改的文件</p>
            </div>
            <div v-if="generationProgress" class="generation-progress">
              <div class="progress-content">
                <div class="progress-text">{{ generationProgress }}</div>
                <div v-if="isGenerating" class="progress-bar">
                  <div class="progress-fill"></div>
                </div>
              </div>
            </div>
            <!-- AI生成的提交消息预览 - 简化版本 -->
            <div v-if="commitMessage && isAIGenerated" class="message-preview">
              <div class="preview-header">
                <span class="preview-label">AI生成的提交消息</span>
                <div class="preview-actions">
                  <button @click="clearCommitMessage" class="preview-action-btn" title="清空消息">
                    清空
                  </button>
                </div>
              </div>
            </div>
          </div>

          <!-- 工作区更改 -->
          <div class="unstaged-files" v-if="gitStatus && gitStatus.unstaged_files.length > 0">
            <div class="section-title">
              <h4>📝 更改 ({{ gitStatus?.unstaged_files?.length || 0 }})</h4>
              <div class="section-actions">
                <button @click="toggleBatchMode" class="batch-mode-btn" :class="{ active: batchMode }" title="批量操作模式">
                  {{ batchMode ? '✅ 批量模式' : '☑️ 批量选择' }}
                </button>
                <button @click="stageAll" class="action-btn" title="暂存所有">
                  ➕
                </button>
              </div>
            </div>

            <!-- 批量操作工具栏 -->
            <div v-if="batchMode && selectedFilesCount > 0" class="batch-toolbar">
              <div class="batch-info">
                <span>已选择 {{ selectedFilesCount }} 个文件</span>
              </div>
              <div class="batch-actions">
                <button v-if="canBatchStage" @click="batchStageFiles" class="batch-btn stage-btn" :disabled="loading"
                  title="批量暂存选中文件">
                  暂存选中
                </button>
                <button @click="batchRevertFiles" class="batch-btn revert-btn" :disabled="loading" title="批量回滚选中文件">
                  回滚选中
                </button>
                <button @click="selectAllUnstaged" class="batch-btn select-all-btn" title="全选工作区文件">
                  全选
                </button>
                <button @click="clearSelection" class="batch-btn clear-btn" title="清空选择">
                  清空
                </button>
              </div>
            </div>

            <div class="file-list">
              <FileItem v-for="file in gitStatus?.unstaged_files || []" :key="file.path" :file="file" :is-staged="false"
                :batch-mode="batchMode" :selected="selectedFiles.has(file.path)" @toggle-stage="toggleStage"
                @revert="revertFile" @viewDiff="openDiffViewer" @toggle-select="toggleFileSelection" />
            </div>
          </div>

          <!-- 未跟踪文件 -->
          <div class="file-section" v-if="gitStatus && gitStatus.untracked_files.length > 0">
            <div class="section-header">
              <h4>❓ 未跟踪的文件 ({{ gitStatus?.untracked_files?.length || 0 }})</h4>
              <div class="section-actions">
                <button @click="stageAllUntracked" class="action-btn" title="暂存所有">
                  ➕
                </button>
              </div>
            </div>
            <div class="file-list">
              <FileItem v-for="file in gitStatus?.untracked_files || []" :key="file.path" :file="file"
                :is-staged="false" :batch-mode="batchMode" :selected="selectedFiles.has(file.path)"
                @toggle-stage="toggleStage" @revert="revertFile" @viewDiff="openDiffViewer"
                @toggle-select="toggleFileSelection" />
            </div>
          </div>

          <!-- 冲突文件 -->
          <div class="file-section" v-if="gitStatus && gitStatus.conflicted_files.length > 0">
            <div class="section-header">
              <h4>⚠️ 合并冲突 ({{ gitStatus?.conflicted_files?.length || 0 }})</h4>
            </div>
            <div class="file-list">
              <FileItem v-for="file in gitStatus?.conflicted_files || []" :key="file.path" :file="file"
                :is-staged="false" @toggle-stage="toggleStage" @revert="revertFile" @viewDiff="openDiffViewer" />
            </div>

            <!-- 无更改状态 -->
            <div v-if="gitStatus && !gitStatus.has_changes" class="no-changes">
              <p>✨ 工作区干净，没有待提交的更改</p>
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
          </div>
        </div>
      </div>
    </div>

    <!-- 日报生成Tab页 -->
    <!-- Author: Evilek, Date: 2025-01-08 -->
    <div v-show="activeTab === 'daily-report'" class="tab-pane">
      <div class="construction-container">
        <div class="construction-content">
          <div class="construction-icon">🚧</div>
          <h2 class="construction-title">日报生成功能</h2>
          <p class="construction-subtitle">施工中...</p>
        </div>
      </div>
    </div>
  </div>

  <!-- Toast通知组件 -->
  <Toast ref="toastRef" />

  <!-- 确认对话框组件 -->
  <ConfirmDialog :visible="globalConfirm.visible.value" :options="globalConfirm.options.value"
    @confirm="globalConfirm.confirm" @cancel="globalConfirm.cancel" @close="globalConfirm.close" />

  <!-- 分层提交进度弹窗 -->
  <LayeredCommitProgress :visible="layeredProgress.visible" :session-id="layeredProgress.sessionId"
    :current-step="layeredProgress.currentStep" :total-steps="layeredProgress.totalSteps"
    :current-status="layeredProgress.currentStatus" :current-file="layeredProgress.currentFile"
    :file-summaries="layeredProgress.fileSummaries" :ai-stream-content="layeredProgress.aiStreamContent"
    @cancel="cancelLayeredCommit" />

  <!-- 调试设置弹窗 -->
  <div v-if="showDebugSettings" class="modal-overlay debug-settings-overlay" @click="closeDebugSettings">
    <div class="modal-content debug-settings-modal" @click.stop>
      <div class="modal-header">
        <h3>🛠️ 开发设置</h3>
        <button @click="closeDebugSettings" class="close-btn">×</button>
      </div>
      <div class="modal-body">
        <DebugSettings />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed, watch, nextTick } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import FileItem from './FileItem.vue'
import Toast from './Toast.vue'
import ConfirmDialog from './ConfirmDialog.vue'
import LayeredCommitProgress from './LayeredCommitProgress.vue'
import DebugSettings from './DebugSettings.vue'
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
// 批量操作相关状态
const batchMode = ref(false)
const selectedFiles = ref<Set<string>>(new Set())
const tauriReady = ref(false)
const selectedTemplate = ref('standard')
const isGenerating = ref(false)
const generationProgress = ref('')
const isAIGenerated = ref(false)
const isLayeredCommit = ref(false)
// 推理内容相关状态 - Author: Evilek, Date: 2025-01-10
const reasoningContent = ref<string | null>(null)
const reasoningExpanded = ref(false)
const layeredProgress = ref({
  visible: false,
  sessionId: '',
  currentStep: 0,
  totalSteps: 0,
  currentStatus: '',
  currentFile: '',
  fileSummaries: [],
  aiStreamContent: ''  // AI实时输出内容 - Author: Evilek, Date: 2025-01-10
})

// 模板相关状态
const availableTemplates = ref<any[]>([])
const templatesLoaded = ref(false)
// 刷新状态指示
const isRefreshing = ref(false)
const refreshCount = ref(0)

// 最近仓库相关状态
const recentRepos = ref<RecentRepo[]>([])
const showRecentDropdown = ref(false)

// 菜单状态
const showMenu = ref(false)

// Tab页状态管理
// Author: Evilek
// Date: 2025-01-08
const activeTab = ref('message-generation')
const tabs = ref([
  {
    id: 'message-generation',
    name: '消息生成',
    icon: '💬'
  },
  {
    id: 'daily-report',
    name: '日报生成',
    icon: '📊'
  }
  // 预留其他tab页扩展空间
])

// 调试设置状态
const showDebugSettings = ref(false)

// 提交区域高度自适应相关状态
const commitTextareaHeight = ref(60) // 默认高度约3行
const commitTextarea = ref<HTMLTextAreaElement | null>(null)

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

// 批量操作相关计算属性
const allFiles = computed(() => {
  if (!gitStatus.value) return []
  return [
    ...gitStatus.value.staged_files.map((f: any) => ({ ...f, isStaged: true })),
    ...gitStatus.value.unstaged_files.map((f: any) => ({ ...f, isStaged: false })),
    ...gitStatus.value.untracked_files.map((f: any) => ({ ...f, isStaged: false }))
  ]
})

const selectedFilesCount = computed(() => selectedFiles.value.size)

const canBatchStage = computed(() => {
  return Array.from(selectedFiles.value).some(filePath => {
    const file = allFiles.value.find(f => f.path === filePath)
    return file && !file.isStaged
  })
})

const canBatchUnstage = computed(() => {
  return Array.from(selectedFiles.value).some(filePath => {
    const file = allFiles.value.find(f => f.path === filePath)
    return file && file.isStaged
  })
})

// 差异查看器已改为独立窗口，不再需要本地状态

// 加载状态管理
const setLoading = (isLoading: boolean, text = '') => {
  loading.value = isLoading
  loadingText.value = text
}

// 方法
const openRepository = async () => {
  if (!tauriReady.value) {
    toast.warning('应用正在初始化，请稍后再试', '请稍候')
    return
  }

  try {
    setLoading(true, '正在打开文件夹选择器...')

    const selectedPath = await invoke('open_folder_dialog') as string | null
    if (selectedPath) {
      setLoading(true, '正在加载仓库信息...')
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
// 作者：Evilek
// 编写日期：2025-08-04
const openRepoByPath = async (path: string) => {
  try {
    setLoading(true, '正在选择仓库...')
    currentRepoPath.value = path

    // 清空之前的提示信息和状态
    clearRepositoryState()

    setLoading(true, '正在初始化仓库...')
    await invoke('select_repository', { path })

    setLoading(true, '正在获取Git状态...')
    await refreshGitStatus(true)

    setLoading(true, '正在加载提交历史...')
    await refreshHistory()

    setLoading(true, '正在保存配置...')
    // 保存到最近仓库列表
    RecentReposManager.addRecentRepo(path)
    loadRecentRepos()

    // 关闭下拉菜单
    showRecentDropdown.value = false

    setLoading(true, '完成')
    setTimeout(() => setLoading(false), 500)
  } catch (error) {
    console.error('打开仓库失败:', error)
    toast.error(`打开仓库失败: ${error}`, '操作失败')
    setLoading(false)
    // 重置仓库路径
    currentRepoPath.value = ''
  }
}

// 智能防抖刷新Git状态
const refreshGitStatus = async (force = false) => {
  const now = Date.now()

  // 如果有正在进行的刷新请求，直接返回该Promise
  if (refreshPromise && !force) {
    return refreshPromise
  }

  // 检查最小刷新间隔
  if (!force && now - lastRefreshTime < MIN_REFRESH_INTERVAL) {
    // 如果距离上次刷新时间太短，使用防抖
    if (refreshTimeout) {
      clearTimeout(refreshTimeout)
    }

    return new Promise<void>((resolve) => {
      refreshTimeout = setTimeout(async () => {
        await refreshGitStatus(true)
        resolve()
      }, REFRESH_DEBOUNCE_DELAY)
    })
  }

  // 执行实际的刷新操作
  refreshPromise = (async () => {
    try {
      isRefreshing.value = true
      refreshCount.value++
      const status = await invoke('get_git_status')
      gitStatus.value = status
      lastRefreshTime = Date.now()
    } catch (error) {
      console.error('Failed to get git status:', error)
      // 如果没有仓库打开，不显示错误提示
      if (currentRepoPath.value) {
        toast.error(`获取Git状态失败: ${error}`, '状态更新失败')
      }
      gitStatus.value = null
    } finally {
      isRefreshing.value = false
      refreshPromise = null
    }
  })()

  return refreshPromise
}

// 历史记录刷新（较少频率，不需要防抖）
let historyRefreshPromise: Promise<void> | null = null

const refreshHistory = async () => {
  // 如果有正在进行的历史刷新请求，直接返回该Promise
  if (historyRefreshPromise) {
    return historyRefreshPromise
  }

  historyRefreshPromise = (async () => {
    try {
      const history = await invoke('get_commit_history', { limit: 10 }) as any[]
      commitHistory.value = history
    } catch (error) {
      console.error('Failed to get commit history:', error)
      // 如果没有仓库打开，不显示错误提示
      if (currentRepoPath.value) {
        toast.error(`获取提交历史失败: ${error}`, '历史加载失败')
      }
      commitHistory.value = []
    } finally {
      historyRefreshPromise = null
    }
  })()

  return historyRefreshPromise
}

// 批量操作优化：收集多个操作后一次性刷新
let pendingOperations = new Set<string>()
let operationTimeout: number | null = null
const OPERATION_BATCH_DELAY = 200 // 200ms内的操作会被批量处理

const scheduleRefresh = () => {
  if (operationTimeout) {
    clearTimeout(operationTimeout)
  }

  operationTimeout = setTimeout(async () => {
    if (pendingOperations.size > 0) {
      pendingOperations.clear()
      await refreshGitStatus()
    }
  }, OPERATION_BATCH_DELAY)
}

const toggleStage = async (filePath: string, shouldStage: boolean) => {
  try {
    const result = await invoke('stage_files', {
      request: {
        file_paths: [filePath],
        stage: shouldStage
      }
    }) as any

    // 显示操作结果信息
    if (result.details) {
      toast.warning(result.details, result.message)
    } else {
      toast.success(result.message, '操作成功')
    }

    // 添加到待处理操作集合，延迟刷新
    pendingOperations.add(filePath)
    scheduleRefresh()
  } catch (error) {
    console.error('Failed to toggle stage:', error)
    toast.error('暂存操作失败: ' + error, '操作失败')
  }
}

const stageAll = async () => {
  if (!gitStatus.value?.unstaged_files?.length) return

  try {
    const filePaths = gitStatus.value.unstaged_files.map((f: any) => f.path)
    const result = await invoke('stage_files', {
      request: { file_paths: filePaths, stage: true }
    }) as any

    // 显示操作结果信息
    if (result.details) {
      toast.warning(result.details, result.message)
    } else {
      toast.success(result.message, '操作成功')
    }

    // 批量操作直接刷新，不使用防抖
    await refreshGitStatus(true)
  } catch (error) {
    console.error('Failed to stage all:', error)
    toast.error('暂存所有文件失败: ' + error, '操作失败')
  }
}

const unstageAll = async () => {
  if (!gitStatus.value?.staged_files?.length) return

  try {
    const filePaths = gitStatus.value.staged_files.map((f: any) => f.path)
    await invoke('stage_files', {
      request: { file_paths: filePaths, stage: false }
    })

    // 批量操作直接刷新，不使用防抖
    await refreshGitStatus(true)
  } catch (error) {
    console.error('Failed to unstage all:', error)
    toast.error('取消暂存所有文件失败: ' + error, '操作失败')
  }
}

const stageAllUntracked = async () => {
  if (!gitStatus.value?.untracked_files?.length) return

  try {
    const filePaths = gitStatus.value.untracked_files.map((f: any) => f.path)
    const result = await invoke('stage_files', {
      request: { file_paths: filePaths, stage: true }
    }) as any

    // 显示操作结果信息
    if (result.details) {
      toast.warning(result.details, result.message)
    } else {
      toast.success(result.message, '操作成功')
    }

    // 批量操作直接刷新，不使用防抖
    await refreshGitStatus(true)
  } catch (error) {
    console.error('Failed to stage untracked files:', error)
    toast.error('暂存未跟踪文件失败: ' + error, '操作失败')
  }
}

// 防抖生成函数
let generateTimeout: number | null = null

// 刷新防抖和缓存机制
let refreshTimeout: number | null = null
let lastRefreshTime = 0
const REFRESH_DEBOUNCE_DELAY = 500 // 500ms防抖延迟
const MIN_REFRESH_INTERVAL = 1000 // 最小刷新间隔1秒
let refreshPromise: Promise<void> | null = null

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
      if (!gitStatus.value?.staged_files?.length) {
        generationProgress.value = '暂存区为空，正在自动暂存所有修改的文件...'

        // 暂存所有未暂存的文件
        if (gitStatus.value?.unstaged_files?.length > 0) {
          const unstagedPaths = gitStatus.value.unstaged_files.map((f: any) => f.path)
          const result = await invoke('stage_files', {
            request: { file_paths: unstagedPaths, stage: true }
          }) as any

          // 如果有跳过的文件，记录但不中断流程
          if (result.details) {
            console.warn('暂存时跳过了一些文件:', result.details)
          }
        }

        // 暂存所有未跟踪的文件
        if (gitStatus.value?.untracked_files?.length > 0) {
          const untrackedPaths = gitStatus.value.untracked_files.map((f: any) => f.path)
          const result = await invoke('stage_files', {
            request: { file_paths: untrackedPaths, stage: true }
          }) as any

          // 如果有跳过的文件，记录但不中断流程
          if (result.details) {
            console.warn('暂存时跳过了一些文件:', result.details)
          }
        }

        // 刷新Git状态（强制刷新，因为这是重要操作）
        await refreshGitStatus(true)
      }

      const filePaths = gitStatus.value?.staged_files?.map((f: any) => f.path) || []

      // 获取暂存文件的差异摘要
      generationProgress.value = '正在获取差异信息...'
      // const diffContent = await invoke('get_staged_diff_summary') as string

      // 统一使用分层提交逻辑 - 移除普通提交分支
      // Author: Evilek, Date: 2025-01-08
      generationProgress.value = '准备分层提交处理...'

      // 调试信息：检查当前选择的模板
      console.log('🔍 [GitPanel] 当前选择的模板ID:', selectedTemplate.value)
      console.log('🔍 [GitPanel] 可用模板列表:', availableTemplates.value.map(t => ({ id: t.id, name: t.name })))

      // 确保模板已加载且选择的模板存在
      if (!templatesLoaded.value || availableTemplates.value.length === 0) {
        throw new Error('模板尚未加载完成，请稍后再试')
      }

      const selectedTemplateExists = availableTemplates.value.some(t => t.id === selectedTemplate.value)
      if (!selectedTemplateExists) {
        console.warn('⚠️ [GitPanel] 选择的模板不存在，使用第一个可用模板')
        selectedTemplate.value = availableTemplates.value[0].id
      }

      // 检查单文件token限制并进行预处理
      generationProgress.value = '检查文件token限制...'
      const processedFiles = await checkAndProcessFileTokens(filePaths)

      // 统一使用分层提交（移除普通提交逻辑）
      generationProgress.value = '开始分层提交处理...'
      await executeLayeredCommit(processedFiles, gitStatus.value?.branch || 'main')

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

// 清空提交消息
const clearCommitMessage = () => {
  commitMessage.value = ''
  isAIGenerated.value = false
  // 同时清空推理内容 - Author: Evilek, Date: 2025-01-10
  reasoningContent.value = null
  reasoningExpanded.value = false
}

// 推理内容折叠展开切换 - Author: Evilek, Date: 2025-01-10
const toggleReasoningExpanded = () => {
  reasoningExpanded.value = !reasoningExpanded.value
}

/**
 * 清空仓库状态 - 切换仓库时重置所有相关状态
 * 作者：Evilek
 * 编写日期：2025-08-04
 */
const clearRepositoryState = () => {
  // 清空提交相关状态
  commitMessage.value = ''
  isAIGenerated.value = false
  isGenerating.value = false
  generationProgress.value = ''
  // 清空推理内容 - Author: Evilek, Date: 2025-01-10
  reasoningContent.value = null
  reasoningExpanded.value = false

  // 清空Git状态
  gitStatus.value = null
  commitHistory.value = []

  // 清空批量操作状态
  batchMode.value = false
  selectedFiles.value.clear()

  // 重置其他状态
  isRefreshing.value = false
  refreshCount.value = 0

  // 清空分层提交状态
  isLayeredCommit.value = false
  layeredProgress.value.visible = false

  console.log('🧹 [GitPanel] 已清空仓库状态')
}

/**
 * 检查并处理文件token限制
 * Author: Evilek
 * Date: 2025-01-08
 * 对单文件变更和新增文件进行token检查和分割处理
 */
const checkAndProcessFileTokens = async (filePaths: string[]): Promise<string[]> => {
  try {
    generationProgress.value = '分析文件token使用情况...'

    // 调用后端检查文件token限制
    const result = await invoke('check_and_process_file_tokens', {
      filePaths: filePaths,
      template_id: selectedTemplate.value
    }) as { processedFiles: string[], needsSplit: boolean }

    if (result.needsSplit) {
      generationProgress.value = '检测到大文件，已自动分割处理...'
      console.log('🔧 [GitPanel] 文件已分割处理:', result.processedFiles)
    } else {
      generationProgress.value = '文件token检查完成...'
    }

    return result.processedFiles
  } catch (error) {
    console.warn('⚠️ [GitPanel] 文件token检查失败，使用原始文件列表:', error)
    // 如果检查失败，返回原始文件列表
    return filePaths
  }
}

/**
 * 执行分层提交
 * 作者：Evilek
 * 编写日期：2025-08-04
 */
const executeLayeredCommit = async (stagedFiles: string[], branchName: string | null) => {
  try {
    // 显示分层提交进度弹窗
    layeredProgress.value.visible = true
    loading.value = false // 关闭主加载状态
    isGenerating.value = false

    // 监听进度更新事件
    const unlisten = await listen('layered-commit-progress', (event: any) => {
      const progress = event.payload
      // 修复Vue响应式更新问题 - Author: Evilek, Date: 2025-01-09
      // 使用Object.assign避免直接替换整个对象导致的Vue内部错误
      Object.assign(layeredProgress.value, {
        visible: true,
        sessionId: progress.session_id,
        currentStep: progress.current_step,
        totalSteps: progress.total_steps,
        currentStatus: progress.status,
        currentFile: progress.current_file || '',
        fileSummaries: progress.file_summaries || [],
        aiStreamContent: progress.ai_stream_content || ''  // AI实时输出内容 - Author: Evilek, Date: 2025-01-10
      })
    })

    // 执行分层提交
    const result = await invoke('execute_layered_commit', {
      templateId: selectedTemplate.value,
      stagedFiles: stagedFiles,
      branchName: branchName || 'main'
    }) as any

    // 设置最终结果
    commitMessage.value = result.finalMessage
    isAIGenerated.value = true
    isLayeredCommit.value = true
    // 设置推理内容 - Author: Evilek, Date: 2025-01-10
    reasoningContent.value = result.reasoningContent || null
    reasoningExpanded.value = false // 默认折叠

    toast.success('分层提交消息生成成功', '操作完成')

    // 清理进度状态
    generationProgress.value = '分层提交完成！'
    setTimeout(() => {
      generationProgress.value = ''
    }, 1000)

    // 清理
    unlisten()
    layeredProgress.value.visible = false
  } catch (error) {
    layeredProgress.value.visible = false
    generationProgress.value = '分层提交失败'
    setTimeout(() => {
      generationProgress.value = ''
    }, 2000)
    throw error
  }
}

/**
 * 取消分层提交
 * 作者：Evilek
 * 编写日期：2025-08-04
 * 更新日期：2025-01-09 - 添加真正的任务取消机制
 */
const cancelLayeredCommit = async () => {
  try {
    // 调用后端取消命令，真正中断任务 - Author: Evilek, Date: 2025-01-09
    await invoke('cancel_layered_commit')

    layeredProgress.value.visible = false
    loading.value = false
    isGenerating.value = false
    generationProgress.value = '分层提交已取消'
    setTimeout(() => {
      generationProgress.value = ''
    }, 1000)
    toast.info('分层提交已取消', '操作取消')
  } catch (error) {
    console.error('取消分层提交失败:', error)
    // 即使取消失败，也要关闭UI
    layeredProgress.value.visible = false
    loading.value = false
    isGenerating.value = false
    toast.warning('取消操作可能未完全生效', '操作警告')
  }
}

// 批量操作相关方法
const toggleBatchMode = () => {
  batchMode.value = !batchMode.value
  if (!batchMode.value) {
    selectedFiles.value.clear()
  }
}

const toggleFileSelection = (filePath: string) => {
  if (selectedFiles.value.has(filePath)) {
    selectedFiles.value.delete(filePath)
  } else {
    selectedFiles.value.add(filePath)
  }
}

const selectAllUnstaged = () => {
  if (!gitStatus.value) return
  gitStatus.value.unstaged_files.forEach((file: any) => {
    selectedFiles.value.add(file.path)
  })
  gitStatus.value.untracked_files.forEach((file: any) => {
    selectedFiles.value.add(file.path)
  })
}

const selectAllStaged = () => {
  if (!gitStatus.value) return
  gitStatus.value.staged_files.forEach((file: any) => {
    selectedFiles.value.add(file.path)
  })
}

const clearSelection = () => {
  selectedFiles.value.clear()
}

const batchStageFiles = async () => {
  const selectedPaths = Array.from(selectedFiles.value)
  const confirmed = await confirm.info(
    '批量暂存文件',
    `确定要暂存选中的 ${selectedPaths.length} 个文件吗？`,
    selectedPaths.join('\n')
  )

  if (!confirmed) return

  try {
    setLoading(true, '正在批量暂存文件...')
    const result = await invoke('stage_files', {
      request: { file_paths: selectedPaths, stage: true }
    }) as any

    setLoading(true, '正在刷新状态...')
    await refreshGitStatus(true)

    selectedFiles.value.clear()
    setLoading(true, '批量暂存完成')

    // 显示详细的操作结果
    if (result.details) {
      toast.warning(result.details, result.message)
    } else {
      toast.success(result.message, '操作完成')
    }

    setTimeout(() => setLoading(false), 1000)
  } catch (error) {
    console.error('Failed to batch stage files:', error)
    toast.error('批量暂存失败: ' + error, '操作失败')
    setLoading(false)
  }
}

const batchRevertFiles = async () => {
  const selectedPaths = Array.from(selectedFiles.value)
  const confirmed = await confirm.danger(
    '批量回滚文件',
    `确定要回滚选中的 ${selectedPaths.length} 个文件吗？`,
    '此操作将丢失这些文件的所有未提交更改，且无法撤销。\n\n文件列表：\n' + selectedPaths.join('\n')
  )

  if (!confirmed) return

  try {
    setLoading(true, '正在批量回滚文件...')

    // 分别处理暂存区和工作区的文件
    const stagedFiles = selectedPaths.filter(path => {
      const file = allFiles.value.find(f => f.path === path)
      return file && file.isStaged
    })

    const unstagedFiles = selectedPaths.filter(path => {
      const file = allFiles.value.find(f => f.path === path)
      return file && !file.isStaged
    })

    if (stagedFiles.length > 0) {
      await invoke('revert_files', {
        request: {
          file_paths: stagedFiles,
          revert_type: 'Staged'
        }
      })
    }

    if (unstagedFiles.length > 0) {
      await invoke('revert_files', {
        request: {
          file_paths: unstagedFiles,
          revert_type: 'WorkingTree'
        }
      })
    }

    setLoading(true, '正在刷新状态...')
    await refreshGitStatus(true)

    selectedFiles.value.clear()
    setLoading(true, '批量回滚完成')
    toast.success(`成功回滚 ${selectedPaths.length} 个文件`, '操作完成')
    setTimeout(() => setLoading(false), 1000)
  } catch (error) {
    console.error('Failed to batch revert files:', error)
    toast.error('批量回滚失败: ' + error, '操作失败')
    setLoading(false)
  }
}

const batchUnstageFiles = async () => {
  const selectedPaths = Array.from(selectedFiles.value)
  const confirmed = await confirm.info(
    '批量取消暂存文件',
    `确定要取消暂存选中的 ${selectedPaths.length} 个文件吗？`,
    selectedPaths.join('\n')
  )

  if (!confirmed) return

  try {
    setLoading(true, '正在批量取消暂存文件...')
    await invoke('stage_files', {
      request: { file_paths: selectedPaths, stage: false }
    })

    setLoading(true, '正在刷新状态...')
    await refreshGitStatus(true)

    selectedFiles.value.clear()
    setLoading(true, '批量取消暂存完成')
    toast.success(`成功取消暂存 ${selectedPaths.length} 个文件`, '操作完成')
    setTimeout(() => setLoading(false), 1000)
  } catch (error) {
    console.error('Failed to batch unstage files:', error)
    toast.error('批量取消暂存失败: ' + error, '操作失败')
    setLoading(false)
  }
}

const commitChanges = async () => {
  if (!commitMessage.value.trim() || !hasCommittableFiles.value) return

  try {
    setLoading(true, '准备提交...')

    // 如果暂存区为空，先暂存所有修改的文件
    if (!gitStatus.value.staged_files.length) {
      setLoading(true, '正在暂存文件...')

      // 暂存所有未暂存的文件
      if (gitStatus.value.unstaged_files.length > 0) {
        const unstagedPaths = gitStatus.value.unstaged_files.map((f: any) => f.path)
        const result = await invoke('stage_files', {
          request: { file_paths: unstagedPaths, stage: true }
        }) as any

        // 如果有跳过的文件，记录但不中断流程
        if (result.details) {
          console.warn('提交前暂存时跳过了一些文件:', result.details)
        }
      }

      // 暂存所有未跟踪的文件
      if (gitStatus.value.untracked_files.length > 0) {
        const untrackedPaths = gitStatus.value.untracked_files.map((f: any) => f.path)
        const result = await invoke('stage_files', {
          request: { file_paths: untrackedPaths, stage: true }
        }) as any

        // 如果有跳过的文件，记录但不中断流程
        if (result.details) {
          console.warn('提交前暂存时跳过了一些文件:', result.details)
        }
      }

      setLoading(true, '正在刷新状态...')
      // 刷新Git状态（强制刷新，因为这是重要操作）
      await refreshGitStatus(true)
    }

    setLoading(true, '正在提交更改...')
    await invoke('commit_changes', {
      request: {
        message: commitMessage.value,
        selected_files: [],
        additional_context: null,
        amend: false
      }
    })

    setLoading(true, '正在更新状态...')
    commitMessage.value = ''
    await refreshGitStatus(true)
    await refreshHistory()

    setLoading(true, '提交完成！')
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
    setLoading(true, `正在回滚${revertType}文件...`)
    await invoke('revert_files', {
      request: {
        file_paths: [filePath],
        revert_type: isStaged ? 'Staged' : 'WorkingTree'
      }
    })

    setLoading(true, '正在刷新状态...')
    await refreshGitStatus(true)

    setLoading(true, '回滚完成')
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

// 菜单功能切换
const toggleMenu = () => {
  showMenu.value = !showMenu.value
}

// Tab页切换方法
// Author: Evilek
// Date: 2025-01-08
const switchTab = (tabId: string) => {
  activeTab.value = tabId
  // 关闭菜单下拉框（如果打开的话）
  showMenu.value = false
}

// 调试设置功能
const openDebugSettings = () => {
  showDebugSettings.value = true
  showMenu.value = false
}

const closeDebugSettings = () => {
  showDebugSettings.value = false
}

// 关于功能
const openAbout = () => {
  // TODO: 实现关于对话框
  console.log('打开关于对话框')
  showMenu.value = false
}

// 自动加载上次打开的仓库
const autoLoadLastRepo = async () => {
  const lastRepoPath = RecentReposManager.getLastOpenedRepo()
  if (lastRepoPath && tauriReady.value) {
    try {
      // 验证路径是否仍然有效
      await invoke('select_repository', { path: lastRepoPath })
      currentRepoPath.value = lastRepoPath
      await refreshGitStatus(true)
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

// 加载可用模板列表
// 作者：Evilek
// 编写日期：2025-01-29
const loadAvailableTemplates = async () => {
  try {
    console.log('📝 [GitPanel] 加载可用模板列表')

    // 获取默认模板和自定义模板
    const [defaultTemplates, customTemplates] = await Promise.all([
      invoke('get_default_templates') as Promise<any[]>,
      invoke('get_custom_templates') as Promise<any[]>
    ])

    // 合并模板列表
    availableTemplates.value = [...defaultTemplates, ...customTemplates]
    templatesLoaded.value = true

    // 如果当前选择的模板不在列表中，选择第一个可用模板
    if (availableTemplates.value.length > 0) {
      const currentTemplateExists = availableTemplates.value.some(t => t.id === selectedTemplate.value)
      if (!currentTemplateExists) {
        console.log('⚠️ [GitPanel] 当前选择的模板不存在，从', selectedTemplate.value, '切换到', availableTemplates.value[0].id)
        selectedTemplate.value = availableTemplates.value[0].id
      } else {
        console.log('✅ [GitPanel] 当前选择的模板存在:', selectedTemplate.value)
      }
    }

    console.log('✅ [GitPanel] 模板列表加载完成，共', availableTemplates.value.length, '个模板')
  } catch (error) {
    console.error('❌ [GitPanel] 加载模板列表失败:', error)
    // 如果加载失败，使用默认的硬编码模板
    availableTemplates.value = [
      { id: 'standard', name: '标准提交', description: '生成符合常规规范的英文提交消息' },
      { id: 'chinese', name: '中文提交', description: '生成简洁明了的中文提交消息' },
      { id: 'detailed', name: '详细提交', description: '生成包含详细描述的提交消息' },
      { id: 'conventional', name: '约定式提交', description: '生成符合约定式提交规范的消息' }
    ]
    templatesLoaded.value = true
  }
}

// 打开模板配置窗口
// 作者：Evilek
// 编写日期：2025-01-29
const openTemplateConfig = async () => {
  try {
    console.log('📝 [GitPanel] 打开模板配置窗口')

    // 使用WindowManager打开模板配置窗口
    await WindowManager.openTemplateConfig()
    console.log('✅ [GitPanel] 已打开模板配置窗口')

    // 模板配置窗口关闭后重新加载模板列表
    // 注意：这里可能需要监听窗口关闭事件，暂时先在这里重新加载
    setTimeout(() => {
      loadAvailableTemplates()
    }, 1000)
  } catch (error) {
    console.error('❌ [GitPanel] 打开模板配置窗口失败:', error)
    alert(`打开模板配置失败: ${error instanceof Error ? error.message : '未知错误'}`)
  }
}

// 打开对话记录窗口
// 作者：Evilek
// 编写日期：2025-01-30
const openConversationHistory = async () => {
  try {
    console.log('📊 [GitPanel] 打开对话记录窗口')
    showMenu.value = false

    // 使用WindowManager打开对话记录窗口
    await WindowManager.openConversationHistory()
    console.log('✅ [GitPanel] 已打开对话记录窗口')
  } catch (error) {
    console.error('❌ [GitPanel] 打开对话记录窗口失败:', error)
    alert(`打开对话记录失败: ${error instanceof Error ? error.message : '未知错误'}`)
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

/**
 * 调整提交消息输入框高度
 * Author: Evilek
 * Date: 2025-01-29
 * 根据内容行数自适应高度，最大10行，超过则显示滚动条
 */
const adjustTextareaHeight = () => {
  if (!commitTextarea.value) return

  const textarea = commitTextarea.value
  const lineHeight = 20 // 每行高度
  const padding = 24 // 上下padding总和
  const minHeight = lineHeight * 3 + padding // 最小3行
  const maxHeight = lineHeight * 10 + padding // 最大10行

  // 重置高度以获取正确的scrollHeight
  textarea.style.height = 'auto'

  // 计算内容高度
  const contentHeight = textarea.scrollHeight

  // 设置高度：在最小和最大高度之间
  const newHeight = Math.max(minHeight, Math.min(contentHeight, maxHeight))
  commitTextareaHeight.value = newHeight

  // 如果内容超过最大高度，启用滚动
  if (contentHeight > maxHeight) {
    textarea.style.overflowY = 'auto'
  } else {
    textarea.style.overflowY = 'hidden'
  }

  // 强制重新布局，确保父容器能够感知高度变化
  nextTick(() => {
    textarea.style.height = newHeight + 'px'
  })
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

// 监听提交消息变化，自动调整高度并重置AI生成标记
watch(commitMessage, (newValue, oldValue) => {
  nextTick(() => {
    adjustTextareaHeight()
  })

  // 如果用户手动修改了消息，重置AI生成标记
  if (newValue !== oldValue && isAIGenerated.value) {
    // 延迟重置，避免在AI生成时误触发
    setTimeout(() => {
      if (!isGenerating.value) {
        isAIGenerated.value = false
      }
    }, 100)
  }
})

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

      // 加载可用模板列表
      await loadAvailableTemplates()

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

  // 初始化提交输入框高度
  nextTick(() => {
    adjustTextareaHeight()
  })
})

// 清理
onUnmounted(() => {
  document.removeEventListener('keydown', handleKeydown)
  if (generateTimeout) {
    clearTimeout(generateTimeout)
  }
  if (refreshTimeout) {
    clearTimeout(refreshTimeout)
  }
  if (operationTimeout) {
    clearTimeout(operationTimeout)
  }
})
</script>

<style scoped>
.git-panel {
  position: relative;
  /* 为绝对定位的加载状态提供定位上下文 */
  display: flex;
  flex-direction: column;
  gap: 12px;
  /* 移除固定高度，改为根据内容自适应 - 修复暂存区为空时占用大量空间的问题 */
  min-height: 100vh;
  /* 允许内容超出视口高度时滚动 */
}

/* 菜单栏样式 - 优化高度以节省垂直空间 */
.menu-bar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 6px 16px;
  background: #667eea;
  color: white;
  margin-bottom: 12px;
}

.menu-left .app-title {
  font-size: 16px;
  font-weight: 600;
}

.menu-dropdown {
  position: relative;
}

.menu-btn {
  background: none;
  border: none;
  color: white;
  font-size: 16px;
  cursor: pointer;
  padding: 4px 8px;
  border-radius: 4px;
  transition: background-color 0.2s ease;
}

.menu-btn:hover {
  background: rgba(255, 255, 255, 0.1);
}

.menu-dropdown-content {
  position: absolute;
  right: 0;
  top: 100%;
  background: white;
  border: 1px solid #e2e8f0;
  border-radius: 6px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  min-width: 150px;
  z-index: 1000;
}

.menu-item {
  display: block;
  width: 100%;
  padding: 8px 12px;
  background: none;
  border: none;
  text-align: left;
  cursor: pointer;
  font-size: 14px;
  color: #374151;
  transition: background-color 0.2s ease;
}

.menu-item:hover:not(:disabled) {
  background: #f3f4f6;
}

.menu-item:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* Tab导航样式 */
/* Author: Evilek, Date: 2025-01-08 */
.tab-navigation {
  background: #f8fafc;
  border-bottom: 1px solid #e2e8f0;
  padding: 0 16px;
}

.tab-list {
  display: flex;
  gap: 2px;
}

.tab-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 12px 16px;
  background: none;
  border: none;
  border-radius: 8px 8px 0 0;
  cursor: pointer;
  font-size: 14px;
  font-weight: 500;
  color: #6b7280;
  transition: all 0.2s ease;
  position: relative;
}

.tab-item:hover {
  background: rgba(99, 102, 241, 0.1);
  color: #4f46e5;
}

.tab-item.active {
  background: white;
  color: #4f46e5;
  border: 1px solid #e2e8f0;
  border-bottom: 1px solid white;
  margin-bottom: -1px;
}

.tab-icon {
  font-size: 16px;
}

.tab-name {
  font-weight: 500;
}

/* Tab内容区域 */
.tab-content {
  flex: 1;
  display: flex;
  flex-direction: column;
}

.tab-pane {
  flex: 1;
  display: flex;
  flex-direction: column;
}

/* 施工中页面样式 */
/* Author: Evilek, Date: 2025-01-08 */
.construction-container {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 40px 20px;
  background: linear-gradient(135deg, #f8fafc 0%, #e2e8f0 100%);
}

.construction-content {
  text-align: center;
  max-width: 500px;
  background: white;
  padding: 40px 30px;
  border-radius: 16px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.1);
  border: 1px solid #e2e8f0;
}

.construction-icon {
  font-size: 4rem;
  margin-bottom: 20px;
  animation: bounce 2s infinite;
}

@keyframes bounce {

  0%,
  20%,
  50%,
  80%,
  100% {
    transform: translateY(0);
  }

  40% {
    transform: translateY(-10px);
  }

  60% {
    transform: translateY(-5px);
  }
}

.construction-title {
  color: #374151;
  font-size: 1.8rem;
  font-weight: 600;
  margin: 0 0 10px 0;
}

.construction-subtitle {
  color: #6b7280;
  font-size: 1.2rem;
  margin: 0 0 30px 0;
  font-weight: 500;
}

.construction-details {
  text-align: left;
  background: #f8fafc;
  padding: 20px;
  border-radius: 8px;
  border: 1px solid #e2e8f0;
}

.construction-details p {
  margin: 8px 0;
  color: #4b5563;
  font-size: 14px;
  line-height: 1.5;
}

/* 仓库头部 */
.repo-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px;
  background: #f8fafc;
  border: 1px solid #e2e8f0;
  border-radius: 8px;
  margin-bottom: 16px;
}

.repo-info {
  display: flex;
  align-items: center;
  gap: 16px;
  flex: 1;
}

.repo-actions {
  display: flex;
  gap: 8px;
  align-items: center;
}

/* 加载状态 */
.loading-status {
  position: absolute;
  top: 120px;
  /* 位于仓库信息下方 */
  left: 16px;
  right: 16px;
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 12px 16px;
  background: #f8fafc;
  border: 1px solid #e2e8f0;
  border-radius: 8px;
  z-index: 15;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.loading-info {
  display: flex;
  align-items: center;
  gap: 8px;
}

.loading-spinner {
  width: 14px;
  height: 14px;
  border: 2px solid #e2e8f0;
  border-top: 2px solid #667eea;
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
  font-size: 12px;
  color: #6b7280;
  font-weight: 500;
}



/* 选择仓库按钮 - 中等尺寸，重要操作 */
.select-repo-btn {
  padding: 6px 12px;
  border: none;
  border-radius: 5px;
  cursor: pointer;
  font-size: 13px;
  font-weight: 500;
  transition: all 0.2s ease;
  white-space: nowrap;
  background: #667eea;
  color: white;
  height: 32px;
  min-width: 70px;
}

.select-repo-btn:hover:not(:disabled) {
  background: #5a67d8;
}

.select-repo-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

/* 仓库名称样式 */
.repo-name {
  color: #1a202c;
  font-size: 16px;
  font-weight: 600;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

/* 最近仓库下拉菜单样式 */
.recent-repos-dropdown {
  position: relative;
}

/* 最近仓库下拉按钮 - 紧凑尺寸，辅助功能 */
.recent-dropdown-btn {
  padding: 6px 8px;
  background: rgba(255, 255, 255, 0.9);
  border: 1px solid #ddd;
  border-radius: 4px;
  cursor: pointer;
  font-size: 14px;
  transition: all 0.3s ease;
  height: 32px;
  width: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
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

/* 清空历史按钮 - 小尺寸文本按钮 */
.clear-recent-btn {
  background: none;
  border: none;
  cursor: pointer;
  font-size: 12px;
  padding: 2px 6px;
  border-radius: 3px;
  transition: background-color 0.2s;
  color: #666;
  font-weight: 500;
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

/* 提交操作区域样式 */
.commit-actions-row {
  margin-top: 12px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 12px;
}

.ai-generate-section {
  display: flex;
  gap: 8px;
  align-items: center;
  flex: 1;
}

.template-select {
  padding: 6px 8px;
  border: 1px solid #d1d5db;
  border-radius: 4px;
  font-size: 13px;
  background: white;
  min-width: 120px;
}

/* 统一的操作按钮样式 - 移除图标，统一尺寸 */
.action-btn {
  padding: 8px 16px;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-size: 14px;
  font-weight: 500;
  transition: all 0.2s ease;
  white-space: nowrap;
  min-width: 80px;
  height: 36px;
}

.action-btn.generate-btn {
  background: #10b981;
  color: white;
}

.action-btn.generate-btn:hover:not(:disabled) {
  background: #059669;
}

.action-btn.commit-btn {
  background: #667eea;
  color: white;
}

.action-btn.commit-btn:hover:not(:disabled) {
  background: #5a67d8;
}

.action-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
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
  flex: 1;
  overflow: hidden;
}

/* 主要内容区域 - 修改为根据内容自适应高度，避免暂存区为空时占用大量空间 */
.main-content {
  display: flex;
  flex-direction: column;
  /* 移除 flex: 1，改为根据内容自适应高度 */
  gap: 16px;
  overflow-y: auto;
  /* 允许整体滚动 */
  padding: 16px;
  /* 添加内边距，让内容与边界有适当距离 */
  padding-bottom: 60px;
  /* 为绝对定位的提示信息留出空间 */
}

/* 文件区域样式 */
.staged-files,
.unstaged-files,
.file-section {
  border: 1px solid #e2e8f0;
  border-radius: 8px;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

/* 暂存区 - 根据内容自适应高度 */
.staged-files {
  flex: 0 1 auto;
  /* 移除 min-height，让暂存区根据内容自适应 */
  max-height: 280px;
}

/* 工作区 - 根据内容自适应高度 */
.unstaged-files {
  flex: 0 1 auto;
  /* 移除 min-height，让工作区根据内容自适应 */
  max-height: 300px;
}

/* 未跟踪文件和冲突文件 - 根据内容自适应高度 */
.file-section {
  flex: 0 1 auto;
  /* 移除 min-height，让未跟踪文件区域根据内容自适应 */
  max-height: 220px;
}

.section-title,
.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  background: #f7fafc;
  border-bottom: 1px solid #e2e8f0;
}

.section-title h4,
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
  /* 移除 flex: 1，改为根据内容自适应高度 - 修复暂存区空时占用大量空间的问题 */
  overflow-y: auto;
  /* 设置最大高度约为10条文件的高度(每条约24px) + padding */
  max-height: 248px;
}

/* 提交区域 - 优化高度以节省垂直空间 */
.commit-area {
  position: relative;
  /* 为绝对定位的进度条提供定位上下文 */
  padding: 12px;
  background: #f7fafc;
  border: 1px solid #e2e8f0;
  border-radius: 8px;
  flex: 0 0 auto;
  /* 不参与flex空间分配，根据内容自适应 */
  min-height: 140px;
  display: flex;
  flex-direction: column;
}

.commit-input {
  width: 100%;
  padding: 12px;
  border: 1px solid #e2e8f0;
  border-radius: 6px;
  font-family: inherit;
  font-size: 14px;
  line-height: 20px;
  /* 固定行高，便于计算 */
  resize: none;
  /* 禁用手动调整大小，使用自动调整 */
  margin-bottom: 12px;
  transition: height 0.2s ease;
  /* 高度变化动画 */
  overflow-y: hidden;
  /* 默认隐藏滚动条 */
  min-height: 60px;
  /* 最小高度约3行 */
  max-height: 224px;
  /* 最大高度约10行 */
}

/* 推理内容展示样式 - Author: Evilek, Date: 2025-01-10 */
.reasoning-content-section {
  margin-bottom: 12px;
  border: 1px solid #e2e8f0;
  border-radius: 6px;
  background: #f8f9fa;
  overflow: hidden;
}

.reasoning-header {
  display: flex;
  align-items: center;
  padding: 8px 12px;
  cursor: pointer;
  background: #f1f3f4;
  border-bottom: 1px solid #e2e8f0;
  transition: background-color 0.2s ease;
}

.reasoning-header:hover {
  background: #e9ecef;
}

.reasoning-icon {
  margin-right: 8px;
  font-size: 16px;
}

.reasoning-title {
  flex: 1;
  font-size: 14px;
  font-weight: 500;
  color: #495057;
}

.reasoning-toggle {
  font-size: 12px;
  color: #6c757d;
  transition: transform 0.2s ease;
}

.reasoning-content {
  padding: 12px;
  background: #ffffff;
  border-top: 1px solid #e2e8f0;
}

.reasoning-text {
  margin: 0;
  font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
  font-size: 13px;
  line-height: 1.5;
  color: #495057;
  white-space: pre-wrap;
  word-wrap: break-word;
  max-height: 200px;
  overflow-y: auto;
  background: #f8f9fa;
  padding: 8px;
  border-radius: 4px;
  border: 1px solid #e2e8f0;
}

/* 优化后的水平布局控制区域 - 节省垂直空间 */
.commit-controls-horizontal {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-top: 8px;
  gap: 12px;
}

.left-controls {
  display: flex;
  align-items: center;
  gap: 8px;
}

.right-controls {
  display: flex;
  align-items: center;
  gap: 8px;
}

/* 保留原有样式以防兼容性问题 */
.commit-controls {
  display: flex;
  justify-content: space-between;
  align-items: center;
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
  min-height: 120px;
  max-height: 300px;
  flex: 1;
  transition: border-color 0.2s ease;
  overflow-y: auto;
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
  position: absolute;
  top: 100%;
  left: 0;
  right: 0;
  margin-top: 4px;
  padding: 8px 12px;
  background: #e3f2fd;
  border: 1px solid #2196f3;
  border-radius: 6px;
  font-size: 12px;
  color: #1976d2;
  z-index: 10;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
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
  position: absolute;
  top: 100%;
  left: 0;
  right: 0;
  margin-top: 4px;
  padding: 8px 12px;
  background: #f0f9ff;
  border: 1px solid #0ea5e9;
  border-radius: 6px;
  font-size: 12px;
  z-index: 9;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
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

/* 重复的按钮样式已移除，使用统一的 .action-btn 样式 */

.commit-hint {
  /* 移除绝对定位，改为正常文档流 - 修复挡住其他元素的问题 */
  margin-top: 8px;
  padding: 8px 12px;
  background: #fff3cd;
  border: 1px solid #ffeaa7;
  border-radius: 4px;
  color: #856404;
  font-size: 12px;
  text-align: center;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
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

/* 批量操作样式 */
.batch-mode-btn {
  padding: 4px 8px;
  border: 1px solid #ddd;
  border-radius: 4px;
  background: #f8f9fa;
  color: #333;
  cursor: pointer;
  font-size: 12px;
  transition: all 0.2s ease;
}

.batch-mode-btn:hover {
  background: #e9ecef;
}

.batch-mode-btn.active {
  background: #007bff;
  color: white;
  border-color: #007bff;
}

.batch-toolbar {
  background: #f8f9fa;
  border: 1px solid #dee2e6;
  border-radius: 4px;
  padding: 8px 12px;
  margin-bottom: 8px;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.batch-info {
  font-size: 14px;
  color: #495057;
  font-weight: 500;
}

.batch-actions {
  display: flex;
  gap: 6px;
}

/* 批量操作按钮 - 较小尺寸，次要功能 */
.batch-btn {
  padding: 3px 6px;
  border: 1px solid #ddd;
  border-radius: 3px;
  background: white;
  color: #333;
  cursor: pointer;
  font-size: 11px;
  font-weight: 500;
  transition: all 0.2s ease;
  height: 24px;
  min-width: 50px;
}

.batch-btn:hover:not(:disabled) {
  background: #e9ecef;
}

.batch-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.batch-btn.stage-btn:hover:not(:disabled) {
  background: #d4edda;
  border-color: #28a745;
  color: #155724;
}

.batch-btn.unstage-btn:hover:not(:disabled) {
  background: #fff3cd;
  border-color: #ffc107;
  color: #856404;
}

.batch-btn.revert-btn:hover:not(:disabled) {
  background: #f8d7da;
  border-color: #dc3545;
  color: #721c24;
}

.batch-btn.select-all-btn:hover:not(:disabled) {
  background: #d1ecf1;
  border-color: #17a2b8;
  color: #0c5460;
}

.batch-btn.clear-btn:hover:not(:disabled) {
  background: #e2e3e5;
  border-color: #6c757d;
  color: #383d41;
}

/* 刷新状态指示器 */
.refresh-indicator {
  display: inline-block;
  animation: spin 1s linear infinite;
  margin-left: 4px;
  font-size: 12px;
}

@keyframes spin {
  from {
    transform: rotate(0deg);
  }

  to {
    transform: rotate(360deg);
  }
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

/* 响应式设计 */
@media (max-height: 800px) {

  /* 在较小屏幕上进一步优化区域设置 */
  .staged-files {
    max-height: 180px;
  }

  .unstaged-files {
    max-height: 200px;
  }

  .commit-area {
    min-height: 120px;
  }

  .file-section {
    max-height: 160px;
  }
}

@media (max-height: 600px) {

  /* 在很小的屏幕上进一步优化压缩 */
  .staged-files {
    max-height: 120px;
  }

  .unstaged-files {
    max-height: 140px;
  }

  .file-section {
    max-height: 100px;
  }

  .commit-area {
    min-height: 90px;
  }

  .commit-input {
    min-height: 40px;
    max-height: 80px;
  }
}

.modal-overlay {
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

/* 调试设置弹窗样式 - 必须在modal-overlay之后定义以确保优先级 */
.debug-settings-overlay {
  z-index: 9999 !important;
}

.debug-settings-modal {
  width: 90%;
  max-width: 600px;
  max-height: 80vh;
  overflow-y: auto;
}

.modal-content {
  background: var(--color-bg);
  border-radius: 8px;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.3);
  border: 1px solid var(--color-border);
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px 20px 0 20px;
  border-bottom: 1px solid var(--color-border);
  margin-bottom: 0;
}

.modal-header h3 {
  margin: 0;
  color: var(--color-text);
  font-size: 1.2rem;
}

.close-btn {
  background: none;
  border: none;
  font-size: 24px;
  cursor: pointer;
  color: var(--color-text-secondary);
  padding: 0;
  width: 30px;
  height: 30px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 4px;
  transition: all 0.2s;
}

.close-btn:hover {
  background: var(--color-bg-secondary);
  color: var(--color-text);
}

.modal-body {
  padding: 0;
}

.menu-divider {
  height: 1px;
  background: var(--color-border);
  margin: 5px 0;
}
</style>
