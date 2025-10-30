<template>
  <section class="checkout-panel">
    <header class="panel-header">
      <div>
        <h3>克隆新仓库</h3>
        <p class="panel-subtitle">填写仓库信息并选择目标路径，支持分支和浅克隆设置。</p>
      </div>
      <span v-if="validationState.status === 'valid'" class="badge success">已验证</span>
      <span v-else-if="validationState.status === 'invalid'" class="badge danger">验证失败</span>
      <span v-else-if="validationState.status === 'checking'" class="badge info">验证中</span>
    </header>

    <form class="form-body" @submit.prevent="handleClone">
      <div class="form-row">
        <label for="repository-url">仓库 URL</label>
        <div class="input-actions">
          <input
            id="repository-url"
            type="text"
            :value="form.repositoryUrl"
            placeholder="https://github.com/user/project.git"
            @input="updateField('repositoryUrl', ($event.target as HTMLInputElement).value)"
            :disabled="isCloning"
          />
          <button type="button" class="secondary" @click="triggerValidate" :disabled="isCloning || !form.repositoryUrl">
            验证
          </button>
        </div>
        <p
          v-if="validationState.message"
          class="validation-message"
          :class="{ error: validationState.status === 'invalid', success: validationState.status === 'valid' }"
        >
          {{ validationState.message }}
        </p>
      </div>

      <div class="form-row">
        <label for="target-path">目标路径</label>
        <div class="input-actions">
          <input
            id="target-path"
            type="text"
            :value="form.targetPath"
            placeholder="选择要保存仓库的位置"
            @input="updateField('targetPath', ($event.target as HTMLInputElement).value)"
            :disabled="isCloning"
          />
          <button type="button" class="ghost" @click="emit('browse')" :disabled="isCloning">
            浏览
          </button>
        </div>
      </div>

      <div class="form-grid">
        <div class="form-row">
          <label for="branch">分支（可选）</label>
          <input
            id="branch"
            type="text"
            :value="form.branch"
            placeholder="main / master / feature-name"
            @input="updateField('branch', ($event.target as HTMLInputElement).value)"
            :disabled="isCloning"
          />
        </div>

        <div class="form-row">
          <label for="depth">浅克隆深度</label>
          <input
            id="depth"
            type="number"
            min="0"
            :value="form.depth ?? ''"
            placeholder="0 表示完整克隆"
            @input="updateDepth($event)"
            :disabled="isCloning"
          />
        </div>
      </div>

      <div class="form-row checkbox-row">
        <label>
          <input
            type="checkbox"
            :checked="form.recursive"
            @change="updateField('recursive', ($event.target as HTMLInputElement).checked)"
            :disabled="isCloning"
          />
          递归克隆子模块
        </label>
      </div>

      <div class="form-actions">
        <button type="submit" class="primary" :disabled="isCloning">
          {{ isCloning ? '克隆中...' : '克隆仓库' }}
        </button>
      </div>
    </form>
  </section>
</template>

<script setup lang="ts">
type ValidationStatus = 'idle' | 'checking' | 'valid' | 'invalid'

interface CloneForm {
  repositoryUrl: string
  targetPath: string
  branch: string
  depth: number | null
  recursive: boolean
}

interface ValidationState {
  status: ValidationStatus
  message?: string
}

const props = defineProps<{
  form: CloneForm
  isCloning: boolean
  validationState: ValidationState
}>()

const emit = defineEmits<{
  (event: 'update:form', value: CloneForm): void
  (event: 'validate'): void
  (event: 'browse'): void
  (event: 'clone'): void
}>()

const updateField = <K extends keyof CloneForm>(field: K, value: CloneForm[K]) => {
  emit('update:form', {
    ...props.form,
    [field]: value,
  })
}

const updateDepth = (event: Event) => {
  const value = (event.target as HTMLInputElement).value
  if (value === '') {
    updateField('depth', null)
    return
  }
  const parsed = Number(value)
  updateField('depth', Number.isFinite(parsed) && parsed >= 0 ? parsed : null)
}

const triggerValidate = () => {
  emit('validate')
}

const handleClone = () => {
  emit('clone')
}
</script>

<style scoped>
.checkout-panel {
  background: #ffffff;
  border: 1px solid #e2e8f0;
  border-radius: 12px;
  padding: 20px;
  display: flex;
  flex-direction: column;
  gap: 16px;
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

.badge {
  display: inline-flex;
  align-items: center;
  padding: 4px 10px;
  border-radius: 999px;
  font-size: 12px;
  font-weight: 500;
}

.badge.success {
  background: #dcfce7;
  color: #166534;
}

.badge.danger {
  background: #fee2e2;
  color: #b91c1c;
}

.badge.info {
  background: #e0f2fe;
  color: #0369a1;
}

.form-body {
  display: flex;
  flex-direction: column;
  gap: 16px;
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

.input-actions {
  display: flex;
  gap: 8px;
}

.input-actions input {
  flex: 1;
}

input[type='text'],
input[type='number'] {
  border: 1px solid #d1d5db;
  border-radius: 8px;
  padding: 10px 12px;
  font-size: 14px;
  color: #111827;
  background: #ffffff;
}

input[type='text']:disabled,
input[type='number']:disabled {
  background: #f3f4f6;
}

.form-grid {
  display: grid;
  gap: 16px;
  grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
}

.checkbox-row label {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  color: #374151;
}

.form-actions {
  display: flex;
  justify-content: flex-end;
}

button.primary,
button.secondary,
button.ghost {
  border-radius: 8px;
  padding: 10px 16px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: background 0.2s ease, color 0.2s ease;
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

button.secondary {
  background: #f3f4f6;
  color: #1f2937;
}

button.secondary:hover:not(:disabled) {
  background: #e5e7eb;
}

button.ghost {
  background: transparent;
  color: #2563eb;
  border: 1px solid #bfdbfe;
}

button.ghost:hover:not(:disabled) {
  background: #eff6ff;
}

.validation-message {
  font-size: 12px;
}

.validation-message.error {
  color: #b91c1c;
}

.validation-message.success {
  color: #15803d;
}
</style>
