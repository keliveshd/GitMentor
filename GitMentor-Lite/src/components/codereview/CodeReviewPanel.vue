<template>
  <div class="code-review-panel">
    <!-- 页面标题 -->
    <div class="panel-header">
      <h2>
        <el-icon><Document /></el-icon>
        Code Review
      </h2>
      <div class="header-actions">
        <el-button
          type="primary"
          :loading="loading"
          :disabled="(fileFilter === 'commits' && selectedCommits.length === 0) || (fileFilter !== 'commits' && selectedFiles.length === 0)"
          @click="handleStartReview"
        >
          <el-icon><MagicStick /></el-icon>
          {{ fileFilter === 'commits' ? '开始审查提交' : '开始 AI 审查' }}
        </el-button>
        <el-button @click="showHistory">
          <el-icon><Clock /></el-icon>
          历史记录
        </el-button>
        <el-button @click="showSettings">
          <el-icon><Setting /></el-icon>
          设置
        </el-button>
      </div>
    </div>

    <!-- 当前仓库信息 -->
    <div v-if="currentRepo" class="repo-info">
      <el-card class="repo-card">
        <div class="repo-details">
          <div class="repo-name">
            <el-icon><FolderOpened /></el-icon>
            {{ getRepoName(currentRepo) }}
          </div>
          <div class="repo-stats" v-if="gitStatus">
            <el-tag type="info">
              <el-icon><Collection /></el-icon>
              {{ gitStatus.branch }}
            </el-tag>
            <el-tag v-if="gitStatus.ahead > 0" type="success">
              ahead {{ gitStatus.ahead }}
            </el-tag>
            <el-tag v-if="gitStatus.behind > 0" type="warning">
              behind {{ gitStatus.behind }}
            </el-tag>
            <el-tag v-if="unstagedCount > 0" type="danger">
              {{ unstagedCount }} 未暂存
            </el-tag>
          </div>
        </div>
      </el-card>
    </div>

    <!-- 文件选择区域 -->
    <div class="file-selection-section">
      <el-card>
        <template #header>
          <div class="card-header">
            <span>
              <el-icon><Files /></el-icon>
              选择审查文件 ({{ selectedFileCount }}/{{ availableFiles.length }})
            </span>
            <div class="selection-actions">
              <el-button size="small" @click="selectAllFiles(availableFiles.map(f => f.path))">
                全选
              </el-button>
              <el-button size="small" @click="clearFileSelection">
                清空
              </el-button>
            </div>
          </div>
        </template>

        <!-- 筛选标签 -->
        <div class="file-filter-tabs">
          <el-radio-group v-model="fileFilter" size="small">
            <el-radio-button label="staged">暂存文件 ({{ stagedFiles.length }})</el-radio-button>
            <el-radio-button label="modified">修改文件 ({{ modifiedFiles.length }})</el-radio-button>
            <el-radio-button label="commits">提交记录 ({{ commitHistory.length }})</el-radio-button>
            <el-radio-button label="all">所有文件</el-radio-button>
          </el-radio-group>
        </div>

        <!-- 文件列表或提交记录列表 -->
        <div class="file-list">
          <!-- 文件列表 -->
          <template v-if="fileFilter !== 'commits'">
            <el-empty v-if="filteredFiles.length === 0" description="没有可审查的文件" />
            <el-checkbox-group
              v-else
              v-model="selectedFiles"
            >
              <div
                v-for="file in filteredFiles"
                :key="file.path"
                class="file-item"
                :class="{ selected: isFileSelected(file.path) }"
              >
                <el-checkbox
                  :value="file.path"
                  @change="toggleFileSelection(file.path)"
                >
                  <div class="file-info">
                    <div class="file-path">
                      <el-icon>
                        <Document />
                      </el-icon>
                      {{ file.path }}
                    </div>
                    <div class="file-meta">
                      <el-tag size="small" :type="getFileStatusType(file.status)">
                        {{ getFileStatusText(file.status) }}
                      </el-tag>
                      <span class="file-size">{{ formatFileSize(file.size) }}</span>
                    </div>
                  </div>
                </el-checkbox>
              </div>
            </el-checkbox-group>
          </template>

          <!-- 提交记录列表 -->
          <template v-else>
            <el-empty v-if="commitHistory.length === 0" description="没有提交记录" />
            <el-checkbox-group
              v-else
              v-model="selectedCommits"
            >
              <div
                v-for="commit in commitHistory"
                :key="commit.hash"
                class="commit-item"
                :class="{ selected: selectedCommits.includes(commit.hash) }"
              >
                <el-checkbox
                  :value="commit.hash"
                  @change="toggleCommitSelection(commit.hash)"
                >
                  <div class="commit-info">
                    <div class="commit-header">
                      <el-icon><Timer /></el-icon>
                      <span class="commit-message">{{ commit.message }}</span>
                    </div>
                    <div class="commit-meta">
                      <el-tag size="small" type="info">{{ commit.hash.substring(0, 8) }}</el-tag>
                      <span class="commit-author">{{ commit.author }}</span>
                      <span class="commit-date">{{ formatDate(commit.date) }}</span>
                    </div>
                  </div>
                </el-checkbox>
              </div>
            </el-checkbox-group>
          </template>
        </div>

        <!-- 选择提示 -->
        <div v-if="!validateSelection().valid" class="selection-warning">
          <el-alert
            :title="validateSelection().message"
            type="warning"
            :closable="false"
          />
        </div>
      </el-card>
    </div>

    <!-- 审查配置 -->
    <div class="review-config-section">
      <el-card>
        <template #header>
          <div class="card-header">
            <span>
              <el-icon><Setting /></el-icon>
              审查配置
            </span>
          </div>
        </template>

        <el-form :model="reviewForm" label-width="100px">
          <el-form-item label="审查深度">
            <el-radio-group v-model="reviewForm.depth">
              <el-radio value="BASIC">
                <el-tooltip content="检查语法错误、明显问题" placement="top">
                  <span>基础 (快速)</span>
                </el-tooltip>
              </el-radio>
              <el-radio value="STANDARD">
                <el-tooltip content="代码质量、性能、最佳实践" placement="top">
                  <span>标准 (推荐)</span>
                </el-tooltip>
              </el-radio>
              <el-radio value="DEEP">
                <el-tooltip content="深度分析架构、安全、设计" placement="top">
                  <span>深度 (详细)</span>
                </el-tooltip>
              </el-radio>
            </el-radio-group>
          </el-form-item>

          <el-form-item label="AI 提供商">
            <el-select v-model="reviewForm.provider" placeholder="选择 AI 提供商">
              <el-option
                v-for="provider in aiProviders"
                :key="provider.id"
                :label="provider.name"
                :value="provider.id"
              />
            </el-select>
          </el-form-item>

          <el-form-item label="模型">
            <el-select v-model="reviewForm.model" placeholder="选择模型">
              <el-option
                v-for="model in availableModels"
                :key="model"
                :label="model"
                :value="model"
              />
            </el-select>
          </el-form-item>
        </el-form>

        <!-- 自定义提示词 (可选) -->
        <el-collapse>
          <el-collapse-item title="高级设置" name="1">
            <el-form label-width="120px">
              <el-form-item label="自定义提示词">
                <el-input
                  v-model="reviewForm.customPrompt"
                  type="textarea"
                  :rows="4"
                  placeholder="输入自定义提示词 (可选)"
                />
              </el-form-item>
              <el-form-item>
                <el-checkbox v-model="reviewForm.streaming">
                  启用流式响应
                </el-checkbox>
              </el-form-item>
            </el-form>
          </el-collapse-item>
        </el-collapse>
      </el-card>
    </div>

    <!-- 最近审查 -->
    <div v-if="hasHistory" class="recent-reviews-section">
      <el-card>
        <template #header>
          <div class="card-header">
            <span>
              <el-icon><Clock /></el-icon>
              最近审查
            </span>
            <el-button size="small" @click="showHistory">
              查看全部
            </el-button>
          </div>
        </template>

        <div class="recent-reviews">
          <div
            v-for="review in recentReviews"
            :key="review.id"
            class="review-item"
            @click="viewReview(review.id)"
          >
            <div class="review-summary">
              <div class="review-meta">
                <el-icon><Timer /></el-icon>
                {{ formatDate(review.timestamp) }}
              </div>
              <div class="review-files">
                <el-icon><Files /></el-icon>
                {{ review.files.length }} 个文件
              </div>
              <div class="review-issues" v-if="review.results.ai">
                <el-tag size="small" type="danger">
                  {{ review.results.ai.issues.length }} 问题
                </el-tag>
              </div>
            </div>
            <div class="review-status">
              <el-tag :type="getStatusType(review.status)">
                {{ getStatusText(review.status) }}
              </el-tag>
            </div>
          </div>
        </div>
      </el-card>
    </div>

    <!-- 错误提示 -->
    <el-alert
      v-if="hasError"
      :title="error"
      type="error"
      :closable="true"
      @close="error = null"
      class="error-alert"
    />

    <!-- 审查结果对话框 -->
    <el-dialog
      v-model="showReviewDialog"
      title="审查结果"
      width="80%"
      :fullscreen="true"
    >
      <ReviewDetailView
        v-if="currentReview"
        :review="currentReview"
        @close="showReviewDialog = false"
        @delete="handleDeleteReview"
      />
    </el-dialog>

    <!-- 历史记录对话框 -->
    <el-dialog
      v-model="showHistoryDialog"
      title="审查历史"
      width="90%"
      :fullscreen="true"
    >
      <ReviewHistoryList
        @view-review="viewReview"
        @close="showHistoryDialog = false"
      />
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { ElMessage, ElMessageBox } from 'element-plus';
import {
  Document, Files, Clock, Setting, MagicStick,
  FolderOpened, Collection, Timer
} from '@element-plus/icons-vue';
import { useCodeReview } from '../../composables/useCodeReview';
import type {
  ReviewRecord,
  ReviewDepth,
  ReviewFile,
  GitStatus
} from '../../types/review';
import ReviewDetailView from './ReviewDetailView.vue';
import ReviewHistoryList from './ReviewHistoryList.vue';

