<template>
  <div class="git-panel">
    <!-- 菜单栏 -->
    <div class="menu-bar">
      <div class="menu-left">
        <span class="app-title">GitMentor</span>
      </div>
      <div class="menu-right">
        <div class="menu-dropdown">
          <button @click="toggleMenu" class="menu-btn" title="菜单">
            ☰
          </button>
          <div v-if="showMenu" class="menu-dropdown-content">
            <button @click="openAISettings" class="menu-item" :disabled="loading || !tauriReady">
              🤖 AI服务设置
            </button>
            <button @click="openTemplateConfig" class="menu-item" :disabled="loading || !tauriReady">
              📝 模板配置
            </button>
            <button @click="openConversationHistory" class="menu-item" :disabled="loading || !tauriReady">
              📊 对话记录
            </button>
            <div class="menu-divider"></div>
            <button @click="checkForUpdates" class="menu-item" :disabled="loading || !tauriReady">
              🔄 检查更新
            </button>
            <button @click="openDebugSettings" class="menu-item">
              🛠️ 开发设置
            </button>
            <button @click="openAbout" class="menu-item">
              ℹ️ 关于
            </button>
          </div>
        </div>
      </div>
    </div>



    <!-- Tab导航栏 -->
    <!-- Author: Evilek, Date: 2025-01-08 -->
    <div class="tab-navigation">
      <div class="tab-list">
        <button v-for="tab in tabs" :key="tab.id" @click="switchTab(tab.id)"
          :class="['tab-item', { active: activeTab === tab.id }]" :title="tab.name">
          <span class="tab-icon">{{ tab.icon }}</span>
          <span class="tab-name">{{ tab.name }}</span>
        </button>
      </div>
    </div>

    <!-- 加载状态 -->
    <div v-if="loading" class="loading-status">
      <div class="loading-info">
        <div class="loading-spinner"></div>
        <span class="loading-text">{{ loadingText || '加载中...' }}</span>
      </div>
    </div>

    <!-- Tab页内容区域 -->
    <!-- Author: Evilek, Date: 2025-01-08 -->
    <div class="tab-content">
      <!-- 消息生成Tab页 -->
      <div v-show="activeTab === 'message-generation'" class="tab-pane">
        <!-- 仓库信息 -->
        <div class="repo-header">
          <div class="repo-info" v-if="currentRepoPath">
            <span class="repo-name">📂 {{ getRepoName(currentRepoPath) }}</span>
            <span class="branch-info" v-if="gitStatus">
              <BranchSwitcher :current-branch="gitStatus.branch" @branch-changed="handleBranchChanged" />
              <span v-if="gitStatus.ahead > 0" class="ahead">↑{{ gitStatus.ahead }}</span>
              <span v-if="gitStatus.behind > 0" class="behind">↓{{ gitStatus.behind }}</span>
              <span v-if="isRefreshing" class="refresh-indicator" title="正在刷新Git状态">🔄</span>

              <!-- Git 快捷操作按钮 -->
              <div class="git-quick-actions">
                <button @click="quickPull" class="quick-action-btn" title="拉取当前分支" :disabled="isGitOperating">
                  {{ isGitOperating && gitOperation === 'pull' ? '⏳' : '⬇️' }}
                </button>
                <button @click="quickPush" class="quick-action-btn" title="推送当前分支" :disabled="isGitOperating">
                  {{ isGitOperating && gitOperation === 'push' ? '⏳' : '⬆️' }}
                </button>
              </div>
            </span>
          </div>

          <div class="repo-actions">
            <button @click="openRepository" class="select-repo-btn" :disabled="loading || !tauriReady">
              {{ loading ? '加载中...' : !tauriReady ? '初始化中...' : '选择仓库' }}
            </button>

            <!-- 最近仓库按钮 -->
            <div class="recent-repos-dropdown" v-if="recentRepos.length > 0">
              <button @click="toggleRecentDropdown" class="recent-dropdown-btn" :disabled="loading || !tauriReady"
                title="最近打开的仓库">
                ⋯
              </button>
              <div v-if="showRecentDropdown" class="recent-dropdown-menu">
                <div class="recent-dropdown-header">
                  <span>最近打开的仓库</span>
                  <button @click="clearRecentRepos" class="clear-recent-btn" title="清空历史">清空</button>
                </div>
                <div class="recent-repo-item" v-for="repo in recentRepos" :key="repo.path"
                  @click="openRecentRepo(repo.path)" :class="{ active: repo.path === currentRepoPath }">
                  <div class="repo-item-info">
                    <div class="repo-item-name">{{ repo.name }}</div>
                    <div class="repo-item-path">{{ repo.path }}</div>
                    <div class="repo-item-time">{{ getRepoDisplayTime(repo) }}</div>
                  </div>
                  <button @click.stop="removeRecentRepo(repo.path)" class="remove-repo-btn" title="从历史中移除">×</button>
                </div>
              </div>
            </div>
          </div>
        </div>

        <div v-if="remoteManagerVisible" class="remote-manager">
          <div class="remote-manager-header">
            <h4>����������</h4>
            <button v-if="remoteFormMode === 'edit'" type="button" class="link-btn" @click="resetRemoteForm">ȡ���༭</button>
          </div>
          <form class="remote-form" @submit.prevent="submitRemoteForm">
            <div class="form-row">
              <label>��������</label>
              <input v-model="remoteForm.name" :disabled="remoteFormMode === 'edit'" placeholder="���� origin" />
            </div>
            <div class="form-row">
              <label>Զ��URL</label>
              <input v-model="remoteForm.url" placeholder="https://..." />
            </div>
            <div class="form-actions">
              <button type="submit" class="primary" :disabled="remoteLoading">
                {{ remoteFormMode === 'add' ? '��������' : '��������' }}
              </button>
              <button type="button" class="ghost" @click="resetRemoteForm" :disabled="remoteLoading && remoteFormMode === 'add'">
                ����
              </button>
            </div>
          </form>
          <div v-if="remoteLoading" class="remote-loading">����Զ����Ϣ...</div>
          <div v-else class="remote-list">
            <p v-if="!remoteConfig || remoteConfig.remotes.length === 0" class="empty-state">��δ��⵽Զ�̿⣬���Ƚ���һ����</p>
            <div v-for="remote in remoteConfig?.remotes || []" :key="remote.name" class="remote-card">
              <div class="remote-card-header">
                <div>
                  <span class="remote-name">{{ remote.name }}</span>
                  <span v-if="remote.is_current_upstream" class="remote-tag">��ǰ����</span>
                </div>
                <div class="remote-card-actions">
                  <button type="button" class="link-btn" @click="startEditRemote(remote)">�༭</button>
                  <button type="button" class="link-btn danger" @click="removeRemote(remote.name)">ɾ��</button>
                </div>
              </div>
              <div class="remote-urls">
                <div>Fetch: {{ remote.fetch_url || 'δ����' }}</div>
                <div>Push: {{ remote.push_url || remote.fetch_url || 'δ����' }}</div>
              </div>
              <div v-if="remote.branches.length > 0" class="remote-branches">
                <div class="branch-row" v-for="branch in remote.branches" :key="branch.full_name">
                  <span class="branch-name">{{ branch.name }}</span>
                  <span v-if="branch.is_tracking_current" class="branch-tag">��ǰ��Ӧ</span>
                  <button v-else type="button" class="mini-btn" @click="setUpstream(remote.name, branch.name)" :disabled="remoteLoading">������Ӧ</button>
                </div>
              </div>
            </div>
          </div>
        </div>

        <div class="main-content">
          <!-- 暂存区 -->
          <div class="staged-files" v-if="gitStatus && gitStatus.staged_files.length > 0">
            <div class="section-title">
              <h4>📋 暂存的更改 ({{ gitStatus?.staged_files?.length || 0 }})</h4>
              <div class="section-actions">
                <button @click="toggleBatchMode" class="batch-mode-btn" :class="{ active: batchMode }" title="批量操作模式">
                  {{ batchMode ? '✅ 批量模式' : '☑️ 批量选择' }}
                </button>
                <button @click="unstageAll" class="action-btn" title="取消暂存所有">
                  ↩️
                </button>
              </div>
            </div>

            <!-- 批量操作工具栏 -->
            <div v-if="batchMode && selectedFilesCount > 0" class="batch-toolbar">
              <div class="batch-info">
                <span>已选择 {{ selectedFilesCount }} 个文件</span>
              </div>
              <div class="batch-actions">
                <button v-if="canBatchUnstage" @click="batchUnstageFiles" class="batch-btn unstage-btn"
                  :disabled="loading" title="批量取消暂存选中文件">
                  取消暂存
                </button>
                <button @click="batchRevertFiles" class="batch-btn revert-btn" :disabled="loading" title="批量回滚选中文件">
                  回滚选中
                </button>
                <button @click="selectAllStaged" class="batch-btn select-all-btn" title="全选暂存区文件">
                  全选
                </button>
                <button @click="clearSelection" class="batch-btn clear-btn" title="清空选择">
                  清空
                </button>
              </div>
            </div>

            <div class="file-list">
              <FileItem v-for="file in gitStatus?.staged_files || []" :key="file.path" :file="file" :is-staged="true"
                :batch-mode="batchMode" :selected="selectedFiles.has(file.path)" @toggle-stage="toggleStage"
                @revert="revertFile" @viewDiff="openDiffViewer" @toggle-select="toggleFileSelection"
                @refresh="refreshGitStatus" @contextMenu="handleFileContextMenu" />
            </div>
          </div>

          <!-- 提交区域 -->
          <div class="commit-area" v-if="gitStatus">
            <textarea v-model="commitMessage" placeholder="输入提交消息..." class="commit-input"
              :disabled="!hasCommittableFiles" :style="{ height: commitTextareaHeight + 'px' }"
              @input="adjustTextareaHeight" ref="commitTextarea"></textarea>

            <!-- 推理内容折叠展示区域 - Author: Evilek, Date: 2025-01-10 -->
            <div v-if="reasoningContent" class="reasoning-content-section">
              <div class="reasoning-header" @click="toggleReasoningExpanded">
                <span class="reasoning-icon">🤔</span>
                <span class="reasoning-title">AI推理过程</span>
                <span class="reasoning-toggle">{{ reasoningExpanded ? '▼' : '▶' }}</span>
              </div>
              <div v-if="reasoningExpanded" class="reasoning-content">
                <pre class="reasoning-text">{{ reasoningContent }}</pre>
              </div>
            </div>

            <!-- 优化后的水平布局按钮区域 - 节省垂直空间 -->
            <div class="commit-controls-horizontal">
              <div class="left-controls">
                <select v-model="selectedTemplate" class="template-select" title="选择提交消息模板风格">
                  <option v-for="template in availableTemplates" :key="template.id" :value="template.id"
                    :title="template.description">
                    {{ template.name }}
                  </option>
                </select>
              </div>
              <div class="right-controls">
                <button @click="generateCommitMessage" class="action-btn generate-btn"
                  :disabled="loading || !hasCommittableFiles" title="快捷键: Ctrl+G">
                  <span v-if="!isGenerating">AI生成</span>
                  <span v-else>生成中...</span>
                </button>
                <button @click="commitChanges" class="action-btn commit-btn"
                  :disabled="!commitMessage.trim() || loading || !hasCommittableFiles" title="快捷键: Ctrl+Enter">
                  提交更改
                </button>
              </div>
            </div>
            <div v-if="!hasCommittableFiles" class="commit-hint">
              <p>工作区干净，没有待提交的更改</p>
            </div>
            <div v-else-if="gitStatus && !gitStatus.staged_files.length" class="commit-hint">
              <p>暂存区为空，AI生成和提交将自动暂存所有修改的文件</p>
            </div>
            <div v-if="generationProgress" class="generation-progress">
              <div class="progress-content">
                <div class="progress-text">{{ generationProgress }}</div>
                <div v-if="isGenerating" class="progress-bar">
                  <div class="progress-fill"></div>
                </div>
              </div>
            </div>
            <!-- AI生成的提交消息预览 - 简化版本 -->
            <div v-if="commitMessage && isAIGenerated" class="message-preview">
              <div class="preview-header">
                <span class="preview-label">AI生成的提交消息</span>
                <div class="preview-actions">
                  <button @click="clearCommitMessage" class="preview-action-btn" title="清空消息">
                    清空
                  </button>
                </div>
              </div>
            </div>
          </div>

          <!-- 工作区更改 -->
          <div class="unstaged-files" v-if="gitStatus && gitStatus.unstaged_files.length > 0">
            <div class="section-title">
              <h4>📝 更改 ({{ gitStatus?.unstaged_files?.length || 0 }})</h4>
              <div class="section-actions">
                <button @click="toggleBatchMode" class="batch-mode-btn" :class="{ active: batchMode }" title="批量操作模式">
                  {{ batchMode ? '✅ 批量模式' : '☑️ 批量选择' }}
                </button>
                <button @click="stageAll" class="action-btn" title="暂存所有">
                  ➕
                </button>
              </div>
            </div>

            <!-- 批量操作工具栏 -->
            <div v-if="batchMode && selectedFilesCount > 0" class="batch-toolbar">
              <div class="batch-info">
                <span>已选择 {{ selectedFilesCount }} 个文件</span>
              </div>
              <div class="batch-actions">
                <button v-if="canBatchStage" @click="batchStageFiles" class="batch-btn stage-btn" :disabled="loading"
                  title="批量暂存选中文件">
                  暂存选中
                </button>
                <button @click="batchRevertFiles" class="batch-btn revert-btn" :disabled="loading" title="批量回滚选中文件">
                  回滚选中
                </button>
                <button @click="selectAllUnstaged" class="batch-btn select-all-btn" title="全选工作区文件">
                  全选
                </button>
                <button @click="clearSelection" class="batch-btn clear-btn" title="清空选择">
                  清空
                </button>
              </div>
            </div>

            <div class="file-list">
              <FileItem v-for="file in gitStatus?.unstaged_files || []" :key="file.path" :file="file" :is-staged="false"
                :batch-mode="batchMode" :selected="selectedFiles.has(file.path)" @toggle-stage="toggleStage"
                @revert="revertFile" @viewDiff="openDiffViewer" @toggle-select="toggleFileSelection"
                @refresh="refreshGitStatus" @contextMenu="handleFileContextMenu" />
            </div>
          </div>

          <!-- 未跟踪文件 -->
          <div class="file-section" v-if="gitStatus && gitStatus.untracked_files.length > 0">
            <div class="section-header">
              <h4>❓ 未跟踪的文件 ({{ gitStatus?.untracked_files?.length || 0 }})</h4>
              <div class="section-actions">
                <button @click="stageAllUntracked" class="action-btn" title="暂存所有">
                  ➕
                </button>
              </div>
            </div>
            <div class="file-list">
              <FileItem v-for="file in gitStatus?.untracked_files || []" :key="file.path" :file="file"
                :is-staged="false" :batch-mode="batchMode" :selected="selectedFiles.has(file.path)"
                @toggle-stage="toggleStage" @revert="revertFile" @viewDiff="openDiffViewer"
                @toggle-select="toggleFileSelection" @refresh="refreshGitStatus" @contextMenu="handleFileContextMenu" />
            </div>
          </div>

          <!-- 冲突文件 -->
          <div class="file-section" v-if="gitStatus && gitStatus.conflicted_files.length > 0">
            <div class="section-header">
              <h4>⚠️ 合并冲突 ({{ gitStatus?.conflicted_files?.length || 0 }})</h4>
            </div>
            <div class="file-list">
              <FileItem v-for="file in gitStatus?.conflicted_files || []" :key="file.path" :file="file"
                :is-staged="false" @toggle-stage="toggleStage" @revert="revertFile" @viewDiff="openDiffViewer"
                @refresh="refreshGitStatus" @contextMenu="handleFileContextMenu" />
            </div>

            <!-- 无更改状态 -->
            <div v-if="gitStatus && !gitStatus.has_changes" class="no-changes">
              <p>✨ 工作区干净，没有待提交的更改</p>
            </div>

            <!-- 提交历史 -->
            <div class="commit-history" v-if="commitHistory.length > 0">
              <div class="section-header">
                <h4>📜 提交历史</h4>
                <button @click="refreshHistory" class="action-btn">🔄</button>
              </div>
              <div class="history-list">
                <div v-for="commit in commitHistory" :key="commit.hash" class="commit-item">
                  <div class="commit-info">
                    <div class="commit-message">{{ commit.message }}</div>
                    <div class="commit-meta">
                      <span class="commit-author">{{ commit.author }}</span>
                      <span class="commit-hash">{{ commit.short_hash }}</span>
                      <span class="commit-time">{{ formatTime(commit.timestamp) }}</span>
                    </div>
                  </div>
                </div>
              </div>
            </div>


          </div>
        </div>
      </div>
    </div>

    <!-- 日报生成Tab页 -->
    <!-- Author: Evilek, Date: 2025-08-21 -->
      <div v-show="activeTab === 'gitflow'" class="tab-pane gitflow-pane">
        <GitflowDashboard />
      </div>

      <div v-show="activeTab === 'daily-report'" class="tab-pane">
      <div class="daily-report-container">
        <!-- 步骤指示器 -->
        <div class="steps-indicator">
          <div class="step-item" :class="{ active: dailyReportStep >= 1, completed: dailyReportStep > 1 }">
            <div class="step-number">1</div>
            <div class="step-label">选择仓库</div>
          </div>
          <div class="step-connector" :class="{ active: dailyReportStep > 1 }"></div>
          <div class="step-item" :class="{ active: dailyReportStep >= 2, completed: dailyReportStep > 2 }">
            <div class="step-number">2</div>
            <div class="step-label">选择日期</div>
          </div>
          <div class="step-connector" :class="{ active: dailyReportStep > 2 }"></div>
          <div class="step-item" :class="{ active: dailyReportStep >= 3, completed: dailyReportStep > 3 }">
            <div class="step-number">3</div>
            <div class="step-label">选择用户</div>
          </div>
          <div class="step-connector" :class="{ active: dailyReportStep > 3 }"></div>
          <div class="step-item" :class="{ active: dailyReportStep >= 4 }">
            <div class="step-number">4</div>
            <div class="step-label">生成报告</div>
          </div>
        </div>

        <!-- 主要内容区域 -->
        <div class="daily-report-content">
          <div class="content-layout">
            <!-- 上方：选择代码仓库区域 -->
            <div class="repo-section">
              <!-- 步骤1: 仓库选择 -->
              <div v-if="dailyReportStep === 1" class="step-content">
                <div class="step-card">
                  <div class="card-header">
                    <h3>📁 选择代码仓库</h3>
                    <p>选择需要生成日报的代码仓库，支持多选</p>
                  </div>
                  <div class="card-body">
                    <div class="repo-search">
                      <div class="search-input-wrapper">
                        <svg class="search-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor">
                          <circle cx="11" cy="11" r="8"></circle>
                          <path d="m21 21-4.35-4.35"></path>
                        </svg>
                        <input v-model="repoSearchQuery" type="text" placeholder="搜索仓库..." class="search-input" />
                      </div>
                      <button @click="selectAllRepos" class="select-all-btn" :disabled="!availableRepos.length">
                        {{ selectedRepos.length === availableRepos.length ? '取消全选' : '全选' }}
                      </button>
                    </div>

                    <div class="repo-list">
                      <div v-for="repo in filteredRepos" :key="repo.path" class="repo-item"
                        :class="{ selected: selectedRepos.includes(repo.path) }"
                        @click="toggleRepoSelection(repo.path)">
                        <div class="repo-checkbox">
                          <svg v-if="selectedRepos.includes(repo.path)" class="check-icon" viewBox="0 0 24 24"
                            fill="currentColor">
                            <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z" />
                          </svg>
                        </div>
                        <div class="repo-info">
                          <div class="repo-name">{{ repo.name }}</div>
                          <div class="repo-path">{{ repo.path }}</div>
                        </div>
                        <div class="repo-status">
                          <span class="status-badge">{{ repo.status || '就绪' }}</span>
                        </div>
                      </div>
                    </div>

                    <div v-if="!filteredRepos.length" class="empty-state">
                      <div class="empty-icon">📂</div>
                      <p>{{ repoSearchQuery ? '未找到匹配的仓库' : '暂无可用仓库' }}</p>
                    </div>
                  </div>
                  <div class="card-footer">
                    <div class="selection-summary">
                      已选择 {{ selectedRepos.length }} 个仓库
                    </div>
                    <button @click="nextStep" class="next-btn" :disabled="!selectedRepos.length">
                      下一步：选择日期
                      <svg class="arrow-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor">
                        <path d="m9 18 6-6-6-6" />
                      </svg>
                    </button>
                  </div>
                </div>
              </div>

              <!-- 步骤2: 日期选择 -->
              <div v-if="dailyReportStep === 2" class="step-content">
                <div class="step-card">
                  <div class="card-header">
                    <h3>📅 选择日期范围</h3>
                    <p>选择需要生成日报的日期范围</p>
                  </div>
                  <div class="card-body">
                    <div class="date-selection">
                      <div class="date-presets">
                        <button @click="setDatePreset('today')" class="preset-btn"
                          :class="{ active: isDatePresetActive('today') }">
                          今天
                        </button>
                        <button @click="setDatePreset('yesterday')" class="preset-btn"
                          :class="{ active: isDatePresetActive('yesterday') }">
                          昨天
                        </button>
                        <button @click="setDatePreset('thisWeek')" class="preset-btn"
                          :class="{ active: isDatePresetActive('thisWeek') }">
                          本周
                        </button>
                        <button @click="setDatePreset('lastWeek')" class="preset-btn"
                          :class="{ active: isDatePresetActive('lastWeek') }">
                          上周
                        </button>
                        <button @click="setDatePreset('thisMonth')" class="preset-btn"
                          :class="{ active: isDatePresetActive('thisMonth') }">
                          本月
                        </button>
                      </div>

                      <div class="date-inputs">
                        <div class="date-input-group">
                          <label>开始日期</label>
                          <input v-model="dateRange.start" type="date" class="date-input"
                            :max="dateRange.end || today" />
                        </div>
                        <div class="date-separator">至</div>
                        <div class="date-input-group">
                          <label>结束日期</label>
                          <input v-model="dateRange.end" type="date" class="date-input" :min="dateRange.start"
                            :max="today" />
                        </div>
                      </div>

                      <div v-if="dateRange.start && dateRange.end" class="date-summary">
                        <div class="summary-item">
                          <span class="summary-label">日期范围：</span>
                          <span class="summary-value">{{ formatDateRange() }}</span>
                        </div>
                        <div class="summary-item">
                          <span class="summary-label">天数：</span>
                          <span class="summary-value">{{ calculateDaysDiff() }} 天</span>
                        </div>
                      </div>
                    </div>
                  </div>
                  <div class="card-footer">
                    <button @click="prevStep" class="prev-btn">
                      <svg class="arrow-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor">
                        <path d="m15 18-6-6 6-6" />
                      </svg>
                      上一步
                    </button>
                    <div class="selection-summary">
                      {{ dateRange.start && dateRange.end ? '已选择日期范围' : '请选择日期范围' }}
                    </div>
                    <button @click="nextStep" class="next-btn" :disabled="!dateRange.start || !dateRange.end">
                      下一步：选择用户
                      <svg class="arrow-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor">
                        <path d="m9 18 6-6-6-6" />
                      </svg>
                    </button>
                  </div>
                </div>
              </div>

              <!-- 步骤3: 用户选择 -->
              <div v-if="dailyReportStep === 3" class="step-content">
                <div class="step-card">
                  <div class="card-header">
                    <h3>👥 选择提交用户</h3>
                    <p>从所选仓库的提交记录中选择需要生成日报的用户（可多选，留空表示所有用户）</p>
                  </div>
                  <div class="card-body">
                    <div class="loading-users" v-if="loadingUsers">
                      <div class="loading-spinner"></div>
                      <p>正在获取用户列表...</p>
                    </div>

                    <div v-else class="user-selection">
                      <div class="user-search">
                        <div class="search-input-wrapper">
                          <svg class="search-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor">
                            <circle cx="11" cy="11" r="8"></circle>
                            <path d="m21 21-4.35-4.35"></path>
                          </svg>
                          <input v-model="userSearchQuery" type="text" placeholder="搜索用户..." class="search-input" />
                        </div>
                        <div class="user-selection-actions">
                          <button @click="selectAllUsers" class="select-all-btn" :disabled="!availableUsers.length">
                            {{ selectedUsers.length === availableUsers.length ? '取消全选' : '全选' }}
                          </button>
                          <button @click="clearUserSelection" class="clear-btn" :disabled="!selectedUsers.length">
                            清空选择
                          </button>
                        </div>
                      </div>

                      <div class="user-list">
                        <div v-for="user in filteredUsers" :key="user.email" class="user-item"
                          :class="{ selected: selectedUsers.includes(user.email) }"
                          @click="toggleUserSelection(user.email)">
                          <div class="user-checkbox">
                            <svg v-if="selectedUsers.includes(user.email)" class="check-icon" viewBox="0 0 24 24"
                              fill="currentColor">
                              <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z" />
                            </svg>
                          </div>
                          <div class="user-avatar">
                            {{ user.name.charAt(0).toUpperCase() }}
                          </div>
                          <div class="user-info">
                            <div class="user-name">{{ user.name }}</div>
                            <div class="user-email">{{ user.email }}</div>
                          </div>
                          <div class="user-stats">
                            <span class="commit-count">{{ user.commitCount }} 次提交</span>
                          </div>
                        </div>
                      </div>

                      <div v-if="!filteredUsers.length" class="empty-state">
                        <div class="empty-icon">👤</div>
                        <p>{{ userSearchQuery ? '未找到匹配的用户' : '暂无用户数据' }}</p>
                      </div>
                    </div>
                  </div>
                  <div class="card-footer">
                    <button @click="prevStep" class="prev-btn">
                      <svg class="arrow-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor">
                        <path d="m15 18-6-6 6-6" />
                      </svg>
                      上一步
                    </button>
                    <div class="selection-summary">
                      已选择 {{ selectedUsers.length }} 个用户
                    </div>
                    <button @click="nextStep" class="next-btn">
                      下一步：生成报告
                      <svg class="arrow-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor">
                        <path d="m9 18 6-6-6-6" />
                      </svg>
                    </button>
                  </div>
                </div>
              </div>

              <!-- 步骤4: 生成报告 -->
              <div v-if="dailyReportStep === 4" class="step-content">
                <div class="step-card">
                  <div class="card-header">
                    <h3>📊 生成日报</h3>
                    <p>确认配置信息并生成日报</p>
                  </div>
                  <div class="card-body">
                    <div class="config-summary">
                      <div class="summary-section">
                        <h4>📁 选择的仓库 ({{ selectedRepos.length }})</h4>
                        <div class="summary-list">
                          <div v-for="repoPath in selectedRepos" :key="repoPath" class="summary-item-small">
                            {{ getRepoDisplayName(repoPath) }}
                          </div>
                        </div>
                      </div>

                      <div class="summary-section">
                        <h4>👥 选择的用户 ({{ selectedUsers.length }})</h4>
                        <div class="summary-list">
                          <div v-for="userEmail in selectedUsers" :key="userEmail" class="summary-item-small">
                            {{ getUserName(userEmail) }}
                          </div>
                        </div>
                      </div>

                      <div class="summary-section">
                        <h4>📅 日期范围</h4>
                        <div class="summary-value-large">
                          {{ formatDateRange() }} ({{ calculateDaysDiff() }} 天)
                        </div>
                      </div>
                    </div>

                    <!-- AI分析选项 -->
                    <div class="ai-option-simple">
                      <label class="ai-simple-toggle">
                        <input type="checkbox" v-model="useAIAnalysis" :disabled="generatingReport">
                        <span class="toggle-label">
                          <span class="toggle-icon">🤖</span>
                          启用AI增强分析
                          <span class="toggle-description">使用AI智能分析和汇总提交内容</span>
                        </span>
                      </label>
                    </div>

                    <div v-if="generatingReport" class="generating-state">
                      <div class="loading-spinner"></div>
                      <p>正在生成日报...</p>
                      <div class="progress-details">
                        <div class="progress-step">{{ reportProgress.currentStep }}</div>
                      </div>
                    </div>

                    <div v-if="reportGenerated" class="report-result">
                      <div class="result-header">
                        <svg class="success-icon" viewBox="0 0 24 24" fill="currentColor">
                          <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z" />
                        </svg>
                        <h4>日报生成完成</h4>
                      </div>
                      <div class="result-actions">
                        <button @click="viewReport" class="action-btn primary">
                          <svg class="btn-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor">
                            <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z" />
                            <circle cx="12" cy="12" r="3" />
                          </svg>
                          查看报告
                        </button>
                        <button @click="exportReport" class="action-btn secondary">
                          <svg class="btn-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor">
                            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
                            <polyline points="7,10 12,15 17,10" />
                            <line x1="12" y1="15" x2="12" y2="3" />
                          </svg>
                          导出报告
                        </button>
                      </div>
                    </div>
                  </div>
                  <div class="card-footer">
                    <button @click="prevStep" class="prev-btn" :disabled="generatingReport">
                      <svg class="arrow-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor">
                        <path d="m15 18-6-6 6-6" />
                      </svg>
                      上一步
                    </button>
                    <div class="selection-summary">
                      {{ generatingReport ? '正在生成...' : reportGenerated ? '生成完成' : '准备生成' }}
                    </div>
                    <button v-if="!reportGenerated" @click="generateReport" class="generate-btn"
                      :disabled="generatingReport">
                      <svg v-if="!generatingReport" class="btn-icon" viewBox="0 0 24 24" fill="none"
                        stroke="currentColor">
                        <path d="M13 2L3 14h9l-1 8 10-12h-9l1-8z" />
                      </svg>
                      <div v-else class="loading-spinner-small"></div>
                      {{ generatingReport ? '生成中...' : '开始生成' }}
                    </button>
                    <button v-else @click="resetWizard" class="reset-btn">
                      <svg class="btn-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor">
                        <path d="M3 12a9 9 0 0 1 9-9 9.75 9.75 0 0 1 6.74 2.74L21 8" />
                        <path d="M21 3v5h-5" />
                        <path d="M21 12a9 9 0 0 1-9 9 9.75 9.75 0 0 1-6.74-2.74L3 16" />
                        <path d="M3 21v-5h5" />
                      </svg>
                      重新开始
                    </button>
                  </div>
                </div>
              </div>
            </div>

            <!-- 下方：历史报告区域 -->
            <div class="history-section">
              <div class="history-card">
                <div class="history-header">
                  <h3>📋 历史报告</h3>
                  <p>查看之前生成的日报</p>
                </div>

                <div class="history-content">
                  <div v-if="historyReports.length === 0" class="history-empty">
                    <div class="empty-icon">📄</div>
                    <p>暂无历史报告</p>
                    <span class="empty-hint">生成第一份日报后将显示在这里</span>
                  </div>

                  <div v-else class="history-list">
                    <div v-for="report in historyReports" :key="report.id" class="history-item"
                      @click="viewHistoryReport(report)">
                      <div class="history-item-header">
                        <div class="history-title">{{ report.title }}</div>
                        <div class="history-date">{{ formatHistoryDate(report.createdAt) }}</div>
                      </div>
                      <div class="history-meta">
                        <span class="meta-item">
                          <svg class="meta-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor">
                            <path d="M9 19c-5 0-8-3-8-8s3-8 8-8 8 3 8 8-3 8-8 8z" />
                            <path d="M9 9h3l-3 3" />
                          </svg>
                          {{ report.repos.length }} 仓库
                        </span>
                        <span class="meta-item">
                          <svg class="meta-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor">
                            <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2" />
                            <circle cx="12" cy="7" r="4" />
                          </svg>
                          {{ report.users.length }} 用户
                        </span>
                        <span class="meta-item">
                          <svg class="meta-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor">
                            <rect x="3" y="4" width="18" height="18" rx="2" ry="2" />
                            <line x1="16" y1="2" x2="16" y2="6" />
                            <line x1="8" y1="2" x2="8" y2="6" />
                            <line x1="3" y1="10" x2="21" y2="10" />
                          </svg>
                          {{ report.dayCount }} 天
                        </span>
                      </div>
                      <div class="history-actions">
                        <button @click.stop="viewHistoryReport(report)" class="action-btn-small view">
                          <svg class="btn-icon-small" viewBox="0 0 24 24" fill="none" stroke="currentColor">
                            <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z" />
                            <circle cx="12" cy="12" r="3" />
                          </svg>
                          查看
                        </button>
                        <button @click.stop="exportHistoryReport(report)" class="action-btn-small export">
                          <svg class="btn-icon-small" viewBox="0 0 24 24" fill="none" stroke="currentColor">
                            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
                            <polyline points="7,10 12,15 17,10" />
                            <line x1="12" y1="15" x2="12" y2="3" />
                          </svg>
                          导出
                        </button>
                        <button @click.stop="deleteHistoryReport(report)" class="action-btn-small delete">
                          <svg class="btn-icon-small" viewBox="0 0 24 24" fill="none" stroke="currentColor">
                            <polyline points="3,6 5,6 21,6" />
                            <path d="m19,6v14a2,2 0 0,1-2,2H7a2,2 0 0,1-2-2V6m3,0V4a2,2 0 0,1,2-2h4a2,2 0 0,1,2,2v2" />
                          </svg>
                          删除
                        </button>
                      </div>
                    </div>
                  </div>
                </div>

                <div class="history-footer">
                  <button @click="clearAllHistory" class="clear-all-btn" :disabled="historyReports.length === 0">
                    <svg class="btn-icon-small" viewBox="0 0 24 24" fill="none" stroke="currentColor">
                      <polyline points="3,6 5,6 21,6" />
                      <path d="m19,6v14a2,2 0 0,1-2,2H7a2,2 0 0,1-2-2V6m3,0V4a2,2 0 0,1,2-2h4a2,2 0 0,1,2,2v2" />
                      <line x1="10" y1="11" x2="10" y2="17" />
                      <line x1="14" y1="11" x2="14" y2="17" />
                    </svg>
                    清空历史
                  </button>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Toast通知组件 -->
    <Toast ref="toastRef" />

    <!-- 确认对话框组件 -->
    <ConfirmDialog :visible="globalConfirm.visible.value" :options="globalConfirm.options.value"
      @confirm="globalConfirm.confirm" @cancel="globalConfirm.cancel" @close="globalConfirm.close" />

    <!-- 分层提交进度弹窗 -->
    <LayeredCommitProgress :visible="layeredProgress.visible" :session-id="layeredProgress.sessionId"
      :current-step="layeredProgress.currentStep" :total-steps="layeredProgress.totalSteps"
      :current-status="layeredProgress.currentStatus" :current-file="layeredProgress.currentFile"
      :file-summaries="layeredProgress.fileSummaries" :ai-stream-content="layeredProgress.aiStreamContent"
      @cancel="cancelLayeredCommit" />

    <!-- 调试设置弹窗 -->
    <div v-if="showDebugSettings" class="modal-overlay debug-settings-overlay" @click="closeDebugSettings">
      <div class="modal-content debug-settings-modal" @click.stop>
        <div class="modal-header">
          <h3>🛠️ 开发设置</h3>
          <button @click="closeDebugSettings" class="close-btn">×</button>
        </div>
        <div class="modal-body">
          <DebugSettings />
        </div>
      </div>
    </div>

    <!-- 全局右键菜单 -->
    <ContextMenu :visible="contextMenuVisible" :position="contextMenuPosition" :menuItems="contextMenuItems"
      @itemClick="handleContextMenuAction" @close="closeContextMenu" />

    <!-- 更新对话框 -->
    <UpdateDialog :visible="showUpdateDialog" @close="closeUpdateDialog" @updateStarted="handleUpdateStarted"
      @updateCompleted="handleUpdateCompleted" />

    <!-- 关于对话框 -->
    <AboutDialog :visible="showAboutDialog" @close="closeAboutDialog" />
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted, onUnmounted, computed, watch, nextTick } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import FileItem from './FileItem.vue'
import Toast from './Toast.vue'
import ConfirmDialog from './ConfirmDialog.vue'
import ContextMenu, { type ContextMenuItem } from './ContextMenu.vue'
import LayeredCommitProgress from './LayeredCommitProgress.vue'
import BranchSwitcher from './BranchSwitcher.vue'
import DebugSettings from './DebugSettings.vue'
import UpdateDialog from './UpdateDialog.vue'
import AboutDialog from './AboutDialog.vue'
import WindowManager from '../utils/WindowManager'
import { RecentReposManager, type RecentRepo } from '../utils/RecentRepos'
import { useToast, setToastInstance } from '../composables/useToast'
import { confirm, globalConfirm } from '../composables/useConfirm'
import GitflowDashboard from './gitflow/GitflowDashboard.vue'

