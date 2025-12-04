<template>
  <div class="review-history-list">
    <!-- 搜索和筛选 -->
    <div class="search-section">
      <el-card>
        <el-form :model="searchForm" label-width="80px" inline>
          <el-form-item label="日期范围">
            <el-date-picker
              v-model="searchForm.dateRange"
              type="daterange"
              range-separator="至"
              start-placeholder="开始日期"
              end-placeholder="结束日期"
              @change="handleSearch"
            />
          </el-form-item>

          <el-form-item label="审查类型">
            <el-select v-model="searchForm.reviewType" placeholder="全部" clearable @change="handleSearch">
              <el-option label="AI 审查" value="AI" />
              <el-option label="人工审查" value="MANUAL" />
              <el-option label="Lint 检查" value="LINT" />
            </el-select>
          </el-form-item>

          <el-form-item label="审查状态">
            <el-select v-model="searchForm.status" placeholder="全部" clearable @change="handleSearch">
              <el-option label="待审查" value="PENDING" />
              <el-option label="审查中" value="IN_PROGRESS" />
              <el-option label="已完成" value="COMPLETED" />
              <el-option label="已批准" value="APPROVED" />
              <el-option label="需修改" value="CHANGES_REQUESTED" />
              <el-option label="已拒绝" value="REJECTED" />
            </el-select>
          </el-form-item>

          <el-form-item label="关键词">
            <el-input
              v-model="searchForm.keyword"
              placeholder="搜索文件名或摘要"
              clearable
              @input="handleSearch"
            >
              <template #prefix>
                <el-icon><Search /></el-icon>
              </template>
            </el-input>
          </el-form-item>

          <el-form-item>
            <el-button @click="resetSearch" :icon="Refresh">重置</el-button>
            <el-button @click="handleSearch" type="primary" :icon="Search">搜索</el-button>
          </el-form-item>
        </el-form>
      </el-card>
    </div>

    <!-- 工具栏 -->
    <div class="toolbar-section">
      <div class="toolbar-left">
        <span class="results-count">
          共找到 {{ pagination.total }} 条记录
        </span>
      </div>
      <div class="toolbar-right">
        <el-select v-model="pagination.pageSize" @change="handleSearch" style="width: 120px">
          <el-option label="10 条/页" :value="10" />
          <el-option label="20 条/页" :value="20" />
          <el-option label="50 条/页" :value="50" />
          <el-option label="100 条/页" :value="100" />
        </el-select>

        <el-button @click="exportData" :icon="Download">
          导出
        </el-button>

        <el-popconfirm
          title="确定要清空所有审查记录吗？"
          @confirm="handleClearAll"
        >
          <template #reference>
            <el-button type="danger" :icon="Delete">
              清空所有
            </el-button>
          </template>
        </el-popconfirm>
      </div>
    </div>

    <!-- 历史记录列表 -->
    <div class="history-section">
      <el-card v-loading="loading">
        <el-empty v-if="!loading && reviewHistory.length === 0" description="没有找到审查记录" />

        <div v-else class="history-list">
          <div
            v-for="review in reviewHistory"
            :key="review.id"
            class="history-item"
            @click="$emit('view-review', review.id)"
          >
            <div class="item-header">
              <div class="item-title">
                <el-icon><Document /></el-icon>
                <span class="review-id">{{ formatReviewId(review.id) }}</span>
                <el-tag :type="getStatusType(review.status)" size="small">
                  {{ getStatusText(review.status) }}
                </el-tag>
                <el-tag :type="getTypeTagType(review.reviewType)" size="small">
                  {{ getTypeText(review.reviewType) }}
                </el-tag>
              </div>
              <div class="item-actions">
                <el-button
                  size="small"
                  @click.stop="viewReview(review.id)"
                  :icon="View"
                >
                  查看
                </el-button>
                <el-popconfirm
                  title="确定要删除这条记录吗？"
                  @confirm.stop="deleteReview(review.id)"
                >
                  <template #reference>
                    <el-button
                      size="small"
                      type="danger"
                      :icon="Delete"
                      @click.stop
                    />
                  </template>
                </el-popconfirm>
              </div>
            </div>

            <div class="item-content">
              <div class="item-meta">
                <span class="meta-item">
                  <el-icon><Timer /></el-icon>
                  {{ formatDate(review.timestamp) }}
                </span>
                <span class="meta-item">
                  <el-icon><Files /></el-icon>
                  {{ review.files.length }} 个文件
                </span>
                <span class="meta-item">
                  <el-icon><Clock /></el-icon>
                  {{ formatDuration(review.duration) }}
                </span>
                <span class="meta-item">
                  <el-icon><Branch /></el-icon>
                  {{ review.branch }}
                </span>
              </div>

              <div class="item-summary">
                {{ getSummaryText(review) }}
              </div>

              <div v-if="review.summary" class="item-description">
                {{ review.summary }}
              </div>
            </div>

            <!-- 展开的详细信息 -->
            <div v-if="expandedItems.has(review.id)" class="item-details">
              <el-divider />

              <div class="details-section">
                <h4>审查的文件</h4>
                <div class="files-grid">
                  <el-tag
                    v-for="file in review.files"
                    :key="file.path"
                    size="small"
                    :type="getFileTypeTagType(file.language)"
                  >
                    {{ file.path }}
                  </el-tag>
                </div>
              </div>

              <div v-if="review.results.ai" class="details-section">
                <h4>AI 审查结果</h4>
                <div class="ai-stats">
                  <el-tag type="info">Token: {{ review.results.ai.tokenCount }}</el-tag>
                  <el-tag type="success">提供商: {{ review.results.ai.provider }}</el-tag>
                  <el-tag type="warning">模型: {{ review.results.ai.model }}</el-tag>
                  <el-tag v-if="review.results.ai.issues.length > 0" type="danger">
                    问题: {{ review.results.ai.issues.length }}
                  </el-tag>
                </div>
                <div v-if="review.results.ai.summary" class="ai-summary">
                  <p>{{ review.results.ai.summary }}</p>
                </div>
              </div>
            </div>

            <div class="item-footer">
              <el-button
                size="small"
                @click.stop="toggleExpanded(review.id)"
                :icon="expandedItems.has(review.id) ? ArrowUp : ArrowDown"
                text
              >
                {{ expandedItems.has(review.id) ? '收起' : '展开详情' }}
              </el-button>
            </div>
          </div>
        </div>
      </el-card>
    </div>

    <!-- 分页 -->
    <div class="pagination-section" v-if="pagination.total > 0">
      <el-pagination
        v-model:current-page="pagination.page"
        v-model:page-size="pagination.pageSize"
        :total="pagination.total"
        :page-sizes="[10, 20, 50, 100]"
        layout="total, sizes, prev, pager, next, jumper"
        @size-change="handleSearch"
        @current-change="handleSearch"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { ElMessage, ElMessageBox } from 'element-plus';
