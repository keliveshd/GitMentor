<template>
  <div class="review-detail-view">
    <!-- 审查摘要信息 -->
    <div class="review-header">
      <div class="review-info">
        <div class="review-title">
          <el-icon><Document /></el-icon>
          审查结果详情
          <el-tag :type="getStatusType(review.status)">
            {{ getStatusText(review.status) }}
          </el-tag>
        </div>
        <div class="review-meta">
          <span>
            <el-icon><Timer /></el-icon>
            {{ formatDuration(review.duration) }}
          </span>
          <span>
            <el-icon><Files /></el-icon>
            {{ review.files.length }} 个文件
          </span>
          <span v-if="review.results.ai">
            <el-icon><MagicStick /></el-icon>
            {{ review.results.ai.provider }} - {{ review.results.ai.model }}
          </span>
        </div>
      </div>
      <div class="header-actions">
        <el-button @click="$emit('close')">
          <el-icon><Close /></el-icon>
          关闭
        </el-button>
        <el-button @click="exportReport" type="primary">
          <el-icon><Download /></el-icon>
          导出报告
        </el-button>
        <el-button @click="handleDelete" type="danger">
          <el-icon><Delete /></el-icon>
          删除
        </el-button>
      </div>
    </div>

    <!-- 问题统计卡片 -->
    <div v-if="review.results.ai" class="issues-stats">
      <el-row :gutter="16">
        <el-col :span="6">
          <el-card class="stat-card total">
            <div class="stat-content">
              <div class="stat-number">{{ review.results.ai.issues.length }}</div>
              <div class="stat-label">总问题数</div>
            </div>
          </el-card>
        </el-col>
        <el-col :span="6">
          <el-card class="stat-card critical">
            <div class="stat-content">
              <div class="stat-number">{{ severityCounts.critical }}</div>
              <div class="stat-label">严重问题</div>
            </div>
          </el-card>
        </el-col>
        <el-col :span="6">
          <el-card class="stat-card high">
            <div class="stat-content">
              <div class="stat-number">{{ severityCounts.high }}</div>
              <div class="stat-label">高优先级</div>
            </div>
          </el-card>
        </el-col>
        <el-col :span="6">
          <el-card class="stat-card medium">
            <div class="stat-content">
              <div class="stat-number">{{ severityCounts.medium + severityCounts.low }}</div>
              <div class="stat-label">其他问题</div>
            </div>
          </el-card>
        </el-col>
      </el-row>
    </div>

    <!-- 审查摘要 -->
    <div v-if="review.results.ai?.summary" class="ai-summary">
      <el-card>
        <template #header>
          <div class="card-header">
            <el-icon><ChatLineRound /></el-icon>
            AI 审查摘要
          </div>
        </template>
        <div class="summary-content">
          {{ review.results.ai.summary }}
        </div>
      </el-card>
    </div>

    <!-- 问题列表 -->
    <div class="issues-section">
      <el-card>
        <template #header>
          <div class="card-header">
            <el-icon><Warning /></el-icon>
            发现的问题 ({{ review.results.ai?.issues.length || 0 }})
          </div>
        </template>

        <!-- 筛选和排序 -->
        <div class="issues-toolbar">
          <el-radio-group v-model="issueFilter" size="small">
            <el-radio-button label="all">全部</el-radio-button>
            <el-radio-button label="critical">严重</el-radio-button>
            <el-radio-button label="high">高</el-radio-button>
            <el-radio-button label="medium">中</el-radio-button>
            <el-radio-button label="low">低</el-radio-button>
          </el-radio-group>

          <el-select v-model="issueSortBy" placeholder="排序方式" size="small">
            <el-option label="严重性" value="severity" />
            <el-option label="类型" value="type" />
            <el-option label="文件" value="file" />
          </el-select>
        </div>

        <!-- 问题列表 -->
        <div class="issues-list">
          <el-empty v-if="filteredIssues.length === 0" description="没有问题" />

          <div
            v-for="issue in filteredIssues"
            :key="issue.id"
            class="issue-item"
            :class="`severity-${issue.severity.toLowerCase()}`"
          >
            <div class="issue-header">
              <div class="issue-title">
                <span class="severity-icon">
                  {{ getSeverityIcon(issue.severity) }}
                </span>
                <span class="issue-title-text">{{ issue.title }}</span>
                <el-tag size="small" :type="getSeverityTagType(issue.severity)">
                  {{ getSeverityText(issue.severity) }}
                </el-tag>
                <el-tag size="small">
                  {{ getIssueTypeIcon(issue.issueType) }} {{ getIssueTypeText(issue.issueType) }}
                </el-tag>
              </div>
              <div class="issue-actions">
                <el-button
                  size="small"
                  @click="toggleIssueExpanded(issue.id)"
                  :icon="expandedIssues.has(issue.id) ? ArrowUp : ArrowDown"
                >
                  {{ expandedIssues.has(issue.id) ? '收起' : '展开' }}
                </el-button>
              </div>
            </div>

            <div v-if="expandedIssues.has(issue.id)" class="issue-details">
              <div class="issue-location">
                <el-icon><Location /></el-icon>
                <span>{{ issue.file }}</span>
                <span v-if="issue.line">第 {{ issue.line }} 行</span>
              </div>

              <div class="issue-description">
                <h4>问题描述</h4>
                <p>{{ issue.description }}</p>
              </div>

              <div v-if="issue.codeSnippet" class="issue-code">
                <h4>相关代码</h4>
                <pre><code>{{ issue.codeSnippet }}</code></pre>
              </div>

              <div v-if="issue.suggestion" class="issue-suggestion">
                <h4>修复建议</h4>
                <p>{{ issue.suggestion }}</p>
              </div>

              <div v-if="issue.cweId" class="issue-references">
                <h4>相关链接</h4>
                <el-link :href="`https://cwe.mitre.org/data/definitions/${issue.cweId.replace('CWE-', '')}.html`" target="_blank">
                  {{ issue.cweId }}
                </el-link>
              </div>
            </div>
          </div>
        </div>
      </el-card>
    </div>

    <!-- 文件列表 -->
    <div class="files-section">
      <el-card>
        <template #header>
          <div class="card-header">
            <el-icon><Files /></el-icon>
            审查的文件 ({{ review.files.length }})
          </div>
        </template>

        <div class="files-list">
          <div
            v-for="file in review.files"
            :key="file.path"
            class="file-item"
          >
            <div class="file-info">
              <div class="file-path">
                <el-icon><Document /></el-icon>
                {{ file.path }}
              </div>
              <div class="file-meta">
                <el-tag size="small">{{ file.language }}</el-tag>
                <span>{{ formatFileSize(file.size) }}</span>
              </div>
            </div>
          </div>
        </div>
      </el-card>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import { ElMessage, ElMessageBox } from 'element-plus';