// 响应式数据
const currentRepoPath = ref<string>('')

const emitRepoChangedEvent = (path: string) => {
  if (typeof window !== 'undefined') {
    window.dispatchEvent(new CustomEvent('gitflow:repo-changed', { detail: { path } }))
  }
}
const gitStatus = ref<any>(null)
const commitMessage = ref('')
const commitHistory = ref<any[]>([])
const loading = ref(false)
const loadingText = ref('')
// 批量操作相关状态
const batchMode = ref(false)
const selectedFiles = ref<Set<string>>(new Set())
const tauriReady = ref(false)
const selectedTemplate = ref('standard')
const isGenerating = ref(false)
const generationProgress = ref('')
const isAIGenerated = ref(false)
const isLayeredCommit = ref(false)
const remoteConfig = ref<RemoteConfiguration | null>(null)
const remoteManagerVisible = ref(false)
const remoteLoading = ref(false)
const remoteFormMode = ref<'add' | 'edit'>('add')
const remoteForm = reactive({ name: '', url: '', originalName: '' })
const upstreamSelection = reactive({ remote: '', branch: '' })

// 推理内容相关状态 - Author: Evilek, Date: 2025-01-10
const reasoningContent = ref<string | null>(null)
const reasoningExpanded = ref(false)
const layeredProgress = ref({
  visible: false,
  sessionId: '',
  currentStep: 0,
  totalSteps: 0,
  currentStatus: '',
  currentFile: '',
  fileSummaries: [],
  aiStreamContent: ''  // AI实时输出内容 - Author: Evilek, Date: 2025-01-10
})

// 日报生成相关状态 - Author: Evilek, Date: 2025-08-21
const dailyReportStep = ref(1)
const selectedRepos = ref<string[]>([])
const selectedUsers = ref<string[]>([])
const dateRange = ref<{ start: string; end: string }>({ start: '', end: '' })
const repoSearchQuery = ref('')
const userSearchQuery = ref('')
const availableRepos = ref<any[]>([])
const availableUsers = ref<any[]>([])
const loadingUsers = ref(false)
const generatingReport = ref(false)
const reportGenerated = ref(false)
const reportProgress = ref({ currentStep: '' })
const today = ref(new Date().toISOString().split('T')[0])
const historyReports = ref<any[]>([]) // 历史报告列表
const currentReportContent = ref('') // 当前报告内容

// AI增强分析选项
const useAIAnalysis = ref(true)

// 模板相关状态
const availableTemplates = ref<any[]>([])
const templatesLoaded = ref(false)
// 刷新状态指示
const isRefreshing = ref(false)
const refreshCount = ref(0)
// Git 操作状态
const isGitOperating = ref(false)
const gitOperation = ref<string | null>(null)

// 最近仓库相关状态
const recentRepos = ref<RecentRepo[]>([])
const showRecentDropdown = ref(false)

