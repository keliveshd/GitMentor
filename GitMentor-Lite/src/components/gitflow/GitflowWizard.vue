<template>
  <teleport to="body">
    <transition name="fade">
      <div v-if="state.visible" class="gitflow-wizard-overlay" @click.self="handleClose">
        <div class="gitflow-wizard" role="dialog" aria-modal="true">
          <header class="wizard-header">
            <div class="header-left">
              <h2>创建 {{ meta.label }} 分支</h2>
              <p>{{ meta.description }}</p>
            </div>
            <button class="close-btn" @click="handleClose" aria-label="关闭向导">×</button>
          </header>

          <div class="wizard-body">
            <ol class="wizard-steps">
              <li
                v-for="stepItem in steps"
                :key="stepItem.id"
                :class="{ active: state.step === stepItem.id, done: state.step > stepItem.id }"
                @click="jumpToStep(stepItem.id)"
              >
                <span class="step-index">{{ stepItem.id }}</span>
                <div class="step-meta">
                  <span class="step-title">{{ stepItem.title }}</span>
                  <span class="step-desc">{{ stepItem.desc }}</span>
                </div>
              </li>
            </ol>

            <section class="wizard-content">
              <div v-if="state.step === 1" class="step-content">
                <h3>选择基线与命名规则</h3>
                <p class="description">
                  默认以 {{ defaultBase }} 为基线，命名遵循前缀 <code>{{ normalizedPrefix }}</code>。
                </p>

                <div class="form-grid">
                  <label class="form-field">
                    <span class="field-label">基线分支</span>
                    <select v-model="baseBranch" class="input">
                      <option :value="configSnapshot.developBranch">
                        {{ configSnapshot.developBranch }}
                      </option>
                      <option :value="configSnapshot.mainBranch">{{ configSnapshot.mainBranch }}</option>
                    </select>
                    <span class="field-help">如需从其他分支派生，可稍后手动调整</span>
                  </label>

                  <label class="form-field">
                    <span class="field-label">命名前缀</span>
                    <input
                      class="input"
                      v-model.trim="branchPrefixModel"
                      :placeholder="`例如：${defaultPrefix}`"
                    />
                    <span class="field-help">
                      默认 {{ defaultPrefix }}，修改后会记住下次设置。
                    </span>
                  </label>
                </div>
              </div>

              <div v-else-if="state.step === 2" class="step-content">
                <h3>填写分支信息</h3>
                <p class="description">{{ stepTwoDescription }}</p>

                <div class="form-grid">
                  <label class="form-field">
                    <span class="field-label">分支名称</span>
                    <input
                      class="input"
                      v-model="branchNameModel"
                      :placeholder="branchNamePlaceholder"
                    />
                    <span class="field-error" v-if="errors.branchName">{{ errors.branchName }}</span>
                  </label>

                  <label class="form-field" v-if="showIssueField">
                    <span class="field-label">关联工单</span>
                    <input
                      class="input"
                      v-model.trim="state.metadata.issueId"
                      placeholder="例如：ABC-123"
                    />
                  </label>

                  <label class="form-field">
                    <span class="field-label">负责人</span>
                    <input
                      class="input"
                      v-model.trim="state.metadata.owner"
                      placeholder="例如：Alice"
                    />
                  </label>

                  <label class="form-field">
                    <span class="field-label">分支说明</span>
                    <textarea
                      class="textarea"
                      rows="3"
                      v-model.trim="state.metadata.purpose"
                      placeholder="分支目标、风险提示或测试要求"
                    ></textarea>
                  </label>
                </div>
              </div>

              <div v-else class="step-content">
                <h3>即将执行的操作</h3>
                <p class="description">确认创建分支时需要执行的动作，稍后可在面板继续推进。</p>
                <ul class="summary-list">
                  <li>
                    <span class="summary-icon">🌱</span>
                    <div>
                      <strong>从 {{ baseBranch }} 创建新分支</strong>
                      <p>{{ finalBranchName }}</p>
                    </div>
                  </li>
                  <li>
                    <span class="summary-icon">🧾</span>
                    <div>
                      <strong>{{ metaSummary.title }}</strong>
                      <p>{{ metaSummary.content }}</p>
                    </div>
                  </li>
                  <li>
                    <span class="summary-icon">🧠</span>
                    <div>
                      <strong>准备 AI 草稿</strong>
                      <p>按照分支类型生成状态播报和检查清单。</p>
                    </div>
                  </li>
                  <li>
                    <span class="summary-icon">☁️</span>
                    <div>
                      <strong>{{ state.autoPush ? '创建后自动推送' : '暂不推送至远端' }}</strong>
                      <p>
                        {{ state.autoPush ? '分支会立即同步到远端仓库' : '可在 Gitflow 面板一键推送' }}
                      </p>
                    </div>
                  </li>
                </ul>
              </div>
            </section>
          </div>

          <footer class="wizard-footer">
            <div class="footer-left">
              <label class="checkbox">
                <input type="checkbox" v-model="state.autoPush" />
                <span>创建完成后自动推送</span>
              </label>
            </div>
            <div class="footer-actions">
              <button class="ghost-btn" @click="handleBack" :disabled="state.step === 1">上一步</button>
              <button class="primary-btn" @click="handleContinue">
                {{ state.step === 3 ? '创建分支' : '下一步' }}
              </button>
            </div>
          </footer>
        </div>
      </div>
    </transition>
  </teleport>