import {
  Document, Timer, Files, MagicStick, Close, Download, Delete,
  ChatLineRound, Warning, Location, ArrowUp, ArrowDown
} from '@element-plus/icons-vue';
import type { ReviewRecord, CodeIssue } from '../../types/review';

// Props
interface Props {
  review: ReviewRecord;
}

const props = defineProps<Props>();

// Emits
const emit = defineEmits<{
  close: [];
  delete: [id: string];
}>();

// 响应式数据
const issueFilter = ref<'all' | 'critical' | 'high' | 'medium' | 'low'>('all');
const issueSortBy = ref<'severity' | 'type' | 'file'>('severity');
const expandedIssues = ref<Set<string>>(new Set());

// 计算属性
const severityCounts = computed(() => {
  if (!props.review.results.ai) return { critical: 0, high: 0, medium: 0, low: 0, info: 0 };

  return props.review.results.ai.issues.reduce((acc, issue) => {
    const severity = issue.severity.toLowerCase();
    acc[severity as keyof typeof acc] = (acc[severity as keyof typeof acc] || 0) + 1;
    return acc;
  }, { critical: 0, high: 0, medium: 0, low: 0, info: 0 } as any);
});

const filteredIssues = computed(() => {
  if (!props.review.results.ai) return [];

  let issues = [...props.review.results.ai.issues];

  // 筛选
  if (issueFilter.value !== 'all') {
    issues = issues.filter(issue => issue.severity.toLowerCase() === issueFilter.value);
  }

  // 排序
  issues.sort((a, b) => {
    switch (issueSortBy.value) {
      case 'severity':
        return getSeverityOrder(a.severity) - getSeverityOrder(b.severity);
      case 'type':
        return a.issueType.localeCompare(b.issueType);
      case 'file':
        return a.file.localeCompare(b.file);
      default:
        return 0;
    }
  });

  return issues;
});

