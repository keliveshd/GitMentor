<template>
  <div class="detail-body">
    <section class="detail-meta">
      <dl>
        <div class="detail-row">
          <dt>基线</dt>
          <dd>{{ branch.base }}</dd>
        </div>
        <div class="detail-row">
          <dt>状态</dt>
          <dd>{{ statusLabel }}</dd>
        </div>
        <div class="detail-row">
          <dt>负责人</dt>
          <dd>{{ branch.metrics?.owner ?? '未指定' }}</dd>
        </div>
        <div class="detail-row">
          <dt>最近更新</dt>
          <dd>{{ formatDate(branch.lastUpdatedAt) }}</dd>
        </div>
        <div class="detail-row" v-if="branch.metrics?.relatedWork?.length">
          <dt>关联事项</dt>
          <dd>{{ branch.metrics.relatedWork.join(', ') }}</dd>
        </div>
        <div class="detail-row" v-if="branch.notes">
          <dt>备注</dt>
          <dd>{{ branch.notes }}</dd>
        </div>
      </dl>
    </section>

    <section v-if="branch.progress?.length" class="detail-section">
      <h4>阶段进展</h4>
      <GitflowStageTimeline :items="branch.progress" />
    </section>

    <section
      v-if="branch.branchType === 'release' && branch.includeInRelease?.length"
      class="detail-section"
    >
      <h4>纳入功能</h4>
      <ul class="feature-list">
        <li v-for="feature in branch.includeInRelease" :key="feature">
          <span class="bullet">•</span>{{ feature }}
        </li>
      </ul>
    </section>

    <section v-if="branch.qaChecklist?.length" class="detail-section">
      <h4>QA 检查项</h4>
      <ul class="task-list">
        <li v-for="task in branch.qaChecklist" :key="task.id" :class="{ done: task.done }">
          <span class="checkbox">{{ task.done ? '✔' : '○' }}</span>
          <span>{{ task.label }}</span>
        </li>
      </ul>
    </section>

    <section v-if="branch.tasks?.length" class="detail-section">
      <h4>工作待办</h4>
      <ul class="task-list">
        <li v-for="task in branch.tasks" :key="task.id" :class="{ done: task.done }">
          <span class="checkbox">{{ task.done ? '✔' : '○' }}</span>
          <div class="task-content">
            <span>{{ task.label }}</span>
            <small v-if="task.owner">责任人：{{ task.owner }}</small>
          </div>
        </li>
      </ul>
    </section>

    <section v-if="branch.timeline?.length" class="detail-section">
      <h4>事件时间线</h4>
      <ul class="timeline">
        <li v-for="node in branch.timeline" :key="node.id" :class="['timeline-item', node.status]">
          <div class="timeline-icon">{{ node.status === 'done' ? '✅' : '⏳' }}</div>
          <div class="timeline-body">
            <div class="timeline-head">
              <strong>{{ node.label }}</strong>
              <span>{{ formatDate(node.timestamp, true) }}</span>
            </div>
            <p v-if="node.description">{{ node.description }}</p>
          </div>
        </li>
      </ul>
    </section>

    <section v-if="branch.nextActions?.length" class="detail-section actions">
      <h4>快捷操作</h4>
      <div class="actions-grid">
        <button
          v-for="action in branch.nextActions"
          :key="action.id"
          class="action-chip"
          :disabled="action.disabled"
          :title="action.description"
        >
          <span class="chip-icon">{{ action.icon }}</span>
          <span>{{ action.label }}</span>
        </button>
      </div>
    </section>

    <section v-if="branch.aiDrafts?.length" class="detail-section ai-drafts">
      <h4>AI 草稿</h4>
      <div class="draft-grid">
        <article v-for="draft in branch.aiDrafts" :key="draft.id" class="draft-card">
          <header>
            <span class="tone">{{ toneLabel(draft.tone) }}</span>
            <h5>{{ draft.title }}</h5>
          </header>
          <p>{{ draft.content }}</p>
          <footer>
            <button class="outline-btn">复制文案</button>
          </footer>
        </article>
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { GitflowBranch, GitflowBranchStatus, GitflowDraft } from '../../composables/useGitflow'
import GitflowStageTimeline from './GitflowStageTimeline.vue'

interface Props {
  branch: GitflowBranch
}

