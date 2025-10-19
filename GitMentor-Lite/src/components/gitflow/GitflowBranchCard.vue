<template>
  <div
    class="gitflow-branch-card"
    :class="[branch.branchType, { active: isActive }]"
    @click="$emit('select', branch.id)"
  >
    <header class="card-header">
      <div class="title-block">
        <span class="type-chip" :style="{ borderColor: meta.accent, color: meta.accent }">
          {{ meta.label }}
        </span>
        <h3 class="branch-name" :title="branch.name">{{ branch.name }}</h3>
      </div>
      <span v-if="branch.status !== 'merged'" class="status-chip">{{ statusText }}</span>
    </header>

    <div class="card-body">
      <div class="meta-row">
        <span class="meta-label">基线</span>
        <span class="meta-value">{{ branch.base }}</span>
      </div>
      <div class="meta-row">
        <span class="meta-label">最近更新</span>
        <span class="meta-value">{{ lastUpdatedLabel }}</span>
      </div>
      <div v-if="branch.latestCommit" class="meta-row">
        <span class="meta-label">最新提交</span>
        <span class="meta-value commit">{{ branch.latestCommit }}</span>
      </div>
      <div class="meta-row divergence">
        <span class="meta-label">差异</span>
        <span class="meta-value">
          <span class="ahead" v-if="branch.divergence?.ahead">+{{ branch.divergence.ahead }}</span>
          <span class="behind" v-if="branch.divergence?.behind">-{{ branch.divergence.behind }}</span>
        </span>
      </div>
      <div v-if="branch.includeInRelease?.length" class="meta-row release-linked">
        <span class="meta-label">包含功能</span>
        <span class="meta-value">{{ branch.includeInRelease.join(', ') }}</span>
      </div>
      <div v-if="branch.slaDeadline" class="meta-row sla">
        <span class="meta-label">SLA</span>
        <span class="meta-value">{{ slaRemaining }}</span>
      </div>
      <div v-if="branch.metrics?.riskLevel" class="meta-row risk">
        <span class="meta-label">风险</span>
        <span class="meta-value">{{ riskLabel }}</span>
      </div>
      <p v-if="branch.notes" class="notes">{{ branch.notes }}</p>
    </div>

    <footer class="card-footer">
      <button class="card-action" @click.stop="$emit('open-wizard', branch.branchType)">
        快速操作
      </button>
      <button class="card-action secondary" @click.stop="$emit('view-detail', branch.id)">
        查看详情
      </button>
    </footer>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { GitflowBranch, GitflowBranchType } from '../../composables/useGitflow'
import { branchTypeMeta } from '../../composables/useGitflow'

interface Props {
  branch: GitflowBranch
  isActive?: boolean
}

defineEmits<{
  (e: 'select', id: string): void
  (e: 'open-wizard', type: GitflowBranchType): void
  (e: 'view-detail', id: string): void
}>()

const props = defineProps<Props>()

const meta = computed(() => branchTypeMeta[props.branch.branchType])
const lastUpdatedLabel = computed(() => {
  if (!props.branch.lastUpdatedAt) return '—'
  return new Intl.DateTimeFormat('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit'
  }).format(new Date(props.branch.lastUpdatedAt))
})

const statusText = computed(() => {
  switch (props.branch.status) {
    case 'in_progress':
      return '开发中'
    case 'awaiting_merge':
      return '等待合并'
    case 'merged':
      return '已合并'
    default:
      return '准备中'
  }
})

const slaRemaining = computed(() => {
  if (!props.branch.slaDeadline) return ''
  const diff = new Date(props.branch.slaDeadline).getTime() - Date.now()
  if (diff <= 0) return '已超时'
  const hours = diff / (1000 * 60 * 60)
  if (hours >= 24) {
    const days = Math.floor(hours / 24)
    const remainHours = Math.round(hours % 24)
    return `${days} 天 ${remainHours} 小时`
  }
  return `${Math.ceil(hours)} 小时`
})

const riskLabel = computed(() => {
  const risk = props.branch.metrics?.riskLevel
  switch (risk) {
    case 'high':
      return '高'
    case 'medium':
      return '中'
    case 'low':
      return '低'
    default:
      return '未知'
  }
})
</script>

<style scoped>
.gitflow-branch-card {
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 16px;
  border: 1px solid #e2e8f0;
  border-radius: 12px;
  background: #ffffff;
  cursor: pointer;
  transition: border-color 0.2s ease, box-shadow 0.2s ease, transform 0.2s ease;
  min-height: 180px;
}

.gitflow-branch-card:hover {
  border-color: #cbd5f5;
  box-shadow: 0 8px 16px rgba(15, 23, 42, 0.08);
  transform: translateY(-2px);
}

.gitflow-branch-card.active {
  border-color: #6366f1;
  box-shadow: 0 12px 24px rgba(99, 102, 241, 0.16);
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 12px;
}

.title-block {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.type-chip {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 2px 8px;
  border-radius: 999px;
  font-size: 12px;
  font-weight: 600;
  border: 1px solid currentColor;
  text-transform: uppercase;
}

.branch-name {
  font-size: 16px;
  font-weight: 600;
  color: #0f172a;
  margin: 0;
}

.status-chip {
  padding: 2px 10px;
  border-radius: 999px;
  font-size: 12px;
  background: #f1f5f9;
  color: #1e293b;
  font-weight: 500;
}

.card-body {
  display: flex;
  flex-direction: column;
  gap: 6px;
  color: #475569;
  font-size: 13px;
}

.meta-row {
  display: flex;
  justify-content: space-between;
  gap: 12px;
}

.meta-label {
  color: #94a3b8;
  font-weight: 500;
}

.meta-value {
  text-align: right;
  flex: 1;
}

.meta-value.commit {
  color: #0f172a;
  font-weight: 500;
}

.meta-row.divergence .meta-value {
  display: flex;
  justify-content: flex-end;
  gap: 6px;
}

.ahead {
  color: #15803d;
  font-weight: 600;
}

.behind {
  color: #dc2626;
  font-weight: 600;
}

.meta-row.release-linked .meta-value {
  max-height: 42px;
  overflow: hidden;
  text-overflow: ellipsis;
}

.meta-row.sla .meta-value {
  color: #f97316;
  font-weight: 600;
}

.meta-row.risk .meta-value {
  color: #dc2626;
  font-weight: 600;
}

.notes {
  margin: 6px 0 0;
  line-height: 1.5;
  color: #475569;
}

.card-footer {
  display: flex;
  gap: 8px;
  margin-top: auto;
}

.card-action {
  flex: 1;
  padding: 8px 12px;
  border-radius: 8px;
  border: 1px solid #6366f1;
  background: #eef2ff;
  color: #4338ca;
  font-size: 13px;
  font-weight: 600;
  transition: background 0.2s ease;
}

.card-action:hover {
  background: #e0e7ff;
}

.card-action.secondary {
  border-color: #cbd5f5;
  background: #ffffff;
  color: #475569;
}

.card-action.secondary:hover {
  background: #f8fafc;
}
</style>