// 方法
const getSeverityOrder = (severity: string): number => {
  switch (severity.toUpperCase()) {
    case 'CRITICAL': return 0;
    case 'HIGH': return 1;
    case 'MEDIUM': return 2;
    case 'LOW': return 3;
    case 'INFO': return 4;
    default: return 5;
  }
};

const getSeverityText = (severity: string): string => {
  switch (severity.toUpperCase()) {
    case 'CRITICAL': return '严重';
    case 'HIGH': return '高';
    case 'MEDIUM': return '中';
    case 'LOW': return '低';
    case 'INFO': return '信息';
    default: return '未知';
  }
};

const getSeverityType = (status: string): 'success' | 'warning' | 'danger' | 'info' => {
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

const getSeverityTagType = (severity: string): 'danger' | 'warning' | 'info' | 'success' => {
  switch (severity.toUpperCase()) {
    case 'CRITICAL': return 'danger';
    case 'HIGH': return 'warning';
    case 'MEDIUM':
    case 'LOW':
    case 'INFO': return 'info';
    default: return 'info';
  }
};

const getSeverityIcon = (severity: string): string => {
  switch (severity.toUpperCase()) {
    case 'CRITICAL': return '🔴';
    case 'HIGH': return '🟠';
    case 'MEDIUM': return '🟡';
    case 'LOW': return '🟢';
    case 'INFO': return 'ℹ️';
    default: return '❓';
  }
};

const getIssueTypeText = (type: string): string => {
  const typeMap: Record<string, string> = {
    'PERFORMANCE': '性能',
    'SECURITY': '安全',
    'BUG': '错误',
    'STYLE': '风格',
    'CODE_SMELL': '代码异味',
    'COMPLEXITY': '复杂度',
    'BEST_PRACTICE': '最佳实践',
    'ERROR': '错误',
    'MAINTAINABILITY': '可维护性',
    'READABILITY': '可读性',
    'DOCUMENTATION': '文档',
    'DEPRECATED': '废弃',
    'UNUSED': '未使用',
    'VULNERABILITY': '漏洞',
    'INEFFICIENT': '低效'
  };
  return typeMap[type] || type;
};

const getIssueTypeIcon = (type: string): string => {
  const iconMap: Record<string, string> = {
    'PERFORMANCE': '⚡',
    'SECURITY': '🔒',
    'BUG': '🐛',
    'STYLE': '🎨',
    'CODE_SMELL': '👃',
    'COMPLEXITY': '🧩',
    'BEST_PRACTICE': '✅',
    'ERROR': '❌',
    'MAINTAINABILITY': '🔧',
    'READABILITY': '📖',
    'DOCUMENTATION': '📝',
    'DEPRECATED': '⚠️',
    'UNUSED': '🗑️',
    'VULNERABILITY': '🛡️',
    'INEFFICIENT': '🐌'
  };
  return iconMap[type] || '📋';
};

const formatDuration = (duration: number): string => {
  if (duration < 1000) return `${duration}ms`;
  return `${(duration / 1000).toFixed(2)}s`;
};

const formatFileSize = (bytes: number): string => {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
};

const toggleIssueExpanded = (issueId: string) => {
  if (expandedIssues.value.has(issueId)) {
    expandedIssues.value.delete(issueId);
  } else {
    expandedIssues.value.add(issueId);
  }
};

const exportReport = () => {
  ElMessage.info('导出功能开发中...');
};

const handleDelete = () => {
  ElMessageBox.confirm('确定要删除这个审查记录吗？', '确认删除', {
    type: 'warning'
  }).then(() => {
    emit('delete', props.review.id);
  }).catch(() => {});
};
</script>

<style scoped>
.review-detail-view {
  padding: 20px;
  height: calc(100vh - 100px);
  overflow-y: auto;
}

.review-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
  padding: 20px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  border-radius: 8px;
}