// 菜单状态
const showMenu = ref(false)

// 更新对话框状态
const showUpdateDialog = ref(false)

// 关于对话框状态
const showAboutDialog = ref(false)

// Tab页状态管理
// Author: Evilek
// Date: 2025-01-08
const activeTab = ref('message-generation')
const tabs = ref([
  {
    id: 'message-generation',
    name: '消息生成',
    icon: '💬'
  },
  {
    id: 'gitflow',
    name: 'Gitflow 面板',
    icon: '🔀'
  },
  {
    id: 'daily-report',
    name: '日报生成',
    icon: '📊'
  }
  // 预留其他tab页扩展空间
])
const SMART_CHECKOUT_EVENT = 'gitpanel:open-smart-checkout'


// 调试设置状态
const showDebugSettings = ref(false)

// 全局右键菜单状态
const contextMenuVisible = ref(false)
const contextMenuPosition = ref({ x: 0, y: 0 })
const contextMenuFile = ref<any>(null) // 当前右键的文件

// 提交区域高度自适应相关状态
const commitTextareaHeight = ref(60) // 默认高度约3行
const commitTextarea = ref<HTMLTextAreaElement | null>(null)

// Toast通知系统
const toast = useToast()
const toastRef = ref<InstanceType<typeof Toast> | null>(null)

// 计算属性：判断是否有可提交的文件
const hasCommittableFiles = computed(() => {
  if (!gitStatus.value) return false
  return gitStatus.value.staged_files.length > 0 ||
    gitStatus.value.unstaged_files.length > 0 ||
    gitStatus.value.untracked_files.length > 0
})

// 批量操作相关计算属性
const allFiles = computed(() => {
  if (!gitStatus.value) return []
  return [
    ...gitStatus.value.staged_files.map((f: any) => ({ ...f, isStaged: true })),
    ...gitStatus.value.unstaged_files.map((f: any) => ({ ...f, isStaged: false })),
    ...gitStatus.value.untracked_files.map((f: any) => ({ ...f, isStaged: false }))
  ]
})

const selectedFilesCount = computed(() => selectedFiles.value.size)

const canBatchStage = computed(() => {
  return Array.from(selectedFiles.value).some(filePath => {
    const file = allFiles.value.find(f => f.path === filePath)
    return file && !file.isStaged
  })
})

const canBatchUnstage = computed(() => {
  return Array.from(selectedFiles.value).some(filePath => {
    const file = allFiles.value.find(f => f.path === filePath)
    return file && file.isStaged
  })
})

// 日报生成相关计算属性 - Author: Evilek, Date: 2025-08-21
const filteredRepos = computed(() => {
  if (!repoSearchQuery.value) return availableRepos.value
  return availableRepos.value.filter(repo =>
    repo.name.toLowerCase().includes(repoSearchQuery.value.toLowerCase()) ||
    repo.path.toLowerCase().includes(repoSearchQuery.value.toLowerCase())
  )
})

const filteredUsers = computed(() => {
  if (!userSearchQuery.value) return availableUsers.value
  return availableUsers.value.filter(user =>
    user.name.toLowerCase().includes(userSearchQuery.value.toLowerCase()) ||
    user.email.toLowerCase().includes(userSearchQuery.value.toLowerCase())
  )
})

// 差异查看器已改为独立窗口，不再需要本地状态

// 加载状态管理
const setLoading = (isLoading: boolean, text = '') => {
  loading.value = isLoading
  loadingText.value = text
}

// 方法
const openRepository = async () => {
  if (!tauriReady.value) {
    toast.warning('应用正在初始化，请稍后再试', '请稍候')
    return
  }

  try {
    setLoading(true, '正在打开文件夹选择器...')

    const selectedPath = await invoke('open_folder_dialog') as string | null
    if (selectedPath) {
      setLoading(true, '正在加载仓库信息...')
      await openRepoByPath(selectedPath)
    }
    // 如果 selectedPath 为 null，说明用户取消了选择或选择的不是有效的Git仓库
    // 这种情况下不需要显示错误消息，因为后端已经处理了
  } catch (error) {
    console.error('Failed to open repository:', error)
    toast.error('打开仓库失败: ' + error, '操作失败')
  } finally {
    setLoading(false)
  }
}

// 通过路径打开仓库的通用方法
// 作者：Evilek
// 编写日期：2025-08-04
const openRepoByPath = async (path: string) => {
  try {
    setLoading(true, '正在选择仓库...')

    await clearRepositoryState()

    setLoading(true, '正在初始化仓库...')
    await invoke('select_repository', { path })

    currentRepoPath.value = path
    emitRepoChangedEvent(path)

    setLoading(true, '正在获取Git状态...')
    await refreshGitStatus(true)

    setLoading(true, '正在加载提交历史...')
    await refreshHistory()

    setLoading(true, '正在缓存配置...')
    RecentReposManager.addRecentRepo(path)
    loadRecentRepos()

    showRecentDropdown.value = false

    setLoading(true, '完成')
    setTimeout(() => setLoading(false), 500)

    await ensureRepoWatcherListener()
  } catch (error) {
    console.error('打开仓库失败:', error)
    toast.error(`打开仓库失败: ${error}`, '操作失败')
    setLoading(false)
    currentRepoPath.value = ''
    emitRepoChangedEvent('')

    if (repoWatcherDebounce) {
      clearTimeout(repoWatcherDebounce)
      repoWatcherDebounce = null
    }

    try {
      await invoke('close_repository')
    } catch (closeError) {
      console.warn('关闭仓库时出错:', closeError)
    }
  }
}


// 智能防抖刷新Git状态
const refreshGitStatus = async (force = false) => {
  const now = Date.now()

  // 如果有正在进行的刷新请求，直接返回该Promise
  if (refreshPromise && !force) {
    return refreshPromise
  }

  // 检查最小刷新间隔
  if (!force && now - lastRefreshTime < MIN_REFRESH_INTERVAL) {
    // 如果距离上次刷新时间太短，使用防抖
    if (refreshTimeout) {
      clearTimeout(refreshTimeout)
    }

    return new Promise<void>((resolve) => {
      refreshTimeout = setTimeout(async () => {
        await refreshGitStatus(true)
        resolve()
      }, REFRESH_DEBOUNCE_DELAY)
    })
  }

  // 执行实际的刷新操作
  refreshPromise = (async () => {
    try {
      isRefreshing.value = true
      refreshCount.value++
      const status = await invoke('get_git_status')
      gitStatus.value = status
      lastRefreshTime = Date.now()
    } catch (error) {
      console.error('Failed to get git status:', error)
      // 如果没有仓库打开，不显示错误提示
      if (currentRepoPath.value) {
        toast.error(`获取Git状态失败: ${error}`, '状态更新失败')
      }
      gitStatus.value = null
    } finally {
      isRefreshing.value = false
      refreshPromise = null
    }
  })()

  return refreshPromise
}

// 历史记录刷新（较少频率，不需要防抖）
let historyRefreshPromise: Promise<void> | null = null

const refreshHistory = async () => {
  // 如果有正在进行的历史刷新请求，直接返回该Promise
  if (historyRefreshPromise) {
    return historyRefreshPromise
  }

  historyRefreshPromise = (async () => {
    try {
      const history = await invoke('get_commit_history', { limit: 10 }) as any[]
      commitHistory.value = history
    } catch (error) {
      console.error('Failed to get commit history:', error)
      // 如果没有仓库打开，不显示错误提示
      if (currentRepoPath.value) {
        toast.error(`获取提交历史失败: ${error}`, '历史加载失败')
      }
      commitHistory.value = []
    } finally {
      historyRefreshPromise = null
    }
  })()

  return historyRefreshPromise
}

// 批量操作优化：收集多个操作后一次性刷新
const loadRemoteConfiguration = async () => {
  if (!currentRepoPath.value) {
    remoteConfig.value = null
    upstreamSelection.remote = ''
    upstreamSelection.branch = ''
    return
  }

  try {
    remoteLoading.value = true
    const config = await invoke('get_remote_configuration') as RemoteConfiguration
    remoteConfig.value = config

    if (config?.current_upstream) {
      const [remoteName, ...rest] = config.current_upstream.split('/')
      upstreamSelection.remote = remoteName || ''
      upstreamSelection.branch = rest.join('/') || ''
    } else {
      upstreamSelection.remote = ''
      upstreamSelection.branch = ''
    }
  } catch (error: any) {
    console.error('获取远程配置失败:', error)
    toast.error(`获取远程配置失败: ${error?.message || error}`, '操作失败')
  } finally {
    remoteLoading.value = false
  }
}

const toggleRemoteManager = () => {
  remoteManagerVisible.value = !remoteManagerVisible.value
  if (remoteManagerVisible.value) {
    void loadRemoteConfiguration()
  } else {
    resetRemoteForm()
  }
}

const resetRemoteForm = () => {
  remoteFormMode.value = 'add'
  remoteForm.name = ''
  remoteForm.url = ''
  remoteForm.originalName = ''
}

const startEditRemote = (remote: RemoteInfo) => {
  remoteManagerVisible.value = true
  remoteFormMode.value = 'edit'
  remoteForm.name = remote.name
  remoteForm.url = remote.fetch_url || remote.push_url || ''
  remoteForm.originalName = remote.name
}

const submitRemoteForm = async () => {
  const name = remoteForm.name.trim()
  const url = remoteForm.url.trim()

  if (!name || !url) {
    toast.error('请输入远程名称和地址', '信息不完整')
    return
  }

  try {
    remoteLoading.value = true

    if (remoteFormMode.value === 'add') {
      await invoke('add_remote', { name, url })
      toast.success(`远程 ${name} 已添加`, '操作完成')
    } else {
      const target = remoteForm.originalName || name
      await invoke('update_remote', { name: target, url })
      toast.success(`远程 ${target} 已更新`, '操作完成')
    }

    resetRemoteForm()
    await loadRemoteConfiguration()
  } catch (error: any) {
    console.error('保存远程失败:', error)
    toast.error(`保存远程失败: ${error?.message || error}`, '操作失败')
  } finally {
    remoteLoading.value = false
  }
}

const removeRemote = async (name: string) => {
  const confirmed = await confirm(`确定要移除远程 ${name} 吗？`, '确认操作')
  if (!confirmed) return

  try {
    remoteLoading.value = true
    await invoke('remove_remote', { name })
    toast.success(`远程 ${name} 已移除`, '操作完成')
    if (remoteFormMode.value === 'edit' && remoteForm.originalName === name) {
      resetRemoteForm()
    }
    await loadRemoteConfiguration()
  } catch (error: any) {
    console.error('移除远程失败:', error)
    toast.error(`移除远程失败: ${error?.message || error}`, '操作失败')
  } finally {
    remoteLoading.value = false
  }
}

const setUpstream = async (remote: string, branch: string) => {
  if (!gitStatus.value) {
    toast.error('当前未检测到有效分支', '操作失败')
    return
  }

  try {
    remoteLoading.value = true
    await invoke('set_branch_upstream', {
      branch: gitStatus.value.branch,
      remote,
      remoteBranch: branch,
    })
    toast.success(`已将 ${gitStatus.value.branch} 关联到 ${remote}/${branch}`, '操作完成')
    await loadRemoteConfiguration()
    await refreshGitStatus(true)
  } catch (error: any) {
    console.error('设置上游分支失败:', error)
    toast.error(`设置上游分支失败: ${error?.message || error}`, '操作失败')
  } finally {
    remoteLoading.value = false
  }
}

let pendingOperations = new Set<string>()
let operationTimeout: number | null = null
const OPERATION_BATCH_DELAY = 200 // 200ms内的操作会被批量处理

const scheduleRefresh = () => {
  if (operationTimeout) {
    clearTimeout(operationTimeout)
  }

  operationTimeout = setTimeout(async () => {
    if (pendingOperations.size > 0) {
      pendingOperations.clear()
      await refreshGitStatus()
    }
  }, OPERATION_BATCH_DELAY)
}

// Repository change listener - Updated: 2025-10-15
const GIT_STATUS_EVENT = 'git-status::dirty'
const REPO_EVENT_DEBOUNCE = 500
let repoWatcherUnlisten: (() => void) | null = null
let repoWatcherDebounce: number | null = null

const ensureRepoWatcherListener = async () => {
  if (repoWatcherUnlisten) {
    return
  }

  try {
    repoWatcherUnlisten = await listen(GIT_STATUS_EVENT, (event) => {
      const payload = (event.payload || {}) as { repository?: string; eventKind?: string }

      if (payload.repository && currentRepoPath.value && payload.repository !== currentRepoPath.value) {
        return
      }

      if (repoWatcherDebounce) {
        clearTimeout(repoWatcherDebounce)
      }

      repoWatcherDebounce = window.setTimeout(() => {
        repoWatcherDebounce = null
        refreshGitStatus(true).catch(error => {
          console.warn('自动刷新 Git 状态失败:', error)
        })
        refreshHistory().catch(error => {
          console.warn('自动刷新提交历史失败:', error)
        })
      }, REPO_EVENT_DEBOUNCE)
    })
  } catch (error) {
    console.error('注册仓库文件监听失败:', error)
  }
}

const disposeRepoWatcherListener = () => {
  if (repoWatcherUnlisten) {
    repoWatcherUnlisten()
    repoWatcherUnlisten = null
  }
  if (repoWatcherDebounce) {
    clearTimeout(repoWatcherDebounce)
    repoWatcherDebounce = null
  }
}

const toggleStage = async (filePath: string, shouldStage: boolean) => {
  try {
    const result = await invoke('stage_files', {
      request: {
        file_paths: [filePath],
        stage: shouldStage
      }
    }) as any

    // 显示操作结果信息
    if (result.details) {
      toast.warning(result.details, result.message)
    } else {
      toast.success(result.message, '操作成功')
    }

    // 添加到待处理操作集合，延迟刷新
    pendingOperations.add(filePath)
    scheduleRefresh()
  } catch (error) {
    console.error('Failed to toggle stage:', error)
    toast.error('暂存操作失败: ' + error, '操作失败')
  }
}

const stageAll = async () => {
  if (!gitStatus.value?.unstaged_files?.length) return

  try {
    const filePaths = gitStatus.value.unstaged_files.map((f: any) => f.path)
    const result = await invoke('stage_files', {
      request: { file_paths: filePaths, stage: true }
    }) as any

    // 显示操作结果信息
    if (result.details) {
      toast.warning(result.details, result.message)
    } else {
      toast.success(result.message, '操作成功')
    }

    // 批量操作直接刷新，不使用防抖
    await refreshGitStatus(true)
  } catch (error) {
    console.error('Failed to stage all:', error)
    toast.error('暂存所有文件失败: ' + error, '操作失败')
  }
}

const unstageAll = async () => {
  if (!gitStatus.value?.staged_files?.length) return

  try {
    const filePaths = gitStatus.value.staged_files.map((f: any) => f.path)
    await invoke('stage_files', {
      request: { file_paths: filePaths, stage: false }
    })

    // 批量操作直接刷新，不使用防抖
    await refreshGitStatus(true)
  } catch (error) {
    console.error('Failed to unstage all:', error)
    toast.error('取消暂存所有文件失败: ' + error, '操作失败')
  }
}

const stageAllUntracked = async () => {
  if (!gitStatus.value?.untracked_files?.length) return

  try {
    const filePaths = gitStatus.value.untracked_files.map((f: any) => f.path)
    const result = await invoke('stage_files', {
      request: { file_paths: filePaths, stage: true }
    }) as any

    // 显示操作结果信息
    if (result.details) {
      toast.warning(result.details, result.message)
    } else {
      toast.success(result.message, '操作成功')
    }

    // 批量操作直接刷新，不使用防抖
    await refreshGitStatus(true)
  } catch (error) {
    console.error('Failed to stage untracked files:', error)
    toast.error('暂存未跟踪文件失败: ' + error, '操作失败')
  }
}

// 防抖生成函数
let generateTimeout: number | null = null

// 刷新防抖和缓存机制
let refreshTimeout: number | null = null
let lastRefreshTime = 0
const REFRESH_DEBOUNCE_DELAY = 500 // 500ms防抖延迟
const MIN_REFRESH_INTERVAL = 1000 // 最小刷新间隔1秒
let refreshPromise: Promise<void> | null = null

// 文件监控自动刷新机制 - Author: Evilek, Date: 2025-01-15

const generateCommitMessage = async () => {
  if (!hasCommittableFiles.value) return

  // 防抖处理
  if (generateTimeout) {
    clearTimeout(generateTimeout)
  }

  generateTimeout = setTimeout(async () => {
    try {
      isGenerating.value = true
      loading.value = true
      generationProgress.value = '正在分析代码变更...'

      // 如果暂存区为空，先暂存所有修改的文件
      if (!gitStatus.value?.staged_files?.length) {
        generationProgress.value = '暂存区为空，正在自动暂存所有修改的文件...'

        // 暂存所有未暂存的文件
        if (gitStatus.value?.unstaged_files?.length > 0) {
          const unstagedPaths = gitStatus.value.unstaged_files.map((f: any) => f.path)
          const result = await invoke('stage_files', {
            request: { file_paths: unstagedPaths, stage: true }
          }) as any

          // 如果有跳过的文件，记录但不中断流程
          if (result.details) {
            console.warn('暂存时跳过了一些文件:', result.details)
          }
        }

        // 暂存所有未跟踪的文件
        if (gitStatus.value?.untracked_files?.length > 0) {
          const untrackedPaths = gitStatus.value.untracked_files.map((f: any) => f.path)
          const result = await invoke('stage_files', {
            request: { file_paths: untrackedPaths, stage: true }
          }) as any

          // 如果有跳过的文件，记录但不中断流程
          if (result.details) {
            console.warn('暂存时跳过了一些文件:', result.details)
          }
        }

        // 刷新Git状态（强制刷新，因为这是重要操作）
        await refreshGitStatus(true)
      }

      const filePaths = gitStatus.value?.staged_files?.map((f: any) => f.path) || []

      // 获取暂存文件的差异摘要
      generationProgress.value = '正在获取差异信息...'
      // const diffContent = await invoke('get_staged_diff_summary') as string

      // 统一使用分层提交逻辑 - 移除普通提交分支
      // Author: Evilek, Date: 2025-01-08
      generationProgress.value = '准备分层提交处理...'

      // 调试信息：检查当前选择的模板
      console.log('🔍 [GitPanel] 当前选择的模板ID:', selectedTemplate.value)
      console.log('🔍 [GitPanel] 可用模板列表:', availableTemplates.value.map(t => ({ id: t.id, name: t.name })))

      // 确保模板已加载且选择的模板存在
      if (!templatesLoaded.value || availableTemplates.value.length === 0) {
        throw new Error('模板尚未加载完成，请稍后再试')
      }

      const selectedTemplateExists = availableTemplates.value.some(t => t.id === selectedTemplate.value)
      if (!selectedTemplateExists) {
        console.warn('⚠️ [GitPanel] 选择的模板不存在，使用第一个可用模板')
        selectedTemplate.value = availableTemplates.value[0].id
      }

      // 检查单文件token限制并进行预处理
      generationProgress.value = '检查文件token限制...'
      const processedFiles = await checkAndProcessFileTokens(filePaths)

      // 统一使用分层提交（移除普通提交逻辑）
      generationProgress.value = '开始分层提交处理...'
      await executeLayeredCommit(processedFiles, gitStatus.value?.branch || 'main')

    } catch (error) {
      console.error('Failed to generate commit message:', error)
      console.log('生成提交消息失败: ' + error)
      generationProgress.value = '生成失败，请重试'
      setTimeout(() => {
        generationProgress.value = ''
      }, 2000)
    } finally {
      isGenerating.value = false
      loading.value = false
    }
  }, 300) // 300ms防抖
}

