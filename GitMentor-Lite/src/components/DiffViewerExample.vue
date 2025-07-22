<template>
  <div class="diff-viewer-example">
    <h2>差异查看器示例</h2>
    
    <!-- 选择组件类型 -->
    <div class="component-selector">
      <label>
        <input type="radio" v-model="selectedComponent" value="basic" />
        基础差异查看器 (DiffViewer)
      </label>
      <label>
        <input type="radio" v-model="selectedComponent" value="enhanced" />
        增强差异查看器 (EnhancedDiffViewer)
      </label>
    </div>

    <!-- 文件选择 -->
    <div class="file-selector">
      <label>
        文件路径:
        <input 
          type="text" 
          v-model="filePath" 
          placeholder="例如: src/main.rs"
          @keyup.enter="openDiffViewer"
        />
      </label>
      
      <label>
        差异类型:
        <select v-model="diffType">
          <option value="WorkingTree">工作区 vs 暂存区</option>
          <option value="Staged">暂存区 vs HEAD</option>
          <option value="HeadToWorking">HEAD vs 工作区</option>
        </select>
      </label>
      
      <button @click="openDiffViewer" :disabled="!filePath">
        打开差异查看器
      </button>
    </div>

    <!-- 功能说明 -->
    <div class="feature-description">
      <h3>功能特性对比</h3>
      
      <div class="comparison-table">
        <table>
          <thead>
            <tr>
              <th>功能</th>
              <th>基础版本</th>
              <th>增强版本</th>
            </tr>
          </thead>
          <tbody>
            <tr>
              <td>并排/统一视图切换</td>
              <td>✅</td>
              <td>✅</td>
            </tr>
            <tr>
              <td>代码换行控制</td>
              <td>✅</td>
              <td>✅</td>
            </tr>
            <tr>
              <td>语法高亮</td>
              <td>✅</td>
              <td>✅ (可切换)</td>
            </tr>
            <tr>
              <td>差异导航</td>
              <td>✅ (基础)</td>
              <td>✅ (增强)</td>
            </tr>
            <tr>
              <td>差异统计</td>
              <td>❌</td>
              <td>✅</td>
            </tr>
            <tr>
              <td>空白字符显示</td>
              <td>❌</td>
              <td>✅</td>
            </tr>
            <tr>
              <td>复制差异</td>
              <td>❌</td>
              <td>✅</td>
            </tr>
            <tr>
              <td>下载差异文件</td>
              <td>❌</td>
              <td>✅</td>
            </tr>
            <tr>
              <td>键盘快捷键</td>
              <td>❌</td>
              <td>✅</td>
            </tr>
            <tr>
              <td>二进制文件信息</td>
              <td>基础</td>
              <td>详细</td>
            </tr>
            <tr>
              <td>响应式设计</td>
              <td>基础</td>
              <td>完整</td>
            </tr>
          </tbody>
        </table>
      </div>

      <div class="keyboard-shortcuts">
        <h4>键盘快捷键 (增强版本)</h4>
        <ul>
          <li><kbd>Alt</kbd> + <kbd>↑</kbd> - 上一个差异</li>
          <li><kbd>Alt</kbd> + <kbd>↓</kbd> - 下一个差异</li>
          <li><kbd>Esc</kbd> - 关闭差异查看器</li>
        </ul>
      </div>
    </div>

    <!-- 差异查看器 -->
    <div v-if="showDiffViewer" class="diff-viewer-container">
      <!-- 基础版本 -->
      <DiffViewer
        v-if="selectedComponent === 'basic'"
        :filePath="filePath"
        :diffType="diffType"
        @close="closeDiffViewer"
      />
      
      <!-- 增强版本 -->
      <EnhancedDiffViewer
        v-if="selectedComponent === 'enhanced'"
        :filePath="filePath"
        :diffType="diffType"
        @close="closeDiffViewer"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import DiffViewer from './DiffViewer.vue'
import EnhancedDiffViewer from './EnhancedDiffViewer.vue'

/**
 * 差异查看器示例组件
 * 作者：Evilek
 * 编写日期：2025-07-22
 */

// 响应式数据
const selectedComponent = ref<'basic' | 'enhanced'>('enhanced')
const filePath = ref('')
const diffType = ref<'WorkingTree' | 'Staged' | 'HeadToWorking'>('WorkingTree')
const showDiffViewer = ref(false)

// 方法
const openDiffViewer = () => {
  if (filePath.value.trim()) {
    showDiffViewer.value = true
  }
}

const closeDiffViewer = () => {
  showDiffViewer.value = false
}
</script>

<style scoped>
.diff-viewer-example {
  padding: 20px;
  max-width: 1200px;
  margin: 0 auto;
}

.component-selector {
  margin-bottom: 20px;
  padding: 16px;
  background: #f6f8fa;
  border-radius: 8px;
}

.component-selector label {
  display: block;
  margin-bottom: 8px;
  cursor: pointer;
}

.component-selector input[type="radio"] {
  margin-right: 8px;
}

.file-selector {
  margin-bottom: 30px;
  padding: 16px;
  border: 1px solid #d1d5da;
  border-radius: 8px;
  display: flex;
  gap: 16px;
  align-items: end;
  flex-wrap: wrap;
}

.file-selector label {
  display: flex;
  flex-direction: column;
  gap: 4px;
  font-weight: 500;
}

.file-selector input,
.file-selector select {
  padding: 8px 12px;
  border: 1px solid #d1d5da;
  border-radius: 4px;
  font-size: 14px;
  min-width: 200px;
}

.file-selector button {
  padding: 8px 16px;
  background: #0366d6;
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 14px;
  transition: background 0.2s;
}

.file-selector button:hover:not(:disabled) {
  background: #0256cc;
}

.file-selector button:disabled {
  background: #94a3b8;
  cursor: not-allowed;
}

.feature-description {
  margin-bottom: 30px;
}

.comparison-table {
  overflow-x: auto;
  margin-bottom: 20px;
}

.comparison-table table {
  width: 100%;
  border-collapse: collapse;
  font-size: 14px;
}

.comparison-table th,
.comparison-table td {
  padding: 12px;
  text-align: left;
  border: 1px solid #d1d5da;
}

.comparison-table th {
  background: #f6f8fa;
  font-weight: 600;
}

.comparison-table tr:nth-child(even) {
  background: #f9f9f9;
}

.keyboard-shortcuts {
  padding: 16px;
  background: #f6f8fa;
  border-radius: 8px;
}

.keyboard-shortcuts ul {
  margin: 8px 0 0 0;
  padding-left: 20px;
}

.keyboard-shortcuts li {
  margin-bottom: 4px;
}

kbd {
  background: #e1e4e8;
  border: 1px solid #c6cbd1;
  border-radius: 3px;
  padding: 2px 6px;
  font-size: 12px;
  font-family: monospace;
}

.diff-viewer-container {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  z-index: 1000;
  background: white;
}

/* 响应式设计 */
@media (max-width: 768px) {
  .file-selector {
    flex-direction: column;
    align-items: stretch;
  }
  
  .file-selector input,
  .file-selector select {
    min-width: auto;
  }
}
</style>
