<template>
  <div class="gitflow-dashboard" :class="{ 'detail-open': !!focusBranch }">
    <header class="gitflow-topbar">
      <div class="heading">
        <h1>Gitflow 仪表盘</h1>
        <p>通过统一面板掌握 feature、release、bugfix、hotfix 四类分支的进度与风险。</p>
      </div>
      <div class="topbar-actions">
        <button class="ghost-refresh" @click="refresh" :disabled="loading">
          <span v-if="loading">刷新中…</span>
          <span v-else>刷新</span>
        </button>
        <button
          v-for="type in branchActionOrder"
          :key="type"
          class="type-action"
          :style="{ '--accent': branchTypeInfo(type).accent }"
          @click="openWizard(type)"
        >
          <span class="icon">＋</span>
          <span class="label">新建 {{ branchTypeInfo(type).label }}</span>
        </button>
      </div>
    </header>

    <p v-if="error" class="error-banner">⚠️ {{ error }}</p>
    <p v-if="usingSampleData" class="info-banner">当前显示示例数据，打开仓库后可获取真实分支。</p>
    <p v-if="!hasOriginRemote && !usingSampleData" class="warning-banner">当前仓库未检测到名为 origin 的远程，涉及推送或创建 PR 的操作已禁用。请先配置远程，例如运行 <code>git remote add origin &lt;url&gt;</code> 后重试。</p>

    <div v-if="activeHotfix" class="hotfix-banner">
      <div class="banner-content">
        <span class="banner-icon">🚨</span>
        <div class="banner-text">
          <strong>Hotfix 进行中：</strong>
          <span>{{ activeHotfix.name }}</span>
        </div>
      </div>
      <div class="banner-actions">
        <button class="banner-btn" @click="selectBranch(activeHotfix.id)">查看进展</button>
        <button class="banner-btn secondary" @click="openWizard('hotfix')">新建热修补丁</button>
      </div>
    </div>

    <section class="gitflow-status">
      <div class="status-card">
        <span class="status-label">基线分支</span>
        <div class="status-values">
          <span class="status-value">Develop：{{ configSnapshot.developBranch }}</span>
          <span class="status-value">Main：{{ configSnapshot.mainBranch }}</span>
        </div>
        <small v-if="lastSyncedAt" class="status-footnote">
          最近同步：{{ formatTimestamp(lastSyncedAt) }}
        </small>
      </div>
      <div v-if="bugfixWithSLA.length" class="status-card warning">
        <span class="status-label">Bugfix SLA</span>
        <ul>
          <li v-for="branch in bugfixWithSLA.slice(0, 2)" :key="branch.id">
            {{ branch.name }} ｜ 剩余 {{ formatSLA(branch.slaDeadline!) }}
          </li>
        </ul>
      </div>
      <div v-if="unsyncedBranches.length" class="status-card attention">
        <span class="status-label">待同步分支</span>
        <ul>
          <li v-for="branch in unsyncedBranches" :key="branch.id">
            {{ branch.name }} 需从 {{ branch.base }} 更新
          </li>
        </ul>
      </div>
      <div class="status-card info" v-if="hasPendingAttention">
        <span class="status-label">提醒</span>
        <p>部分分支仍在开发或等待合并，请关注最新进展。</p>
      </div>
    </section>

    <section class="gitflow-grid" :class="{ loading }">
      <article v-for="type in branchActionOrder" :key="type" class="grid-column">
        <header class="column-header">
          <h2>{{ branchTypeInfo(type).label }}</h2>
          <p>{{ branchTypeInfo(type).description }}</p>
        </header>
        <div class="column-body">
          <div v-if="loading" class="placeholder-card" />
          <GitflowBranchCard
            v-else
            v-for="branch in groupedBranches[type]"
            :key="branch.id"
            :branch="branch"
            :is-active="branch.id === selectedBranchId"
            :primary-action-label="primaryActionLabel(branch)"
            @select="selectBranch"
            @primary-action="handlePrimaryAction"
            @quick-action="handleQuickAction"
            @view-detail="selectBranch"
            @switch-branch="handleSwitchBranch"
          />
          <p v-if="!loading && !groupedBranches[type].length" class="empty-placeholder">
            暂无 {{ branchTypeInfo(type).label }} 分支，点击上方按钮快速创建。
          </p>
        </div>
      </article>
    </section>

    <aside v-if="focusBranch" class="gitflow-detail">
      <header class="detail-header">
        <div>
          <span
            class="detail-type"
            :style="{ color: focusBranchTypeMeta?.accent }"
          >
            {{ focusBranchTypeMeta?.label || "" }}
          </span>
          <h3>{{ focusBranch.name }}</h3>
        </div>
        <button class="detail-close" @click="resetSelection">×</button>
      </header>
      <GitflowBranchDetail :branch="focusBranch" @quick-action="handleQuickAction" />
    </aside>

    <GitflowWizard
      :state="gitflowWizard"
      @close="closeWizard"
      @submit="handleSubmitWizard"
      @update:state="handleWizardUpdate"
    />
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import GitflowBranchCard from './GitflowBranchCard.vue'
import GitflowBranchDetail from './GitflowBranchDetail.vue'
import GitflowWizard from './GitflowWizard.vue'
import { useGitflow, branchTypeMeta } from '../../composables/useGitflow'
import type { GitflowBranch, GitflowBranchType, GitflowWizardState } from '../../composables/useGitflow'
import { useToast } from '../../composables/useToast'

