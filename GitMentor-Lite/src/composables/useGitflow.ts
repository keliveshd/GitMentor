import { computed, reactive, ref, onMounted, onBeforeUnmount } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export type GitflowBranchType = 'feature' | 'release' | 'bugfix' | 'hotfix'

export type GitflowBranchStatus = 'idle' | 'in_progress' | 'awaiting_merge' | 'merged'

export interface GitflowConfig {
  developBranch: string
  mainBranch: string
  featurePrefix: string
  releasePrefix: string
  bugfixPrefix: string
  hotfixPrefix: string
}

export interface GitflowDivergence {
  ahead: number
  behind: number
}

export interface GitflowProgressItem {
  id: string
  label: string
  status: 'todo' | 'doing' | 'done'
  description?: string
}

export interface GitflowTaskItem {
  id: string
  label: string
  done: boolean
  owner?: string
}

export interface GitflowTimelineEntry {
  id: string
  label: string
  timestamp: string
  status: 'pending' | 'done'
  description?: string
}

export interface GitflowDraft {
  id: string
  title: string
  content: string
  tone: 'status' | 'release' | 'incident' | 'retro'
}

export interface GitflowQuickAction {
  id: string
  label: string
  icon: string
  disabled?: boolean
  description?: string
}

export interface GitflowBranch {
  id: string
  name: string
  branchType: GitflowBranchType
  base: string
  status: GitflowBranchStatus
  createdAt?: string
  lastUpdatedAt?: string
  latestCommit?: string
  divergence: GitflowDivergence
  upstream?: string
  isCurrent: boolean
  includeInRelease?: string[]
  slaDeadline?: string
  notes?: string
  progress?: GitflowProgressItem[]
  tasks?: GitflowTaskItem[]
  qaChecklist?: GitflowTaskItem[]
  timeline?: GitflowTimelineEntry[]
  aiDrafts?: GitflowDraft[]
  metrics?: {
    riskLevel?: 'low' | 'medium' | 'high'
    owner?: string
    relatedWork?: string[]
  }
  nextActions?: GitflowQuickAction[]
}

export interface GitflowSummary {
  config: GitflowConfig
  branches: GitflowBranch[]
}

export interface GitflowWizardState {
  visible: boolean
  step: 1 | 2 | 3
  branchType: GitflowBranchType
  branchName: string
  metadata: Record<string, string>
  autoPush: boolean
}

export const branchTypeMeta: Record<
  GitflowBranchType,
  {
    label: string
    accent: string
    description: string
  }
> = {
  feature: {
    label: 'Feature',
    accent: '#3b82f6',
    description: '用于新增功能的常规开发分支'
  },
  release: {
    label: 'Release',
    accent: '#22c55e',
    description: '聚合多个功能分支，准备发布版本'
  },
  bugfix: {
    label: 'Bugfix',
    accent: '#f59e0b',
    description: '处理测试或近期回归的缺陷'
  },
  hotfix: {
    label: 'Hotfix',
    accent: '#ef4444',
    description: '紧急修复线上事故，需回流至 main/develop'
  }
}

interface GitflowCreatePayload {
  branchType: GitflowBranchType
  branchName: string
  baseBranch?: string
  autoPush: boolean
  metadata: Record<string, string>
}

const gitflowConfig = ref<GitflowConfig | null>(null)
const gitflowBranches = ref<GitflowBranch[]>([])
const loading = ref(false)
const error = ref<string | null>(null)
const lastSyncedAt = ref<number | null>(null)
const usingSampleData = ref(false)

const gitflowWizard = reactive<GitflowWizardState>({
  visible: false,
  step: 1,
  branchType: 'feature',
  branchName: '',
  metadata: {},
  autoPush: false
})

let repoChangeListener: ((event: CustomEvent<{ path?: string }>) => void) | null = null
let repoListenerRefCount = 0

const selectedBranchId = ref<string | null>(null)

const groupedBranches = computed(() => {
  return gitflowBranches.value.reduce<Record<GitflowBranchType, GitflowBranch[]>>(
    (acc, branch) => {
      acc[branch.branchType].push(branch)
      return acc
    },
    { feature: [], release: [], bugfix: [], hotfix: [] }
  )
})

const focusBranch = computed<GitflowBranch | null>(() => {
  if (!selectedBranchId.value) return null
  return gitflowBranches.value.find(branch => branch.id === selectedBranchId.value) ?? null
})

