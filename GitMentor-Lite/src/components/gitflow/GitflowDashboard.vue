<template>
  <div class="gitflow-dashboard">
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
          :style="{ '--accent': branchTypeMeta[type].accent }"
          @click="openWizard(type)"
        >
          <span class="icon">＋</span>
          <span class="label">新建 {{ branchTypeMeta[type].label }}</span>
        </button>
      </div>
    </header>

    <p v-if="error" class="error-banner">⚠️ {{ error }}</p>
    <p v-if="usingSampleData" class="info-banner">当前显示示例数据，打开仓库后可获取真实分支。</p>

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
          <h2>{{ branchTypeMeta[type].label }}</h2>
          <p>{{ branchTypeMeta[type].description }}</p>
        </header>
        <div class="column-body">
          <div v-if="loading" class="placeholder-card" />
          <GitflowBranchCard
            v-else
            v-for="branch in groupedBranches[type]"
            :key="branch.id"
            :branch="branch"
            :is-active="branch.id === selectedBranchId"
            @select="selectBranch"
            @open-wizard="openWizard"
            @view-detail="selectBranch"
          />
          <p v-if="!loading && !groupedBranches[type].length" class="empty-placeholder">
            暂无 {{ branchTypeMeta[type].label }} 分支，点击上方按钮快速创建。
          </p>
        </div>
      </article>
    </section>

    <aside v-if="focusBranch" class="gitflow-detail">
      <header class="detail-header">
        <div>
          <span
            class="detail-type"
            :style="{ color: branchTypeMeta[focusBranch.branchType].accent }"
          >
            {{ branchTypeMeta[focusBranch.branchType].label }}
          </span>
          <h3>{{ focusBranch.name }}</h3>
        </div>
        <button class="detail-close" @click="resetSelection">×</button>
      </header>
      <GitflowBranchDetail :branch="focusBranch" />
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
import GitflowBranchCard from './GitflowBranchCard.vue'
import GitflowBranchDetail from './GitflowBranchDetail.vue'
import GitflowWizard from './GitflowWizard.vue'
import { useGitflow, branchTypeMeta } from '../../composables/useGitflow'
import type { GitflowBranchType, GitflowWizardState } from '../../composables/useGitflow'

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
  lastSyncedAt
} = useGitflow()

const branchActionOrder: GitflowBranchType[] = ['feature', 'release', 'bugfix', 'hotfix']

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

const handleSubmitWizard = async (state: GitflowWizardState) => {
  await createGitflowBranch({
    branchType: state.branchType,
    branchName: state.branchName,
    baseBranch: state.metadata.base ?? undefined,
    autoPush: state.autoPush,
    metadata: state.metadata
  })
  closeWizard()
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

onMounted(() => {
  fetchGitflowBranches()
})
</script>

<style scoped>
.gitflow-dashboard {
  display: flex;
  flex-direction: column;
  gap: 24px;
  padding: 24px 0;
  position: relative;
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
  position: sticky;
  bottom: 16px;
  align-self: flex-end;
  margin-top: -8px;
  margin-right: 4px;
  padding: 20px 24px;
  width: min(440px, 100%);
  border: 1px solid #cbd5f5;
  border-radius: 16px;
  background: #ffffff;
  box-shadow: 0 12px 32px rgba(15, 23, 42, 0.12);
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


