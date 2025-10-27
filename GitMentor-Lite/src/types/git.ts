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