interface GitOperationResult {
  success: boolean
  message: string
  details?: string
}

const {
  loading,
  error,
  gitflowConfig,
  groupedBranches,
  focusBranch,
  selectedBranchId,
  gitflowWizard,
  hasPendingAttention,
  unsyncedBranches,
  activeHotfix,
  bugfixWithSLA,
  usingSampleData,
  openWizard,
  closeWizard,
  selectBranch,
  resetSelection,
  fetchGitflowBranches,
  createGitflowBranch,
  setCustomPrefixForType,
  setLastBranchNameForType,
  setLastOwnerForType,
  setReleaseStageForBranch,
  lastSyncedAt,
  hasOriginRemote
} = useGitflow()

const branchActionOrder: GitflowBranchType[] = ['feature', 'release', 'bugfix', 'hotfix']

const branchTypeInfo = (type: GitflowBranchType) => branchTypeMeta[type]
const focusBranchTypeMeta = computed(() =>
  focusBranch.value ? branchTypeInfo(focusBranch.value.branchType) : null
)
const toast = useToast()

const configSnapshot = computed(() => {
  return (
    gitflowConfig.value ?? {
      developBranch: 'develop',
      mainBranch: 'main',
      featurePrefix: 'feature/',
      releasePrefix: 'release/',
      bugfixPrefix: 'bugfix/',
      hotfixPrefix: 'hotfix/'
    }
  )
})

const refresh = () => fetchGitflowBranches()

const resolvePrimaryActionId = (branch: GitflowBranch): string | null => {
  if (branch.branchType === 'release') {
    if (branch.lifecycleStage === 'published') return 'finalize-release'
    if (branch.lifecycleStage === 'finished') return null
    return 'finish-release'
  }
  if (branch.branchType === 'hotfix') {
    return 'backport'
  }
  if (branch.branchType === 'feature') {
    const divergence = branch.divergence ?? { ahead: 0, behind: 0 }
    if (!hasOriginRemote.value) {
      return 'finish-local'
    }
    if (divergence.behind > 0) {
      return 'sync-base'
    }
    if (divergence.ahead > 0) {
      return 'finish-feature'
    }
    return 'generate-status'
  }
  if (branch.branchType === 'bugfix') {
    if (!hasOriginRemote.value) {
      return 'finish-local'
    }
    if (branch.status === 'awaiting_merge') {
      return 'request-review'
    }
    return 'generate-status'
  }
  return null
}

const primaryActionLabel = (branch: GitflowBranch): string | undefined => {
  const actionId = resolvePrimaryActionId(branch)
  if (!actionId) {
    return undefined
  }
  const quickAction = branch.nextActions?.find(action => action.id === actionId)
  if (quickAction) {
    return quickAction.label
  }
  switch (actionId) {
    case 'finish-local':
      return `合并到 ${branch.base}`
    case 'finish-feature':
      return `合并回 ${branch.base}`
    case 'open-pr':
      return '创建 PR'
    case 'sync-base':
      return `同步 ${branch.base}`
    case 'generate-status':
      return branch.branchType === 'bugfix' ? '生成事件记录' : '生成状态播报'
    default:
      return undefined
  }
}

