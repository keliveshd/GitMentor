<template>
  <div class="template-versioning-test">
    <h1>🧪 模板版本管理测试</h1>

    <div class="test-section">
      <h2>测试面板</h2>

      <div class="test-actions">
        <button @click="runAllTests" class="test-btn">🚀 运行所有测试</button>
        <button @click="clearResults" class="clear-btn">🗑️ 清除结果</button>
      </div>

      <div class="test-results">
        <h3>测试结果：</h3>
        <div v-for="(result, index) in testResults" :key="index"
             :class="['test-result', result.success ? 'success' : 'error']">
          <div class="result-header">
            <span class="result-icon">{{ result.success ? '✅' : '❌' }}</span>
            <span class="result-title">{{ result.title }}</span>
          </div>
          <div class="result-content" v-if="result.message">
            {{ result.message }}
          </div>
          <div class="result-details" v-if="result.data">
            <pre>{{ JSON.stringify(result.data, null, 2) }}</pre>
          </div>
        </div>
      </div>
    </div>

    <div class="info-section">
      <h2>📊 模板信息</h2>

      <div class="template-info">
        <h3>提交模板 ({{ commitTemplates.length }})</h3>
        <div v-for="template in commitTemplates" :key="template.id" class="template-card">
          <h4>{{ template.name }} ({{ template.version || 'N/A' }})</h4>
          <p>{{ template.description }}</p>
          <div class="template-meta">
            <span class="version-info">版本: {{ template.version || '未知' }}</span>
            <span class="custom-info" :class="{ custom: template.is_custom }">
              {{ template.is_custom ? '自定义' : '系统' }}
            </span>
          </div>
        </div>
      </div>

      <div class="version-info-section">
        <h3>版本历史示例</h3>
        <div v-if="versionHistory.length > 0">
          <div v-for="version in versionHistory" :key="version.id" class="version-card">
            <span class="version-number">{{ version.version }}</span>
            <span class="version-name">{{ version.name }}</span>
            <span class="version-date">{{ new Date(version.created_at).toLocaleString() }}</span>
          </div>
        </div>
        <div v-else class="no-data">
          暂无版本历史数据
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';

// 测试结果
const testResults = ref<any[]>([]);

// 模板数据
const commitTemplates = ref<any[]>([]);
const versionHistory = ref<any[]>([]);

// 添加测试结果
function addTestResult(title: string, success: boolean, message?: string, data?: any) {
  testResults.value.push({
    title,
    success,
    message,
    data,
    timestamp: new Date().toISOString()
  });
}

// 清除测试结果
function clearResults() {
  testResults.value = [];
}

// 测试函数
async function testGetCommitTemplates() {
  try {
    const templates = await invoke('get_all_commit_templates');
    commitTemplates.value = templates as any[];
    addTestResult(
      '获取所有提交模板',
      true,
      `成功获取 ${templates.length} 个提交模板`,
      { count: templates.length }
    );
  } catch (error) {
    addTestResult(
      '获取所有提交模板',
      false,
      `失败: ${error}`
    );
  }
}

async function testGetVersionHistory() {
  try {
    const versions = await invoke('get_commit_template_version_history', {
      templateId: 'commit_standard'
    });
    versionHistory.value = versions as any[];
    addTestResult(
      '获取版本历史',
      true,
      `成功获取 ${versions.length} 个版本`,
      { versions: versions }
    );
  } catch (error) {
    addTestResult(
      '获取版本历史',
      false,
      `失败: ${error}`
    );
  }
}

async function testCheckUpdates() {
  try {
    const updates = await invoke('check_commit_template_updates');
    addTestResult(
      '检查系统更新',
      true,
      `发现 ${updates.length} 个可用更新`,
      { updateCount: updates.length }
    );
  } catch (error) {
    addTestResult(
      '检查系统更新',
      false,
      `失败: ${error}`
    );
  }
}

async function testUpdateTemplate() {
  try {
    const versionId = await invoke('update_commit_template_with_version', {
      templateId: 'commit_chinese',
      content: `这是测试更新的中文提交模板内容

变更的文件：{{staged_files}}
代码差异：{{diff}}

请根据以上变更生成简洁的中文提交消息。`,
      versionName: 'v1.0.1 - 测试更新',
      versionDescription: '通过测试页面创建的测试版本'
    });
    addTestResult(
      '更新模板并创建版本',
      true,
      `成功创建新版本: ${versionId}`,
      { versionId }
    );
  } catch (error) {
    addTestResult(
      '更新模板并创建版本',
      false,
      `失败: ${error}`
    );
  }
}

