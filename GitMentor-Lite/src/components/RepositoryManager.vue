<template>
  <div class="repository-manager">
    <div class="panels">
      <CheckoutPanel
        :form="cloneForm"
        :is-cloning="isCloning"
        :validation-state="validationState"
        @update:form="updateCloneForm"
        @validate="validateRepositoryUrl"
        @browse="selectTargetPath"
        @clone="handleCloneRepository"
      />

      <RemoteConfigPanel
        :configuration="remoteConfig"
        :loading="remoteLoading"
        :form="remoteForm"
        :mode="remoteMode"
        :current-branch="remoteConfig?.current_branch ?? null"
        @update:form="updateRemoteForm"
        @submit="handleRemoteSubmit"
        @reset-form="resetRemoteForm"
        @edit-remote="startEditRemote"
        @remove-remote="removeRemote"
        @set-upstream="setUpstream"
        @refresh="loadRemoteConfiguration"
      />
    </div>

    <OperationResult :result="operationResult" @clear="operationResult = null" />
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'

import CheckoutPanel from './CheckoutPanel.vue'
import RemoteConfigPanel from './RemoteConfigPanel.vue'
import OperationResult from './OperationResult.vue'

import type { CheckoutResult, OperationFeedback, RemoteConfiguration, RemoteInfo } from '../types/git'
import { useToast } from '../composables/useToast'

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

interface RemoteFormState {
  name: string
  url: string
}

const emit = defineEmits<{
  (event: 'clone-success', repositoryPath: string): void
}>()

const toast = useToast()

const cloneForm = reactive<CloneForm>({
  repositoryUrl: '',
  targetPath: '',
  branch: '',
  depth: null,
  recursive: true,
})

const validationState = reactive<ValidationState>({ status: 'idle', message: undefined })
const isCloning = ref(false)
const operationResult = ref<OperationFeedback | null>(null)
let resultTimer: ReturnType<typeof setTimeout> | null = null

const remoteConfig = ref<RemoteConfiguration | null>(null)
const remoteLoading = ref(false)
const remoteForm = reactive<RemoteFormState>({ name: '', url: '' })
const remoteMode = ref<'add' | 'edit'>('add')
const baseTargetDirectory = ref<string | null>(null)
const autoTargetPath = ref<string>('')

const setOperationResult = (result: OperationFeedback | null, autoClear = true) => {
  operationResult.value = result
  if (resultTimer) {
    clearTimeout(resultTimer)
    resultTimer = null
  }
  if (result && autoClear) {
    resultTimer = setTimeout(() => {
      operationResult.value = null
      resultTimer = null
    }, 8000)
  }
}

const updateCloneForm = (value: CloneForm) => {
  Object.assign(cloneForm, value)
}

const updateRemoteForm = (value: RemoteFormState) => {
  Object.assign(remoteForm, value)
}

const setValidationState = (status: ValidationStatus, message?: string) => {
  validationState.status = status
  validationState.message = message
}

const validateRepositoryUrl = async () => {
  const url = cloneForm.repositoryUrl.trim()
  if (!url) {
    setValidationState('invalid', '请先填写仓库 URL')
    return
  }

  setValidationState('checking', '正在验证远程连接...')

  try {
    const isValid = await invoke<boolean>('validate_remote_connection', { url })
    if (isValid) {
      setValidationState('valid', '远程仓库连接正常')
    } else {
      setValidationState('invalid', '无法连接远程仓库，请检查地址或网络')
    }
  } catch (error: any) {
    console.error('Validate remote failed', error)
    const message = String(error) || '远程校验失败，请稍后再试'
    setValidationState('invalid', message)
    toast.error(message, '验证失败')
  }
}

const normalizePath = (path: string) => path.replace(/[\\/]+$/, '')

const detectSeparator = (path: string) => (path.includes('\\') && !path.includes('/')) ? '\\' : '/'

const extractRepoSlug = (url: string): string | null => {
  if (!url) return null
  let normalized = url.trim()
  if (!normalized) return null

  if (/^[^@]+@[^:]+:.+/.test(normalized) && !normalized.startsWith('http')) {
    const index = normalized.indexOf(':')
    if (index !== -1) {
      normalized = `${normalized.slice(0, index)}/${normalized.slice(index + 1)}`
    }
  }

  normalized = normalized.replace(/\.git(\s*)$/i, '')
  const segments = normalized.split(/[\\/]/).filter(Boolean)
  return segments.length ? segments[segments.length - 1] : null
}