const props = defineProps<Props>()

const statusLabel = computed(() => statusText(props.branch.status))

const formatDate = (value?: string, withTime = false) => {
  if (!value) return '—'
  const date = new Date(value)
  return withTime
    ? new Intl.DateTimeFormat('zh-CN', {
        month: '2-digit',
        day: '2-digit',
        hour: '2-digit',
        minute: '2-digit'
      }).format(date)
    : new Intl.DateTimeFormat('zh-CN', {
        year: 'numeric',
        month: '2-digit',
        day: '2-digit'
      }).format(date)
}

const toneLabel = (tone: GitflowDraft['tone']) => {
  switch (tone) {
    case 'status':
      return '状态播报'
    case 'release':
      return '发布说明'
    case 'incident':
      return '事件通报'
    case 'retro':
      return '复盘提纲'
    default:
      return 'AI 草稿'
  }
}

const statusText = (status: GitflowBranchStatus) => {
  switch (status) {
    case 'in_progress':
      return '开发中'
    case 'awaiting_merge':
      return '等待合并'
    case 'merged':
      return '已合并'
    default:
      return '准备中'
  }
}
</script>

<style scoped>
.detail-body {
  display: flex;
  flex-direction: column;
  gap: 24px;
  color: #475569;
}

.detail-meta dl {
  display: grid;
  grid-template-columns: auto 1fr;
  row-gap: 12px;
  column-gap: 16px;
  margin: 0;
}

.detail-row dt {
  font-weight: 600;
  color: #1f2937;
}

.detail-row dd {
  margin: 0;
}

.detail-section h4 {
  margin: 0 0 12px;
  font-size: 15px;
  color: #111827;
}

.feature-list,
.task-list {
  margin: 0;
  padding: 0;
  list-style: none;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.feature-list .bullet {
  margin-right: 8px;
  color: #6366f1;
}

.task-list li {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  font-size: 13px;
}

.task-list li.done {
  color: #94a3b8;
  text-decoration: line-through;
}

.checkbox {
  width: 18px;
  height: 18px;
  border-radius: 50%;
  background: #eef2ff;
  color: #4338ca;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 11px;
}

.task-content {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.task-content small {
  color: #9ca3af;
}

.timeline {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.timeline-item {
  display: flex;
  gap: 12px;
}

.timeline-icon {
  width: 28px;
  height: 28px;
  border-radius: 50%;
  background: #eef2ff;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #4338ca;
  flex-shrink: 0;
}

.timeline-item.pending .timeline-icon {
  background: #fff7ed;
  color: #f97316;
}

.timeline-body {
  flex: 1;
  background: #f8fafc;
  border: 1px solid #e2e8f0;
  border-radius: 12px;
  padding: 12px;
}

.timeline-head {
  display: flex;
  justify-content: space-between;
  font-size: 13px;
  color: #0f172a;
}

.timeline-body p {
  margin: 6px 0 0;
  font-size: 13px;
}

.actions-grid {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.action-chip {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 8px 12px;
  border-radius: 999px;
  border: 1px solid #cbd5f5;
  background: #f8fafc;
  color: #1e293b;
  font-size: 13px;
  cursor: pointer;
  transition: background 0.2s ease;
}

.action-chip:hover:not(:disabled) {
  background: #e0e7ff;
}

.action-chip:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.chip-icon {
  font-size: 16px;
}

.ai-drafts .draft-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
  gap: 12px;
}

.draft-card {
  border: 1px solid #e2e8f0;
  border-radius: 12px;
  padding: 12px;
  display: flex;
  flex-direction: column;
  gap: 8px;
  background: #ffffff;
}

.draft-card h5 {
  margin: 4px 0 0;
  font-size: 15px;
  color: #1f2937;
}

.draft-card p {
  margin: 0;
  font-size: 13px;
  line-height: 1.5;
  color: #475569;
}

.tone {
  font-size: 11px;
  color: #6366f1;
  font-weight: 600;
  text-transform: uppercase;
}

.outline-btn {
  align-self: flex-start;
  padding: 6px 10px;
  border-radius: 8px;
  border: 1px dashed #6366f1;
  background: transparent;
  color: #4338ca;
  cursor: pointer;
  font-size: 12px;
}

.outline-btn:hover {
  background: #eef2ff;
}
</style>