const hasPendingAttention = computed(() =>
  gitflowBranches.value.some(branch =>
    ['awaiting_merge', 'in_progress'].includes(branch.status)
  )
)

const unsyncedBranches = computed(() =>
  gitflowBranches.value.filter(branch => branch.divergence?.behind > 0)
)

const activeHotfix = computed(() =>
  gitflowBranches.value.find(
    branch => branch.branchType === 'hotfix' && branch.status !== 'merged'
  )
)

const bugfixWithSLA = computed(() =>
  gitflowBranches.value
    .filter(branch => branch.branchType === 'bugfix' && branch.slaDeadline)
    .sort(
      (a, b) =>
        new Date(a.slaDeadline as string).getTime() -
        new Date(b.slaDeadline as string).getTime()
    )
)

const featureBranches = computed(() => groupedBranches.value.feature)

const openWizard = (type: GitflowBranchType) => {
  gitflowWizard.visible = true
  gitflowWizard.step = 1
  gitflowWizard.branchType = type
  gitflowWizard.branchName = ''
  gitflowWizard.metadata = {}
  gitflowWizard.autoPush = type !== 'feature'
}

const closeWizard = () => {
  gitflowWizard.visible = false
}

const selectBranch = (id: string) => {
  selectedBranchId.value = id
}

const resetSelection = () => {
  selectedBranchId.value = null
}

const decorateBranch = (branch: GitflowBranch): GitflowBranch => {
  return {
    ...branch,
    divergence: branch.divergence ?? { ahead: 0, behind: 0 },
    progress: branch.progress ?? [],
    tasks: branch.tasks ?? [],
    qaChecklist: branch.qaChecklist ?? [],
    timeline: branch.timeline ?? [],
    aiDrafts: branch.aiDrafts ?? [],
    nextActions: branch.nextActions ?? []
  }
}

const defaultConfig: GitflowConfig = {
  developBranch: 'develop',
  mainBranch: 'main',
  featurePrefix: 'feature/',
  releasePrefix: 'release/',
  bugfixPrefix: 'bugfix/',
  hotfixPrefix: 'hotfix/'
}