// 清空提交消息
const clearCommitMessage = () => {
  commitMessage.value = ''
  isAIGenerated.value = false
  // 同时清空推理内容 - Author: Evilek, Date: 2025-01-10
  reasoningContent.value = null
  reasoningExpanded.value = false
}

// 推理内容折叠展开切换 - Author: Evilek, Date: 2025-01-10
const toggleReasoningExpanded = () => {
  reasoningExpanded.value = !reasoningExpanded.value
}

/**
 * 清空仓库状态 - 切换仓库时重置所有相关状态
 * 作者：Evilek
 * 编写日期：2025-08-04
 */
const clearRepositoryState = async () => {
  if (repoWatcherDebounce) {
    clearTimeout(repoWatcherDebounce)
    repoWatcherDebounce = null
  }

  try {
    await invoke('close_repository')
  } catch (error) {
    console.warn('关闭仓库时出错:', error)
  }

  emitRepoChangedEvent('')

  // 重置提交状态
  commitMessage.value = ''
  isAIGenerated.value = false
  isGenerating.value = false
  generationProgress.value = ''
  reasoningContent.value = null
  reasoningExpanded.value = false

  // 重置Git状态
  gitStatus.value = null
  commitHistory.value = []

  // 重置批量操作状态
  batchMode.value = false
  selectedFiles.value.clear()

  // 重置刷新状态
  isRefreshing.value = false
  refreshCount.value = 0

  // 重置分层提交状态
  isLayeredCommit.value = false
  layeredProgress.value.visible = false

  console.log('[GitPanel] 清理当前仓库状态')
}


/**
 * 检查并处理文件token限制
 * Author: Evilek
 * Date: 2025-01-08
 * 对单文件变更和新增文件进行token检查和分割处理
 */
const checkAndProcessFileTokens = async (filePaths: string[]): Promise<string[]> => {
  try {
    generationProgress.value = '分析文件token使用情况...'

    // 调用后端检查文件token限制
    const result = await invoke('check_and_process_file_tokens', {
      filePaths: filePaths,
      template_id: selectedTemplate.value
    }) as { processedFiles: string[], needsSplit: boolean }

    if (result.needsSplit) {
      generationProgress.value = '检测到大文件，已自动分割处理...'
      console.log('🔧 [GitPanel] 文件已分割处理:', result.processedFiles)
    } else {
      generationProgress.value = '文件token检查完成...'
    }

    return result.processedFiles
  } catch (error) {
    console.warn('⚠️ [GitPanel] 文件token检查失败，使用原始文件列表:', error)
    // 如果检查失败，返回原始文件列表
    return filePaths
  }
}

/**
 * 执行分层提交
 * 作者：Evilek
 * 编写日期：2025-08-04
 */
const executeLayeredCommit = async (stagedFiles: string[], branchName: string | null) => {
  try {
    // 显示分层提交进度弹窗
    layeredProgress.value.visible = true
    loading.value = false // 关闭主加载状态
    isGenerating.value = false

    // 监听进度更新事件
    const unlisten = await listen('layered-commit-progress', (event: any) => {
      const progress = event.payload
      // 修复Vue响应式更新问题 - Author: Evilek, Date: 2025-01-09
      // 使用Object.assign避免直接替换整个对象导致的Vue内部错误
      Object.assign(layeredProgress.value, {
        visible: true,
        sessionId: progress.session_id,
        currentStep: progress.current_step,
        totalSteps: progress.total_steps,
        currentStatus: progress.status,
        currentFile: progress.current_file || '',
        fileSummaries: progress.file_summaries || [],
        aiStreamContent: progress.ai_stream_content || ''  // AI实时输出内容 - Author: Evilek, Date: 2025-01-10
      })
    })

    // 执行分层提交
    const result = await invoke('execute_layered_commit', {
      templateId: selectedTemplate.value,
      stagedFiles: stagedFiles,
      branchName: branchName || 'main'
    }) as any

    // 设置最终结果
    commitMessage.value = result.finalMessage
    isAIGenerated.value = true
    isLayeredCommit.value = true
    // 设置推理内容 - Author: Evilek, Date: 2025-01-10
    reasoningContent.value = result.reasoningContent || null
    reasoningExpanded.value = false // 默认折叠

    toast.success('分层提交消息生成成功', '操作完成')

    // 清理进度状态
    generationProgress.value = '分层提交完成！'
    setTimeout(() => {
      generationProgress.value = ''
    }, 1000)

    // 清理
    unlisten()
    layeredProgress.value.visible = false
  } catch (error) {
    layeredProgress.value.visible = false
    generationProgress.value = '分层提交失败'
    // 打印详细错误信息便于调试
    console.error('分层提交执行失败:', error)
    if (typeof error === 'string') {
      console.log('错误字符串:', error)
    } else if (error && typeof error === 'object') {
      console.log('错误对象:', JSON.stringify(error, null, 2))
      if ('code' in error) {
        console.log('错误代码:', error.code)
      }
      if ('message' in error) {
        console.log('错误消息:', error.message)
      }
    }
    setTimeout(() => {
      generationProgress.value = ''
    }, 2000)
    throw error
  }
}

/**
 * 取消分层提交
 * 作者：Evilek
 * 编写日期：2025-08-04
 * 更新日期：2025-01-09 - 添加真正的任务取消机制
 */
const cancelLayeredCommit = async () => {
  try {
    // 调用后端取消命令，真正中断任务 - Author: Evilek, Date: 2025-01-09
    await invoke('cancel_layered_commit')

    layeredProgress.value.visible = false
    loading.value = false
    isGenerating.value = false
    generationProgress.value = '分层提交已取消'
    setTimeout(() => {
      generationProgress.value = ''
    }, 1000)
    toast.info('分层提交已取消', '操作取消')
  } catch (error) {
    console.error('取消分层提交失败:', error)
    // 即使取消失败，也要关闭UI
    layeredProgress.value.visible = false
    loading.value = false
    isGenerating.value = false
    toast.warning('取消操作可能未完全生效', '操作警告')
  }
}

// 批量操作相关方法
const toggleBatchMode = () => {
  batchMode.value = !batchMode.value
  if (!batchMode.value) {
    selectedFiles.value.clear()
  }
}

const toggleFileSelection = (filePath: string) => {
  if (selectedFiles.value.has(filePath)) {
    selectedFiles.value.delete(filePath)
  } else {
    selectedFiles.value.add(filePath)
  }
}

const selectAllUnstaged = () => {
  if (!gitStatus.value) return
  gitStatus.value.unstaged_files.forEach((file: any) => {
    selectedFiles.value.add(file.path)
  })
  gitStatus.value.untracked_files.forEach((file: any) => {
    selectedFiles.value.add(file.path)
  })
}

const selectAllStaged = () => {
  if (!gitStatus.value) return
  gitStatus.value.staged_files.forEach((file: any) => {
    selectedFiles.value.add(file.path)
  })
}

const clearSelection = () => {
  selectedFiles.value.clear()
}

const batchStageFiles = async () => {
  const selectedPaths = Array.from(selectedFiles.value)
  const confirmed = await confirm.info(
    '批量暂存文件',
    `确定要暂存选中的 ${selectedPaths.length} 个文件吗？`,
    selectedPaths.join('\n')
  )

  if (!confirmed) return

  try {
    setLoading(true, '正在批量暂存文件...')
    const result = await invoke('stage_files', {
      request: { file_paths: selectedPaths, stage: true }
    }) as any

    setLoading(true, '正在刷新状态...')
    await refreshGitStatus(true)

    selectedFiles.value.clear()
    setLoading(true, '批量暂存完成')

    // 显示详细的操作结果
    if (result.details) {
      toast.warning(result.details, result.message)
    } else {
      toast.success(result.message, '操作完成')
    }

    setTimeout(() => setLoading(false), 1000)
  } catch (error) {
    console.error('Failed to batch stage files:', error)
    toast.error('批量暂存失败: ' + error, '操作失败')
    setLoading(false)
  }
}

const batchRevertFiles = async () => {
  const selectedPaths = Array.from(selectedFiles.value)
  const confirmed = await confirm.danger(
    '批量回滚文件',
    `确定要回滚选中的 ${selectedPaths.length} 个文件吗？`,
    '此操作将丢失这些文件的所有未提交更改，且无法撤销。\n\n文件列表：\n' + selectedPaths.join('\n')
  )

  if (!confirmed) return

  try {
    setLoading(true, '正在批量回滚文件...')

    // 分别处理暂存区和工作区的文件
    const stagedFiles = selectedPaths.filter(path => {
      const file = allFiles.value.find(f => f.path === path)
      return file && file.isStaged
    })

    const unstagedFiles = selectedPaths.filter(path => {
      const file = allFiles.value.find(f => f.path === path)
      return file && !file.isStaged
    })

    if (stagedFiles.length > 0) {
      await invoke('revert_files', {
        request: {
          file_paths: stagedFiles,
          revert_type: 'DiscardAll'  // 暂存区文件撤销所有更改
        }
      })
    }

    if (unstagedFiles.length > 0) {
      await invoke('revert_files', {
        request: {
          file_paths: unstagedFiles,
          revert_type: 'WorkingTree'  // 工作区文件只撤销工作区更改
        }
      })
    }

    setLoading(true, '正在刷新状态...')
    await refreshGitStatus(true)

    selectedFiles.value.clear()
    setLoading(true, '批量回滚完成')
    toast.success(`成功回滚 ${selectedPaths.length} 个文件`, '操作完成')
    setTimeout(() => setLoading(false), 1000)
  } catch (error) {
    console.error('Failed to batch revert files:', error)
    toast.error('批量回滚失败: ' + error, '操作失败')
    setLoading(false)
  }
}

const batchUnstageFiles = async () => {
  const selectedPaths = Array.from(selectedFiles.value)
  const confirmed = await confirm.info(
    '批量取消暂存文件',
    `确定要取消暂存选中的 ${selectedPaths.length} 个文件吗？`,
    selectedPaths.join('\n')
  )

  if (!confirmed) return

  try {
    setLoading(true, '正在批量取消暂存文件...')
    await invoke('stage_files', {
      request: { file_paths: selectedPaths, stage: false }
    })

    setLoading(true, '正在刷新状态...')
    await refreshGitStatus(true)

    selectedFiles.value.clear()
    setLoading(true, '批量取消暂存完成')
    toast.success(`成功取消暂存 ${selectedPaths.length} 个文件`, '操作完成')
    setTimeout(() => setLoading(false), 1000)
  } catch (error) {
    console.error('Failed to batch unstage files:', error)
    toast.error('批量取消暂存失败: ' + error, '操作失败')
    setLoading(false)
  }
}

const commitChanges = async () => {
  if (!commitMessage.value.trim() || !hasCommittableFiles.value) return

  try {
    setLoading(true, '准备提交...')

    // 如果暂存区为空，先暂存所有修改的文件
    if (!gitStatus.value.staged_files.length) {
      setLoading(true, '正在暂存文件...')

      // 暂存所有未暂存的文件
      if (gitStatus.value.unstaged_files.length > 0) {
        const unstagedPaths = gitStatus.value.unstaged_files.map((f: any) => f.path)
        const result = await invoke('stage_files', {
          request: { file_paths: unstagedPaths, stage: true }
        }) as any

        // 如果有跳过的文件，记录但不中断流程
        if (result.details) {
          console.warn('提交前暂存时跳过了一些文件:', result.details)
        }
      }

      // 暂存所有未跟踪的文件
      if (gitStatus.value.untracked_files.length > 0) {
        const untrackedPaths = gitStatus.value.untracked_files.map((f: any) => f.path)
        const result = await invoke('stage_files', {
          request: { file_paths: untrackedPaths, stage: true }
        }) as any

        // 如果有跳过的文件，记录但不中断流程
        if (result.details) {
          console.warn('提交前暂存时跳过了一些文件:', result.details)
        }
      }

      setLoading(true, '正在刷新状态...')
      // 刷新Git状态（强制刷新，因为这是重要操作）
      await refreshGitStatus(true)
    }

    setLoading(true, '正在提交更改...')
    await invoke('commit_changes', {
      request: {
        message: commitMessage.value,
        selected_files: [],
        additional_context: null,
        amend: false
      }
    })

    setLoading(true, '正在更新状态...')
    commitMessage.value = ''
    await refreshGitStatus(true)
    await refreshHistory()

    setLoading(true, '提交完成！')
    toast.success('提交成功！', '操作完成')
    setTimeout(() => setLoading(false), 1000)
  } catch (error) {
    console.error('Failed to commit:', error)
    toast.error('提交失败: ' + error, '操作失败')
    setLoading(false)
  }
}

const revertFile = async (filePath: string, isStaged: boolean) => {
  const fileName = filePath.split(/[/\\]/).pop() || filePath
  const revertType = isStaged ? '暂存区' : '工作区'

  const confirmed = await confirm.danger(
    '撤销文件更改',
    `确定要撤销${revertType}中的文件 "${fileName}" 的更改吗？`,
    '此操作将丢失该文件的所有未提交更改，且无法撤销。'
  )

  if (!confirmed) return

  try {
    setLoading(true, `正在撤销${revertType}文件更改...`)
    await invoke('revert_files', {
      request: {
        file_paths: [filePath],
        // 对于暂存区文件，撤销所有更改；对于工作区文件，只撤销工作区更改
        revert_type: isStaged ? 'DiscardAll' : 'WorkingTree'
      }
    })

    setLoading(true, '正在刷新状态...')
    await refreshGitStatus(true)

    setLoading(true, '撤销完成')
    toast.success(`${revertType}文件 ${fileName} 的更改已撤销`, '操作完成')
    setTimeout(() => setLoading(false), 1000)
  } catch (error) {
    console.error('Failed to revert file:', error)
    toast.error('撤销文件更改失败: ' + error, '操作失败')
    setLoading(false)
  }
}

// 工具函数
const getRepoName = (path: string) => {
  return path.split(/[/\\]/).pop() || path
}

const formatTime = (timestamp: number) => {
  return new Date(timestamp * 1000).toLocaleString()
}

// 最近仓库相关方法
const loadRecentRepos = () => {
  recentRepos.value = RecentReposManager.getRecentRepos()
}

const toggleRecentDropdown = () => {
  showRecentDropdown.value = !showRecentDropdown.value
}

const openRecentRepo = async (path: string) => {
  if (!tauriReady.value || loading.value) return

  try {
    loading.value = true
    await openRepoByPath(path)
  } catch (error) {
    console.error('Failed to open recent repository:', error)
    alert('打开仓库失败: ' + error)
  } finally {
    loading.value = false
  }
}

const removeRecentRepo = async (path: string) => {
  const repoName = path.split(/[/\\]/).pop() || path
  const confirmed = await confirm.warning(
    '移除仓库记录',
    `确定要从历史记录中移除 "${repoName}" 吗？`
  )

  if (confirmed) {
    RecentReposManager.removeRecentRepo(path)
    loadRecentRepos()
    toast.success('已从历史记录中移除', '操作完成')
  }
}

const clearRecentRepos = async () => {
  const confirmed = await confirm.warning(
    '清空历史记录',
    '确定要清空所有最近打开的仓库记录吗？此操作无法撤销。'
  )

  if (confirmed) {
    RecentReposManager.clearRecentRepos()
    loadRecentRepos()
    showRecentDropdown.value = false
    toast.success('历史记录已清空', '操作完成')
  }
}

const getRepoDisplayTime = (repo: RecentRepo) => {
  return RecentReposManager.getDisplayText(repo)
}

// 菜单功能切换
const toggleMenu = () => {
  showMenu.value = !showMenu.value
}

// Tab页切换方法
// Author: Evilek
// Date: 2025-01-08
const switchTab = (tabId: string) => {
  activeTab.value = tabId
  // 关闭菜单下拉框（如果打开的话）
  showMenu.value = false
}

const handleSmartCheckoutRequest = (event: Event) => {
  const detail = (event as CustomEvent<{ targetBranch?: string }>).detail
  switchTab('message-generation')
  const targetBranch = detail?.targetBranch
  const message = targetBranch
    ? `已打开 Smart Checkout，请处理当前改动后再尝试切换到 ${targetBranch}。`
    : '已打开 Smart Checkout，请处理当前改动后再尝试切换分支。'
  toast.info(message, 'Smart Checkout')
}

// 调试设置功能
const openDebugSettings = () => {
  showDebugSettings.value = true
  showMenu.value = false
}

const closeDebugSettings = () => {
  showDebugSettings.value = false
}

// 关于功能
const openAbout = () => {
  console.log('🔍 [GitPanel] 打开关于对话框')
  showAboutDialog.value = true
  showMenu.value = false
}

const closeAboutDialog = () => {
  console.log('🔍 [GitPanel] 关闭关于对话框')
  showAboutDialog.value = false
}

// 自动加载上次打开的仓库
const autoLoadLastRepo = async () => {
  const lastRepoPath = RecentReposManager.getLastOpenedRepo()
  if (lastRepoPath && tauriReady.value) {
    try {
      // 验证路径是否仍然有效
      await invoke('select_repository', { path: lastRepoPath })
      currentRepoPath.value = lastRepoPath
      await refreshGitStatus(true)
      await refreshHistory()
      console.log('自动加载上次仓库:', lastRepoPath)
    } catch (error) {
      console.warn('自动加载上次仓库失败:', error)
      // 如果加载失败，从最近列表中移除该路径
      RecentReposManager.removeRecentRepo(lastRepoPath)
      loadRecentRepos()
    }
  }
}

// 处理仓库刷新事件 Author: Evilek, Date: 2025-01-10
const handleRepositoryRefresh = async () => {
  console.log('收到仓库刷新事件，重新加载最新仓库')
  // 重新加载最近仓库列表
  loadRecentRepos()
  // 自动加载最新的仓库
  await autoLoadLastRepo()
}

// AI服务设置方法
// 作者：Evilek
// 编写日期：2025-07-25
const openAISettings = async () => {
  try {
    console.log('🤖 [GitPanel] 打开AI服务设置窗口')

    // 使用WindowManager打开AI设置窗口
    await WindowManager.openAISettings()
    console.log('✅ [GitPanel] 已打开AI服务设置窗口')
  } catch (error) {
    console.error('❌ [GitPanel] 打开AI服务设置窗口失败:', error)
    alert(`打开AI服务设置失败: ${error instanceof Error ? error.message : '未知错误'}`)
  }
}

// 检查更新方法
// 作者：Evilek
// 编写日期：2025-01-18
const checkForUpdates = () => {
  console.log('🔄 [GitPanel] 用户点击检查更新按钮')
  console.log('🔄 [GitPanel] 当前 showUpdateDialog 状态:', showUpdateDialog.value)
  console.log('🔄 [GitPanel] 设置 showUpdateDialog = true')
  showUpdateDialog.value = true
  console.log('🔄 [GitPanel] 关闭菜单')
  showMenu.value = false
  console.log('🔄 [GitPanel] 检查更新对话框应该已显示')
}

const closeUpdateDialog = () => {
  showUpdateDialog.value = false
}

const handleUpdateStarted = () => {
  console.log('📥 [GitPanel] 更新下载开始')
  toast.info('开始下载更新包...')
}

const handleUpdateCompleted = () => {
  console.log('✅ [GitPanel] 更新安装完成')
  toast.success('更新安装完成，应用将重启')
  // 这里可以添加重启应用的逻辑
}