import {
  Search, Refresh, Download, Delete, View,
  Document, Timer, Files, Clock, Branch, ArrowUp, ArrowDown
} from '@element-plus/icons-vue';
import { useCodeReview } from '../../composables/useCodeReview';
import type { ReviewRecord, ReviewType, ReviewStatus } from '../../types/review';

// Emits
const emit = defineEmits<{
  'view-review': [id: string];
  close: [];
}>();

// 响应式数据
const loading = ref(false);
const expandedItems = ref<Set<string>>(new Set());
const searchForm = ref({
  dateRange: null as any,
  reviewType: '',
  status: '',
  keyword: ''
});

const pagination = ref({
  page: 1,
  pageSize: 20,
  total: 0
});

// Composables
const {
  reviewHistory,
  loadHistory,
  deleteReview: deleteReviewRecord,
  clearAllReviews
} = useCodeReview();

// 计算属性
const reviewHistoryList = computed(() => reviewHistory.value?.data || []);

// 方法
const handleSearch = () => {
  loading.value = true;

  const filters = {
    dateFrom: searchForm.value.dateRange?.[0],
    dateTo: searchForm.value.dateRange?.[1],
    reviewTypes: searchForm.value.reviewType ? [searchForm.value.reviewType as ReviewType] : undefined,
    statuses: searchForm.value.status ? [searchForm.value.status as ReviewStatus] : undefined,
    query: searchForm.value.keyword
  };

  const paginationOptions = {
    page: pagination.value.page,
    pageSize: pagination.value.pageSize,
    sortBy: 'timestamp',
    sortOrder: 'desc'
  };

  loadHistory(filters as any, paginationOptions as any)
    .then(() => {
      pagination.value.total = reviewHistory.value?.total || 0;
    })
    .catch((err) => {
      ElMessage.error(err.message || '加载历史记录失败');
    })
    .finally(() => {
      loading.value = false;
    });
};

const resetSearch = () => {
  searchForm.value = {
    dateRange: null,
    reviewType: '',
    status: '',
    keyword: ''
  };
  pagination.value.page = 1;
  handleSearch();
};

const viewReview = (id: string) => {
  emit('view-review', id);
};

const deleteReview = async (id: string) => {
  try {
    await deleteReviewRecord(id);
    ElMessage.success('删除成功');
    handleSearch();
  } catch (err: any) {
    ElMessage.error(err.message || '删除失败');
  }
};

const handleClearAll = async () => {
  try {
    await ElMessageBox.confirm(
      '此操作将永久删除所有审查记录，且不可恢复。确定要继续吗？',
      '警告',
      {
        confirmButtonText: '确定删除',
        cancelButtonText: '取消',
        type: 'warning'
      }
    );

    await clearAllReviews();
    ElMessage.success('清空成功');
    handleSearch();
  } catch (err) {
    if (err !== 'cancel') {
      ElMessage.error('清空失败');
    }
  }
};

