/**
 * Code Review Vue Composables
 * 提供前端组件使用的状态管理和 API 调用
 * 作者：Evilek
 * 日期：2025-01-04
 */

import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type {
  ReviewRecord,
  ReviewConfig,
  ReviewDepth,
  ReviewFilters,
  PaginationOptions,
  PaginatedResult,
  CodeIssue,
  AIReviewResult,
  ReviewFile
} from '../types/review';

// 响应式状态
const loading = ref(false);
const currentReview = ref<ReviewRecord | null>(null);
const reviewHistory = ref<PaginatedResult<ReviewRecord> | null>(null);
const error = ref<string | null>(null);
const selectedFiles = ref<string[]>([]);
const reviewConfig = ref<ReviewConfig | null>(null);

/**
 * Code Review 组合式函数
 */
export function useCodeReview() {
  /**
   * 开始 AI 审查
   */
  const startReview = async (
    files: string[],
    depth: ReviewDepth,
    config?: ReviewConfig,
    currentBranch?: string
  ): Promise<ReviewRecord> => {
    loading.value = true;
    error.value = null;

    try {
      const result = await invoke<ReviewRecord>('create_ai_review', {
        files,
        depth,
        config,
        currentBranch,
        creator: 'current_user' // TODO: 从用户状态获取
      });

      currentReview.value = result;
      await loadHistory(); // 刷新历史记录
      return result;
    } catch (err: any) {
      error.value = err.message || '审查失败';
      throw new Error(error.value);
    } finally {
      loading.value = false;
    }
  };

  /**
   * 获取审查记录
   */
  const getReview = async (id: string): Promise<ReviewRecord | null> => {
    try {
      const result = await invoke<ReviewRecord | null>('get_review_record', { id });
      return result;
    } catch (err: any) {
      error.value = err.message || '获取审查记录失败';
      throw new Error(error.value);
    }
  };

  /**
   * 加载审查历史
   */
  const loadHistory = async (
    filters?: ReviewFilters,
    pagination?: PaginationOptions
  ): Promise<void> => {
    loading.value = true;
    error.value = null;

    try {
      const result = await invoke<PaginatedResult<ReviewRecord>>('get_review_history', {
        filters,
        pagination
      });
      reviewHistory.value = result;
    } catch (err: any) {
      error.value = err.message || '获取历史记录失败';
      throw new Error(error.value);
    } finally {
      loading.value = false;
    }
  };

  /**
   * 删除审查记录
   */
  const deleteReview = async (id: string): Promise<void> => {
    try {
      await invoke('delete_review_record', { id });
      await loadHistory(); // 刷新列表
    } catch (err: any) {
      error.value = err.message || '删除失败';
      throw new Error(error.value);
    }
  };

  /**
   * 清除所有审查记录
   */
  const clearAllReviews = async (): Promise<void> => {
    try {
      await invoke('clear_all_reviews');
      await loadHistory(); // 刷新列表
      currentReview.value = null;
    } catch (err: any) {
      error.value = err.message || '清除失败';
      throw new Error(error.value);
    }
  };

  /**
   * 获取审查配置
   */
  const loadConfig = async (): Promise<ReviewConfig> => {
    try {
      const config = await invoke<ReviewConfig>('get_review_config');
      reviewConfig.value = config;
      return config;
    } catch (err: any) {
      error.value = err.message || '获取配置失败';
      throw new Error(error.value);
    }
  };

  /**
   * 获取缓存大小
   */
  const getCacheSize = async (): Promise<number> => {
    try {
      return await invoke<number>('get_review_cache_size');
    } catch (err: any) {
      error.value = err.message || '获取缓存大小失败';
      throw new Error(error.value);
    }
  };

  /**
   * 清除缓存
   */
  const clearCache = async (): Promise<void> => {
    try {
      await invoke('clear_review_cache');
    } catch (err: any) {
      error.value = err.message || '清除缓存失败';
      throw new Error(error.value);
    }
  };

  /**
   * 获取存储路径信息
   */
  const getStoragePaths = async () => {
    try {
      return await invoke<{
        config_dir: string;
        data_path: string;
        cache_dir: string;
      }>('get_review_storage_paths');
    } catch (err: any) {
      error.value = err.message || '获取存储路径失败';
      throw new Error(error.value);
    }
  };

  /**
   * 获取审查统计
   */
  const getStatistics = async () => {
    try {
      return await invoke('get_review_statistics');
    } catch (err: any) {
      error.value = err.message || '获取统计失败';
      throw new Error(error.value);
    }
  };

  /**
   * 格式化审查结果摘要
   */
  const formatReviewSummary = (review: ReviewRecord): string => {
    if (!review.results.ai) {
      return '无审查结果';
    }

    const aiResult = review.results.ai;
    const totalIssues = aiResult.issues.length;
    const severityCounts = aiResult.issues.reduce((acc, issue) => {
      acc[issue.severity] = (acc[issue.severity] || 0) + 1;
      return acc;
    }, {} as Record<string, number>);

    const parts = [
      `审查了 ${review.files.length} 个文件`,
      `发现 ${totalIssues} 个问题`
    ];

    if (severityCounts.CRITICAL) {
      parts.push(`🔴 严重: ${severityCounts.CRITICAL}`);
    }
    if (severityCounts.HIGH) {
      parts.push(`🟠 高: ${severityCounts.HIGH}`);
    }
    if (severityCounts.MEDIUM) {
      parts.push(`🟡 中: ${severityCounts.MEDIUM}`);
    }
    if (severityCounts.LOW) {
      parts.push(`🟢 低: ${severityCounts.LOW}`);
    }

    return parts.join(', ');
  };

  /**
   * 获取问题严重性颜色
   */
  const getSeverityColor = (severity: string): string => {
    switch (severity) {
      case 'CRITICAL':
        return '#f56c6c';
      case 'HIGH':
        return '#e6a23c';
      case 'MEDIUM':
        return '#409eff';
      case 'LOW':
        return '#67c23a';
      case 'INFO':
        return '#909399';
      default:
        return '#909399';
    }
  };

  /**
   * 获取问题严重性图标
   */
  const getSeverityIcon = (severity: string): string => {
    switch (severity) {
      case 'CRITICAL':
        return '🔴';
      case 'HIGH':
        return '🟠';
      case 'MEDIUM':
        return '🟡';
      case 'LOW':
        return '🟢';
      case 'INFO':
        return 'ℹ️';
      default:
        return '❓';
    }
  };

  /**
   * 获取问题类型图标
   */
  const getIssueTypeIcon = (type: string): string => {
    switch (type) {
      case 'PERFORMANCE':
        return '⚡';
      case 'SECURITY':
        return '🔒';
      case 'BUG':
        return '🐛';
      case 'STYLE':
        return '🎨';
      case 'CODE_SMELL':
        return '👃';
      case 'COMPLEXITY':
        return '🧩';
      case 'BEST_PRACTICE':
        return '✅';
      case 'ERROR':
        return '❌';
      case 'MAINTAINABILITY':
        return '🔧';
      case 'READABILITY':
        return '📖';
      case 'DOCUMENTATION':
        return '📝';
      case 'DEPRECATED':
        return '⚠️';
      case 'UNUSED':
        return '🗑️';
      default:
        return '📋';
    }
  };

  /**
   * 检查文件是否被选中
   */
  const isFileSelected = (filePath: string): boolean => {
    return selectedFiles.value.includes(filePath);
  };

  /**
   * 切换文件选择
   */
  const toggleFileSelection = (filePath: string): void => {
    const index = selectedFiles.value.indexOf(filePath);
    if (index > -1) {
      selectedFiles.value.splice(index, 1);
    } else {
      selectedFiles.value.push(filePath);
    }
  };

  /**
   * 选择所有文件
   */
  const selectAllFiles = (filePaths: string[]): void => {
    selectedFiles.value = [...filePaths];
  };

  /**
   * 清空文件选择
   */
  const clearFileSelection = (): void => {
    selectedFiles.value = [];
  };

  /**
   * 验证文件选择
   */
  const validateSelection = (): { valid: boolean; message?: string } => {
    if (selectedFiles.value.length === 0) {
      return { valid: false, message: '请选择至少一个文件进行审查' };
    }

    if (selectedFiles.value.length > 20) {
      return { valid: false, message: '一次最多只能审查 20 个文件' };
    }

    return { valid: true };
  };

  /**
   * 重置状态
   */
  const reset = (): void => {
    loading.value = false;
    currentReview.value = null;
    error.value = null;
    selectedFiles.value = [];
  };

  // 计算属性
  const hasCurrentReview = computed(() => currentReview.value !== null);
  const hasHistory = computed(() => reviewHistory.value && reviewHistory.value.data.length > 0);
  const hasError = computed(() => error.value !== null);
  const selectedFileCount = computed(() => selectedFiles.value.length);

  // 返回
  return {
    // 状态
    loading: readonly(loading),
    currentReview: readonly(currentReview),
    reviewHistory: readonly(reviewHistory),
    error: readonly(error),
    selectedFiles: readonly(selectedFiles),
    reviewConfig: readonly(reviewConfig),

    // 计算属性
    hasCurrentReview,
    hasHistory,
    hasError,
    selectedFileCount,

    // 方法
    startReview,
    getReview,
    loadHistory,
    deleteReview,
    clearAllReviews,
    loadConfig,
    getCacheSize,
    clearCache,
    getStoragePaths,
    getStatistics,

    // 工具方法
    formatReviewSummary,
    getSeverityColor,
    getSeverityIcon,
    getIssueTypeIcon,

    // 文件选择
    isFileSelected,
    toggleFileSelection,
    selectAllFiles,
    clearFileSelection,
    validateSelection,

    // 状态管理
    reset
  };
}

// 只读包装器
function readonly<T>(ref: any): any {
  return ref;
}
