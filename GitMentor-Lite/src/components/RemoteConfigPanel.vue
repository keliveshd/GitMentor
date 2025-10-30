<template>
  <section class="remote-panel">
    <header class="panel-header">
      <div>
        <h3>远程仓库配置</h3>
        <p class="panel-subtitle">
          查看当前远程仓库列表，支持添加、更新、删除以及设置上游分支。
        </p>
      </div>
      <button type="button" class="refresh-btn" @click="emit('refresh')" :disabled="loading">
        {{ loading ? '加载中...' : '刷新' }}
      </button>
    </header>

    <div class="panel-content">
      <form class="remote-form" @submit.prevent="submitForm">
        <div class="form-grid">
          <div class="form-row">
            <label for="remote-name">远程名称</label>
            <input
              id="remote-name"
              type="text"
              :value="form.name"
              placeholder="例如 origin / upstream"
              @input="updateField('name', ($event.target as HTMLInputElement).value)"
              :disabled="loading || mode === 'edit'"
            />
          </div>
          <div class="form-row">
            <label for="remote-url">远程 URL</label>
            <input
              id="remote-url"
              type="text"
              :value="form.url"
              placeholder="https:// 或 git@..."
              @input="updateField('url', ($event.target as HTMLInputElement).value)"
              :disabled="loading"
            />
          </div>
        </div>

        <div class="form-actions">
          <button type="submit" class="primary" :disabled="loading || !form.name || !form.url">
            {{ mode === 'add' ? '添加远程' : '更新远程' }}
          </button>
          <button type="button" class="ghost" @click="emit('reset-form')" :disabled="loading && mode === 'add'">
            {{ mode === 'add' ? '清空' : '取消编辑' }}
          </button>
        </div>
      </form>

      <div class="remote-list">
        <p v-if="!loading && (!configuration || configuration.remotes.length === 0)" class="empty-state">
          尚未检测到远程仓库，先添加一个吧。
        </p>

        <div v-for="remote in configuration?.remotes || []" :key="remote.name" class="remote-card">
          <div class="remote-card-header">
            <div>
              <span class="remote-name">{{ remote.name }}</span>
              <span v-if="remote.is_current_upstream" class="remote-tag">当前上游</span>
            </div>
            <div class="remote-card-actions">
              <button type="button" class="link-btn" @click="emit('edit-remote', remote)" :disabled="loading">
                编辑
              </button>
              <button type="button" class="link-btn danger" @click="emit('remove-remote', remote.name)" :disabled="loading">
                删除
              </button>
            </div>
          </div>

          <div class="remote-urls">
            <div>Fetch: {{ remote.fetch_url || '未配置' }}</div>
            <div>Push: {{ remote.push_url || remote.fetch_url || '未配置' }}</div>
          </div>

          <div v-if="remote.branches.length > 0" class="remote-branches">
            <div class="branch-row" v-for="branch in remote.branches" :key="branch.full_name">
              <span class="branch-name">{{ branch.name }}</span>
              <span v-if="branch.is_tracking_current" class="branch-tag">已关联</span>
              <button
                v-else
                type="button"
                class="mini-btn"
                @click="emit('set-upstream', remote.name, branch.name)"
                :disabled="loading || !currentBranch"
              >
                设为上游
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
import type { RemoteConfiguration, RemoteInfo } from '../types/git'

type RemoteForm = {
  name: string
  url: string
}

const props = defineProps<{
  configuration: RemoteConfiguration | null
  loading: boolean
  form: RemoteForm
  mode: 'add' | 'edit'
  currentBranch: string | null
}>()

const emit = defineEmits<{
  (event: 'update:form', value: RemoteForm): void
  (event: 'submit'): void
  (event: 'reset-form'): void
  (event: 'edit-remote', value: RemoteInfo): void
  (event: 'remove-remote', name: string): void
  (event: 'set-upstream', remote: string, branch: string): void
  (event: 'refresh'): void
}>()

const updateField = <K extends keyof RemoteForm>(field: K, value: RemoteForm[K]) => {
  emit('update:form', {
    ...props.form,
    [field]: value,
  })
}

const submitForm = () => {
  emit('submit')
}
</script>

<style scoped>
.remote-panel {
  background: #ffffff;
  border: 1px solid #e2e8f0;
  border-radius: 12px;
  padding: 20px;
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.panel-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 16px;
}

.panel-header h3 {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
  color: #1f2937;
}

.panel-subtitle {
  margin: 4px 0 0;
  color: #6b7280;
  font-size: 13px;
}

.refresh-btn {
  border-radius: 8px;
  padding: 8px 14px;
  background: #eff6ff;
  color: #2563eb;
  border: 1px solid #bfdbfe;
  font-size: 13px;
  cursor: pointer;
}

.refresh-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.panel-content {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.remote-form {
  display: flex;
  flex-direction: column;
  gap: 16px;
  border: 1px solid #e5e7eb;
  border-radius: 10px;
  padding: 16px;
  background: #fafafa;
}

.form-grid {
  display: grid;
  gap: 16px;
  grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
}

.form-row {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.form-row label {
  font-size: 13px;
  color: #374151;
  font-weight: 500;
}

.form-row input {
  border: 1px solid #d1d5db;
  border-radius: 8px;
  padding: 10px 12px;
  font-size: 14px;
  color: #111827;
  background: #ffffff;
}

.form-actions {
  display: flex;
  gap: 12px;
}

.form-actions button {
  border-radius: 8px;
  padding: 10px 16px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  border: none;
}

button.primary {
  background: #2563eb;
  color: #ffffff;
}

button.primary:disabled {
  background: #93c5fd;
  cursor: not-allowed;
}

button.ghost {
  background: transparent;
  color: #2563eb;
  border: 1px solid #bfdbfe;
}

.remote-list {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.empty-state {
  font-size: 13px;
  color: #6b7280;
}

.remote-card {
  border: 1px solid #e5e7eb;
  border-radius: 10px;
  padding: 16px;
  background: #ffffff;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.remote-card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 12px;
}

.remote-card-actions {
  display: flex;
  gap: 8px;
}

.link-btn {
  background: none;
  border: none;
  font-size: 13px;
  color: #2563eb;
  cursor: pointer;
}

.link-btn.danger {
  color: #dc2626;
}

.remote-name {
  font-size: 15px;
  font-weight: 600;
  color: #111827;
}

.remote-tag {
  margin-left: 8px;
  border-radius: 999px;
  padding: 2px 8px;
  font-size: 11px;
  color: #1e3a8a;
  background: #dbeafe;
}

.remote-urls {
  font-size: 12px;
  color: #4b5563;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.remote-branches {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.branch-row {
  display: flex;
  align-items: center;
  gap: 12px;
  font-size: 12px;
}

.branch-name {
  min-width: 120px;
  font-weight: 500;
}

.branch-tag {
  background: #dcfce7;
  color: #15803d;
  padding: 2px 8px;
  border-radius: 999px;
  font-size: 11px;
}

.mini-btn {
  border-radius: 6px;
  border: 1px solid #2563eb;
  background: transparent;
  color: #2563eb;
  padding: 4px 10px;
  font-size: 12px;
  cursor: pointer;
}

.mini-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}
</style>