const joinPath = (basePath: string, slug: string) => {
  if (!slug) return basePath
  const separator = detectSeparator(basePath || '')
  const sanitized = normalizePath(basePath)
  if (!sanitized) return slug
  return `${sanitized}${separator}${slug}`
}

const applyAutoTargetPath = (basePath: string) => {
  const trimmedBase = normalizePath(basePath)
  const slug = extractRepoSlug(cloneForm.repositoryUrl)
  let finalPath = trimmedBase

  if (slug) {
    const endsWithSlug = trimmedBase.toLowerCase().endsWith(slug.toLowerCase())
    finalPath = endsWithSlug ? trimmedBase : joinPath(trimmedBase, slug)
  }

  cloneForm.targetPath = finalPath
  autoTargetPath.value = finalPath
}

const selectTargetPath = async () => {
  try {
    const selected = await invoke<string | null>('open_folder_dialog', { requireGitRepo: false })
    if (selected) {
      baseTargetDirectory.value = selected
      applyAutoTargetPath(selected)
      setValidationState('idle')
    }
  } catch (error: any) {
    console.error('Select path failed', error)
    toast.error(String(error), '选择路径失败')
  }
}

watch(
  () => cloneForm.repositoryUrl,
  () => {
    const slug = extractRepoSlug(cloneForm.repositoryUrl)

    if (!baseTargetDirectory.value && !cloneForm.targetPath && slug) {
      cloneForm.targetPath = slug
      autoTargetPath.value = slug
      return
    }

    if (
      baseTargetDirectory.value &&
      (!cloneForm.targetPath || cloneForm.targetPath === autoTargetPath.value)
    ) {
      applyAutoTargetPath(baseTargetDirectory.value)
    }
  }
)

watch(
  () => cloneForm.targetPath,
  (newValue) => {
    if (newValue !== autoTargetPath.value) {
      autoTargetPath.value = ''
    }
  }
)

const buildClonePayload = () => {
  const repository_url = cloneForm.repositoryUrl.trim()
  const target_path = cloneForm.targetPath.trim()
  const branch = cloneForm.branch.trim()

  return {
    repository_url,
    target_path,
    branch: branch ? branch : null,
    depth: cloneForm.depth && cloneForm.depth > 0 ? cloneForm.depth : null,
    recursive: cloneForm.recursive,
  }
}

const handleCloneRepository = async () => {
  if (isCloning.value) return
  const payload = buildClonePayload()

  if (!payload.repository_url) {
    toast.warning('请填写有效的仓库 URL', '提示')
    return
  }

  if (!payload.target_path) {
    toast.warning('请指定克隆目标路径', '提示')
    return
  }

  isCloning.value = true
  setOperationResult(null, false)
  setValidationState('idle')

  try {
    const result = await invoke<CheckoutResult>('clone_repository', { request: payload })
    processCloneResult(result)
  } catch (error: any) {
    console.error('Clone repository failed', error)
    toast.error(String(error), '克隆失败')
    setOperationResult({
      success: false,
      title: '克隆失败',
      message: String(error),
    })
  } finally {
    isCloning.value = false
  }
}

const processCloneResult = (result: CheckoutResult) => {
  if (result.success) {
    toast.success('仓库克隆完成', '操作成功')
    setOperationResult(
      {
        success: true,
        title: '克隆成功',
        message: `仓库已克隆至 ${result.repository_path}`,
        suggestion: result.commit_info
          ? `最新提交：${result.commit_info.short_hash} ${result.commit_info.message}`
          : undefined,
        duration_ms: result.duration_ms,
      },
      false
    )
    emit('clone-success', result.repository_path)
    baseTargetDirectory.value = null
    loadRemoteConfiguration()
  } else {
    const suggestion = result.suggestion || result.error?.suggestion || null
    const message = result.error_message || result.error?.user_message || '未知错误'
    toast.error(message, '克隆失败')
    setOperationResult(
      {
        success: false,
        title: '克隆失败',
        message,
        suggestion,
        duration_ms: result.duration_ms,
      },
      false
    )
  }
}