async function testGetUnifiedTemplates() {
  try {
    const unified = await invoke('get_all_templates_unified');
    addTestResult(
      '获取统一模板列表',
      true,
      `提交模板: ${unified.commit_templates.length}, 版本化模板: ${unified.versioned_templates.length}`,
      {
        commitCount: unified.commit_templates.length,
        versionedCount: unified.versioned_templates.length
      }
    );
  } catch (error) {
    addTestResult(
      '获取统一模板列表',
      false,
      `失败: ${error}`
    );
  }
}

// 运行所有测试
async function runAllTests() {
  clearResults();
  addTestResult('开始测试序列', true, '正在运行所有测试...');

  await testGetCommitTemplates();
  await new Promise(resolve => setTimeout(resolve, 100));

  await testGetVersionHistory();
  await new Promise(resolve => setTimeout(resolve, 100));

  await testCheckUpdates();
  await new Promise(resolve => setTimeout(resolve, 100));

  await testGetUnifiedTemplates();
  await new Promise(resolve => setTimeout(resolve, 100));

  // 暂时注释掉更新测试，避免频繁创建版本
  // await testUpdateTemplate();

  addTestResult('测试序列完成', true, '所有测试已完成');
}

// 组件挂载时加载基础数据
onMounted(async () => {
  await testGetCommitTemplates();
  await testGetVersionHistory();
});
</script>

<style scoped>
.template-versioning-test {
  max-width: 1200px;
  margin: 0 auto;
  padding: 20px;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
}

h1 {
  text-align: center;
  color: #2c3e50;
  margin-bottom: 30px;
}

.test-section {
  background: #f8f9fa;
  border-radius: 10px;
  padding: 20px;
  margin-bottom: 30px;
}

.test-actions {
  display: flex;
  gap: 10px;
  margin-bottom: 20px;
}

.test-btn {
  background: #42b883;
  color: white;
  border: none;
  padding: 10px 20px;
  border-radius: 5px;
  cursor: pointer;
  font-size: 16px;
  transition: background 0.3s;
}

.test-btn:hover {
  background: #3aa876;
}

.clear-btn {
  background: #e74c3c;
  color: white;
  border: none;
  padding: 10px 20px;
  border-radius: 5px;
  cursor: pointer;
  font-size: 16px;
  transition: background 0.3s;
}

.clear-btn:hover {
  background: #c0392b;
}

.test-results {
  max-height: 400px;
  overflow-y: auto;
}

.test-result {
  background: white;
  border-radius: 5px;
  padding: 15px;
  margin-bottom: 10px;
  border-left: 4px solid;
}

.test-result.success {
  border-left-color: #27ae60;
}

.test-result.error {
  border-left-color: #e74c3c;
}

.result-header {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 5px;
}

.result-title {
  font-weight: bold;
  font-size: 16px;
}

.result-content {
  color: #555;
  margin-bottom: 10px;
}

.result-details {
  background: #f8f9fa;
  padding: 10px;
  border-radius: 3px;
  overflow-x: auto;
}

.result-details pre {
  margin: 0;
  font-size: 12px;
  color: #2c3e50;
}

.info-section {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 30px;
}

@media (max-width: 768px) {
  .info-section {
    grid-template-columns: 1fr;
  }
}

.template-card, .version-card {
  background: white;
  border-radius: 8px;
  padding: 15px;
  margin-bottom: 15px;
  box-shadow: 0 2px 4px rgba(0,0,0,0.1);
}

.template-card h4 {
  margin: 0 0 5px 0;
  color: #2c3e50;
}

.template-card p {
  margin: 0 0 10px 0;
  color: #7f8c8d;
  font-size: 14px;
}

.template-meta {
  display: flex;
  gap: 15px;
  font-size: 13px;
}

.version-info {
  color: #3498db;
}

.custom-info {
  color: #95a5a6;
}

.custom-info.custom {
  color: #e67e22;
}

.version-card {
  display: flex;
  align-items: center;
  gap: 15px;
}

.version-number {
  background: #3498db;
  color: white;
  padding: 4px 8px;
  border-radius: 4px;
  font-size: 12px;
  font-weight: bold;
}

.version-name {
  font-weight: 500;
  flex: 1;
}

.version-date {
  color: #95a5a6;
  font-size: 13px;
}

.no-data {
  text-align: center;
  color: #95a5a6;
  padding: 40px;
  background: #f8f9fa;
  border-radius: 8px;
}
</style>