</template>

<script setup lang="ts">
import { computed, reactive, watch } from 'vue'
import { branchTypeMeta, useGitflow } from '../../composables/useGitflow'
import type { GitflowWizardState } from '../../composables/useGitflow'

interface Props {
  state: GitflowWizardState
}

const props = defineProps<Props>()

const emit = defineEmits<{
  (e: 'close'): void
  (e: 'submit', payload: GitflowWizardState): void
  (e: 'update:state', payload: Partial<GitflowWizardState>): void
}>()

const steps = [
  { id: 1, title: '选择类型', desc: '确认基线与命名规则' },
  { id: 2, title: '填写信息', desc: '输入分支名称与元数据' },
  { id: 3, title: '操作预览', desc: '核对即将执行的步骤' }
] as const

const { gitflowConfig, gitflowBranches, getDefaultBaseForType } = useGitflow()

const meta = computed(() => branchTypeMeta[props.state.branchType])

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

const defaultPrefix = computed(() => {
  switch (props.state.branchType) {
    case 'feature':
      return configSnapshot.value.featurePrefix
    case 'release':
      return configSnapshot.value.releasePrefix
    case 'bugfix':
      return configSnapshot.value.bugfixPrefix
    case 'hotfix':
      return configSnapshot.value.hotfixPrefix
  }
})

const branchPrefixModel = computed({
  get() {
    return props.state.branchPrefix || defaultPrefix.value
  },
  set(value: string) {
    emit('update:state', { branchPrefix: value.trim() })
  }
})

const normalizedPrefix = computed(() => {
  const trimmed = (props.state.branchPrefix || defaultPrefix.value).trim()
  return trimmed || defaultPrefix.value
})

const baseBranch = computed({
  get() {
    return props.state.metadata.base ?? defaultBase.value
  },
  set(value: string) {
    emit('update:state', {
      metadata: {
        ...props.state.metadata,
        base: value
      }
    })
  }
})

const defaultBase = computed(() => getDefaultBaseForType(props.state.branchType))

const fallbackSuffix = computed(() => {
  switch (props.state.branchType) {
    case 'release': {
      const today = new Date()
      const month = String(today.getMonth() + 1).padStart(2, '0')
      const day = String(today.getDate()).padStart(2, '0')
      return `${today.getFullYear()}${month}${day}`
    }
    case 'bugfix':
      return 'fix'
    case 'hotfix':
      return 'hotfix'
    default:
      return 'task'
  }
})

const sanitizeNameFragment = (value?: string) =>
  (value ?? '').trim().replace(/\s+/g, '-')

const buildBranchName = (raw?: string) => {
  const fragment = sanitizeNameFragment(raw)
  if (!fragment) return ''
  const prefix = normalizedPrefix.value
  if (fragment.startsWith(prefix)) {
    return fragment
  }
  return `${prefix}${fragment}`
}

const issueSuffix = computed(() =>
  props.state.metadata.issueId
    ? props.state.metadata.issueId.replace(/\s+/g, '-').toUpperCase()
    : ''
)