.review-title {
  font-size: 24px;
  font-weight: bold;
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 8px;
}

.review-meta {
  display: flex;
  gap: 20px;
  font-size: 14px;
  opacity: 0.9;
}

.review-meta span {
  display: flex;
  align-items: center;
  gap: 6px;
}

.header-actions {
  display: flex;
  gap: 10px;
}

.issues-stats {
  margin-bottom: 20px;
}

.stat-card {
  text-align: center;
}

.stat-card.total {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
}

.stat-card.critical {
  background: linear-gradient(135deg, #ff6b6b 0%, #ee5a6f 100%);
  color: white;
}

.stat-card.high {
  background: linear-gradient(135deg, #feca57 0%, #ff9ff3 100%);
  color: white;
}

.stat-card.medium {
  background: linear-gradient(135deg, #48dbfb 0%, #0abde3 100%);
  color: white;
}

.stat-number {
  font-size: 36px;
  font-weight: bold;
  margin-bottom: 8px;
}

.stat-label {
  font-size: 14px;
  opacity: 0.9;
}

.ai-summary {
  margin-bottom: 20px;
}

.summary-content {
  padding: 16px;
  background: #f8f9fa;
  border-radius: 4px;
  line-height: 1.6;
}

.issues-section {
  margin-bottom: 20px;
}

.issues-toolbar {
  display: flex;
  justify-content: space-between;
  margin-bottom: 16px;
}

.issues-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.issue-item {
  border: 1px solid #e4e7ed;
  border-radius: 8px;
  padding: 16px;
  transition: all 0.3s;
}

.issue-item.severity-critical {
  border-left: 4px solid #f56c6c;
}

.issue-item.severity-high {
  border-left: 4px solid #e6a23c;
}

.issue-item.severity-medium {
  border-left: 4px solid #409eff;
}

.issue-item.severity-low {
  border-left: 4px solid #67c23a;
}

.issue-item:hover {
  box-shadow: 0 2px 12px 0 rgba(0, 0, 0, 0.1);
}

.issue-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 12px;
}

.issue-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-weight: 500;
}

.severity-icon {
  font-size: 18px;
}

.issue-title-text {
  font-size: 16px;
}

.issue-details {
  margin-top: 12px;
  padding-top: 12px;
  border-top: 1px solid #f0f0f0;
}

.issue-location {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 12px;
  font-size: 14px;
  color: #666;
}

.issue-description h4,
.issue-code h4,
.issue-suggestion h4,
.issue-references h4 {
  margin: 12px 0 8px 0;
  font-size: 14px;
  font-weight: 600;
  color: #333;
}

.issue-description p,
.issue-suggestion p {
  margin: 0;
  line-height: 1.6;
  color: #666;
}

.issue-code pre {
  background: #f5f7fa;
  padding: 12px;
  border-radius: 4px;
  overflow-x: auto;
  font-size: 13px;
  line-height: 1.5;
}

.issue-code code {
  font-family: 'Courier New', monospace;
}

.files-section {
  margin-bottom: 20px;
}

.files-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.file-item {
  padding: 12px;
  border: 1px solid #e4e7ed;
  border-radius: 4px;
  display: flex;
  justify-content: space-between;
  align-items: center;
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

.card-header {
  display: flex;
  align-items: center;
  gap: 8px;
  font-weight: 600;
}
</style>
