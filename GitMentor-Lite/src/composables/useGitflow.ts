import { computed, reactive, ref } from 'vue'
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

const gitflowWizard = reactive<GitflowWizardState>({
  visible: false,
  step: 1,
  branchType: 'feature',
  branchName: '',
  metadata: {},
  autoPush: false
})

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

const fetchGitflowBranches = async () => {
  try {
    loading.value = true
    error.value = null
    const summary = (await invoke('list_gitflow_branches')) as GitflowSummary
    gitflowConfig.value = summary.config
    gitflowBranches.value = summary.branches.map(decorateBranch)
    lastSyncedAt.value = Date.now()
    if (selectedBranchId.value) {
      const exists = gitflowBranches.value.some(branch => branch.id === selectedBranchId.value)
      if (!exists) {
        selectedBranchId.value = null
      }
    }
  } catch (err) {
    console.error('[Gitflow] fetch failed', err)
    error.value = (err as Error).message || '无法获取 Gitflow 分支信息'
  } finally {
    loading.value = false
  }
}

const createGitflowBranch = async (payload: GitflowCreatePayload) => {
  try {
    loading.value = true
    error.value = null
    await invoke('create_gitflow_branch', { request: payload })
    await fetchGitflowBranches()
  } catch (err) {
    console.error('[Gitflow] create branch failed', err)
    error.value = (err as Error).message || '创建 Gitflow 分支失败'
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
    openWizard,
    closeWizard,
    selectBranch,
    resetSelection,
    fetchGitflowBranches,
    createGitflowBranch,
    getDefaultBaseForType
  }
}