// 加载可用模板列表
// 作者：Evilek
// 编写日期：2025-01-29
const loadAvailableTemplates = async () => {
  try {
    console.log('📝 [GitPanel] 加载可用模板列表')

    // 获取默认模板和自定义模板
    const [defaultTemplates, customTemplates] = await Promise.all([
      invoke('get_default_templates') as Promise<any[]>,
      invoke('get_custom_templates') as Promise<any[]>
    ])

    // 合并模板列表
    availableTemplates.value = [...defaultTemplates, ...customTemplates]
    templatesLoaded.value = true

    // 如果当前选择的模板不在列表中，选择第一个可用模板
    if (availableTemplates.value.length > 0) {
      const currentTemplateExists = availableTemplates.value.some(t => t.id === selectedTemplate.value)
      if (!currentTemplateExists) {
        console.log('⚠️ [GitPanel] 当前选择的模板不存在，从', selectedTemplate.value, '切换到', availableTemplates.value[0].id)
        selectedTemplate.value = availableTemplates.value[0].id
      } else {
        console.log('✅ [GitPanel] 当前选择的模板存在:', selectedTemplate.value)
      }
    }

    console.log('✅ [GitPanel] 模板列表加载完成，共', availableTemplates.value.length, '个模板')
  } catch (error) {
    console.error('❌ [GitPanel] 加载模板列表失败:', error)
    // 如果加载失败，使用默认的硬编码模板
    availableTemplates.value = [
      { id: 'standard', name: '标准提交', description: '生成符合常规规范的英文提交消息' },
      { id: 'chinese', name: '中文提交', description: '生成简洁明了的中文提交消息' },
      { id: 'detailed', name: '详细提交', description: '生成包含详细描述的提交消息' },
      { id: 'conventional', name: '约定式提交', description: '生成符合约定式提交规范的消息' }
    ]
    templatesLoaded.value = true
  }
}

// 打开模板配置窗口
// 作者：Evilek
// 编写日期：2025-01-29
const openTemplateConfig = async () => {
  try {
    console.log('📝 [GitPanel] 打开模板配置窗口')

    // 使用WindowManager打开模板配置窗口
    await WindowManager.openTemplateConfig()
    console.log('✅ [GitPanel] 已打开模板配置窗口')

    // 模板配置窗口关闭后重新加载模板列表
    // 注意：这里可能需要监听窗口关闭事件，暂时先在这里重新加载
    setTimeout(() => {
      loadAvailableTemplates()
    }, 1000)
  } catch (error) {
    console.error('❌ [GitPanel] 打开模板配置窗口失败:', error)
    alert(`打开模板配置失败: ${error instanceof Error ? error.message : '未知错误'}`)
  }
}

// 打开对话记录窗口
// 作者：Evilek
// 编写日期：2025-01-30
const openConversationHistory = async () => {
  try {
    console.log('📊 [GitPanel] 打开对话记录窗口')
    showMenu.value = false

    // 使用WindowManager打开对话记录窗口
    await WindowManager.openConversationHistory()
    console.log('✅ [GitPanel] 已打开对话记录窗口')
  } catch (error) {
    console.error('❌ [GitPanel] 打开对话记录窗口失败:', error)
    alert(`打开对话记录失败: ${error instanceof Error ? error.message : '未知错误'}`)
  }
}

// 差异查看器方法
const openDiffViewer = async (filePath: string, isStaged?: boolean) => {
  try {
    console.log(`🔍 [GitPanel] 打开差异查看器: ${filePath}, isStaged: ${isStaged}`)

    // 根据文件状态和用户点击的区域确定差异类型
    const stagedFile = gitStatus.value?.staged_files?.find((f: any) => f.path === filePath)
    const unstagedFile = gitStatus.value?.unstaged_files?.find((f: any) => f.path === filePath)

    let currentDiffType: 'WorkingTree' | 'Staged' | 'HeadToWorking' = 'HeadToWorking'

    // 如果明确指定了isStaged参数，优先使用
    if (isStaged !== undefined) {
      if (isStaged && stagedFile) {
        // 用户点击的是暂存区的文件，显示暂存区与HEAD的差异
        currentDiffType = 'Staged'
      } else if (!isStaged && unstagedFile) {
        // 用户点击的是工作区的文件，显示工作区与暂存区的差异
        currentDiffType = 'WorkingTree'
      } else {
        // 默认显示工作区与HEAD的差异
        currentDiffType = 'HeadToWorking'
      }
    } else {
      // 兼容旧的逻辑（没有isStaged参数时）
      if (stagedFile) {
        currentDiffType = 'Staged'
      } else if (unstagedFile) {
        currentDiffType = 'WorkingTree'
      } else {
        currentDiffType = 'HeadToWorking'
      }
    }

    console.log(`📋 [GitPanel] 差异类型: ${currentDiffType}`)

    // 使用WindowManager打开新窗口
    await WindowManager.openDiffViewer(filePath, currentDiffType)
    console.log(`✅ [GitPanel] 已打开差异查看器窗口: ${filePath}`)
  } catch (error) {
    console.error('❌ [GitPanel] 打开差异查看器失败:', error)
    // 可以在这里添加用户友好的错误提示
    toast.error(`打开差异查看器失败: ${error instanceof Error ? error.message : '未知错误'}`, '操作失败')
  }
}

/**
 * 调整提交消息输入框高度
 * Author: Evilek
 * Date: 2025-01-29
 * 根据内容行数自适应高度，最大10行，超过则显示滚动条
 */
const adjustTextareaHeight = () => {
  if (!commitTextarea.value) return

  const textarea = commitTextarea.value
  const lineHeight = 20 // 每行高度
  const padding = 24 // 上下padding总和
  const minHeight = lineHeight * 3 + padding // 最小3行
  const maxHeight = lineHeight * 10 + padding // 最大10行

  // 重置高度以获取正确的scrollHeight
  textarea.style.height = 'auto'

  // 计算内容高度
  const contentHeight = textarea.scrollHeight

  // 设置高度：在最小和最大高度之间
  const newHeight = Math.max(minHeight, Math.min(contentHeight, maxHeight))
  commitTextareaHeight.value = newHeight

  // 如果内容超过最大高度，启用滚动
  if (contentHeight > maxHeight) {
    textarea.style.overflowY = 'auto'
  } else {
    textarea.style.overflowY = 'hidden'
  }

  // 强制重新布局，确保父容器能够感知高度变化
  nextTick(() => {
    textarea.style.height = newHeight + 'px'
  })
}

// 快捷键处理
const handleKeydown = (event: KeyboardEvent) => {
  if (event.ctrlKey && event.key === 'g') {
    event.preventDefault()
    generateCommitMessage()
  } else if (event.ctrlKey && event.key === 'Enter') {
    event.preventDefault()
    if (commitMessage.value.trim() && hasCommittableFiles.value) {
      commitChanges()
    }
  } else if (event.key === 'Escape' && isGenerating.value) {
    // 取消生成
    if (generateTimeout) {
      clearTimeout(generateTimeout)
      generateTimeout = null
      isGenerating.value = false
      loading.value = false
      generationProgress.value = '已取消生成'
      setTimeout(() => {
        generationProgress.value = ''
      }, 1000)
    }
  }
}

// 监听提交消息变化，自动调整高度并重置AI生成标记
watch(remoteManagerVisible, value => {
  if (value) {
    void loadRemoteConfiguration()
  }
})

watch(commitMessage, (newValue, oldValue) => {
  nextTick(() => {
    adjustTextareaHeight()
  })

  // 如果用户手动修改了消息，重置AI生成标记
  if (newValue !== oldValue && isAIGenerated.value) {
    // 延迟重置，避免在AI生成时误触发
    setTimeout(() => {
      if (!isGenerating.value) {
        isAIGenerated.value = false
      }
    }, 100)
  }
})

// 监听仓库路径变化，重新启动文件监控 - Author: Evilek, Date: 2025-01-15
watch(currentRepoPath, async (newPath, oldPath) => {
  if (!tauriReady.value) return

  emitRepoChangedEvent(newPath || '')

  if (!newPath && oldPath) {
    if (repoWatcherDebounce) {
      clearTimeout(repoWatcherDebounce)
      repoWatcherDebounce = null
    }

    try {
      await invoke('close_repository')
    } catch (error) {
      console.warn('关闭仓库时出错:', error)
    }
    return
  }

  if (newPath) {
    await ensureRepoWatcherListener()
  }
})

// 生命周期
onMounted(async () => {
  // 初始化Toast实例
  if (toastRef.value) {
    setToastInstance(toastRef.value)
  }

  // 等待 Tauri 初始化
  try {
    // 测试 invoke 函数是否可用
    await new Promise(resolve => setTimeout(resolve, 100)) // 等待100ms
    if (typeof invoke === 'function') {
      tauriReady.value = true
      console.log('Tauri API 已就绪')

      // 加载最近仓库列表
      loadRecentRepos()

      // 初始化日报生成可用仓库列表
      initializeAvailableRepos()

      // 初始化历史报告数据
      initializeHistoryReports()

      // 加载可用模板列表
      await loadAvailableTemplates()

      // 自动加载上次打开的仓库
      await autoLoadLastRepo()

      await ensureRepoWatcherListener()
    } else {
      console.error('Tauri API 未正确加载')
    }
  } catch (error) {
    console.error('Tauri 初始化失败:', error)
  }

  // 添加快捷键监听
  document.addEventListener('keydown', handleKeydown)

  // Smart Checkout 请求监听
  window.addEventListener(
    SMART_CHECKOUT_EVENT,
    handleSmartCheckoutRequest as EventListener
  )

  // 监听仓库刷新事件 Author: Evilek, Date: 2025-01-10
  window.addEventListener('refreshRepository', handleRepositoryRefresh)

  // 初始化提交输入框高度
  nextTick(() => {
    adjustTextareaHeight()
  })
})

// 清理
onUnmounted(() => {
  document.removeEventListener('keydown', handleKeydown)
  // 移除 Smart Checkout 请求监听
  window.removeEventListener(
    SMART_CHECKOUT_EVENT,
    handleSmartCheckoutRequest as EventListener
  )
  // 移除仓库刷新事件监听器 Author: Evilek, Date: 2025-01-10
  window.removeEventListener('refreshRepository', handleRepositoryRefresh)

  // 清理仓库事件监听
  disposeRepoWatcherListener()

  void invoke('close_repository').catch(error => {
    console.debug('关闭仓库时出错:', error)
  })


  if (generateTimeout) {
    clearTimeout(generateTimeout)
  }
  if (refreshTimeout) {
    clearTimeout(refreshTimeout)
  }
  if (operationTimeout) {
    clearTimeout(operationTimeout)
  }
})

// 全局右键菜单相关方法
const contextMenuItems = computed((): ContextMenuItem[] => {
  if (!contextMenuFile.value) return []

  const file = contextMenuFile.value
  const items: ContextMenuItem[] = []

  // 根据文件状态显示不同的菜单项
  if (file.isStaged) {
    // 暂存区文件菜单
    items.push({
      id: 'unstage',
      text: '取消暂存',
      icon: '➖',
      action: 'unstage'
    })

    items.push({
      id: 'discardAll',
      text: '撤销所有更改',
      icon: '↩️',
      action: 'discardAll'
    })

    items.push({
      id: 'separator1',
      text: '',
      icon: '',
      action: '',
      separator: true
    })
    items.push({
      id: 'viewDiff',
      text: '查看差异',
      icon: '👁️',
      action: 'viewDiff'
    })
  } else if (file.working_tree_status === 'Untracked') {
    // 未跟踪文件菜单
    items.push({
      id: 'stage',
      text: '暂存文件',
      icon: '➕',
      action: 'stage'
    })
  } else {
    // 工作区文件菜单
    items.push({
      id: 'stage',
      text: '暂存更改',
      icon: '➕',
      action: 'stage'
    })

    items.push({
      id: 'discard',
      text: '撤销更改',
      icon: '↩️',
      action: 'discard'
    })

    items.push({
      id: 'separator1',
      text: '',
      icon: '',
      action: '',
      separator: true
    })
    items.push({
      id: 'viewDiff',
      text: '查看差异',
      icon: '👁️',
      action: 'viewDiff'
    })
  }

  // 通用操作（所有文件都可以）
  items.push({
    id: 'separator2',
    text: '',
    icon: '',
    action: '',
    separator: true
  })
  items.push({
    id: 'addToIgnore',
    text: '添加到 .gitignore',
    icon: '🚫',
    action: 'addToIgnore'
  })
  items.push({
    id: 'delete',
    text: '删除文件',
    icon: '🗑️',
    action: 'deleteFile'
  })

  return items
})

const handleFileContextMenu = (file: any, event: MouseEvent) => {
  event.preventDefault()
  contextMenuFile.value = file
  contextMenuPosition.value = { x: event.clientX, y: event.clientY }
  contextMenuVisible.value = true
}

const closeContextMenu = () => {
  contextMenuVisible.value = false
  contextMenuFile.value = null
}

// 处理分支切换事件
// 作者：Evilek
// 编写日期：2025-08-12
const handleBranchChanged = async (branchName: string) => {
  try {
    console.log(`🌿 [GitPanel] 分支已切换到: ${branchName}`)
    // 刷新Git状态以更新UI
    await refreshGitStatus(true)
    // 刷新提交历史
    await refreshHistory()
    toast.success(`已切换到分支: ${branchName}`, '分支切换成功')
  } catch (error) {
    console.error('处理分支切换事件失败:', error)
    toast.error(`处理分支切换失败: ${error}`, '操作失败')
  }
}

// Git 快捷操作方法
// 作者：Evilek
// 编写日期：2025-08-12
const quickPull = async () => {
  if (isGitOperating.value) return

  try {
    isGitOperating.value = true
    gitOperation.value = 'pull'

    const result = await invoke('pull_current_branch') as any

    if (result.success) {
      toast.success(result.message, '拉取成功')
      // 刷新Git状态以更新UI
      await refreshGitStatus(true)
    } else {
      toast.error(result.message || '拉取失败', '操作失败')
    }
  } catch (error) {
    console.error('快捷拉取失败:', error)
    toast.error(`拉取失败: ${error}`, '操作失败')
  } finally {
    isGitOperating.value = false
    gitOperation.value = null
  }
}

const quickPush = async () => {
  if (isGitOperating.value) return

  try {
    isGitOperating.value = true
    gitOperation.value = 'push'

    const result = await invoke('push_current_branch', {
      force: false
    }) as any

    if (result.success) {
      toast.success(result.message, '推送成功')
    } else {
      toast.error(result.message || '推送失败', '操作失败')
    }
  } catch (error) {
    console.error('快捷推送失败:', error)
    const errorMsg = String(error)

    // 检查是否需要强制推送
    if (errorMsg.includes('rejected') || errorMsg.includes('non-fast-forward')) {
      const confirmed = await confirm.warning(
        '推送冲突',
        '推送被拒绝，可能需要强制推送。是否强制推送？\n警告：强制推送可能会覆盖远程更改！'
      )
      if (confirmed) {
        try {
          const forceResult = await invoke('push_current_branch', {
            force: true
          }) as any

          if (forceResult.success) {
            toast.success(forceResult.message, '强制推送成功')
          } else {
            toast.error(forceResult.message || '强制推送失败', '操作失败')
          }
        } catch (forceError) {
          toast.error(`强制推送失败: ${forceError}`, '操作失败')
        }
      }
    } else {
      toast.error(`推送失败: ${error}`, '操作失败')
    }
  } finally {
    isGitOperating.value = false
    gitOperation.value = null
  }
}

const handleContextMenuAction = async (action: string) => {
  if (!contextMenuFile.value) return

  const file = contextMenuFile.value

  try {
    switch (action) {
      case 'stage':
        await toggleStage(file.path, true)
        break
      case 'unstage':
        // 取消暂存：使用 Staged 类型（只重置暂存区，保留工作区更改）
        if (await confirm.danger('取消暂存', `确定要取消暂存 ${file.path} 吗？`, '')) {
          await invoke('revert_files', {
            request: {
              file_paths: [file.path],
              revert_type: 'Staged'
            }
          })
          await refreshGitStatus()
        }
        break
      case 'discard':
        // 撤销工作区更改
        await revertFile(file.path, false)
        break
      case 'discardAll':
        // 撤销所有更改（暂存区+工作区）
        await revertFile(file.path, true)
        break
      case 'viewDiff':
        await openDiffViewer(file.path, file.isStaged)
        break
      case 'deleteFile':
        // 通用删除文件操作
        const fileType = file.working_tree_status === 'Untracked' ? '未跟踪文件' : '文件'
        if (await confirm.danger('删除文件', `确定要删除${fileType} ${file.path} 吗？`, '此操作不可撤销。')) {
          if (file.working_tree_status === 'Untracked') {
            // 未跟踪文件直接删除
            await invoke('delete_untracked_files', { filePaths: [file.path] })
          } else {
            // 已跟踪文件需要先从Git中移除再删除物理文件
            await invoke('delete_tracked_files', { filePaths: [file.path] })
          }
          await refreshGitStatus()
        }
        break
      case 'addToIgnore':
        await invoke('add_to_gitignore', { filePaths: [file.path] })
        await refreshGitStatus()
        break
    }
  } catch (error) {
    console.error('Context menu action failed:', error)
    toast.error(`操作失败: ${error}`, '操作失败')
  }

  closeContextMenu()
}

// 日报生成相关方法 - Author: Evilek, Date: 2025-08-21
const nextStep = () => {
  if (dailyReportStep.value < 4) {
    dailyReportStep.value++
    if (dailyReportStep.value === 3) {
      loadUsersFromRepos()
    }
  }
}

const prevStep = () => {
  if (dailyReportStep.value > 1) {
    dailyReportStep.value--
  }
}

const selectAllRepos = () => {
  if (selectedRepos.value.length === availableRepos.value.length) {
    selectedRepos.value = []
  } else {
    selectedRepos.value = availableRepos.value.map(repo => repo.path)
  }
}

const toggleRepoSelection = (repoPath: string) => {
  const index = selectedRepos.value.indexOf(repoPath)
  if (index > -1) {
    selectedRepos.value.splice(index, 1)
  } else {
    selectedRepos.value.push(repoPath)
  }
}

const toggleUserSelection = (userEmail: string) => {
  const index = selectedUsers.value.indexOf(userEmail)
  if (index > -1) {
    selectedUsers.value.splice(index, 1)
  } else {
    selectedUsers.value.push(userEmail)
  }
}

const selectAllUsers = () => {
  if (selectedUsers.value.length === availableUsers.value.length) {
    selectedUsers.value = []
  } else {
    selectedUsers.value = availableUsers.value.map(user => user.email)
  }
}

const clearUserSelection = () => {
  selectedUsers.value = []
}

const loadUsersFromRepos = async () => {
  if (!selectedRepos.value.length) return

  try {
    loadingUsers.value = true
    // 调用后端API获取用户列表
    const users = await invoke('get_repo_contributors', { repoPaths: selectedRepos.value }) as any[]
    availableUsers.value = users
  } catch (error) {
    console.error('Failed to load users:', error)
    toast.error('获取用户列表失败: ' + error, '操作失败')
  } finally {
    loadingUsers.value = false
  }
}

// 初始化可用仓库列表
const initializeAvailableRepos = async () => {
  try {
    // 从最近仓库列表获取路径
    const repoPaths = recentRepos.value.map(repo => repo.path)
    if (repoPaths.length > 0) {
      // 调用后端API验证仓库状态
      const repos = await invoke('get_available_repositories', { repoPaths }) as any[]
      availableRepos.value = repos
    } else {
      // 如果没有最近仓库，使用本地数据
      availableRepos.value = recentRepos.value.map(repo => ({
        name: repo.name,
        path: repo.path,
        status: '就绪'
      }))
    }
  } catch (error) {
    console.error('Failed to initialize repositories:', error)
    // 出错时使用本地数据作为备选
    availableRepos.value = recentRepos.value.map(repo => ({
      name: repo.name,
      path: repo.path,
      status: '就绪'
    }))
  }
}