const handlePrimaryAction = async (branch: GitflowBranch) => {
  const actionId = resolvePrimaryActionId(branch)
  if (actionId) {
    const quickAction = branch.nextActions?.find(action => action.id === actionId)
    if (quickAction) {
      if (quickAction.disabled) {
        toast.warning('请先配置远程 origin 后再执行该操作', '操作受限')
        selectBranch(branch.id)
        return
      }
      await handleQuickAction(branch, quickAction)
      return
    }
  }
  selectBranch(branch.id)
}

const handleQuickAction = async (branch: GitflowBranch, action: any) => {
  try {
    const result = await runQuickAction(branch, action)
    if (!result) {
      return
    }
    if (result.success) {
      toast.success(result.message, 'Gitflow 操作')
      if (result.details) {
        toast.info(result.details)
      }
    } else {
      toast.error(result.message || 'Gitflow 操作失败', '操作失败')
      if (result.details) {
        toast.info(result.details)
      }
    }
  } catch (err) {
    console.error('快捷操作执行失败:', err)
    const message =
      err instanceof Error ? err.message : typeof err === 'string' ? err : '未知错误，请查看控制台'
    if (/does not appear to be a git repository/i.test(message) || /could not read from remote repository/i.test(message)) {
      toast.error('检测到仓库未配置名为 origin 的远程，请先配置后再试。', '缺少远程', true)
      hasOriginRemote.value = false
      return
    }
    toast.error(message, '操作失败', true)
  }
}

const handleSwitchBranch = async (branch: GitflowBranch) => {
  try {
    const status = (await invoke('get_git_status')) as { has_changes?: boolean }
    if (status?.has_changes) {
      const confirmed = window.confirm(
        `检测到当前工作区存在未提交改动。
选择“确定”将打开 Smart Checkout 引导，取消则终止切换。`
      )
      if (confirmed) {
        window.dispatchEvent(
          new CustomEvent('gitpanel:open-smart-checkout', {
            detail: { targetBranch: branch.name }
          })
        )
        toast.info('已为你打开 Smart Checkout，请处理完改动后再尝试切换。', '需要先处理改动')
      } else {
        toast.info('已取消分支切换。', '操作取消')
      }
      return
    }

    const result = (await invoke('checkout_branch', {
      branchName: branch.name,
      isRemote: false
    })) as any

    if (result?.success) {
      toast.success(result.message || `已切换到 ${branch.name}`, '分支切换')
      selectBranch(branch.id)
      await fetchGitflowBranches()
    } else {
      toast.error(result?.message || '分支切换失败', '操作失败')
    }
  } catch (error) {
    console.error('切换分支失败:', error)
    const message =
      error instanceof Error ? error.message : typeof error === 'string' ? error : '未知错误，请查看控制台'

    if (/untracked working tree files/i.test(message) || /overwritten by checkout/i.test(message)) {
      toast.warning('检测到工作区仍有未处理文件，请先处理后再切换。', '无法切换')
      return
    }

    toast.error(message, '操作失败')
  }
}

const handleSubmitWizard = async (state: GitflowWizardState) => {
  try {
    setCustomPrefixForType(state.branchType, state.branchPrefix)
    setLastBranchNameForType(state.branchType, state.branchName)
    setLastOwnerForType(state.branchType, state.metadata.owner || '')
    await createGitflowBranch({
      branchType: state.branchType,
      branchName: state.branchName,
      baseBranch: state.metadata.base ?? undefined,
      autoPush: state.autoPush,
      metadata: state.metadata
    })
    toast.success('已创建 Gitflow 分支', 'Gitflow')
    closeWizard()
  } catch (err) {
    console.error('创建 Gitflow 分支失败:', err)
    const message =
      err instanceof Error ? err.message : typeof err === 'string' ? err : '未知错误，请查看控制台'
    toast.error(message, '创建失败', true)
  }
}

const handleWizardUpdate = (payload: Partial<GitflowWizardState>) => {
  Object.assign(gitflowWizard, payload)
}

const formatSLA = (deadline: string) => {
  const diff = new Date(deadline).getTime() - Date.now()
  const hours = Math.max(Math.round(diff / (1000 * 60 * 60)), 0)
  if (hours >= 24) {
    return `${Math.floor(hours / 24)} 天`
  }
  return `${hours} 小时`
}