const sampleBranches = (): GitflowBranch[] => [
  decorateBranch({
    id: 'feature-sample-123',
    name: 'feature/sample-user-onboarding',
    branchType: 'feature',
    base: 'develop',
    status: 'in_progress',
    isCurrent: false,
    createdAt: new Date(Date.now() - 3 * 24 * 60 * 60 * 1000).toISOString(),
    lastUpdatedAt: new Date().toISOString(),
    latestCommit: 'feat: 添加 Gitflow 仪表盘骨架',
    divergence: { ahead: 2, behind: 0 },
    notes: '等待设计评审',
    progress: [
      { id: 'sync', label: '同步基线', status: 'done', description: '已跟进至最新 develop' },
      { id: 'implement', label: '功能开发', status: 'doing', description: '仪表盘 UI 开发中' },
      { id: 'review', label: '评审准备', status: 'todo' },
      { id: 'merge', label: '准备合并', status: 'todo' }
    ],
    tasks: [
      { id: 'design', label: '确认设计稿视觉', done: false, owner: 'Alice' },
      { id: 'copy', label: '完善 AI 提示语', done: false, owner: 'Bob' }
    ],
    aiDrafts: [
      {
        id: 'feature-status',
        title: 'Feature 状态播报',
        tone: 'status',
        content:
          'Gitflow 仪表盘已绘制 UI，正在整合分支详情组件，预计明日完成自测。仍需处理命名校验。'
      }
    ],
    nextActions: [
      { id: 'generate-status', label: '生成状态播报', icon: '🧠' },
      { id: 'open-pr', label: '创建合并请求', icon: '📬', disabled: true }
    ],
    metrics: { riskLevel: 'medium', owner: '产品组', relatedWork: ['UI-451'] }
  }),
  decorateBranch({
    id: 'release-sample-180',
    name: 'release/1.8.0',
    branchType: 'release',
    base: 'develop',
    status: 'awaiting_merge',
    isCurrent: false,
    createdAt: new Date(Date.now() - 7 * 24 * 60 * 60 * 1000).toISOString(),
    lastUpdatedAt: new Date().toISOString(),
    latestCommit: 'chore: bump version to v1.8.0',
    includeInRelease: ['feature/sample-user-onboarding', 'feature/sample-ai-digest'],
    divergence: { ahead: 1, behind: 0 },
    qaChecklist: [
      { id: 'qa-regression', label: '核心流程回归', done: true },
      { id: 'qa-package', label: '安装包烟测', done: false },
      { id: 'qa-doc', label: '发布说明审阅', done: false }
    ],
    aiDrafts: [
      {
        id: 'release-note',
        title: '发布说明草稿',
        tone: 'release',
        content:
          '版本 1.8.0 推出 Gitflow 仪表盘和热修复看板，并修复登录卡顿问题。QA 请补充安装包烟测。'
      }
    ],
    nextActions: [
      { id: 'qa-update', label: '同步 QA 结果', icon: '✅' },
      { id: 'finish-release', label: '完成发布流程', icon: '🚀' }
    ],
    metrics: { riskLevel: 'low', owner: 'Release Manager', relatedWork: ['REL-1.8.0'] }
  }),
  decorateBranch({
    id: 'bugfix-sample-login',
    name: 'bugfix/login-lag',
    branchType: 'bugfix',
    base: 'develop',
    status: 'in_progress',
    isCurrent: false,
    createdAt: new Date(Date.now() - 10 * 60 * 60 * 1000).toISOString(),
    lastUpdatedAt: new Date().toISOString(),
    latestCommit: 'fix: reduce input debounce',
    slaDeadline: new Date(Date.now() + 8 * 60 * 60 * 1000).toISOString(),
    divergence: { ahead: 1, behind: 0 },
    tasks: [
      { id: 'profiling', label: '上传性能分析报告', done: false, owner: 'Chen' },
      { id: 'snapshot', label: '补充异常截图', done: true }
    ],
    aiDrafts: [
      {
        id: 'bugfix-note',
        title: 'Bugfix 事件记录',
        tone: 'status',
        content:
          '登录页输入提示导致性能回退，已定位并优化，等待前端自测结果，晚间计划推送回归包。'
      }
    ],
    nextActions: [
      { id: 'request-review', label: '申请评审', icon: '📝', disabled: true },
      { id: 'generate-retro', label: '生成复盘草稿', icon: '🔁' }
    ],
    metrics: { riskLevel: 'high', owner: 'QA', relatedWork: ['BUG-231'] }
  }),
  decorateBranch({
    id: 'hotfix-sample-tz',
    name: 'hotfix/timezone-crash',
    branchType: 'hotfix',
    base: 'main',
    status: 'awaiting_merge',
    isCurrent: false,
    createdAt: new Date(Date.now() - 5 * 60 * 60 * 1000).toISOString(),
    lastUpdatedAt: new Date(Date.now() - 30 * 60 * 1000).toISOString(),
    latestCommit: 'fix: guard tz calculation on DST switch',
    divergence: { ahead: 1, behind: 0 },
    timeline: [
      {
        id: 'incident-detect',
        label: '报警触发',
        timestamp: new Date(Date.now() - 4 * 60 * 60 * 1000).toISOString(),
        status: 'done',
        description: '监控捕获 0:00 UTC 崩溃'
      },
      {
        id: 'patch-deploy',
        label: '补丁部署',
        timestamp: new Date(Date.now() - 2 * 60 * 60 * 1000).toISOString(),
        status: 'done',
        description: '热修复包部署完成'
      },
      {
        id: 'monitor',
        label: '监控回归',
        timestamp: new Date(Date.now() - 30 * 60 * 1000).toISOString(),
        status: 'pending',
        description: '等待 2 小时无异常后回流 develop'
      }
    ],
    aiDrafts: [
      {
        id: 'incident-broadcast',
        title: 'Hotfix 事件播报',
        tone: 'incident',
        content:
          '生产在切换夏令时出现崩溃，热修复 branch 已恢复服务，监控稳定后将回流 develop 并产出复盘。'
      }
    ],
    nextActions: [
      { id: 'backport', label: '回流 develop', icon: '↩️' },
      { id: 'generate-postmortem', label: '生成复盘', icon: '🧠' }
    ],
    metrics: { riskLevel: 'high', owner: 'Ops', relatedWork: ['INC-512'] }
  })
]

const applySampleData = () => {
  gitflowConfig.value = defaultConfig
  gitflowBranches.value = sampleBranches()
  lastSyncedAt.value = Date.now()
  usingSampleData.value = true
}

applySampleData()