// 日期相关方法 - Author: Evilek, Date: 2025-08-21
const setDatePreset = (preset: string) => {
  const today = new Date()
  const yesterday = new Date(today)
  yesterday.setDate(yesterday.getDate() - 1)

  switch (preset) {
    case 'today':
      dateRange.value.start = today.toISOString().split('T')[0]
      dateRange.value.end = today.toISOString().split('T')[0]
      break
    case 'yesterday':
      dateRange.value.start = yesterday.toISOString().split('T')[0]
      dateRange.value.end = yesterday.toISOString().split('T')[0]
      break
    case 'thisWeek': {
      const thisWeekStart = new Date(today)
      const dayOfWeek = today.getDay() || 7 // 将周日(0)转换为7
      thisWeekStart.setDate(today.getDate() - dayOfWeek + 1) // 周一
      dateRange.value.start = thisWeekStart.toISOString().split('T')[0]
      dateRange.value.end = today.toISOString().split('T')[0]
      break
    }
    case 'lastWeek': {
      const lastWeekEnd = new Date(today)
      const dayOfWeek = today.getDay() || 7 // 将周日(0)转换为7
      lastWeekEnd.setDate(today.getDate() - dayOfWeek) // 上周日
      const lastWeekStart = new Date(lastWeekEnd)
      lastWeekStart.setDate(lastWeekEnd.getDate() - 6) // 上周一
      dateRange.value.start = lastWeekStart.toISOString().split('T')[0]
      dateRange.value.end = lastWeekEnd.toISOString().split('T')[0]
      break
    }
    case 'thisMonth':
      const thisMonthStart = new Date(today.getFullYear(), today.getMonth(), 1)
      dateRange.value.start = thisMonthStart.toISOString().split('T')[0]
      dateRange.value.end = today.toISOString().split('T')[0]
      break
  }
}

const isDatePresetActive = (preset: string) => {
  const today = new Date()
  const yesterday = new Date(today)
  yesterday.setDate(yesterday.getDate() - 1)

  switch (preset) {
    case 'today':
      return dateRange.value.start === today.toISOString().split('T')[0] &&
        dateRange.value.end === today.toISOString().split('T')[0]
    case 'yesterday':
      return dateRange.value.start === yesterday.toISOString().split('T')[0] &&
        dateRange.value.end === yesterday.toISOString().split('T')[0]
    // 其他预设的判断逻辑可以后续完善
    default:
      return false
  }
}

const formatDateRange = () => {
  if (!dateRange.value.start || !dateRange.value.end) return ''
  const start = new Date(dateRange.value.start).toLocaleDateString('zh-CN')
  const end = new Date(dateRange.value.end).toLocaleDateString('zh-CN')
  return start === end ? start : `${start} - ${end}`
}

const calculateDaysDiff = () => {
  if (!dateRange.value.start || !dateRange.value.end) return 0
  const start = new Date(dateRange.value.start)
  const end = new Date(dateRange.value.end)
  const diffTime = Math.abs(end.getTime() - start.getTime())
  return Math.ceil(diffTime / (1000 * 60 * 60 * 24)) + 1
}

const getRepoDisplayName = (repoPath: string) => {
  const repo = availableRepos.value.find(r => r.path === repoPath)
  return repo ? repo.name : repoPath.split('/').pop() || repoPath
}

const getUserName = (userEmail: string) => {
  const user = availableUsers.value.find(u => u.email === userEmail)
  return user ? user.name : userEmail
}

const generateReport = async () => {
  try {
    console.log('开始生成日报...')
    console.log('选择的仓库:', selectedRepos.value)
    console.log('选择的用户:', selectedUsers.value)
    console.log('日期范围:', dateRange.value)
    console.log('AI分析选项:', {
      useAIAnalysis: useAIAnalysis.value
    })
    
    generatingReport.value = true
    reportProgress.value.currentStep = '正在分析提交记录...'

    // 构建分析配置
    const config = {
      repoPaths: selectedRepos.value,
      userEmails: selectedUsers.value, // 可以为空，表示所有用户
      startDate: dateRange.value.start,
      endDate: dateRange.value.end
    }
    
    console.log('分析配置:', config)

    // 根据AI选项选择命令
    reportProgress.value.currentStep = '正在生成报告内容...'
    
    let report: any
    if (useAIAnalysis.value) {
      console.log('调用 generate_ai_enhanced_report 命令...')
      report = await invoke('generate_ai_enhanced_report', {
        config,
        use_ai_summary: true,
        include_tech_analysis: true,
        include_risk_assessment: true,
        report_template: 'daily_summary_optimized'
      }) as any
    } else {
      console.log('调用 generate_enhanced_daily_report 命令...')
      report = await invoke('generate_enhanced_daily_report', { config }) as any
    }
    
    console.log('报告生成成功:', report)
    
    reportProgress.value.currentStep = '正在保存报告...'

    // 保存报告到历史记录
    await invoke('save_report', { report })
    
    console.log('报告已保存')

    // 更新历史报告列表
    await loadHistoryReports()
    
    console.log('历史报告列表已更新')

    // 更新当前报告内容
    currentReportContent.value = report.content
    
    reportGenerated.value = true
    toast.success('日报生成成功！', '操作成功')
  } catch (error) {
    console.error('Failed to generate report:', error)
    toast.error('生成日报失败: ' + error, '操作失败')
  } finally {
    generatingReport.value = false
  }
}