const formatTimestamp = (timestamp: number) => {
  const date = new Date(timestamp)
  return `${date.getMonth() + 1}月${date.getDate()}日 ${String(date.getHours()).padStart(2, '0')}:${String(date.getMinutes()).padStart(2, '0')}`
}

// 快捷操作执行调度
const runQuickAction = async (branch: GitflowBranch, action: any): Promise<GitOperationResult | undefined> => {
  switch (action.id) {
    case 'sync-base':
      return syncWithBase(branch)
    case 'generate-status':
      return generateStatusReport(branch)
    case 'finish-feature':
      return finishFeature(branch)
    case 'open-pr':
      return openPullRequest(branch)
    case 'qa-update':
      return updateQAStatus(branch)
    case 'finish-local':
      return finishLocalMerge(branch)
    case 'finish-release':
      return finishRelease(branch)
    case 'finalize-release':
      return finalizeRelease(branch)
    case 'close-release':
      return closeReleaseLocal(branch)
    case 'backport':
      return backportToDevelop(branch)
    case 'generate-postmortem':
      return generatePostmortem(branch)
    case 'request-review':
      return requestCodeReview(branch)
    case 'generate-retro':
      return generateRetrospective(branch)
    default:
      console.warn('未知快捷操作:', action.id)
      toast.warning(`暂不支持的快捷操作：${action.label}`)
      return undefined
  }
}

// 快捷操作函数实现
const syncWithBase = async (branch: GitflowBranch): Promise<GitOperationResult> => {
  const result = (await invoke('execute_gitflow_action', {
    request: {
      branchName: branch.name,
      action: 'sync_with_base'
    }
  })) as GitOperationResult
  await fetchGitflowBranches()
  return result
}

const generateStatusReport = async (branch: GitflowBranch): Promise<GitOperationResult> => {
  try {
    const result = (await invoke('execute_gitflow_action', {
      request: {
        branchName: branch.name,
        action: 'generate_status_report'
      }
    })) as GitOperationResult
    await fetchGitflowBranches() // 刷新状态
    return result
  } catch (error) {
    console.error('生成状态报告失败:', error)
    throw error
  }
}

const finishFeature = async (branch: GitflowBranch): Promise<GitOperationResult> => {
  try {
    const result = (await invoke('execute_gitflow_action', {
      request: {
        branchName: branch.name,
        action: 'finish_feature'
      }
    })) as GitOperationResult
    if (result.success) {
      await fetchGitflowBranches()
    }
    return result
  } catch (error) {
    console.error('完成 Feature 合并失败:', error)
    throw error
  }
}

const openPullRequest = async (branch: GitflowBranch): Promise<GitOperationResult> => {
  try {
    const result = (await invoke('execute_gitflow_action', {
      request: {
        branchName: branch.name,
        action: 'create_pull_request'
      }
    })) as GitOperationResult
    return result
  } catch (error) {
    console.error('创建合并请求失败:', error)
    throw error
  }
}

const updateQAStatus = async (branch: GitflowBranch): Promise<GitOperationResult> => {
  try {
    const result = (await invoke('execute_gitflow_action', {
      request: {
        branchName: branch.name,
        action: 'update_qa_status'
      }
    })) as GitOperationResult
    await fetchGitflowBranches() // 刷新状态
    return result
  } catch (error) {
    console.error('更新 QA 状态失败:', error)
    throw error
  }
}

const finishLocalMerge = async (branch: GitflowBranch): Promise<GitOperationResult> => {
  try {
    const result = (await invoke('execute_gitflow_action', {
      request: {
        branchName: branch.name,
        action: 'finish_local'
      }
    })) as GitOperationResult
    await fetchGitflowBranches()
    return result
  } catch (error) {
    throw error
  }
}

const finishRelease = async (branch: GitflowBranch): Promise<GitOperationResult> => {
  try {
    const result = (await invoke('execute_gitflow_action', {
      request: {
        branchName: branch.name,
        action: 'finish_release'
      }
    })) as GitOperationResult
    if (result.success) {
      setReleaseStageForBranch(branch.name, 'published')
      await fetchGitflowBranches() // 刷新状态
    }
    return result
  } catch (error) {
    console.error('完成发布失败:', error)
    throw error
  }
}