// Props
interface Props {
  currentRepo?: string | null;
  gitStatus?: GitStatus | null;
  availableFiles?: ReviewFile[];
  commitHistory?: any[];
}

const props = withDefaults(defineProps<Props>(), {
  currentRepo: null,
  gitStatus: null,
  availableFiles: () => [],
  commitHistory: () => []
});

// 响应式数据
const showReviewDialog = ref(false);
const showHistoryDialog = ref(false);
const fileFilter = ref<'staged' | 'modified' | 'all' | 'commits'>('staged');
const reviewForm = ref({
  depth: 'STANDARD' as ReviewDepth,
  provider: 'openai',
  model: 'gpt-4',
  customPrompt: '',
  streaming: true
});
const selectedCommits = ref<string[]>([]);

// Composables
const {
  loading,
  currentReview,
  reviewHistory,
  error,
  selectedFiles,
  selectedFileCount,
  hasCurrentReview,
  hasHistory,
  hasError,

  startReview,
  loadHistory,
  clearFileSelection,
  selectAllFiles,
  isFileSelected,
  toggleFileSelection,
  validateSelection,
  getReview,

  formatReviewSummary,
  reset
} = useCodeReview();

// AI 提供商列表 (从 AI 配置获取)
const aiProviders = ref([
  { id: 'openai', name: 'OpenAI' },
  { id: 'anthropic', name: 'Anthropic' },
  { id: 'ollama', name: 'Ollama' },
  { id: 'deepseek', name: 'DeepSeek' }
]);

