<template>
  <transition name="fade">
    <div v-if="result" class="operation-result" :class="result.success ? 'success' : 'error'">
      <div class="result-header">
        <strong>{{ result.title }}</strong>
        <button type="button" class="close-btn" @click="emit('clear')">×</button>
      </div>
      <p v-if="result.message" class="result-message">{{ result.message }}</p>
      <p v-if="result.suggestion" class="result-suggestion">建议：{{ result.suggestion }}</p>
      <p v-if="result.duration_ms !== undefined" class="result-meta">
        耗时 {{ (result.duration_ms / 1000).toFixed(2) }} 秒
      </p>
    </div>
  </transition>
</template>

<script setup lang="ts">
import type { OperationFeedback } from '../types/git'

const { result } = defineProps<{
  result: OperationFeedback | null
}>()

const emit = defineEmits<{
  (event: 'clear'): void
}>()
</script>

<style scoped>
.operation-result {
  border-radius: 12px;
  padding: 16px;
  border: 1px solid transparent;
  background: #ffffff;
  box-shadow: 0 10px 30px rgba(15, 23, 42, 0.08);
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.operation-result.success {
  border-color: #bbf7d0;
  background: #ecfdf5;
  color: #166534;
}

.operation-result.error {
  border-color: #fecaca;
  background: #fef2f2;
  color: #b91c1c;
}

.result-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.result-header strong {
  font-size: 15px;
  font-weight: 600;
}

.result-message,
.result-suggestion,
.result-meta {
  margin: 0;
  font-size: 13px;
}

.result-suggestion {
  font-style: italic;
}

.result-meta {
  color: rgba(0, 0, 0, 0.6);
}

.close-btn {
  background: none;
  border: none;
  font-size: 18px;
  line-height: 1;
  cursor: pointer;
  color: inherit;
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