const defaultSuffix = computed(() => {
  if (props.state.branchType === 'release') {
    return fallbackSuffix.value
  }
  if (issueSuffix.value) {
    return issueSuffix.value
  }
  return fallbackSuffix.value
})

const branchNamePlaceholder = computed(() => buildBranchName(defaultSuffix.value))

const finalBranchName = computed(() => {
  if (props.state.branchName) {
    return props.state.branchName
  }
  return buildBranchName(defaultSuffix.value)
})

const branchNameModel = computed({
  get() {
    return props.state.branchName || ''
  },
  set(value: string) {
    const sanitized = buildBranchName(value)
    emit('update:state', { branchName: sanitized })
  }
})

const existingBranchNames = computed(() => {
  const set = new Set<string>()
  for (const branch of gitflowBranches.value) {
    set.add(branch.name.toLowerCase())
  }
  return set
})

const showIssueField = computed(() => props.state.branchType !== 'release')

const stepTwoDescription = computed(() =>
  props.state.branchType === 'release'
    ? '填写版本号和负责人，便于后续生成发布说明。'
    : '补充工单、负责人等信息，方便后续生成 AI 状态播报。'
)

const metaSummary = computed(() => {
  if (props.state.branchType === 'release') {
    const owner = props.state.metadata.owner || '未指定'
    const note = props.state.metadata.purpose?.trim() || '暂未填写版本说明'
    return {
      title: '发布附加信息',
      content: `负责人：${owner} ｜ 版本说明：${note}`
    }
  }
  const issue = props.state.metadata.issueId || '未填写'
  const owner = props.state.metadata.owner || '未指定'
  return {
    title: '绑定元数据',
    content: `工单：${issue} ｜ 负责人：${owner}`
  }
})

watch(
  () => normalizedPrefix.value,
  (next: string, prev: string | undefined) => {
    if (next === prev) return
    const current = props.state.branchName
    if (!current) return
    if (current.startsWith(next)) return
    const suffix = prev && current.startsWith(prev) ? current.slice(prev.length) : current
    const updated = buildBranchName(suffix)
    emit('update:state', { branchName: updated })
  }
)

const errors = reactive({
  branchName: ''
})

const handleClose = () => emit('close')

const handleBack = () => {
  if (props.state.step > 1) {
    emit('update:state', { step: (props.state.step - 1) as GitflowWizardState['step'] })
  }
}

const handleContinue = () => {
  if (props.state.step === 2 && !validateStepTwo()) {
    return
  }

  if (props.state.step < 3) {
    emit('update:state', { step: (props.state.step + 1) as GitflowWizardState['step'] })
    return
  }

  const sanitizedPrefix = normalizedPrefix.value
  const targetBranchName = finalBranchName.value
  const metadata = {
    ...props.state.metadata,
    base: baseBranch.value,
    prefix: sanitizedPrefix
  }

  emit('update:state', {
    branchPrefix: sanitizedPrefix,
    branchName: targetBranchName,
    metadata
  })

  emit('submit', {
    ...props.state,
    branchPrefix: sanitizedPrefix,
    branchName: targetBranchName,
    metadata
  })
}

const jumpToStep = (step: GitflowWizardState['step']) => {
  emit('update:state', { step })
}

const validateStepTwo = () => {
  errors.branchName = ''
  const candidate = finalBranchName.value
  if (!candidate) {
    errors.branchName = '请填写分支名称'
    return false
  }
  if (existingBranchNames.value.has(candidate.toLowerCase())) {
    errors.branchName = '当前仓库已存在该分支，请更换名称'
    return false
  }
  return true
}
</script>

<style scoped>
.gitflow-wizard-overlay {
  position: fixed;
  inset: 0;
  background: rgba(15, 23, 42, 0.55);
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 32px;
  z-index: 2000;
  backdrop-filter: blur(2px);
}

.gitflow-wizard {
  width: 760px;
  max-height: 88vh;
  display: flex;
  flex-direction: column;
  gap: 24px;
  border-radius: 20px;
  background: #ffffff;
  box-shadow: 0 24px 48px rgba(15, 23, 42, 0.2);
  padding: 28px 32px;
  overflow: hidden;
}