const viewReport = () => {
  // 查看报告的逻辑
  if (currentReportContent.value) {
    // 创建新窗口显示报告
    const reportWindow = window.open('', '_blank')
    if (reportWindow) {
      // 简单的 Markdown 转 HTML（仅支持基本格式）
      const htmlContent = currentReportContent.value
        .replace(/^# (.*$)/gim, '<h1>$1</h1>')
        .replace(/^## (.*$)/gim, '<h2>$1</h2>')
        .replace(/^### (.*$)/gim, '<h3>$1</h3>')
        .replace(/^\*\*(.*)\*\*/gim, '<strong>$1</strong>')
        .replace(/^\*(.*)\*/gim, '<em>$1</em>')
        .replace(/^\* (.*$)/gim, '<li>$1</li>')
        .replace(/\n/gim, '<br>')
      
      reportWindow.document.write(`
        <!DOCTYPE html>
        <html>
        <head>
          <title>开发日报</title>
          <style>
            body { 
              font-family: 'Segoe UI', Arial, sans-serif; 
              max-width: 800px; 
              margin: 0 auto; 
              padding: 20px;
              line-height: 1.6;
              color: #333;
            }
            h1, h2, h3 { color: #2c3e50; margin-top: 24px; margin-bottom: 16px; }
            h1 { font-size: 28px; border-bottom: 2px solid #eee; padding-bottom: 10px; }
            h2 { font-size: 22px; border-bottom: 1px solid #eee; padding-bottom: 8px; }
            h3 { font-size: 18px; }
            strong { color: #2c3e50; }
            li { margin-left: 20px; }
            pre { 
              background: #f5f5f5; 
              padding: 15px; 
              border-radius: 5px; 
              overflow-x: auto; 
              white-space: pre-wrap;
            }
          </style>
        </head>
        <body>
          ${htmlContent}
        </body>
        </html>
      `)
      reportWindow.document.close()
    }
  } else {
    toast.error('没有可查看的报告内容', '错误')
  }
}

const exportReport = () => {
  // 导出报告的逻辑
  if (currentReportContent.value) {
    const blob = new Blob([currentReportContent.value], { type: 'text/markdown' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = `daily-report-${dateRange.value.start}-to-${dateRange.value.end}.md`
    document.body.appendChild(a)
    a.click()
    document.body.removeChild(a)
    URL.revokeObjectURL(url)
    toast.success('报告导出成功', '操作成功')
  } else {
    toast.error('没有可导出的报告内容', '错误')
  }
}

const resetWizard = () => {
  dailyReportStep.value = 1
  selectedRepos.value = []
  selectedUsers.value = []
  dateRange.value = { start: '', end: '' }
  repoSearchQuery.value = ''
  userSearchQuery.value = ''
  availableUsers.value = []
  generatingReport.value = false
  reportGenerated.value = false
  reportProgress.value = { currentStep: '' }
}

// 历史报告相关方法 - Author: Evilek, Date: 2025-08-21
const formatHistoryDate = (dateStr: string) => {
  const date = new Date(dateStr)
  const now = new Date()
  const diffTime = now.getTime() - date.getTime()
  const diffDays = Math.floor(diffTime / (1000 * 60 * 60 * 24))

  if (diffDays === 0) {
    return '今天'
  } else if (diffDays === 1) {
    return '昨天'
  } else if (diffDays < 7) {
    return `${diffDays}天前`
  } else {
    return date.toLocaleDateString('zh-CN')
  }
}

const viewHistoryReport = (report: any) => {
  // TODO: 实现查看历史报告功能
  toast.success(`查看报告: ${report.title}`, '功能待实现')
}

const exportHistoryReport = (report: any) => {
  // TODO: 实现导出历史报告功能
  toast.success(`导出报告: ${report.title}`, '功能待实现')
}

const deleteHistoryReport = async (report: any) => {
  try {
    // 调用后端删除报告
    await invoke('delete_report', { reportId: report.id })

    // 从本地列表中移除
    const index = historyReports.value.findIndex(r => r.id === report.id)
    if (index > -1) {
      historyReports.value.splice(index, 1)
      toast.success(`已删除报告: ${report.title}`, '删除成功')
    }
  } catch (error) {
    console.error('Failed to delete report:', error)
    toast.error('删除报告失败: ' + error, '操作失败')
  }
}

const clearAllHistory = () => {
  // TODO: 实现清空所有历史报告功能
  historyReports.value = []
  toast.success('已清空所有历史报告', '清空成功')
}

// 加载历史报告数据
const loadHistoryReports = async () => {
  try {
    const reports = await invoke('get_history_reports') as any[]
    historyReports.value = reports
  } catch (error) {
    console.error('Failed to load history reports:', error)
    // 使用模拟数据作为备选
    historyReports.value = [
      {
        id: '1',
        title: '2025-08-20 开发日报',
        createdAt: '2025-08-20T18:30:00Z',
        repos: ['GitMentor', 'ProjectA'],
        users: ['Evilek', 'John'],
        dayCount: 1
      },
      {
        id: '2',
        title: '2025-08-19 周报',
        createdAt: '2025-08-19T17:45:00Z',
        repos: ['GitMentor'],
        users: ['Evilek'],
        dayCount: 7
      }
    ]
  }
}

// 初始化历史报告数据
const initializeHistoryReports = async () => {
  await loadHistoryReports()
}
</script>

<style scoped>
/* 修复层叠上下文问题 - 移除position: relative */
/* Author: Evilek, Date: 2025-08-21 */
.git-panel {
  display: flex;
  flex-direction: column;
  gap: 12px;
  /* 移除固定高度，改为根据内容自适应 - 修复暂存区为空时占用大量空间的问题 */
  min-height: 100vh;
  /* 允许内容超出视口高度时滚动 */
}

/* 简化菜单栏样式 - 移除伪元素避免层叠上下文问题 */
/* Author: Evilek, Date: 2025-08-21 */
.menu-bar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 6px 16px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  margin-bottom: 12px;
  box-shadow: 0 4px 20px rgba(102, 126, 234, 0.3);
  border-radius: 0 0 12px 12px;
  position: relative;
  z-index: 10001;
}

.menu-left .app-title {
  font-size: 16px;
  font-weight: 600;
}

/* 菜单下拉容器 - 超高z-index */
/* Author: Evilek, Date: 2025-08-21 */
.menu-dropdown {
  position: relative;
  z-index: 999998;
}

/* 菜单按钮 - 超高z-index确保可点击 */
/* Author: Evilek, Date: 2025-08-21 */
.menu-btn {
  background: none;
  border: none;
  color: white;
  font-size: 16px;
  cursor: pointer;
  padding: 8px 12px;
  border-radius: 8px;
  transition: all 0.3s ease;
  position: relative;
  z-index: 999999;
}

.menu-btn:hover {
  background: rgba(255, 255, 255, 0.15);
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
}

/* 下拉菜单内容 - 回到absolute定位，移除了所有层叠上下文问题 */
/* Author: Evilek, Date: 2025-08-21 */
.menu-dropdown-content {
  position: absolute;
  right: 0;
  top: 100%;
  background: rgba(255, 255, 255, 0.98);
  border: 1px solid rgba(226, 232, 240, 0.8);
  border-radius: 12px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.12), 0 2px 8px rgba(0, 0, 0, 0.08);
  min-width: 180px;
  z-index: 999997;
  overflow: hidden;
  animation: menuFadeIn 0.2s cubic-bezier(0.4, 0, 0.2, 1);
}

@keyframes menuFadeIn {
  from {
    opacity: 0;
    transform: translateY(-8px) scale(0.95);
  }

  to {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
}

/* 现代化菜单项样式 - Author: Evilek, Date: 2025-08-21 */
.menu-item {
  display: block;
  width: 100%;
  padding: 12px 16px;
  background: none;
  border: none;
  text-align: left;
  cursor: pointer;
  font-size: 14px;
  color: #374151;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  position: relative;
  font-weight: 500;
}

.menu-item::before {
  content: '';
  position: absolute;
  left: 0;
  top: 0;
  bottom: 0;
  width: 3px;
  background: linear-gradient(135deg, #667eea, #764ba2);
  transform: scaleY(0);
  transition: transform 0.2s ease;
}

.menu-item:hover:not(:disabled) {
  background: linear-gradient(135deg, rgba(102, 126, 234, 0.08), rgba(118, 75, 162, 0.08));
  color: #667eea;
  transform: translateX(4px);
}

.menu-item:hover:not(:disabled)::before {
  transform: scaleY(1);
}

.menu-item:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* Tab导航样式 - 移除定位避免层叠上下文问题 */
/* Author: Evilek, Date: 2025-01-08, Updated: 2025-08-21 */
.tab-navigation {
  background: linear-gradient(135deg, #f8fafc 0%, #e2e8f0 100%);
  border-bottom: 1px solid rgba(226, 232, 240, 0.8);
  padding: 0 16px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.05);
}

.tab-list {
  display: flex;
  gap: 4px;
}

/* 简化tab-item避免层叠上下文问题 */
.tab-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 14px 20px;
  background: none;
  border: none;
  border-radius: 12px 12px 0 0;
  cursor: pointer;
  font-size: 14px;
  font-weight: 600;
  color: #6b7280;
  transition: all 0.3s ease;
}

.tab-item:hover {
  background: rgba(102, 126, 234, 0.08);
  color: #4f46e5;
}

.tab-item.active {
  background: white;
  color: #4f46e5;
  border: 1px solid rgba(226, 232, 240, 0.8);
  border-bottom: 1px solid white;
  margin-bottom: -1px;
  box-shadow: 0 4px 12px rgba(102, 126, 234, 0.15);
}

.tab-icon {
  font-size: 16px;
}

.tab-name {
  font-weight: 500;
}

/* Tab内容区域 */
.tab-content {
  flex: 1;
  display: flex;
  flex-direction: column;
}

.tab-pane {
  flex: 1;
  display: flex;
  flex-direction: column;
}

.gitflow-pane {
  padding: 0 24px 32px;
  overflow-y: auto;
}

/* 施工中页面样式 */
/* Author: Evilek, Date: 2025-01-08 */
.construction-container {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 40px 20px;
  background: linear-gradient(135deg, #f8fafc 0%, #e2e8f0 100%);
}

.construction-content {
  text-align: center;
  max-width: 500px;
  background: white;
  padding: 40px 30px;
  border-radius: 16px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.1);
  border: 1px solid #e2e8f0;
}

.construction-icon {
  font-size: 4rem;
  margin-bottom: 20px;
  animation: bounce 2s infinite;
}

@keyframes bounce {

  0%,
  20%,
  50%,
  80%,
  100% {
    transform: translateY(0);
  }

  40% {
    transform: translateY(-10px);
  }

  60% {
    transform: translateY(-5px);
  }
}

.construction-title {
  color: #374151;
  font-size: 1.8rem;
  font-weight: 600;
  margin: 0 0 10px 0;
}

.construction-subtitle {
  color: #6b7280;
  font-size: 1.2rem;
  margin: 0 0 30px 0;
  font-weight: 500;
}

.construction-details {
  text-align: left;
  background: #f8fafc;
  padding: 20px;
  border-radius: 8px;
  border: 1px solid #e2e8f0;
}

.construction-details p {
  margin: 8px 0;
  color: #4b5563;
  font-size: 14px;
  line-height: 1.5;
}

/* 仓库头部 - 与commit-area宽度对齐 */
/* Author: Evilek, Date: 2025-08-21 */
.repo-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px;
  background: linear-gradient(135deg, #f8fafc 0%, #f1f5f9 100%);
  border: 1px solid rgba(226, 232, 240, 0.6);
  border-radius: 16px;
  margin: 0 16px 20px 16px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.08), 0 2px 8px rgba(0, 0, 0, 0.04);
}

/* 移除伪元素和动画避免层叠上下文问题 */

.repo-info {
  display: flex;
  align-items: center;
  gap: 16px;
  flex: 1;
}

.repo-actions {
  display: flex;
  gap: 8px;
  align-items: center;
}

/* 加载状态 */
.loading-status {
  position: absolute;
  top: 120px;
  /* 位于仓库信息下方 */
  left: 16px;
  right: 16px;
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 12px 16px;
  background: #f8fafc;
  border: 1px solid #e2e8f0;
  border-radius: 8px;
  z-index: 15;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.loading-info {
  display: flex;
  align-items: center;
  gap: 8px;
}

.loading-spinner {
  width: 14px;
  height: 14px;
  border: 2px solid #e2e8f0;
  border-top: 2px solid #667eea;
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  0% {
    transform: rotate(0deg);
  }

  100% {
    transform: rotate(360deg);
  }
}

.loading-text {
  font-size: 12px;
  color: #6b7280;
  font-weight: 500;
}



/* 选择仓库按钮 - 正常层级，低于菜单 */
/* Author: Evilek, Date: 2025-08-21 */
.select-repo-btn {
  padding: 8px 16px;
  border: none;
  border-radius: 10px;
  cursor: pointer;
  font-size: 14px;
  font-weight: 600;
  transition: all 0.3s ease;
  white-space: nowrap;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  height: 40px;
  min-width: 90px;
  box-shadow: 0 4px 15px rgba(102, 126, 234, 0.3);
  position: relative;
  z-index: 10;
}

.select-repo-btn:hover:not(:disabled) {
  background: linear-gradient(135deg, #5a67d8 0%, #6b46c1 100%);
  transform: translateY(-2px);
  box-shadow: 0 8px 25px rgba(102, 126, 234, 0.4);
}

.select-repo-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
  transform: none;
  box-shadow: none;
}

/* 仓库名称样式 */
.repo-name {
  color: #1a202c;
  font-size: 16px;
  font-weight: 600;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

/* 最近仓库下拉菜单样式 */
.recent-repos-dropdown {
  position: relative;
}

/* 最近仓库下拉按钮 - 紧凑尺寸，辅助功能 */
.recent-dropdown-btn {
  padding: 6px 8px;
  background: rgba(255, 255, 255, 0.9);
  border: 1px solid #ddd;
  border-radius: 4px;
  cursor: pointer;
  font-size: 14px;
  transition: all 0.3s ease;
  height: 32px;
  width: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.recent-dropdown-btn:hover:not(:disabled) {
  background: rgba(255, 255, 255, 1);
  transform: translateY(-1px);
}

.recent-dropdown-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.recent-dropdown-menu {
  position: absolute;
  top: 100%;
  right: 0;
  background: white;
  border: 1px solid #ddd;
  border-radius: 8px;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.15);
  z-index: 1000;
  min-width: 350px;
  max-height: 400px;
  overflow-y: auto;
  margin-top: 4px;
}

.recent-dropdown-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  border-bottom: 1px solid #eee;
  background: #f8f9fa;
  border-radius: 8px 8px 0 0;
  font-weight: 600;
  color: #333;
}

/* 清空历史按钮 - 小尺寸文本按钮 */
.clear-recent-btn {
  background: none;
  border: none;
  cursor: pointer;
  font-size: 12px;
  padding: 2px 6px;
  border-radius: 3px;
  transition: background-color 0.2s;
  color: #666;
  font-weight: 500;
}

.clear-recent-btn:hover {
  background: rgba(255, 0, 0, 0.1);
}

.recent-repo-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  cursor: pointer;
  border-bottom: 1px solid #f0f0f0;
  transition: background-color 0.2s;
}

.recent-repo-item:hover {
  background: #f8f9fa;
}

.recent-repo-item.active {
  background: rgba(102, 126, 234, 0.1);
  border-left: 3px solid #667eea;
}

.recent-repo-item:last-child {
  border-bottom: none;
}

.repo-item-info {
  flex: 1;
  min-width: 0;
}

.repo-item-name {
  font-weight: 600;
  color: #333;
  margin-bottom: 4px;
}

.repo-item-path {
  font-size: 12px;
  color: #666;
  margin-bottom: 2px;
  word-break: break-all;
}

.repo-item-time {
  font-size: 11px;
  color: #999;
}

.remove-repo-btn {
  background: none;
  border: none;
  cursor: pointer;
  font-size: 18px;
  color: #999;
  padding: 4px 8px;
  border-radius: 4px;
  transition: all 0.2s;
  margin-left: 8px;
}

.remove-repo-btn:hover {
  background: rgba(255, 0, 0, 0.1);
  color: #ff4444;
}

/* 提交操作区域样式 */
.commit-actions-row {
  margin-top: 12px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 12px;
}

.ai-generate-section {
  display: flex;
  gap: 8px;
  align-items: center;
  flex: 1;
}

.template-select {
  padding: 6px 8px;
  border: 1px solid #d1d5db;
  border-radius: 4px;
  font-size: 13px;
  background: white;
  min-width: 120px;
}

/* 统一的操作按钮样式 - 现代化渐变设计 */
/* Author: Evilek, Date: 2025-08-21 */
.action-btn {
  padding: 10px 20px;
  border: none;
  border-radius: 10px;
  cursor: pointer;
  font-size: 14px;
  font-weight: 600;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  white-space: nowrap;
  min-width: 100px;
  height: 42px;
  position: relative;
  overflow: hidden;
  box-shadow: 0 4px 15px rgba(0, 0, 0, 0.1);
}

.action-btn::before {
  content: '';
  position: absolute;
  top: 0;
  left: -100%;
  width: 100%;
  height: 100%;
  background: linear-gradient(90deg, transparent, rgba(255, 255, 255, 0.2), transparent);
  transition: left 0.5s ease;
}

.action-btn:hover:not(:disabled)::before {
  left: 100%;
}

.action-btn.generate-btn {
  background: linear-gradient(135deg, #10b981 0%, #059669 100%);
  color: white;
}

.action-btn.generate-btn:hover:not(:disabled) {
  background: linear-gradient(135deg, #059669 0%, #047857 100%);
  transform: translateY(-2px);
  box-shadow: 0 8px 25px rgba(16, 185, 129, 0.3);
}

.action-btn.commit-btn {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
}

.action-btn.commit-btn:hover:not(:disabled) {
  background: linear-gradient(135deg, #5a67d8 0%, #6b46c1 100%);
  transform: translateY(-2px);
  box-shadow: 0 8px 25px rgba(102, 126, 234, 0.3);
}

.action-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
  transform: none;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.05);
}

.branch-info {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-shrink: 0;
}

.git-quick-actions {
  display: flex;
  align-items: center;
  gap: 4px;
  margin-left: 8px;
}

.quick-action-btn {
  background: none;
  border: 1px solid var(--border-color, #e1e5e9);
  border-radius: 4px;
  cursor: pointer;
  padding: 4px 6px;
  font-size: 11px;
  color: var(--text-color, #24292f);
  transition: all 0.2s ease;
  min-width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.quick-action-btn:hover {
  background: var(--hover-bg, #f6f8fa);
  border-color: var(--border-hover, #d0d7de);
}

.quick-action-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.quick-action-btn:disabled:hover {
  background: none;
  border-color: var(--border-color, #e1e5e9);
}

.branch-name {
  font-weight: 500;
  color: #2d3748;
  font-size: 12px;
}

.ahead {
  background: #48bb78;
  color: white;
  padding: 2px 6px;
  border-radius: 12px;
  font-size: 11px;
  font-weight: 600;
}

.behind {
  background: #ed8936;
  color: white;
  padding: 2px 6px;
  border-radius: 12px;
  font-size: 11px;
  font-weight: 600;
}



/* Git状态面板 - 修复下拉菜单被裁剪问题 */
/* Author: Evilek, Date: 2025-08-21 */
.git-status-panel {
  display: flex;
  flex-direction: column;
  gap: 10px;
  flex: 1;
  overflow: visible;
}

/* 主要内容区域 - 修改为根据内容自适应高度，避免暂存区为空时占用大量空间 */
.main-content {
  display: flex;
  flex-direction: column;
  /* 移除 flex: 1，改为根据内容自适应高度 */
  gap: 16px;
  overflow-y: auto;
  /* 允许整体滚动 */
  padding: 16px;
  /* 添加内边距，让内容与边界有适当距离 */
  padding-bottom: 60px;
  /* 为绝对定位的提示信息留出空间 */
}

/* 文件区域样式 - 现代化卡片设计（移除backdrop-filter避免层叠上下文冲突） */
/* Author: Evilek, Date: 2025-08-21 */
.staged-files,
.unstaged-files,
.file-section {
  border: 1px solid rgba(226, 232, 240, 0.6);
  border-radius: 16px;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  background: rgba(255, 255, 255, 0.95);
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.06), 0 1px 4px rgba(0, 0, 0, 0.04);
  transition: all 0.3s ease;
}

.staged-files:hover,
.unstaged-files:hover,
.file-section:hover {
  box-shadow: 0 8px 30px rgba(0, 0, 0, 0.1), 0 2px 8px rgba(0, 0, 0, 0.06);
  transform: translateY(-2px);
}

/* 暂存区 - 根据内容自适应高度 */
.staged-files {
  flex: 0 1 auto;
  /* 移除 min-height，让暂存区根据内容自适应 */
  max-height: 280px;
}

/* 工作区 - 根据内容自适应高度 */
.unstaged-files {
  flex: 0 1 auto;
  /* 移除 min-height，让工作区根据内容自适应 */
  max-height: 300px;
}

/* 未跟踪文件和冲突文件 - 根据内容自适应高度 */
.file-section {
  flex: 0 1 auto;
  /* 移除 min-height，让未跟踪文件区域根据内容自适应 */
  max-height: 220px;
}

/* 现代化区域标题样式 - Author: Evilek, Date: 2025-08-21 */
.section-title,
.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
  background: linear-gradient(135deg, rgba(247, 250, 252, 0.9) 0%, rgba(241, 245, 249, 0.9) 100%);
  border-bottom: 1px solid rgba(226, 232, 240, 0.6);
  position: relative;
}

.section-title::before,
.section-header::before {
  content: '';
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  height: 1px;
  background: linear-gradient(90deg, transparent, rgba(102, 126, 234, 0.3), transparent);
}

.section-title h4,
.section-header h4 {
  margin: 0;
  font-size: 14px;
  font-weight: 600;
  color: #2d3748;
}

.section-actions {
  display: flex;
  gap: 4px;
}

.action-btn {
  padding: 4px 8px;
  background: transparent;
  border: 1px solid #cbd5e0;
  border-radius: 4px;
  cursor: pointer;
  font-size: 12px;
  transition: all 0.2s ease;
}

.action-btn:hover {
  background: #edf2f7;
  border-color: #a0aec0;
}

.file-list {
  padding: 4px;
  background: white;
  /* 移除 flex: 1，改为根据内容自适应高度 - 修复暂存区空时占用大量空间的问题 */
  overflow-y: auto;
  /* 设置最大高度约为10条文件的高度(每条约24px) + padding */
  max-height: 248px;
}

/* 提交区域 - 移除overflow避免裁剪问题 */
/* Author: Evilek, Date: 2025-08-21 */
.commit-area {
  position: relative;
  /* 为绝对定位的进度条提供定位上下文 */
  padding: 20px;
  background: linear-gradient(135deg, #f7fafc 0%, #f1f5f9 100%);
  border: 1px solid rgba(226, 232, 240, 0.6);
  border-radius: 16px;
  flex: 0 0 auto;
  /* 不参与flex空间分配，根据内容自适应 */
  min-height: 160px;
  display: flex;
  flex-direction: column;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.08), 0 2px 8px rgba(0, 0, 0, 0.04);
}

/* 移除伪元素和动画避免层叠上下文问题 */

/* 现代化提交输入框样式（移除backdrop-filter避免层叠上下文冲突） */
/* Author: Evilek, Date: 2025-08-21 */
.commit-input {
  width: 100%;
  padding: 16px;
  border: 2px solid rgba(226, 232, 240, 0.6);
  border-radius: 12px;
  font-family: inherit;
  font-size: 14px;
  line-height: 20px;
  /* 固定行高，便于计算 */
  resize: none;
  /* 禁用手动调整大小，使用自动调整 */
  margin-bottom: 16px;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  /* 高度变化动画 */
  overflow-y: hidden;
  /* 默认隐藏滚动条 */
  min-height: 60px;
  /* 最小高度约3行 */
  max-height: 224px;
  /* 最大高度约10行 */
  background: rgba(255, 255, 255, 0.95);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.04);
}

.commit-input:focus {
  outline: none;
  border-color: rgba(102, 126, 234, 0.8);
  box-shadow: 0 0 0 3px rgba(102, 126, 234, 0.1), 0 4px 12px rgba(0, 0, 0, 0.08);
  transform: translateY(-1px);
}

/* 推理内容展示样式 - Author: Evilek, Date: 2025-01-10 */
.reasoning-content-section {
  margin-bottom: 12px;
  border: 1px solid #e2e8f0;
  border-radius: 6px;
  background: #f8f9fa;
  overflow: hidden;
}

.reasoning-header {
  display: flex;
  align-items: center;
  padding: 8px 12px;
  cursor: pointer;
  background: #f1f3f4;
  border-bottom: 1px solid #e2e8f0;
  transition: background-color 0.2s ease;
}

.reasoning-header:hover {
  background: #e9ecef;
}

.reasoning-icon {
  margin-right: 8px;
  font-size: 16px;
}

.reasoning-title {
  flex: 1;
  font-size: 14px;
  font-weight: 500;
  color: #495057;
}

.reasoning-toggle {
  font-size: 12px;
  color: #6c757d;
  transition: transform 0.2s ease;
}

.reasoning-content {
  padding: 12px;
  background: #ffffff;
  border-top: 1px solid #e2e8f0;
}

.reasoning-text {
  margin: 0;
  font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
  font-size: 13px;
  line-height: 1.5;
  color: #495057;
  white-space: pre-wrap;
  word-wrap: break-word;
  max-height: 200px;
  overflow-y: auto;
  background: #f8f9fa;
  padding: 8px;
  border-radius: 4px;
  border: 1px solid #e2e8f0;
}

/* 优化后的水平布局控制区域 - 节省垂直空间 */
.commit-controls-horizontal {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-top: 8px;
  gap: 12px;
}

.left-controls {
  display: flex;
  align-items: center;
  gap: 8px;
}

.right-controls {
  display: flex;
  align-items: center;
  gap: 8px;
}

/* 保留原有样式以防兼容性问题 */
.commit-controls {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 12px;
}

.commit-textarea {
  width: 100%;
  padding: 12px;
  border: 2px solid #e2e8f0;
  border-radius: 6px;
  font-family: inherit;
  font-size: 14px;
  resize: vertical;
  min-height: 120px;
  max-height: 300px;
  flex: 1;
  transition: border-color 0.2s ease;
  overflow-y: auto;
}

.commit-textarea:focus {
  outline: none;
  border-color: #667eea;
}

.commit-actions {
  display: flex;
  gap: 8px;
  align-items: center;
}

.ai-generate-section {
  display: flex;
  gap: 8px;
  align-items: center;
}

.template-select {
  padding: 6px 8px;
  border: 1px solid #ddd;
  border-radius: 4px;
  background: white;
  font-size: 12px;
  min-width: 100px;
}

.template-select:focus {
  outline: none;
  border-color: #007acc;
}

.generation-progress {
  position: absolute;
  top: 100%;
  left: 0;
  right: 0;
  margin-top: 4px;
  padding: 8px 12px;
  background: #e3f2fd;
  border: 1px solid #2196f3;
  border-radius: 6px;
  font-size: 12px;
  color: #1976d2;
  z-index: 10;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.progress-content {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.progress-text {
  font-weight: 500;
}

.progress-bar {
  height: 4px;
  background: #bbdefb;
  border-radius: 2px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: linear-gradient(90deg, #2196f3, #1976d2);
  border-radius: 2px;
  animation: progress-fill 2s ease-in-out infinite;
}

@keyframes progress-fill {
  0% {
    width: 0%;
  }

  50% {
    width: 70%;
  }

  100% {
    width: 100%;
  }
}

/* 提交消息预览样式 */
.message-preview {
  position: absolute;
  top: 100%;
  left: 0;
  right: 0;
  margin-top: 4px;
  padding: 8px 12px;
  background: #f0f9ff;
  border: 1px solid #0ea5e9;
  border-radius: 6px;
  font-size: 12px;
  z-index: 9;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.preview-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.preview-label {
  color: #0369a1;
  font-weight: 500;
}

.preview-actions {
  display: flex;
  gap: 4px;
}

.preview-action-btn {
  background: none;
  border: none;
  padding: 4px;
  border-radius: 4px;
  cursor: pointer;
  font-size: 12px;
  color: #0369a1;
  transition: background-color 0.2s ease;
}

.preview-action-btn:hover:not(:disabled) {
  background: rgba(3, 105, 161, 0.1);
}

.preview-action-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* 重新生成按钮样式 */
.regenerate-btn {
  padding: 6px 12px;
  background: #f59e0b;
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 12px;
  font-weight: 500;
  transition: all 0.2s ease;
}

.regenerate-btn:hover:not(:disabled) {
  background: #d97706;
  transform: translateY(-1px);
}

.regenerate-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
  transform: none;
}

@keyframes pulse {
  0% {
    opacity: 1;
  }

  50% {
    opacity: 0.7;
  }

  100% {
    opacity: 1;
  }
}

/* 重复的按钮样式已移除，使用统一的 .action-btn 样式 */

.commit-hint {
  /* 移除绝对定位，改为正常文档流 - 修复挡住其他元素的问题 */
  margin-top: 8px;
  padding: 8px 12px;
  background: #fff3cd;
  border: 1px solid #ffeaa7;
  border-radius: 4px;
  color: #856404;
  font-size: 12px;
  text-align: center;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.commit-hint p {
  margin: 0;
}

.commit-textarea:disabled {
  background-color: #f8f9fa;
  color: #6c757d;
  cursor: not-allowed;
}

/* 无更改状态 */
.no-changes {
  text-align: center;
  padding: 40px 20px;
  color: #718096;
}

.no-changes p {
  margin: 0;
  font-size: 16px;
}

/* 提交历史 */
.commit-history {
  border: 1px solid #e2e8f0;
  border-radius: 8px;
  overflow: hidden;
}

.history-list {
  max-height: 300px;
  overflow-y: auto;
  background: white;
}

.commit-item {
  padding: 12px 16px;
  border-bottom: 1px solid #f1f5f9;
  transition: background-color 0.2s ease;
}

.commit-item:hover {
  background: #f8fafc;
}

.commit-item:last-child {
  border-bottom: none;
}

.commit-message {
  font-weight: 500;
  color: #2d3748;
  margin-bottom: 4px;
  line-height: 1.4;
}

.commit-meta {
  display: flex;
  gap: 12px;
  font-size: 12px;
  color: #718096;
}

.commit-author {
  font-weight: 500;
}

.commit-hash {
  font-family: 'Consolas', 'Monaco', monospace;
  background: #edf2f7;
  padding: 2px 4px;
  border-radius: 3px;
}

/* 批量操作样式 */
.batch-mode-btn {
  padding: 4px 8px;
  border: 1px solid #ddd;
  border-radius: 4px;
  background: #f8f9fa;
  color: #333;
  cursor: pointer;
  font-size: 12px;
  transition: all 0.2s ease;
}

.batch-mode-btn:hover {
  background: #e9ecef;
}

.batch-mode-btn.active {
  background: #007bff;
  color: white;
  border-color: #007bff;
}

.batch-toolbar {
  background: #f8f9fa;
  border: 1px solid #dee2e6;
  border-radius: 4px;
  padding: 8px 12px;
  margin-bottom: 8px;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.batch-info {
  font-size: 14px;
  color: #495057;
  font-weight: 500;
}

.batch-actions {
  display: flex;
  gap: 6px;
}

/* 批量操作按钮 - 较小尺寸，次要功能 */
.batch-btn {
  padding: 3px 6px;
  border: 1px solid #ddd;
  border-radius: 3px;
  background: white;
  color: #333;
  cursor: pointer;
  font-size: 11px;
  font-weight: 500;
  transition: all 0.2s ease;
  height: 24px;
  min-width: 50px;
}

.batch-btn:hover:not(:disabled) {
  background: #e9ecef;
}

.batch-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.batch-btn.stage-btn:hover:not(:disabled) {
  background: #d4edda;
  border-color: #28a745;
  color: #155724;
}

.batch-btn.unstage-btn:hover:not(:disabled) {
  background: #fff3cd;
  border-color: #ffc107;
  color: #856404;
}

.batch-btn.revert-btn:hover:not(:disabled) {
  background: #f8d7da;
  border-color: #dc3545;
  color: #721c24;
}

.batch-btn.select-all-btn:hover:not(:disabled) {
  background: #d1ecf1;
  border-color: #17a2b8;
  color: #0c5460;
}

.batch-btn.clear-btn:hover:not(:disabled) {
  background: #e2e3e5;
  border-color: #6c757d;
  color: #383d41;
}

/* 刷新状态指示器 */
.refresh-indicator {
  display: inline-block;
  animation: spin 1s linear infinite;
  margin-left: 4px;
  font-size: 12px;
}

@keyframes spin {
  from {
    transform: rotate(0deg);
  }

  to {
    transform: rotate(360deg);
  }
}

/* 深色主题支持 */
@media (prefers-color-scheme: dark) {

  .repo-header,
  .file-section,
  .commit-section,
  .commit-history {
    background: #2d3748;
    border-color: #4a5568;
  }

  .section-header {
    background: #1a202c;
    border-color: #4a5568;
  }

  .file-list,
  .history-list {
    background: #2d3748;
  }

  .commit-textarea {
    background: #2d3748;
    border-color: #4a5568;
    color: #e2e8f0;
  }

  .repo-name,
  .section-header h4,
  .commit-message {
    color: #e2e8f0;
  }

  .commit-meta {
    color: #a0aec0;
  }

  .branch-name {
    color: #e2e8f0;
  }

  .commit-item:hover {
    background: #4a5568;
  }

  .commit-hash {
    background: #4a5568;
    color: #e2e8f0;
  }

  .no-changes {
    color: #a0aec0;
  }

  .commit-hint {
    background: #2d3748;
    border-color: #4a5568;
    color: #e2e8f0;
  }

  .commit-textarea:disabled {
    background-color: #2d3748;
    color: #a0aec0;
  }
}

/* 差异查看器弹窗样式 */
.diff-viewer-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  z-index: 1000;
  display: flex;
  align-items: center;
  justify-content: center;
}

.diff-viewer-overlay>* {
  width: 90vw;
  height: 90vh;
  max-width: 1200px;
  max-height: 800px;
  border-radius: 8px;
  overflow: hidden;
  box-shadow: 0 10px 30px rgba(0, 0, 0, 0.3);
}

/* 响应式设计 */
@media (max-height: 800px) {

  /* 在较小屏幕上进一步优化区域设置 */
  .staged-files {
    max-height: 180px;
  }

  .unstaged-files {
    max-height: 200px;
  }

  .commit-area {
    min-height: 120px;
  }

  .file-section {
    max-height: 160px;
  }
}

@media (max-height: 600px) {

  /* 在很小的屏幕上进一步优化压缩 */
  .staged-files {
    max-height: 120px;
  }

  .unstaged-files {
    max-height: 140px;
  }

  .file-section {
    max-height: 100px;
  }

  .commit-area {
    min-height: 90px;
  }

  .commit-input {
    min-height: 40px;
    max-height: 80px;
  }
}

.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

/* 调试设置弹窗样式 - 必须在modal-overlay之后定义以确保优先级 */
.debug-settings-overlay {
  z-index: 9999 !important;
}

.debug-settings-modal {
  width: 90%;
  max-width: 600px;
  max-height: 80vh;
  overflow-y: auto;
}

.modal-content {
  background: var(--color-bg);
  border-radius: 8px;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.3);
  border: 1px solid var(--color-border);
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px 20px 0 20px;
  border-bottom: 1px solid var(--color-border);
  margin-bottom: 0;
}

.modal-header h3 {
  margin: 0;
  color: var(--color-text);
  font-size: 1.2rem;
}

.close-btn {
  background: none;
  border: none;
  font-size: 24px;
  cursor: pointer;
  color: var(--color-text-secondary);
  padding: 0;
  width: 30px;
  height: 30px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 4px;
  transition: all 0.2s;
}

.close-btn:hover {
  background: var(--color-bg-secondary);
  color: var(--color-text);
}

.modal-body {
  padding: 0;
}

.menu-divider {
  height: 1px;
  background: var(--color-border);
  margin: 5px 0;
}

/* 日报生成功能样式 - Author: Evilek, Date: 2025-08-21 */
.daily-report-container {
  padding: 20px;
  max-width: 1000px;
  margin: 0 auto;
}

/* 步骤指示器 */
.steps-indicator {
  display: flex;
  align-items: center;
  justify-content: center;
  margin-bottom: 32px;
  padding: 20px;
  background: linear-gradient(135deg, #f8fafc 0%, #f1f5f9 100%);
  border-radius: 12px;
  border: 1px solid #e2e8f0;
}

.step-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  opacity: 0.5;
  transition: all 0.3s ease;
}

.step-item.active {
  opacity: 1;
}

.step-item.completed {
  opacity: 1;
}

.step-number {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  background: #e2e8f0;
  color: #64748b;
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: 600;
  font-size: 14px;
  transition: all 0.3s ease;
}

.step-item.active .step-number {
  background: #3b82f6;
  color: white;
}

.step-item.completed .step-number {
  background: #10b981;
  color: white;
}

.step-label {
  font-size: 12px;
  color: #64748b;
  font-weight: 500;
  text-align: center;
}

.step-item.active .step-label {
  color: #1f2937;
}

.step-connector {
  width: 60px;
  height: 2px;
  background: #e2e8f0;
  margin: 0 16px;
  transition: all 0.3s ease;
}

.step-connector.active {
  background: #3b82f6;
}

/* 主要内容区域 */
.daily-report-content {
  min-height: 500px;
}

.content-layout {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.repo-section {
  flex: 1;
  min-width: 0;
}

.history-section {
  width: 100%;
}

.history-card {
  background: white;
  border-radius: 16px;
  border: 1px solid #e2e8f0;
  box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1);
  overflow: hidden;
}

.step-content {
  animation: fadeInUp 0.3s ease;
}

@keyframes fadeInUp {
  from {
    opacity: 0;
    transform: translateY(20px);
  }

  to {
    opacity: 1;
    transform: translateY(0);
  }
}

/* 步骤卡片 */
.step-card {
  background: white;
  border-radius: 16px;
  border: 1px solid #e2e8f0;
  box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1);
  overflow: hidden;
}

.card-header {
  padding: 24px 24px 16px;
  border-bottom: 1px solid #f1f5f9;
}

.card-header h3 {
  margin: 0 0 8px 0;
  font-size: 18px;
  font-weight: 600;
  color: #1f2937;
}

.card-header p {
  margin: 0;
  color: #6b7280;
  font-size: 14px;
}

.card-body {
  padding: 24px;
}

.card-footer {
  padding: 16px 24px;
  background: #f8fafc;
  border-top: 1px solid #f1f5f9;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
}

/* 搜索输入框 */
.repo-search,
.user-search {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 20px;
}

.search-input-wrapper {
  position: relative;
  flex: 1;
}

.search-icon {
  position: absolute;
  left: 12px;
  top: 50%;
  transform: translateY(-50%);
  width: 16px;
  height: 16px;
  color: #9ca3af;
}

.search-input {
  width: 100%;
  padding: 10px 12px 10px 40px;
  border: 1px solid #d1d5db;
  border-radius: 8px;
  font-size: 14px;
  transition: all 0.2s ease;
}

.search-input:focus {
  outline: none;
  border-color: #3b82f6;
  box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
}

.select-all-btn {
  padding: 10px 16px;
  background: #f3f4f6;
  border: 1px solid #d1d5db;
  border-radius: 8px;
  font-size: 14px;
  color: #374151;
  cursor: pointer;
  transition: all 0.2s ease;
}

.select-all-btn:hover:not(:disabled) {
  background: #e5e7eb;
}

.select-all-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* 仓库和用户列表 */
.repo-list,
.user-list {
  max-height: 300px;
  overflow-y: auto;
  border: 1px solid #e5e7eb;
  border-radius: 8px;
  background: white;
}

.repo-item,
.user-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
  border-bottom: 1px solid #f3f4f6;
  cursor: pointer;
  transition: all 0.2s ease;
}

.repo-item:last-child,
.user-item:last-child {
  border-bottom: none;
}

.repo-item:hover,
.user-item:hover {
  background: #f8fafc;
}

.repo-item.selected,
.user-item.selected {
  background: #eff6ff;
  border-color: #dbeafe;
}

.repo-checkbox,
.user-checkbox {
  width: 20px;
  height: 20px;
  border: 2px solid #d1d5db;
  border-radius: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s ease;
}

.repo-item.selected .repo-checkbox,
.user-item.selected .user-checkbox {
  background: #3b82f6;
  border-color: #3b82f6;
}

.check-icon {
  width: 12px;
  height: 12px;
  color: white;
}

.repo-info,
.user-info {
  flex: 1;
  min-width: 0;
}

.repo-name,
.user-name {
  font-weight: 500;
  color: #1f2937;
  margin-bottom: 2px;
}

.repo-path,
.user-email {
  font-size: 12px;
  color: #6b7280;
  font-family: 'SFMono-Regular', Consolas, 'Liberation Mono', Menlo, monospace;
}

.repo-status,
.user-stats {
  flex-shrink: 0;
}

.status-badge,
.commit-count {
  background: #f0fdf4;
  color: #166534;
  padding: 2px 8px;
  border-radius: 12px;
  font-size: 11px;
  font-weight: 500;
}

.user-avatar {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  background: linear-gradient(135deg, #3b82f6 0%, #2563eb 100%);
  color: white;
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: 600;
  font-size: 14px;
  flex-shrink: 0;
}

/* 空状态 */
.empty-state {
  text-align: center;
  padding: 40px 20px;
  color: #6b7280;
}

.empty-icon {
  font-size: 48px;
  margin-bottom: 16px;
}

/* 加载状态 */
.loading-users {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
  padding: 40px 20px;
  color: #6b7280;
}

.loading-spinner {
  width: 32px;
  height: 32px;
  border: 3px solid #e5e7eb;
  border-top: 3px solid #3b82f6;
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

.loading-spinner-small {
  width: 16px;
  height: 16px;
  border: 2px solid #e5e7eb;
  border-top: 2px solid #3b82f6;
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

/* 按钮样式 */
.prev-btn,
.next-btn,
.generate-btn,
.reset-btn {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 16px;
  border-radius: 8px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
  border: 1px solid transparent;
}

.prev-btn {
  background: #f3f4f6;
  color: #374151;
  border-color: #d1d5db;
}

.prev-btn:hover:not(:disabled) {
  background: #e5e7eb;
}

.next-btn,
.generate-btn {
  background: #3b82f6;
  color: white;
}

.next-btn:hover:not(:disabled),
.generate-btn:hover:not(:disabled) {
  background: #2563eb;
}

.reset-btn {
  background: #6b7280;
  color: white;
}

.reset-btn:hover {
  background: #4b5563;
}

.prev-btn:disabled,
.next-btn:disabled,
.generate-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.arrow-icon,
.btn-icon {
  width: 16px;
  height: 16px;
  flex-shrink: 0;
}

.selection-summary {
  font-size: 14px;
  color: #6b7280;
  flex: 1;
  text-align: center;
}

/* 日期选择样式 */
.date-selection {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.date-presets {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.preset-btn {
  padding: 8px 16px;
  background: #f3f4f6;
  border: 1px solid #d1d5db;
  border-radius: 6px;
  font-size: 13px;
  color: #374151;
  cursor: pointer;
  transition: all 0.2s ease;
}

.preset-btn:hover {
  background: #e5e7eb;
}

.preset-btn.active {
  background: #3b82f6;
  color: white;
  border-color: #3b82f6;
}

.date-inputs {
  display: flex;
  align-items: center;
  gap: 16px;
}

.date-input-group {
  flex: 1;
}

.date-input-group label {
  display: block;
  font-size: 13px;
  font-weight: 500;
  color: #374151;
  margin-bottom: 6px;
}

.date-input {
  width: 100%;
  padding: 10px 12px;
  border: 1px solid #d1d5db;
  border-radius: 6px;
  font-size: 14px;
  transition: all 0.2s ease;
}

.date-input:focus {
  outline: none;
  border-color: #3b82f6;
  box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
}

.date-separator {
  font-size: 14px;
  color: #6b7280;
  margin-top: 20px;
}

.date-summary {
  background: #f8fafc;
  border: 1px solid #e2e8f0;
  border-radius: 8px;
  padding: 16px;
}

.summary-item {
  display: flex;
  justify-content: space-between;
  margin-bottom: 8px;
}

.summary-item:last-child {
  margin-bottom: 0;
}

.summary-label {
  font-size: 13px;
  color: #6b7280;
}

.summary-value {
  font-size: 13px;
  font-weight: 500;
  color: #1f2937;
}

/* 配置摘要样式 */
.config-summary {
  display: flex;
  flex-direction: column;
  gap: 24px;
  margin-bottom: 24px;
}

.summary-section h4 {
  margin: 0 0 12px 0;
  font-size: 14px;
  font-weight: 600;
  color: #1f2937;
}

.summary-list {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.summary-item-small {
  background: #f3f4f6;
  color: #374151;
  padding: 4px 8px;
  border-radius: 4px;
  font-size: 12px;
}

.summary-value-large {
  font-size: 16px;
  font-weight: 500;
  color: #1f2937;
  background: #f8fafc;
  padding: 12px 16px;
  border-radius: 8px;
  border: 1px solid #e2e8f0;
}

/* 生成状态样式 */
.generating-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
  padding: 40px 20px;
  text-align: center;
}

.progress-details {
  color: #6b7280;
  font-size: 14px;
}

.progress-step {
  font-weight: 500;
}

/* 报告结果样式 */
.report-result {
  text-align: center;
  padding: 32px 20px;
}

.result-header {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  margin-bottom: 24px;
}

.success-icon {
  width: 48px;
  height: 48px;
  color: #10b981;
}

.result-header h4 {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
  color: #1f2937;
}

.result-actions {
  display: flex;
  gap: 12px;
  justify-content: center;
}

.action-btn {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px 20px;
  border-radius: 8px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
  border: 1px solid transparent;
}

.action-btn.primary {
  background: #3b82f6;
  color: white;
}

.action-btn.primary:hover {
  background: #2563eb;
}

.action-btn.secondary {
  background: #f3f4f6;
  color: #374151;
  border-color: #d1d5db;
}

.action-btn.secondary:hover {
  background: #e5e7eb;
}

.action-btn.enhanced {
  background: linear-gradient(135deg, #8b5cf6 0%, #7c3aed 100%);
  color: white;
  border: none;
}

.action-btn.enhanced:hover:not(:disabled) {
  background: linear-gradient(135deg, #7c3aed 0%, #6d28d9 100%);
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(139, 92, 246, 0.3);
}

/* 响应式设计 */
@media (max-width: 768px) {
  .daily-report-container {
    padding: 16px;
  }

  .steps-indicator {
    padding: 16px;
    margin-bottom: 24px;
  }

  .step-connector {
    width: 40px;
    margin: 0 8px;
  }

  .date-inputs {
    flex-direction: column;
    align-items: stretch;
  }

  .date-separator {
    text-align: center;
    margin: 0;
  }

  .result-actions {
    flex-direction: column;
  }

  .card-footer {
    flex-direction: column;
    gap: 12px;
  }

  .selection-summary {
    text-align: left;
  }

  .content-layout {
    flex-direction: column;
  }

  .history-section {
    order: 2;
    width: 100%;
    margin-top: 16px;
  }

  .repo-section {
    order: 1;
  }
}

/* 历史报告区域样式 - 修改为上下布局 */
.history-section {
  width: 100%;
  margin-top: 20px;
}

.history-card {
  background: white;
  border-radius: 16px;
  border: 1px solid #e2e8f0;
  box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1);
  overflow: hidden;
}

.history-header {
  padding: 20px 20px 16px;
  border-bottom: 1px solid #f1f5f9;
}

.history-header h3 {
  margin: 0 0 6px 0;
  font-size: 16px;
  font-weight: 600;
  color: #1f2937;
}

.history-header p {
  margin: 0;
  color: #6b7280;
  font-size: 13px;
}

.history-content {
  max-height: 500px;
  overflow-y: auto;
}

.history-empty {
  text-align: center;
  padding: 40px 20px;
  color: #6b7280;
}

.history-empty .empty-icon {
  font-size: 36px;
  margin-bottom: 12px;
}

.history-empty p {
  margin: 0 0 6px 0;
  font-size: 14px;
  font-weight: 500;
}

.empty-hint {
  font-size: 12px;
  color: #9ca3af;
}

.history-list {
  padding: 8px 0;
}

.history-item {
  padding: 16px 20px;
  border-bottom: 1px solid #f3f4f6;
  cursor: pointer;
  transition: all 0.2s ease;
}

.history-item:last-child {
  border-bottom: none;
}

.history-item:hover {
  background: #f8fafc;
}

.history-item-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 8px;
}

.history-title {
  font-size: 13px;
  font-weight: 500;
  color: #1f2937;
  flex: 1;
  margin-right: 8px;
}

.history-date {
  font-size: 11px;
  color: #6b7280;
  flex-shrink: 0;
}

.history-meta {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-bottom: 12px;
}

.meta-item {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 11px;
  color: #6b7280;
}

.meta-icon {
  width: 12px;
  height: 12px;
  flex-shrink: 0;
}

.history-actions {
  display: flex;
  gap: 6px;
}

.action-btn-small {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 4px 8px;
  border-radius: 4px;
  font-size: 11px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
  border: 1px solid transparent;
}

.action-btn-small.view {
  background: #eff6ff;
  color: #2563eb;
  border-color: #dbeafe;
}

.action-btn-small.view:hover {
  background: #dbeafe;
}

.action-btn-small.export {
  background: #f0fdf4;
  color: #16a34a;
  border-color: #dcfce7;
}

.action-btn-small.export:hover {
  background: #dcfce7;
}

.action-btn-small.delete {
  background: #fef2f2;
  color: #dc2626;
  border-color: #fecaca;
}

.action-btn-small.delete:hover {
  background: #fecaca;
}

.btn-icon-small {
  width: 12px;
  height: 12px;
  flex-shrink: 0;
}

.history-footer {
  padding: 16px 20px;
  border-top: 1px solid #f1f5f9;
  background: #f8fafc;
}

.clear-all-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 12px;
  background: #f3f4f6;
  border: 1px solid #d1d5db;
  border-radius: 6px;
  font-size: 12px;
  color: #374151;
  cursor: pointer;
  transition: all 0.2s ease;
  width: 100%;
  justify-content: center;
}

.clear-all-btn:hover:not(:disabled) {
  background: #e5e7eb;
}

.clear-all-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* AI分析选项样式 - 简化版 */
.ai-option-simple {
  margin: 24px 0;
  padding: 16px;
  background: #f8fafc;
  border-radius: 12px;
  border: 1px solid #e2e8f0;
}

.ai-simple-toggle {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  cursor: pointer;
}

.ai-simple-toggle input[type="checkbox"] {
  width: 20px;
  height: 20px;
  accent-color: #3b82f6;
  margin-top: 2px;
}

.ai-simple-toggle:has(input:disabled) {
  opacity: 0.6;
  cursor: not-allowed;
}

.toggle-label {
  display: flex;
  flex-direction: column;
  gap: 4px;
  flex: 1;
}

.toggle-icon {
  font-size: 20px;
}

.toggle-description {
  font-size: 14px;
  color: #64748b;
  margin-top: 2px;
}
.remote-manager {
  margin-top: 12px;
  padding: 16px;
  border: 1px solid #e5e7eb;
  border-radius: 12px;
  background: #f8fafc;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.remote-manager-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.remote-form {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
  gap: 12px 16px;
  align-items: end;
}

.remote-form .form-row {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.remote-form label {
  font-size: 12px;
  color: #6b7280;
}

.remote-form input {
  border: 1px solid #d1d5db;
  border-radius: 6px;
  padding: 8px 10px;
  font-size: 13px;
}

.remote-form .form-actions {
  display: flex;
  gap: 12px;
}

.remote-form .form-actions .primary {
  background: #2563eb;
  color: #fff;
  border: none;
  padding: 8px 14px;
  border-radius: 6px;
  cursor: pointer;
}

.remote-form .form-actions .ghost {
  background: transparent;
  border: 1px solid #cbd5f5;
  color: #2563eb;
  padding: 8px 14px;
  border-radius: 6px;
  cursor: pointer;
}

.remote-loading {
  padding: 12px;
  font-size: 13px;
  color: #6b7280;
}

.remote-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.remote-card {
  border: 1px solid #e2e8f0;
  border-radius: 10px;
  padding: 12px 16px;
  background: #fff;
}

.remote-card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.remote-card-actions {
  display: flex;
  gap: 8px;
}

.remote-card-actions .link-btn {
  font-size: 12px;
  color: #2563eb;
  background: none;
  border: none;
  cursor: pointer;
}

.remote-card-actions .danger {
  color: #dc2626;
}

.remote-name {
  font-weight: 600;
  font-size: 14px;
}

.remote-tag {
  margin-left: 8px;
  font-size: 11px;
  padding: 2px 6px;
  border-radius: 999px;
  background: #dbeafe;
  color: #1d4ed8;
}

.remote-urls {
  font-size: 12px;
  color: #4b5563;
  display: flex;
  flex-direction: column;
  gap: 2px;
  margin-bottom: 8px;
}

.remote-branches {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.branch-row {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
}

.branch-tag {
  background: #dcfce7;
  color: #15803d;
  padding: 2px 6px;
  border-radius: 999px;
  font-size: 11px;
}

.branch-name {
  min-width: 120px;
}

.mini-btn {
  border: 1px solid #2563eb;
  background: transparent;
  color: #2563eb;
  padding: 4px 8px;
  border-radius: 6px;
  font-size: 11px;
  cursor: pointer;
}

.remote-manager-btn {
  margin-left: 12px;
  padding: 8px 14px;
  border-radius: 8px;
  border: 1px solid #2563eb;
  background: transparent;
  color: #2563eb;
  cursor: pointer;
}

.remote-manager-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.link-btn {
  background: none;
  border: none;
  color: #2563eb;
  cursor: pointer;
}

.link-btn.danger {
  color: #dc2626;
}

.empty-state {
  font-size: 12px;
  color: #6b7280;
}
</style>