// 可用模型 (根据提供商动态获取)
const availableModels = computed(() => {
  // TODO: 根据选择的提供商动态加载模型
  return ['gpt-4', 'gpt-3.5-turbo', 'claude-3', 'llama2'];
});

// 计算属性
const stagedFiles = computed(() => {
  return props.availableFiles.filter(f => f.status === 'staged');
});

const modifiedFiles = computed(() => {
  return props.availableFiles.filter(f => f.status === 'modified');
});

const filteredFiles = computed(() => {
  switch (fileFilter.value) {
    case 'staged':
      return stagedFiles.value;
    case 'modified':
      return modifiedFiles.value;
    default:
      return props.availableFiles;
  }
});

const unstagedCount = computed(() => {
  return (props.gitStatus?.modified?.length || 0) +
         (props.gitStatus?.deleted?.length || 0) +
         (props.gitStatus?.untracked?.length || 0);
});

const recentReviews = computed(() => {
  if (!reviewHistory.value) return [];
  return reviewHistory.value.data.slice(0, 5);
});

// 方法
const handleStartReview = async () => {
  const validation = validateSelection();
  if (!validation.valid) {
    ElMessage.warning(validation.message);
    return;
  }

  try {
    const config = {
      ai: {
        provider: reviewForm.value.provider,
        model: reviewForm.value.model,
        customPrompts: reviewForm.value.customPrompt ? {
          [reviewForm.value.depth]: reviewForm.value.customPrompt
        } : {}
      }
    };

    let review;

    if (fileFilter.value === 'commits') {
      // 提交记录审查
      console.log('[CodeReviewPanel] 开始审查提交:', selectedCommits.value);
      // TODO: 实现提交记录审查逻辑
      // 需要后端支持 get_commit_diff 和 analyze_commits 功能
      ElMessage.info('提交记录审查功能开发中...');
      return;
    } else {
      // 文件审查
      review = await startReview(
        selectedFiles.value,
        reviewForm.value.depth,
        config as any,
        props.gitStatus?.branch
      );
    }

    ElMessage.success('审查完成！');
    showReviewDialog.value = true;
  } catch (err: any) {
    ElMessage.error(err.message || '审查失败');
  }
};

