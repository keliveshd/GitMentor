<template>
  <div class="whitespace-ignore-demo">
    <h2>忽略空白字符差异功能演示</h2>

    <!-- 功能说明 -->
    <div class="feature-description">
      <h3>功能特性</h3>
      <div class="feature-list">
        <div class="feature-item">
          <h4>🔍 忽略换行符差异</h4>
          <p>自动统一不同操作系统的换行符格式（Windows: \r\n → Unix: \n）</p>
        </div>

        <div class="feature-item">
          <h4>✂️ 移除行尾空白</h4>
          <p>忽略行尾的空格和制表符差异，专注于实际内容变更</p>
        </div>

        <div class="feature-item">
          <h4>📄 标准化文件结尾</h4>
          <p>移除文件末尾的多余空行，避免因编辑器设置导致的虚假差异</p>
        </div>
      </div>
    </div>

    <!-- 测试控制 -->
    <div class="test-controls">
      <div class="control-group">
        <label>测试文件:</label>
        <input v-model="testFile" placeholder="例如: LICENSE" />
      </div>

      <div class="control-group">
        <label>差异类型:</label>
        <select v-model="diffType">
          <option value="WorkingTree">工作区 vs 暂存区</option>
          <option value="Staged">暂存区 vs HEAD</option>
          <option value="HeadToWorking">HEAD vs 工作区</option>
        </select>
      </div>

      <button @click="openDiffViewer" :disabled="!testFile">
        打开差异查看器
      </button>
    </div>

    <!-- 使用说明 -->
    <div class="usage-instructions">
      <h3>使用说明</h3>
      <ol>
        <li>输入要查看差异的文件路径（如：LICENSE、README.md等）</li>
        <li>选择差异类型</li>
        <li>点击"打开差异查看器"</li>
        <li>在差异查看器中，注意工具栏中的 <strong>🔍</strong> 按钮（默认已启用）</li>
        <li>可以点击按钮切换"忽略空白字符差异"模式：
          <ul>
            <li><strong>开启状态（🔍）</strong>：忽略空白字符，只显示实际内容差异（默认）</li>
            <li><strong>关闭状态（👁️）</strong>：显示所有差异，包括空白字符</li>
          </ul>
        </li>
      </ol>
    </div>

    <!-- 常见场景 -->
    <div class="common-scenarios">
      <h3>常见应用场景</h3>

      <div class="scenario">
        <h4>📝 文档文件编辑</h4>
        <p>当编辑README.md、LICENSE等文档文件时，不同编辑器可能会改变换行符格式或添加行尾空格。启用此功能可以忽略这些格式差异，专注于内容变更。</p>
      </div>

      <div class="scenario">
        <h4>🔄 跨平台协作</h4>
        <p>团队成员使用不同操作系统（Windows、macOS、Linux）时，Git的autocrlf设置可能导致换行符差异。此功能可以消除这些干扰。</p>
      </div>

      <div class="scenario">
        <h4>🛠️ 代码格式化</h4>
        <p>当代码格式化工具只改变了缩进、空格或换行符时，可以使用此功能查看实际的逻辑变更。</p>
      </div>

      <div class="scenario">
        <h4>📋 配置文件对比</h4>
        <p>对比配置文件（如package.json、pom.xml）时，忽略格式差异，专注于配置项的实际变更。</p>
      </div>
    </div>

    <!-- 技术细节 -->
    <div class="technical-details">
      <h3>技术实现</h3>
      <div class="tech-item">
        <h4>换行符标准化</h4>
        <pre><code>content.replace(/\r\n/g, '\n').replace(/\r/g, '\n')</code></pre>
        <p>将Windows格式(\r\n)和Mac格式(\r)统一转换为Unix格式(\n)</p>
      </div>

      <div class="tech-item">
        <h4>行尾空白移除</h4>
        <pre><code>content.replace(/[ \t]+$/gm, '')</code></pre>
        <p>移除每行末尾的空格和制表符</p>
      </div>

      <div class="tech-item">
        <h4>文件结尾标准化</h4>
        <pre><code>content.replace(/\n+$/, '\n')</code></pre>
        <p>确保文件以单个换行符结尾</p>
      </div>
    </div>

    <!-- 差异查看器 -->
    <div v-if="showDiffViewer" class="diff-viewer-container">
      <div class="diff-viewer-header">
        <h3>差异查看器 - {{ testFile }}</h3>
        <div class="header-tips">
          <span class="tip">💡 默认已启用忽略空白字符模式（🔍），点击可切换到显示所有差异（👁️）</span>
        </div>
        <button @click="closeDiffViewer" class="close-btn">关闭</button>
      </div>

      <DiffViewer :filePath="testFile" :diffType="diffType" @close="closeDiffViewer" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import DiffViewer from './DiffViewer.vue'

