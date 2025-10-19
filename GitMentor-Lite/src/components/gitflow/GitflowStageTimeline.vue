<template>
  <ul class="stage-list">
    <li v-for="item in items" :key="item.id" :class="['stage-item', item.status]">
      <div class="stage-indicator">
        <span class="stage-icon">
          <template v-if="item.status === 'done'">✔</template>
          <template v-else-if="item.status === 'doing'">●</template>
          <template v-else>○</template>
        </span>
        <div class="stage-line"></div>
      </div>
      <div class="stage-body">
        <div class="stage-head">
          <strong>{{ item.label }}</strong>
          <span class="status">{{ statusLabel(item.status) }}</span>
        </div>
        <p v-if="item.description">{{ item.description }}</p>
      </div>
    </li>
  </ul>
</template>

<script setup lang="ts">
import type { GitflowProgressItem } from '../../composables/useGitflow'

interface Props {
  items: GitflowProgressItem[]
}

defineProps<Props>()

const statusLabel = (status: GitflowProgressItem['status']) => {
  switch (status) {
    case 'done':
      return '已完成'
    case 'doing':
      return '进行中'
    default:
      return '待处理'
  }
}
</script>

<style scoped>
.stage-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.stage-item {
  display: flex;
  gap: 12px;
}

.stage-indicator {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
}

.stage-icon {
  width: 20px;
  height: 20px;
  border-radius: 50%;
  border: 2px solid #cbd5f5;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 12px;
  color: #6366f1;
  background: #ffffff;
}

.stage-item.done .stage-icon {
  background: #6366f1;
  color: #ffffff;
  border-color: #6366f1;
}

.stage-item.doing .stage-icon {
  border-color: #4338ca;
  color: #4338ca;
}

.stage-line {
  flex: 1;
  width: 2px;
  background: linear-gradient(180deg, rgba(148, 163, 184, 0.4), rgba(148, 163, 184, 0.1));
}

.stage-item:last-child .stage-line {
  display: none;
}

.stage-body {
  flex: 1;
  background: #f8fafc;
  border-radius: 12px;
  border: 1px solid #e2e8f0;
  padding: 12px 16px;
}

.stage-head {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 13px;
  color: #1f2937;
}

.stage-head strong {
  font-size: 14px;
}

.stage-head .status {
  font-size: 12px;
  color: #64748b;
}

.stage-body p {
  margin: 8px 0 0;
  font-size: 13px;
  color: #64748b;
}
</style>
