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

export type ReleaseLifecycleStage = 'draft' | 'published' | 'finished'

const RELEASE_STAGE_ORDER: Record<ReleaseLifecycleStage, number> = {
  draft: 0,
  published: 1,
  finished: 2
}

const inferStageFromStatus = (status: GitflowBranchStatus): ReleaseLifecycleStage => {
  switch (status) {
    case 'merged':
      return 'finished'
    case 'awaiting_merge':
      return 'published'
    default:
      return 'draft'
  }
}

const normalizeReleaseLifecycleStage = (
  stage: ReleaseLifecycleStage | undefined,
  status: GitflowBranchStatus
): ReleaseLifecycleStage => {
  const statusStage = inferStageFromStatus(status)
  if (!stage) {
    return statusStage
  }
  if (RELEASE_STAGE_ORDER[stage] < RELEASE_STAGE_ORDER[statusStage]) {
    return statusStage
  }
  if (RELEASE_STAGE_ORDER[stage] > RELEASE_STAGE_ORDER[statusStage]) {
    if (statusStage === 'finished') {
      return stage
    }
    return statusStage
  }
  return stage
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
  lifecycleStage?: ReleaseLifecycleStage
}

export interface GitflowSummary {
  config: GitflowConfig
  branches: GitflowBranch[]
  hasOriginRemote?: boolean
}