/**
 * 忽略空白字符差异功能演示组件
 * 作者：Evilek
 * 编写日期：2025-07-22
 */

// 响应式数据
const testFile = ref('LICENSE')
const diffType = ref<'WorkingTree' | 'Staged' | 'HeadToWorking'>('WorkingTree')
const showDiffViewer = ref(false)

// 方法
const openDiffViewer = () => {
  if (testFile.value.trim()) {
    showDiffViewer.value = true
  }
}

const closeDiffViewer = () => {
  showDiffViewer.value = false
}
</script>

<style scoped>
.whitespace-ignore-demo {
  padding: 20px;
  max-width: 1200px;
  margin: 0 auto;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
}

.feature-description {
  margin-bottom: 30px;
  padding: 20px;
  background: #f6f8fa;
  border-radius: 8px;
}

.feature-list {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
  gap: 16px;
  margin-top: 16px;
}

.feature-item {
  padding: 16px;
  background: white;
  border-radius: 6px;
  border: 1px solid #d1d5da;
}

.feature-item h4 {
  margin: 0 0 8px 0;
  color: #24292e;
  font-size: 16px;
}

.feature-item p {
  margin: 0;
  color: #586069;
  font-size: 14px;
  line-height: 1.5;
}

.test-controls {
  display: flex;
  gap: 16px;
  align-items: end;
  margin-bottom: 30px;
  padding: 16px;
  border: 1px solid #d1d5da;
  border-radius: 8px;
  flex-wrap: wrap;
}

.control-group {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.control-group label {
  font-weight: 500;
  font-size: 14px;
  color: #24292e;
}

.control-group input,
.control-group select {
  padding: 8px 12px;
  border: 1px solid #d1d5da;
  border-radius: 4px;
  font-size: 14px;
  min-width: 200px;
}

button {
  padding: 8px 16px;
  background: #0366d6;
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 14px;
  transition: background 0.2s;
}

button:hover:not(:disabled) {
  background: #0256cc;
}

button:disabled {
  background: #94a3b8;
  cursor: not-allowed;
}

.usage-instructions,
.common-scenarios,
.technical-details {
  margin-bottom: 30px;
}

.usage-instructions h3,
.common-scenarios h3,
.technical-details h3 {
  color: #24292e;
  border-bottom: 1px solid #e1e4e8;
  padding-bottom: 8px;
}

.usage-instructions ol {
  padding-left: 20px;
}

.usage-instructions li {
  margin-bottom: 8px;
  line-height: 1.6;
}

.usage-instructions ul {
  margin-top: 8px;
}

.scenario {
  margin-bottom: 20px;
  padding: 16px;
  border-left: 4px solid #0366d6;
  background: #f1f8ff;
}

.scenario h4 {
  margin: 0 0 8px 0;
  color: #0366d6;
}

.scenario p {
  margin: 0;
  color: #586069;
  line-height: 1.5;
}

.tech-item {
  margin-bottom: 20px;
  padding: 16px;
  border: 1px solid #e1e4e8;
  border-radius: 6px;
}

.tech-item h4 {
  margin: 0 0 8px 0;
  color: #24292e;
}

.tech-item pre {
  background: #f6f8fa;
  padding: 8px 12px;
  border-radius: 4px;
  font-size: 13px;
  margin: 8px 0;
  overflow-x: auto;
}

.tech-item code {
  font-family: 'SFMono-Regular', Consolas, 'Liberation Mono', Menlo, monospace;
}

.tech-item p {
  margin: 8px 0 0 0;
  color: #586069;
  font-size: 14px;
}

.diff-viewer-container {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  z-index: 1000;
  background: white;
  display: flex;
  flex-direction: column;
}

.diff-viewer-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px;
  border-bottom: 1px solid #d1d5da;
  background: #f6f8fa;
  flex-shrink: 0;
}

.diff-viewer-header h3 {
  margin: 0;
  color: #24292e;
}

.header-tips {
  flex: 1;
  text-align: center;
}

.tip {
  font-size: 14px;
  color: #586069;
  background: #fff3cd;
  padding: 4px 8px;
  border-radius: 4px;
  border: 1px solid #ffeaa7;
}

.close-btn {
  background: #d73a49;
}

.close-btn:hover {
  background: #cb2431;
}

/* 响应式设计 */
@media (max-width: 768px) {
  .test-controls {
    flex-direction: column;
    align-items: stretch;
  }

  .control-group input,
  .control-group select {
    min-width: auto;
  }

  .feature-list {
    grid-template-columns: 1fr;
  }
}
</style>