const finalizeRelease = async (branch: GitflowBranch): Promise<GitOperationResult> => {
  try {
    const result = (await invoke('execute_gitflow_action', {
      request: {
        branchName: branch.name,
        action: 'finalize_release'
      }
    })) as GitOperationResult
    if (result.success) {
      setReleaseStageForBranch(branch.name, 'finished')
      await fetchGitflowBranches()
    }
    return result
  } catch (error) {
    console.error('收尾发布失败:', error)
    throw error
  }
}

const closeReleaseLocal = async (branch: GitflowBranch): Promise<GitOperationResult> => {
  try {
    const result = (await invoke('execute_gitflow_action', {
      request: {
        branchName: branch.name,
        action: 'close_release_local'
      }
    })) as GitOperationResult
    if (result.success) {
      setReleaseStageForBranch(branch.name, 'finished')
      await fetchGitflowBranches()
    }
    return result
  } catch (error) {
    console.error('关闭发布失败:', error)
    throw error
  }
}

const backportToDevelop = async (branch: GitflowBranch): Promise<GitOperationResult> => {
  try {
    const result = (await invoke('execute_gitflow_action', {
      request: {
        branchName: branch.name,
        action: 'backport_to_develop'
      }
    })) as GitOperationResult
    await fetchGitflowBranches() // 刷新状态
    return result
  } catch (error) {
    console.error('回流失败:', error)
    throw error
  }
}

const generatePostmortem = async (branch: GitflowBranch): Promise<GitOperationResult> => {
  try {
    const result = (await invoke('execute_gitflow_action', {
      request: {
        branchName: branch.name,
        action: 'generate_postmortem'
      }
    })) as GitOperationResult
    return result
  } catch (error) {
    console.error('生成复盘报告失败:', error)
    throw error
  }
}

const requestCodeReview = async (branch: GitflowBranch): Promise<GitOperationResult> => {
  try {
    const result = (await invoke('execute_gitflow_action', {
      request: {
        branchName: branch.name,
        action: 'request_code_review'
      }
    })) as GitOperationResult
    return result
  } catch (error) {
    console.error('请求代码评审失败:', error)
    throw error
  }
}

const generateRetrospective = async (branch: GitflowBranch): Promise<GitOperationResult> => {
  try {
    const result = (await invoke('execute_gitflow_action', {
      request: {
        branchName: branch.name,
        action: 'generate_retrospective'
      }
    })) as GitOperationResult
    return result
  } catch (error) {
    console.error('生成回顾总结失败:', error)
    throw error
  }
}

onMounted(() => {
  fetchGitflowBranches()
})
</script>

<style scoped>
.gitflow-dashboard {
  display: flex;
  flex-direction: column;
  gap: 24px;
  padding: 24px;
  position: relative;
  padding-right: 24px;
}

.gitflow-dashboard.detail-open {
  padding-right: calc(24px + min(440px, 30vw));
}

.gitflow-topbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 24px;
}

.heading h1 {
  margin: 0;
  font-size: 24px;
  color: #0f172a;
}

.heading p {
  margin: 6px 0 0;
  color: #475569;
}

.topbar-actions {
  display: flex;
  gap: 12px;
  flex-wrap: wrap;
  justify-content: flex-end;
}

.ghost-refresh {
  padding: 8px 12px;
  border-radius: 999px;
  border: 1px solid #cbd5f5;
  background: #ffffff;
  color: #1e293b;
  font-weight: 600;
  cursor: pointer;
}