const exportData = () => {
  ElMessage.info('导出功能开发中...');
};

const toggleExpanded = (id: string) => {
  if (expandedItems.value.has(id)) {
    expandedItems.value.delete(id);
  } else {
    expandedItems.value.add(id);
  }
};

// 工具函数
const formatReviewId = (id: string): string => {
  return id.substring(0, 8);
};

const formatDate = (timestamp: string): string => {
  return new Date(timestamp).toLocaleString();
};

const formatDuration = (duration: number): string => {
  if (duration < 1000) return `${duration}ms`;
  return `${(duration / 1000).toFixed(2)}s`;
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

const getStatusType = (status: string): 'success' | 'warning' | 'danger' | 'info' => {
  switch (status) {
    case 'COMPLETED':
    case 'APPROVED': return 'success';
    case 'CHANGES_REQUESTED': return 'warning';
    case 'REJECTED': return 'danger';
    default: return 'info';
  }
};

const getTypeText = (type: string): string => {
  switch (type) {
    case 'AI': return 'AI 审查';
    case 'MANUAL': return '人工审查';
    case 'LINT': return 'Lint 检查';
    case 'MIXED': return '混合审查';
    default: return '未知';
  }
};

const getTypeTagType = (type: string): 'primary' | 'success' | 'warning' | 'info' => {
  switch (type) {
    case 'AI': return 'primary';
    case 'MANUAL': return 'success';
    case 'LINT': return 'warning';
    case 'MIXED': return 'info';
    default: return 'info';
  }
};

const getFileTypeTagType = (language: string): '' | 'success' | 'warning' | 'info' | 'danger' => {
  const types: Record<string, '' | 'success' | 'warning' | 'info' | 'danger'> = {
    'javascript': 'warning',
    'typescript': 'warning',
    'python': 'info',
    'rust': 'success',
    'java': 'info',
    'go': 'success',
    'php': 'warning',
    'ruby': 'danger'
  };
  return types[language] || '';
};

const getSummaryText = (review: ReviewRecord): string => {
  if (review.summary) {
    return review.summary;
  }

  if (review.results.ai) {
    const issueCount = review.results.ai.issues.length;
    return `AI 审查发现 ${issueCount} 个问题`;
  }

  return '无审查摘要';
};

// 生命周期
onMounted(() => {
  handleSearch();
});
</script>

<style scoped>
.review-history-list {
  padding: 20px;
  height: 100%;
  overflow-y: auto;
}

.search-section {
  margin-bottom: 20px;
}

.toolbar-section {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
}

.toolbar-left {
  display: flex;
  align-items: center;
}

.results-count {
  color: #666;
  font-size: 14px;
}

.toolbar-right {
  display: flex;
  gap: 12px;
  align-items: center;
}

.history-section {
  margin-bottom: 20px;
}

.history-list {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.history-item {
  border: 1px solid #e4e7ed;
  border-radius: 8px;
  padding: 16px;
  cursor: pointer;
  transition: all 0.3s;
}

.history-item:hover {
  border-color: #409eff;
  box-shadow: 0 2px 12px 0 rgba(0, 0, 0, 0.1);
}

.item-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}

.item-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-weight: 500;
}

.review-id {
  font-family: 'Courier New', monospace;
  background: #f5f7fa;
  padding: 2px 6px;
  border-radius: 4px;
  font-size: 12px;
}

.item-actions {
  display: flex;
  gap: 8px;
}

.item-content {
  margin-bottom: 8px;
}

.item-meta {
  display: flex;
  gap: 20px;
  margin-bottom: 8px;
  font-size: 13px;
  color: #666;
}

.meta-item {
  display: flex;
  align-items: center;
  gap: 4px;
}

.item-summary {
  font-weight: 500;
  margin-bottom: 8px;
}

.item-description {
  font-size: 14px;
  color: #666;
  line-height: 1.5;
}

.item-details {
  margin-top: 12px;
}

.details-section {
  margin-bottom: 16px;
}

.details-section h4 {
  margin: 0 0 8px 0;
  font-size: 14px;
  font-weight: 600;
  color: #333;
}

.files-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.ai-stats {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-bottom: 12px;
}

.ai-summary {
  background: #f8f9fa;
  padding: 12px;
  border-radius: 4px;
  line-height: 1.6;
  font-size: 14px;
}

.item-footer {
  display: flex;
  justify-content: center;
  margin-top: 8px;
  padding-top: 8px;
  border-top: 1px solid #f0f0f0;
}

.pagination-section {
  display: flex;
  justify-content: center;
  margin-top: 20px;
}
</style>