const viewReview = async (id: string) => {
  try {
    const review = await getReview(id);
    if (review) {
      currentReview.value = review;
      showReviewDialog.value = true;
    }
  } catch (err: any) {
    ElMessage.error(err.message || '获取审查记录失败');
  }
};

const showHistory = () => {
  showHistoryDialog.value = true;
};

const showSettings = () => {
  ElMessage.info('设置功能开发中...');
};

const handleDeleteReview = async (id: string) => {
  try {
    await ElMessageBox.confirm('确定要删除这个审查记录吗？', '确认删除', {
      type: 'warning'
    });
    // TODO: 调用删除 API
    ElMessage.success('删除成功');
    showReviewDialog.value = false;
    await loadHistory();
  } catch (err) {
    if (err !== 'cancel') {
      ElMessage.error('删除失败');
    }
  }
};

// 工具函数
const getRepoName = (path: string): string => {
  const parts = path.split('/');
  return parts[parts.length - 1];
};

const getFileStatusType = (status: string): 'success' | 'warning' | 'info' => {
  switch (status) {
    case 'staged': return 'success';
    case 'modified': return 'warning';
    default: return 'info';
  }
};

const getFileStatusText = (status: string): string => {
  switch (status) {
    case 'staged': return '已暂存';
    case 'modified': return '已修改';
    case 'untracked': return '未跟踪';
    default: return '未知';
  }
};

const formatFileSize = (bytes: number): string => {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
};

const formatDate = (timestamp: string): string => {
  return new Date(timestamp).toLocaleString();
};

// 切换提交选择
const toggleCommitSelection = (commitHash: string) => {
  const index = selectedCommits.value.indexOf(commitHash);
  if (index >= 0) {
    selectedCommits.value.splice(index, 1);
  } else {
    selectedCommits.value.push(commitHash);
  }
  console.log('[CodeReviewPanel] 选中的提交:', selectedCommits.value);
};