const fetchGitflowBranches = async () => {
  try {
    loading.value = true
    error.value = null
    usingSampleData.value = false
    const summary = (await invoke('list_gitflow_branches')) as GitflowSummary
    gitflowConfig.value = summary.config
    gitflowBranches.value = summary.branches.map(decorateBranch)
    if (!gitflowBranches.value.length) {
      applySampleData()
    } else {
      lastSyncedAt.value = Date.now()
    }
    if (selectedBranchId.value) {
      const exists = gitflowBranches.value.some(branch => branch.id === selectedBranchId.value)
      if (!exists) {
        selectedBranchId.value = null
      }
    }
  } catch (err) {
    console.error('[Gitflow] fetch failed', err)
    const message = (err as Error).message || '无法获取 Gitflow 分支信息'
    if (message.includes('No repository opened')) {
      error.value = '请先在消息生成页选择或打开仓库'
      applySampleData()
    } else {
      error.value = message
      if (!gitflowBranches.value.length) {
        applySampleData()
      }
    }
  } finally {
    loading.value = false
  }
}

const createGitflowBranch = async (payload: GitflowCreatePayload) => {
  try {
    loading.value = true
    error.value = null
    usingSampleData.value = false
    await invoke('create_gitflow_branch', { request: payload })
    await fetchGitflowBranches()
  } catch (err) {
    console.error('[Gitflow] create branch failed', err)
    error.value = (err as Error).message || '创建 Gitflow 分支失败'
    if (usingSampleData.value || gitflowBranches.value.length) {
      // 在示例模式下附加一个模拟分支，方便继续演示 UI
      const dummy: GitflowBranch = decorateBranch({
        id: `${payload.branchType}-${Date.now()}`,
        name: payload.branchName,
        branchType: payload.branchType,
        base: payload.baseBranch ?? getDefaultBaseForType(payload.branchType),
        status: 'in_progress',
        isCurrent: false,
        createdAt: new Date().toISOString(),
        lastUpdatedAt: new Date().toISOString(),
        divergence: { ahead: 0, behind: 0 },
        latestCommit: '示例环境：未连接仓库',
        notes: '这是示例数据，打开仓库后可同步真实分支。'
      })
      gitflowBranches.value = [dummy, ...gitflowBranches.value]
      usingSampleData.value = true
      lastSyncedAt.value = Date.now()
    }
    throw err
  } finally {
    loading.value = false
  }
}

const getDefaultBaseForType = (type: GitflowBranchType) => {
  if (!gitflowConfig.value) return type === 'hotfix' ? 'main' : 'develop'
  switch (type) {
    case 'hotfix':
      return gitflowConfig.value.mainBranch
    default:
      return gitflowConfig.value.developBranch
  }
}

export const useGitflow = () => {

  const syncGitflowForPath = (pathValue: string) => {
    if (pathValue) {
      fetchGitflowBranches()
    } else {
      applySampleData()
    }
  }

  onMounted(() => {
    if (typeof window !== 'undefined') {
      if (!repoChangeListener) {
        repoChangeListener = ((event: CustomEvent<{ path?: string }>) => {
          const nextPath = event.detail?.path ?? ''
          syncGitflowForPath(nextPath)
        })
      }

      if (repoListenerRefCount === 0 && repoChangeListener) {
        window.addEventListener('gitflow:repo-changed', repoChangeListener as EventListener)
      }
      repoListenerRefCount += 1
    }

    if (!gitflowBranches.value.length || usingSampleData.value) {
      fetchGitflowBranches()
    }
  })

  onBeforeUnmount(() => {
    if (typeof window !== 'undefined' && repoChangeListener) {
      repoListenerRefCount = Math.max(repoListenerRefCount - 1, 0)
      if (repoListenerRefCount === 0) {
        window.removeEventListener('gitflow:repo-changed', repoChangeListener as EventListener)
      }
    }
  })

  return {
    loading,
    error,
    gitflowConfig,
    gitflowBranches,
    groupedBranches,
    focusBranch,
    selectedBranchId,
    gitflowWizard,
    hasPendingAttention,
    unsyncedBranches,
    activeHotfix,
    bugfixWithSLA,
    featureBranches,
    branchTypeMeta,
    lastSyncedAt,
    usingSampleData,
    openWizard,
    closeWizard,
    selectBranch,
    resetSelection,
    fetchGitflowBranches,
    createGitflowBranch,
    getDefaultBaseForType
  }
}