.ghost-refresh:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.type-action {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 10px 16px;
  border-radius: 999px;
  border: 1px solid var(--accent);
  color: var(--accent);
  background: color-mix(in srgb, var(--accent) 12%, #ffffff);
  cursor: pointer;
  font-weight: 600;
  transition: transform 0.2s ease, box-shadow 0.2s ease;
}

.type-action:hover {
  transform: translateY(-1px);
  box-shadow: 0 6px 16px rgba(99, 102, 241, 0.2);
}

.icon {
  font-size: 18px;
}

.error-banner {
  margin: 0;
  padding: 12px 16px;
  border-radius: 10px;
  background: #fef2f2;
  color: #dc2626;
  border: 1px solid #fecaca;
  font-size: 14px;
}

.info-banner {
  margin: 0;
  padding: 10px 16px;
  border-radius: 10px;
  background: #eef2ff;
  color: #3730a3;
  border: 1px solid #c7d2fe;
  font-size: 14px;
}

.warning-banner {
  margin: 0;
  padding: 10px 16px;
  border-radius: 10px;
  background: #fef3c7;
  color: #92400e;
  border: 1px solid #fcd34d;
  font-size: 14px;
}

.hotfix-banner {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
  border-radius: 14px;
  background: linear-gradient(120deg, rgba(248, 113, 113, 0.16), rgba(239, 68, 68, 0.08));
  border: 1px solid rgba(239, 68, 68, 0.3);
  margin-bottom: 12px;
  gap: 16px;
}

.banner-content {
  display: flex;
  gap: 12px;
  align-items: center;
  color: #b91c1c;
  font-weight: 600;
}

.banner-icon {
  font-size: 20px;
}

.banner-actions {
  display: flex;
  gap: 10px;
}

.banner-btn {
  padding: 8px 14px;
  border-radius: 999px;
  background: #ef4444;
  border: none;
  color: #fff;
  font-weight: 600;
  cursor: pointer;
  transition: opacity 0.2s ease;
}

.banner-btn:hover {
  opacity: 0.86;
}

.banner-btn.secondary {
  background: rgba(239, 68, 68, 0.12);
  color: #b91c1c;
}

.gitflow-status {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
  gap: 16px;
}

.status-card {
  padding: 16px;
  border-radius: 16px;
  border: 1px solid #e2e8f0;
  background: #f8fafc;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.status-card.warning {
  border-color: #f97316;
  background: #fff7ed;
  color: #9a3412;
}

.status-card.attention {
  border-color: #f97316;
  background: #fff7ed;
  color: #9a3412;
}

.status-card.info {
  border-color: #6366f1;
  background: #eef2ff;
  color: #3730a3;
}

.status-label {
  font-weight: 600;
  color: #0f172a;
}

.status-values {
  display: flex;
  flex-direction: column;
  gap: 4px;
  color: #475569;
}

.status-footnote {
  font-size: 12px;
  color: #94a3b8;
}

.gitflow-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));
  gap: 18px;
}

.gitflow-grid.loading {
  opacity: 0.6;
  pointer-events: none;
}

.grid-column {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.column-header h2 {
  margin: 0;
  font-size: 18px;
  color: #111827;
}

.column-header p {
  margin: 4px 0 0;
  color: #6b7280;
}

.column-body {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.placeholder-card {
  height: 180px;
  border-radius: 12px;
  background: repeating-linear-gradient(
    -45deg,
    #f1f5f9,
    #f1f5f9 10px,
    #e2e8f0 10px,
    #e2e8f0 20px
  );
  animation: placeholder 1.2s linear infinite;
}

@keyframes placeholder {
  from {
    background-position: 0 0;
  }
  to {
    background-position: 100px 0;
  }
}

.empty-placeholder {
  margin: 0;
  padding: 16px;
  border: 1px dashed #cbd5f5;
  border-radius: 12px;
  color: #94a3b8;
  font-size: 14px;
}

.gitflow-detail {
  position: fixed;
  top: 92px;
  right: 32px;
  bottom: 32px;
  width: min(440px, 32vw);
  max-height: calc(100vh - 124px);
  overflow-y: auto;
  border: 1px solid #cbd5f5;
  border-radius: 16px;
  background: #ffffff;
  box-shadow: 0 12px 32px rgba(15, 23, 42, 0.12);
  padding: 20px 24px;
  z-index: 200;
}

@media (max-width: 1280px) {
  .gitflow-dashboard {
    padding-right: 24px;
  }

  .gitflow-dashboard.detail-open {
    padding-right: 24px;
  }

  .gitflow-detail {
    width: min(400px, 85vw);
    right: 16px;
  }
}

@media (max-width: 960px) {
  .gitflow-dashboard {
    padding: 16px;
  }

  .gitflow-dashboard.detail-open {
    padding: 16px;
  }

  .gitflow-detail {
    top: 72px;
    right: 12px;
    bottom: 12px;
    left: 12px;
    width: auto;
    max-height: calc(100vh - 96px);
  }
}

.detail-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 16px;
}

.detail-header h3 {
  margin: 6px 0 0;
  font-size: 18px;
  color: #111827;
}

.detail-type {
  font-size: 12px;
  font-weight: 600;
  text-transform: uppercase;
}

.detail-close {
  border: none;
  background: #f8fafc;
  border-radius: 50%;
  width: 32px;
  height: 32px;
  font-size: 18px;
  cursor: pointer;
  color: #475569;
}
</style>