export interface GitflowWizardState {
  visible: boolean
  step: 1 | 2 | 3
  branchType: GitflowBranchType
  branchName: string
  branchPrefix: string
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
const hasOriginRemote = ref(true)

const gitflowWizard = reactive<GitflowWizardState>({
  visible: false,
  step: 1,
  branchType: 'feature',
  branchName: '',
  branchPrefix: '',
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

const getLastOwnerForType = (type: GitflowBranchType) => {
  const value = branchOwners.value[type]?.trim()
  return value && value.length > 0 ? value : undefined
}

const setLastOwnerForType = (type: GitflowBranchType, owner: string) => {
  const normalized = owner.trim()
  if (!normalized) {
    if (branchOwners.value[type]) {
      const next = { ...branchOwners.value }
      delete next[type]
      branchOwners.value = next
      persistBranchOwners()
    }
    return
  }
  branchOwners.value = {
    ...branchOwners.value,
    [type]: normalized
  }
  persistBranchOwners()
}

const openWizard = (type: GitflowBranchType) => {
  gitflowWizard.visible = true
  gitflowWizard.step = 1
  gitflowWizard.branchType = type
  const nextPrefix = getCustomPrefixForType(type)
  gitflowWizard.branchPrefix = nextPrefix
  const lastBranchName = getLastBranchNameForType(type)
  gitflowWizard.branchName =
    lastBranchName && lastBranchName.startsWith(nextPrefix) ? lastBranchName : ''
  gitflowWizard.metadata = {
    base: getDefaultBaseForType(type),
    owner: getLastOwnerForType(type) ?? ''
  }
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

const computeDefaultQuickActions = (branch: GitflowBranch): GitflowQuickAction[] => {
  const actions: GitflowQuickAction[] = []
  const divergence = branch.divergence ?? { ahead: 0, behind: 0 }
  const behindCount = divergence.behind ?? 0
  const remoteUnavailable = !hasOriginRemote.value
  const remoteNote = remoteUnavailable ? '（需先配置 origin 远程）' : ''

  switch (branch.branchType) {
    case 'feature': {
      if (remoteUnavailable) {
        actions.push({
          id: 'finish-local',
          label: `合并到 ${branch.base}`,
          icon: '✅',
          description: `将当前分支合并进 ${branch.base} 并关闭本地分支`
        })
      }
      if (behindCount > 0) {
        const baseSync: GitflowQuickAction = {
          id: 'sync-base',
          label: `同步 ${branch.base}`,
          icon: '🔄',
          description: `将 ${branch.base} 的最新提交合并到此 feature 分支`
        }
        if (remoteUnavailable) {
          baseSync.disabled = true
          baseSync.description += remoteNote
        }
        actions.push(baseSync)
      }
      actions.push({
        id: 'generate-status',
        label: '生成状态播报',
        icon: '🧠',
        description: '基于最近提交生成一份可复制的进度播报'
      })
      const prAction: GitflowQuickAction = {
        id: 'open-pr',
        label: '创建 PR',
        icon: '📬',
        description: '推送分支并给出可用于创建 Pull Request 的链接提示'
      }
      if (remoteUnavailable) {
        prAction.disabled = true
        prAction.description += remoteNote
      }
      actions.push(prAction)
      break
    }
    case 'release': {
      const stage = branch.lifecycleStage ?? 'draft'
      actions.push({
        id: 'qa-update',
        label: '同步 QA 结果',
        icon: '✅',
        description: '记录最新的测试验证结论'
      })
      if (stage === 'draft') {
        const finishAction: GitflowQuickAction = {
          id: 'finish-release',
          label: '发布到远端',
          icon: '🚀',
          description: '推送 release 分支至远程并准备收尾'
        }
        if (remoteUnavailable) {
          finishAction.disabled = true
          finishAction.description += remoteNote
        }
        actions.push(finishAction)
      } else if (stage === 'published') {
        const finalizeAction: GitflowQuickAction = {
          id: 'finalize-release',
          label: 'Finish',
          icon: '🏁',
          description: '合并至主干并清理 release 分支'
        }
        if (remoteUnavailable) {
          finalizeAction.disabled = true
          finalizeAction.description += remoteNote
        }
        actions.push(finalizeAction)
      }
      if (stage !== 'finished') {
        actions.push({
          id: 'close-release',
          label: '直接关闭',
          icon: '🧹',
          description: '不推送远端，直接合并到主干并清理本地分支'
        })
      }
      break
    }
    case 'bugfix': {
      if (remoteUnavailable) {
        actions.push({
          id: 'finish-local',
          label: `合并到 ${branch.base}`,
          icon: '✅',
          description: `将当前分支合并进 ${branch.base} 并关闭本地分支`
        })
      }
      if (behindCount > 0) {
        const syncAction: GitflowQuickAction = {
          id: 'sync-base',
          label: `同步 ${branch.base}`,
          icon: '🔄',
          description: `将 ${branch.base} 的最新提交合并到此 bugfix 分支`
        }
        if (remoteUnavailable) {
          syncAction.disabled = true
          syncAction.description += remoteNote
        }
        actions.push(syncAction)
      }
      actions.push({
        id: 'generate-status',
        label: '生成事件记录',
        icon: '🧠',
        description: '整理缺陷信息与处理进展，生成 AI 草稿'
      })
      const reviewAction: GitflowQuickAction = {
        id: 'request-review',
        label: '申请评审',
        icon: '📝',
        description: '推送分支并提示代码评审的下一步动作'
      }
      if (remoteUnavailable) {
        reviewAction.disabled = true
        reviewAction.description += remoteNote
      }
      actions.push(reviewAction)
      actions.push({
        id: 'generate-retro',
        label: '生成复盘草稿',
        icon: '🔁',
        description: '输出缺陷复盘提纲，便于会后同步'
      })
      break
    }
    case 'hotfix': {
      actions.push(
        {
          id: 'backport',
          label: '回流 develop',
          icon: '↩️',
          description: '将热修补丁快速合并回 develop'
        },
        {
          id: 'generate-postmortem',
          label: '生成复盘报告',
          icon: '🧠',
          description: '提炼事故经过与回溯检查项'
        }
      )
      break
    }
  }

  return actions
}

const mergeQuickActions = (
  existing: GitflowQuickAction[] | undefined,
  fallback: GitflowQuickAction[] | undefined
): GitflowQuickAction[] => {
  const merged = new Map<string, GitflowQuickAction>()
  for (const action of existing ?? []) {
    merged.set(action.id, action)
  }
  for (const action of fallback ?? []) {
    if (!merged.has(action.id)) {
      merged.set(action.id, action)
    }
  }
  return Array.from(merged.values())
}

const defaultConfig: GitflowConfig = {
  developBranch: 'develop',
  mainBranch: 'main',
  featurePrefix: 'feature/',
  releasePrefix: 'release/',
  bugfixPrefix: 'bugfix/',
  hotfixPrefix: 'hotfix/'
}

type CustomPrefixMap = Partial<Record<GitflowBranchType, string>>

const CUSTOM_PREFIX_STORAGE_KEY = 'gitflow:custom-prefixes'
const BRANCH_HISTORY_STORAGE_KEY = 'gitflow:last-branch-names'
const RELEASE_STAGE_STORAGE_KEY = 'gitflow:release-stages'
const BRANCH_OWNER_STORAGE_KEY = 'gitflow:last-owner'

type BranchHistoryMap = Partial<Record<GitflowBranchType, string>>
type BranchOwnerMap = Partial<Record<GitflowBranchType, string>>
type ReleaseStageMap = Partial<Record<string, ReleaseLifecycleStage>>

const loadCustomPrefixes = (): CustomPrefixMap => {
  if (typeof window === 'undefined') return {}
  try {
    const stored = window.localStorage.getItem(CUSTOM_PREFIX_STORAGE_KEY)
    if (stored) {
      const parsed = JSON.parse(stored) as CustomPrefixMap
      return parsed ?? {}
    }
  } catch (error) {
    console.warn('[Gitflow] Failed to load custom prefixes:', error)
  }
  return {}
}

const customPrefixes = ref<CustomPrefixMap>(loadCustomPrefixes())

const persistCustomPrefixes = () => {
  if (typeof window === 'undefined') return
  try {
    if (Object.keys(customPrefixes.value).length === 0) {
      window.localStorage.removeItem(CUSTOM_PREFIX_STORAGE_KEY)
    } else {
      window.localStorage.setItem(
        CUSTOM_PREFIX_STORAGE_KEY,
        JSON.stringify(customPrefixes.value)
      )
    }
  } catch (error) {
    console.warn('[Gitflow] Failed to persist custom prefixes:', error)
  }
}

const getConfigPrefixForType = (type: GitflowBranchType) => {
  const config = gitflowConfig.value ?? defaultConfig
  switch (type) {
    case 'feature':
      return config.featurePrefix
    case 'release':
      return config.releasePrefix
    case 'bugfix':
      return config.bugfixPrefix
    case 'hotfix':
      return config.hotfixPrefix
  }
}

const getCustomPrefixForType = (type: GitflowBranchType) => {
  const stored = customPrefixes.value[type]?.trim()
  if (stored) {
    return stored
  }
  return getConfigPrefixForType(type)
}

const setCustomPrefixForType = (type: GitflowBranchType, prefix: string) => {
  const normalized = prefix.trim()
  const defaultPrefix = getConfigPrefixForType(type)
  if (!normalized || normalized === defaultPrefix) {
    if (customPrefixes.value[type]) {
      const next = { ...customPrefixes.value }
      delete next[type]
      customPrefixes.value = next
      persistCustomPrefixes()
    }
    return
  }
  customPrefixes.value = {
    ...customPrefixes.value,
    [type]: normalized
  }
  persistCustomPrefixes()
}

const loadBranchHistory = (): BranchHistoryMap => {
  if (typeof window === 'undefined') return {}
  try {
    const stored = window.localStorage.getItem(BRANCH_HISTORY_STORAGE_KEY)
    if (stored) {
      const parsed = JSON.parse(stored) as BranchHistoryMap
      return parsed ?? {}
    }
  } catch (error) {
    console.warn('[Gitflow] Failed to load branch history:', error)
  }
  return {}
}

const loadBranchOwners = (): BranchOwnerMap => {
  if (typeof window === 'undefined') return {}
  try {
    const stored = window.localStorage.getItem(BRANCH_OWNER_STORAGE_KEY)
    if (stored) {
      const parsed = JSON.parse(stored) as BranchOwnerMap
      return parsed ?? {}
    }
  } catch (error) {
    console.warn('[Gitflow] Failed to load branch owners:', error)
  }
  return {}
}

const branchOwners = ref<BranchOwnerMap>(loadBranchOwners())

const branchHistory = ref<BranchHistoryMap>(loadBranchHistory())

const persistBranchOwners = () => {
  if (typeof window === 'undefined') return
  try {
    if (Object.keys(branchOwners.value).length === 0) {
      window.localStorage.removeItem(BRANCH_OWNER_STORAGE_KEY)
    } else {
      window.localStorage.setItem(
        BRANCH_OWNER_STORAGE_KEY,
        JSON.stringify(branchOwners.value)
      )
    }
  } catch (error) {
    console.warn('[Gitflow] Failed to persist branch owners:', error)
  }
}

const persistBranchHistory = () => {
  if (typeof window === 'undefined') return
  try {
    if (Object.keys(branchHistory.value).length === 0) {
      window.localStorage.removeItem(BRANCH_HISTORY_STORAGE_KEY)
    } else {
      window.localStorage.setItem(
        BRANCH_HISTORY_STORAGE_KEY,
        JSON.stringify(branchHistory.value)
      )
    }
  } catch (error) {
    console.warn('[Gitflow] Failed to persist branch history:', error)
  }
}

const getLastBranchNameForType = (type: GitflowBranchType) => {
  const value = branchHistory.value[type]?.trim()
  return value && value.length > 0 ? value : undefined
}

const setLastBranchNameForType = (type: GitflowBranchType, branchName: string) => {
  const normalized = branchName.trim()
  if (!normalized) {
    if (branchHistory.value[type]) {
      const next = { ...branchHistory.value }
      delete next[type]
      branchHistory.value = next
      persistBranchHistory()
    }
    return
  }
  branchHistory.value = {
    ...branchHistory.value,
    [type]: normalized
  }
  persistBranchHistory()
}

const releaseStages = ref<ReleaseStageMap>(
  ((): ReleaseStageMap => {
    if (typeof window === 'undefined') return {}
    try {
      const stored = window.localStorage.getItem(RELEASE_STAGE_STORAGE_KEY)
      if (stored) {
        const parsed = JSON.parse(stored) as ReleaseStageMap
        return parsed ?? {}
      }
    } catch (error) {
      console.warn('[Gitflow] Failed to load release stages:', error)
    }
    return {}
  })()
)

const persistReleaseStages = () => {
  if (typeof window === 'undefined') return
  try {
    if (Object.keys(releaseStages.value).length === 0) {
      window.localStorage.removeItem(RELEASE_STAGE_STORAGE_KEY)
    } else {
      window.localStorage.setItem(
        RELEASE_STAGE_STORAGE_KEY,
        JSON.stringify(releaseStages.value)
      )
    }
  } catch (error) {
    console.warn('[Gitflow] Failed to persist release stages:', error)
  }
}

const getReleaseStageForBranch = (branchName: string): ReleaseLifecycleStage => {
  return releaseStages.value[branchName] ?? 'draft'
}

const pruneReleaseStages = (validNames: Set<string>) => {
  let changed = false
  const next: ReleaseStageMap = {}
  for (const [name, stage] of Object.entries(releaseStages.value)) {
    if (validNames.has(name)) {
      next[name] = stage
    } else {
      changed = true
    }
  }
  if (changed) {
    releaseStages.value = next
    persistReleaseStages()
  }
}

const updateReleaseStageInState = (branchName: string, stage: ReleaseLifecycleStage) => {
  const idx = gitflowBranches.value.findIndex(b => b.name === branchName)
  if (idx >= 0) {
    const existing = gitflowBranches.value[idx]
    const updated = decorateBranch({
      ...existing,
      lifecycleStage: stage
    })
    gitflowBranches.value = [
      ...gitflowBranches.value.slice(0, idx),
      updated,
      ...gitflowBranches.value.slice(idx + 1)
    ]
  }
}

const setReleaseStageForBranch = (branchName: string, stage: ReleaseLifecycleStage) => {
  if (stage === 'draft') {
    if (releaseStages.value[branchName]) {
      const next = { ...releaseStages.value }
      delete next[branchName]
      releaseStages.value = next
      persistReleaseStages()
    }
  } else {
    releaseStages.value = {
      ...releaseStages.value,
      [branchName]: stage
    }
    persistReleaseStages()
  }
  updateReleaseStageInState(branchName, stage)
}

const persistReleaseStageSnapshot = (branchName: string, stage: ReleaseLifecycleStage) => {
  if (stage === 'draft') {
    if (!releaseStages.value[branchName]) {
      return
    }
    const next = { ...releaseStages.value }
    delete next[branchName]
    releaseStages.value = next
    persistReleaseStages()
    return
  }
  if (releaseStages.value[branchName] === stage) {
    return
  }
  releaseStages.value = {
    ...releaseStages.value,
    [branchName]: stage
  }
  persistReleaseStages()
}

const decorateBranch = (branch: GitflowBranch): GitflowBranch => {
  let lifecycleStage = branch.lifecycleStage

  if (branch.branchType === 'release') {
    const storedStage = getReleaseStageForBranch(branch.name)
    const inputStage = branch.lifecycleStage ?? storedStage
    const normalizedStage = normalizeReleaseLifecycleStage(inputStage, branch.status)
    lifecycleStage = normalizedStage
    if (normalizedStage !== storedStage) {
      persistReleaseStageSnapshot(branch.name, normalizedStage)
    }
  }

  const normalized: GitflowBranch = {
    ...branch,
    divergence: branch.divergence ?? { ahead: 0, behind: 0 },
    progress: branch.progress ?? [],
    tasks: branch.tasks ?? [],
    qaChecklist: branch.qaChecklist ?? [],
    timeline: branch.timeline ?? [],
    aiDrafts: branch.aiDrafts ?? [],
    nextActions: branch.nextActions ?? [],
    lifecycleStage
  }

  const defaultQuickActions = computeDefaultQuickActions(normalized)
  normalized.nextActions = mergeQuickActions(normalized.nextActions, defaultQuickActions)

  return normalized
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
      },
      {
        id: 'feature-pr-description',
        title: 'PR 描述模板',
        tone: 'status',
        content:
          '## 概述\n本功能实现了 Gitflow 工作流管理仪表盘，提供统一的分支操作界面。\n\n## 主要变更\n- 新增 Gitflow 仪表盘组件\n- 实现分支卡片和快捷操作\n- 集成 AI 状态报告生成\n\n## 测试要点\n- [ ] 分支创建和切换\n- [ ] 快捷操作按钮响应\n- [ ] AI 草稿复制功能'
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
          '## 版本 1.8.0 发布说明\n\n### 🎉 新功能\n- **Gitflow 工作流管理**: 新增统一的 Gitflow 仪表盘，支持 feature、release、bugfix、hotfix 四类分支管理\n- **快捷操作**: 分支卡片提供一键操作按钮，快速执行常用 Gitflow 操作\n- **AI 文案生成**: 自动生成状态报告、发布说明、复盘提纲等多种文案\n\n### 🐛 问题修复\n- 修复登录页面输入提示导致的性能回退问题\n- 优化分支切换响应速度\n\n### 🔧 改进\n- 优化界面交互体验\n- 增强错误处理机制\n\n### ⚠️ 注意事项\n- 请在测试环境充分验证后再部署到生产环境\n- 建议备份现有配置文件'
      },
      {
        id: 'release-checklist',
        title: '发布检查清单',
        tone: 'release',
        content:
          '## 发布前检查清单\n\n### 代码质量\n- [ ] 所有单元测试通过\n- [ ] 代码审查完成\n- [ ] 静态代码分析通过\n\n### 功能测试\n- [ ] 核心功能回归测试\n- [ ] 新功能验收测试\n- [ ] 跨平台兼容性测试\n\n### 性能评估\n- [ ] 性能基准测试\n- [ ] 内存泄漏检查\n- [ ] 并发压力测试\n\n### 安全检查\n- [ ] 安全漏洞扫描\n- [ ] 依赖库安全检查\n- [ ] 权限控制验证\n\n### 部署准备\n- [ ] 安装包构建测试\n- [ ] 升级脚本验证\n- [ ] 回滚方案准备'
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
          '## Bugfix 处理记录\n\n### 问题描述\n- **缺陷编号**: BUG-231\n- **问题描述**: 登录页面输入提示导致性能回退\n- **严重程度**: 高\n- **报告人**: QA团队\n\n### 原因分析\n输入框事件监听器触发频率过高，导致 DOM 操作过于频繁，影响页面响应性能。\n\n### 解决方案\n- 实现输入防抖机制，限制触发频率\n- 优化 DOM 查询和更新逻辑\n- 添加性能监控点\n\n### 修复验证\n- [ ] 本地环境测试通过\n- [ ] 测试环境验证完成\n- [ ] 性能指标回归正常\n\n### 发布计划\n预计今晚发布回归包，修复时间约 2 小时。'
      },
      {
        id: 'bugfix-retrospective',
        title: '缺陷复盘总结',
        tone: 'retro',
        content:
          '## 缺陷复盘总结\n\n### 缺陷概述\n- **缺陷类型**: 性能问题\n- **发现阶段**: 测试阶段\n- **修复耗时**: 8 小时\n- **影响范围**: 登录模块\n\n### 根本原因\n1. **技术原因**: 缺乏输入防抖机制\n2. **流程原因**: 性能测试覆盖不足\n3. **设计原因**: 未考虑高频操作场景\n\n### 改进措施\n#### 短期改进\n- 为所有输入框添加防抖机制\n- 增加性能回归测试用例\n- 建立性能监控基线\n\n#### 长期改进\n- 建立前端性能最佳实践指南\n- 引入性能测试自动化\n- 加强代码审查中的性能关注点\n\n### 经验教训\n- 用户输入场景必须考虑性能影响\n- 高频操作需要增加限制机制\n- 性能问题应该在开发阶段发现\n\n### 预防措施\n- 新功能开发必须包含性能评估\n- 建立性能监控和告警机制\n- 定期进行性能回归测试'
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
          '## 🚨 生产事件通报\n\n### 事件概述\n- **时间**: 2025-03-15 00:00 UTC\n- **影响**: 生产环境应用崩溃\n- **持续时间**: 约 2 小时\n- **用户影响**: 登录功能异常\n\n### 根本原因\n夏令时切换时区计算错误导致应用崩溃\n\n### 修复措施\n- 紧急发布热修复补丁 (hotfix/timezone-crash)\n- 修复时区计算逻辑，添加异常保护\n- 加强时区转换的边界条件测试\n\n### 恢复状态\n✅ 服务已恢复正常\n✅ 监控指标稳定\n✅ 用户功能验证通过\n\n### 后续行动\n- 完成事故复盘报告\n- 改进时区相关代码测试覆盖\n- 完善异常监控机制'
      },
      {
        id: 'postmortem-template',
        title: '事故复盘模板',
        tone: 'retro',
        content:
          '## 事故复盘报告\n\n### 1. 事件时间线\n- **[时间]**: 事件发生\n- **[时间]**: 告警触发\n- **[时间]**: 问题定位\n- **[时间]**: 修复实施\n- **[时间]**: 服务恢复\n\n### 2. 影响评估\n- **业务影响**: 描述具体影响\n- **用户影响**: 受影响用户范围和数量\n- **财务影响**: 如有，说明具体损失\n\n### 3. 根本原因分析\n- **直接原因**: \n- **根本原因**: \n- **促成因素**: \n\n### 4. 改进措施\n- **短期**: \n- **中期**: \n- **长期**: \n\n### 5. 经验教训\n- \n- \n- \n\n### 6. 行动计划\n| 行动项 | 负责人 | 截止时间 | 状态 |\n|--------|--------|----------|------|\n|        |        |          |      |'
      }
    ],
    nextActions: [
      { id: 'backport', label: '回流 develop', icon: '↩️' },
      { id: 'generate-postmortem', label: '生成复盘', icon: '🧠' }
    ],
    metrics: { riskLevel: 'high', owner: 'Ops', relatedWork: ['INC-512'] }
  })
]