.wizard-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 24px;
}

.wizard-header h2 {
  margin: 0;
  font-size: 22px;
  color: #0f172a;
}

.wizard-header p {
  margin: 6px 0 0;
  color: #64748b;
  line-height: 1.5;
}

.close-btn {
  border: none;
  background: transparent;
  font-size: 22px;
  padding: 0;
  cursor: pointer;
  color: #94a3b8;
}

.wizard-body {
  display: flex;
  gap: 24px;
  flex: 1 1 auto;
  min-height: 0;
  overflow: hidden;
}

.wizard-steps {
  list-style: none;
  margin: 0;
  padding: 0;
  width: 220px;
  display: flex;
  flex-direction: column;
  gap: 12px;
  overflow-y: auto;
}

.wizard-steps li {
  display: flex;
  gap: 12px;
  padding: 12px;
  border-radius: 12px;
  cursor: pointer;
  border: 1px solid #e2e8f0;
  background: #f8fafc;
  transition: border-color 0.2s ease, background 0.2s ease;
}

.wizard-steps li.active {
  border-color: #6366f1;
  background: #eef2ff;
}

.wizard-steps li.done {
  border-color: #22c55e;
}

.step-index {
  width: 28px;
  height: 28px;
  border-radius: 50%;
  border: 2px solid currentColor;
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: 600;
  color: #475569;
}

.step-meta {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.step-title {
  font-weight: 600;
  color: #0f172a;
}

.step-desc {
  font-size: 12px;
  color: #94a3b8;
}

.wizard-content {
  flex: 1;
  min-height: 280px;
  border-radius: 16px;
  border: 1px dashed #cbd5f5;
  padding: 24px;
  background: #ffffff;
  display: flex;
  flex-direction: column;
  overflow: auto;
  min-width: 0;
}

.step-content {
  display: flex;
  flex-direction: column;
  gap: 18px;
  width: 100%;
  color: #475569;
}

.step-content h3 {
  margin: 0;
  font-size: 18px;
  color: #0f172a;
}

.description {
  margin: 0;
  color: #64748b;
  font-size: 14px;
}

.form-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
  gap: 16px;
}

.form-field {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.field-label {
  font-weight: 600;
  color: #1f2937;
}

.input,
.textarea,
select.input {
  width: 100%;
  padding: 10px 12px;
  border-radius: 10px;
  border: 1px solid #cbd5f5;
  background: #f8fafc;
  font-size: 14px;
  color: #1f2937;
  transition: border-color 0.2s ease, background 0.2s ease;
}

.textarea {
  resize: vertical;
}

.input:focus,
.textarea:focus,
select.input:focus {
  outline: none;
  border-color: #6366f1;
  background: #ffffff;
  box-shadow: 0 0 0 3px rgba(99, 102, 241, 0.1);
}

.field-help {
  font-size: 12px;
  color: #94a3b8;
}

.field-error {
  font-size: 12px;
  color: #dc2626;
}

.summary-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.summary-list li {
  display: flex;
  gap: 12px;
  align-items: flex-start;
  padding: 12px;
  border-radius: 12px;
  border: 1px solid #e2e8f0;
  background: #f8fafc;
}

.summary-icon {
  font-size: 20px;
}

.wizard-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 16px;
  flex-wrap: wrap;
  padding-top: 12px;
}

.checkbox {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  color: #475569;
}

.checkbox input {
  width: 16px;
  height: 16px;
  accent-color: #6366f1;
}

.footer-actions {
  display: flex;
  gap: 12px;
}

.ghost-btn,
.primary-btn {
  min-width: 110px;
  padding: 10px 16px;
  border-radius: 10px;
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s ease;
}

.ghost-btn {
  border: 1px solid #cbd5f5;
  background: #ffffff;
  color: #475569;
}

.ghost-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.ghost-btn:not(:disabled):hover {
  background: #f8fafc;
}

.primary-btn {
  border: none;
  background: linear-gradient(120deg, #6366f1, #8b5cf6);
  color: #ffffff;
  box-shadow: 0 6px 12px rgba(99, 102, 241, 0.25);
}

.primary-btn:hover {
  box-shadow: 0 12px 24px rgba(99, 102, 241, 0.35);
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
