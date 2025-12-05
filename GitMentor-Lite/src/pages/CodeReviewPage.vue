<template>
  <div class="code-review-page">
    <CodeReviewPanel
      :current-repo="currentRepo"
      :git-status="gitStatus"
      :available-files="availableFiles"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import CodeReviewPanel from '../components/codereview/CodeReviewPanel.vue';
import type { ReviewFile, GitStatus } from '../types/review';

const currentRepo = ref<string | null>(null);
const gitStatus = ref<GitStatus | null>(null);
const availableFiles = ref<ReviewFile[]>([]);

onMounted(async () => {
  try {
    // 获取当前仓库信息
    currentRepo.value = await invoke('get_current_repository');

    // 获取 Git 状态
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
    }
  } catch (error) {
    console.error('获取代码审查页面数据失败:', error);
    // 静默处理错误，避免页面空白
    currentRepo.value = null;
    gitStatus.value = null;
    availableFiles.value = [];
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