const ENABLE_GITFLOW_SAMPLE_DATA = false

const applySampleData = () => {
  gitflowConfig.value = defaultConfig
  const samples = sampleBranches()
  pruneReleaseStages(new Set(samples.map(branch => branch.name)))
  gitflowBranches.value = samples
  hasOriginRemote.value = true
  lastSyncedAt.value = Date.now()
  usingSampleData.value = true
}

if (ENABLE_GITFLOW_SAMPLE_DATA) {
  applySampleData()
}

const fetchGitflowBranches = async () => {
  try {
    loading.value = true
    error.value = null
    usingSampleData.value = false
    const summary = (await invoke('list_gitflow_branches')) as GitflowSummary
    gitflowConfig.value = summary.config
    hasOriginRemote.value =
      typeof summary.hasOriginRemote === 'boolean' ? summary.hasOriginRemote : true
    const availableNames = new Set(summary.branches.map(branch => branch.name))
    pruneReleaseStages(availableNames)
    gitflowBranches.value = summary.branches.map(decorateBranch)
    if (!gitflowBranches.value.length) {
      usingSampleData.value = false
      if (ENABLE_GITFLOW_SAMPLE_DATA) {
        applySampleData()
      }
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
      error.value = '��������Ϣ����ҳѡ���򿪲ֿ�'
      gitflowBranches.value = []
      usingSampleData.value = false
      gitflowConfig.value = null
      hasOriginRemote.value = true
      if (ENABLE_GITFLOW_SAMPLE_DATA) {
        applySampleData()
      }
    } else {
      error.value = message
      if (!gitflowBranches.value.length) {
        usingSampleData.value = false
        if (ENABLE_GITFLOW_SAMPLE_DATA) {
          applySampleData()
        }
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
      gitflowBranches.value = []
      gitflowConfig.value = null
      usingSampleData.value = false
      hasOriginRemote.value = true
      if (ENABLE_GITFLOW_SAMPLE_DATA) {
        applySampleData()
      }
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
    hasOriginRemote,
    usingSampleData,
    openWizard,
    closeWizard,
    selectBranch,
    resetSelection,
    fetchGitflowBranches,
    createGitflowBranch,
    getDefaultBaseForType,
    getCustomPrefixForType,
    setCustomPrefixForType,
    setLastBranchNameForType,
    setLastOwnerForType,
    setReleaseStageForBranch
  }
}