// 验证选择
const validateSelection = (): { valid: boolean; message?: string } => {
  if (fileFilter.value === 'commits') {
    // 验证提交记录选择
    if (selectedCommits.value.length === 0) {
      return { valid: false, message: '请选择至少一个提交进行审查' };
    }

    if (selectedCommits.value.length > 10) {
      return { valid: false, message: '一次最多只能审查 10 个提交' };
    }

    return { valid: true };
  } else {
    // 验证文件选择
    if (selectedFiles.value.length === 0) {
      return { valid: false, message: '请选择至少一个文件进行审查' };
    }

    if (selectedFiles.value.length > 20) {
      return { valid: false, message: '一次最多只能审查 20 个文件' };
    }

    return { valid: true };
  }
};

const getStatusType = (status: string): 'success' | 'warning' | 'danger' | 'info' => {
  switch (status) {
    case 'COMPLETED': return 'success';
    case 'APPROVED': return 'success';
    case 'CHANGES_REQUESTED': return 'warning';
    case 'REJECTED': return 'danger';
    default: return 'info';
  }
};

const getStatusText = (status: string): string => {
  switch (status) {
    case 'PENDING': return '待审查';
    case 'IN_PROGRESS': return '审查中';
    case 'COMPLETED': return '已完成';
    case 'APPROVED': return '已批准';
    case 'CHANGES_REQUESTED': return '需修改';
    case 'REJECTED': return '已拒绝';
    default: return '未知';
  }
};

// 生命周期
onMounted(() => {
  loadHistory();
});
</script>

<style scoped>
.code-review-panel {
  padding: 20px;
  height: 100%;
  overflow-y: auto;
}

.panel-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
}

.panel-header h2 {
  margin: 0;
  display: flex;
  align-items: center;
  gap: 10px;
}

.header-actions {
  display: flex;
  gap: 10px;
}

.repo-info {
  margin-bottom: 20px;
}

.repo-card {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
}

.repo-details {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.repo-name {
  font-size: 18px;
  font-weight: bold;
  display: flex;
  align-items: center;
  gap: 8px;
}

.repo-stats {
  display: flex;
  gap: 8px;
}

.file-selection-section,
.review-config-section,
.recent-reviews-section {
  margin-bottom: 20px;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.file-filter-tabs {
  margin-bottom: 20px;
}

.file-list {
  max-height: 300px;
  overflow-y: auto;
}

.file-item {
  padding: 8px 0;
  border-bottom: 1px solid #f0f0f0;
}

.file-item:last-child {
  border-bottom: none;
}

.file-item.selected {
  background-color: #f0f8ff;
}

.file-info {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.file-path {
  display: flex;
  align-items: center;
  gap: 8px;
  font-weight: 500;
}

.file-meta {
  display: flex;
  align-items: center;
  gap: 12px;
  font-size: 12px;
  color: #666;
}

/* 提交记录样式 */
.commit-item {
  padding: 8px 0;
  border-bottom: 1px solid #f0f0f0;
}

.commit-item:last-child {
  border-bottom: none;
}

.commit-item.selected {
  background-color: #f0f8ff;
}

.commit-info {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.commit-header {
  display: flex;
  align-items: center;
  gap: 8px;
  font-weight: 500;
}

.commit-message {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.commit-meta {
  display: flex;
  align-items: center;
  gap: 12px;
  font-size: 12px;
  color: #666;
}

.commit-author {
  font-weight: 500;
}

.commit-date {
  color: #999;
}

.selection-warning {
  margin-top: 16px;
}

.recent-reviews {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.review-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px;
  border: 1px solid #e4e7ed;
  border-radius: 4px;
  cursor: pointer;
  transition: all 0.3s;
}

.review-item:hover {
  border-color: #409eff;
  background-color: #f0f8ff;
}

.review-summary {
  display: flex;
  gap: 20px;
  align-items: center;
}

.review-meta,
.review-files {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 14px;
  color: #666;
}

.error-alert {
  margin-top: 20px;
}

.selection-actions {
  display: flex;
  gap: 8px;
}
</style>