const loadRemoteConfiguration = async () => {
  try {
    remoteLoading.value = true
    const config = await invoke<RemoteConfiguration>('get_remote_configuration')
    remoteConfig.value = config
  } catch (error: any) {
    console.error('Load remote configuration failed', error)
    const message = String(error)
    if (!message.includes('No repository opened')) {
      toast.error(message, '加载远程仓库失败')
    } else {
      remoteConfig.value = null
    }
  } finally {
    remoteLoading.value = false
  }
}

const resetRemoteForm = () => {
  remoteForm.name = ''
  remoteForm.url = ''
  remoteMode.value = 'add'
}

interface GitOperationResultPayload {
  success: boolean
  message: string
  details?: string | null
}

const showGitOperationResult = (result: GitOperationResultPayload, title: string) => {
  if (result.success) {
    toast.success(result.message, title)
  } else {
    toast.error(result.message, `${title}失败`)
  }

  setOperationResult(
    {
      success: result.success,
      title: result.success ? `${title}成功` : `${title}失败`,
      message: result.message,
      suggestion: result.details || undefined,
    },
    true
  )
}

const handleRemoteSubmit = async () => {
  const name = remoteForm.name.trim()
  const url = remoteForm.url.trim()
  if (!name) {
    toast.warning('请输入远程名称', '提示')
    return
  }
  if (!url) {
    toast.warning('请输入远程 URL', '提示')
    return
  }

  remoteLoading.value = true
  try {
    const operation = remoteMode.value === 'add' ? 'Add' : 'Update'
    const response = await invoke<GitOperationResultPayload>('configure_remote', {
      request: {
        remote_name: name,
        remote_url: url,
        operation,
      },
    })

    showGitOperationResult(response, remoteMode.value === 'add' ? '添加远程' : '更新远程')
    await loadRemoteConfiguration()
    resetRemoteForm()
  } catch (error: any) {
    console.error('Configure remote failed', error)
    toast.error(String(error), '操作失败')
    setOperationResult({
      success: false,
      title: '远程操作失败',
      message: String(error),
    })
  } finally {
    remoteLoading.value = false
  }
}

const startEditRemote = (remote: RemoteInfo) => {
  remoteMode.value = 'edit'
  remoteForm.name = remote.name
  remoteForm.url = remote.fetch_url || remote.push_url || ''
}

const removeRemote = async (name: string) => {
  if (!name) return
  remoteLoading.value = true
  try {
    const response = await invoke<GitOperationResultPayload>('configure_remote', {
      request: {
        remote_name: name,
        remote_url: null,
        operation: 'Remove',
      },
    })
    showGitOperationResult(response, '移除远程')
    await loadRemoteConfiguration()
    resetRemoteForm()
  } catch (error: any) {
    console.error('Remove remote failed', error)
    toast.error(String(error), '移除远程失败')
    setOperationResult({
      success: false,
      title: '移除远程失败',
      message: String(error),
    })
  } finally {
    remoteLoading.value = false
  }
}

const setUpstream = async (remoteName: string, branch: string) => {
  const currentBranch = remoteConfig.value?.current_branch
  if (!currentBranch) {
    toast.warning('当前仓库未检测到本地分支', '提示')
    return
  }

  remoteLoading.value = true
  try {
    const response = await invoke<GitOperationResultPayload>('configure_remote', {
      request: {
        remote_name: remoteName,
        remote_url: null,
        operation: {
          SetUpstream: {
            branch: currentBranch,
            remote_branch: branch,
          },
        },
      },
    })
    showGitOperationResult(response, '设置上游分支')
    await loadRemoteConfiguration()
  } catch (error: any) {
    console.error('Set upstream failed', error)
    toast.error(String(error), '设置上游失败')
    setOperationResult({
      success: false,
      title: '设置上游失败',
      message: String(error),
    })
  } finally {
    remoteLoading.value = false
  }
}

onMounted(() => {
  loadRemoteConfiguration()
})

watch(
  () => cloneForm.repositoryUrl,
  () => {
    if (validationState.status === 'invalid' && !cloneForm.repositoryUrl.trim()) {
      setValidationState('idle')
    }
  }
)
</script>

<style scoped>
.repository-manager {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.panels {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(320px, 1fr));
  gap: 24px;
}
</style>
