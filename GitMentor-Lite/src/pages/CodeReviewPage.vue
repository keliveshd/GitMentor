<template>
  <div class="code-review-page">
    <CodeReviewPanel
      :current-repo="currentRepo"
      :git-status="gitStatus"
      :available-files="availableFiles"
      :commit-history="commitHistory"
      @load-commit-history="loadCommitHistory"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import CodeReviewPanel from '../components/codereview/CodeReviewPanel.vue';
import type { ReviewFile, GitStatus, GitCommit } from '../types/review';

const currentRepo = ref<string | null>(null);
const gitStatus = ref<GitStatus | null>(null);
const availableFiles = ref<ReviewFile[]>([]);
const commitHistory = ref<GitCommit[]>([]);

// 加载提交历史
const loadCommitHistory = async (limit: number = 50) => {
  if (!currentRepo.value) return;

  try {
    console.log('[CodeReviewPage] 加载提交历史...');
    const commits = await invoke<GitCommit[]>('get_commit_history', {
      repoPath: currentRepo.value,
      limit
    });
    commitHistory.value = commits;
    console.log('[CodeReviewPage] 提交历史加载完成:', commits.length, '条记录');
  } catch (error) {
    console.error('[CodeReviewPage] 加载提交历史失败:', error);
    commitHistory.value = [];
  }
};

onMounted(async () => {
  try {
    // 注意：get_current_repository 命令未实现，暂不使用
    // 后续可以从 GitPanel 或全局状态获取当前仓库

    // 暂时设置为空，等待用户手动选择仓库
    currentRepo.value = null;

    // 尝试获取 Git 状态（如果已有仓库）
    if (currentRepo.value) {
      gitStatus.value = await invoke('get_repository_status', {
        repoPath: currentRepo.value
      });

      // 生成可审查文件列表
      const status = gitStatus.value;
      availableFiles.value = [
        ...(status.staged?.map(f => ({
          ...f,
          status: 'staged' as const
        })) || []),
        ...(status.modified?.map(f => ({
          ...f,
          status: 'modified' as const
        })) || []),
      ];

      // 预加载提交历史
      await loadCommitHistory(20);
    } else {
      // 没有仓库时显示空状态
      gitStatus.value = null;
      availableFiles.value = [];
      commitHistory.value = [];
    }
  } catch (error) {
    console.error('获取代码审查页面数据失败:', error);
    // 静默处理错误，避免页面空白
    currentRepo.value = null;
    gitStatus.value = null;
    availableFiles.value = [];
    commitHistory.value = [];
  }
});
</script>

<style scoped>
.code-review-page {
  width: 100%;
  height: 100%;
  padding: 16px;
  box-sizing: border-box;
}
</style>
