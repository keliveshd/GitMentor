// Git 相关类型定义

export interface RemoteInfo {
  name: string
  fetch_url?: string | null
  push_url?: string | null
  branches: RemoteBranchInfo[]
  is_current_upstream: boolean
}

export interface RemoteBranchInfo {
  name: string
  full_name: string
  is_tracking_current: boolean
}

export interface RemoteConfiguration {
  current_branch?: string | null
  current_upstream?: string | null
  remotes: RemoteInfo[]
}

export interface CommitInfo {
  hash: string
  short_hash: string
  message: string
  author: string
  email: string
  timestamp: number
  files_changed: string[]
}

export type GitErrorKind =
  | 'Network'
  | 'FileSystem'
  | 'GitOperation'
  | 'Configuration'
  | 'Validation'

export interface GitErrorInfo {
  kind: GitErrorKind
  user_message: string
  suggestion?: string | null
  raw_message?: string | null
}

export interface CheckoutResult {
  success: boolean
  repository_path: string
  duration_ms: number
  commit_info?: CommitInfo | null
  error_message?: string | null
  suggestion?: string | null
  error?: GitErrorInfo | null
}

export interface OperationFeedback {
  success: boolean
  title: string
  message?: string
  suggestion?: string | null
  duration_ms?: number
}
