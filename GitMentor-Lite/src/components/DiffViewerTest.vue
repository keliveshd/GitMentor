<template>
  <div class="diff-viewer-test">
    <h2>DiffViewer修复验证</h2>
    
    <!-- 测试控制面板 -->
    <div class="test-controls">
      <div class="control-group">
        <label>测试文件:</label>
        <input v-model="testFile" placeholder="例如: backend/pom.xml" />
      </div>
      
      <div class="control-group">
        <label>差异类型:</label>
        <select v-model="diffType">
          <option value="WorkingTree">工作区 vs 暂存区</option>
          <option value="Staged">暂存区 vs HEAD</option>
          <option value="HeadToWorking">HEAD vs 工作区</option>
        </select>
      </div>
      
      <button @click="runTest" :disabled="!testFile || testing">
        {{ testing ? '测试中...' : '开始测试' }}
      </button>
      
      <button @click="clearResults">清空结果</button>
    </div>

    <!-- 测试结果 -->
    <div class="test-results" v-if="testResults.length > 0">
      <h3>测试结果</h3>
      <div class="result-list">
        <div 
          v-for="(result, index) in testResults" 
          :key="index" 
          :class="['result-item', result.status]"
        >
          <div class="result-header">
            <span class="result-time">{{ result.timestamp }}</span>
            <span class="result-status">{{ result.status.toUpperCase() }}</span>
            <span class="result-title">{{ result.title }}</span>
          </div>
          <div class="result-details" v-if="result.details">
            <pre>{{ result.details }}</pre>
          </div>
        </div>
      </div>
    </div>

    <!-- DiffViewer组件测试 -->
    <div v-if="showDiffViewer" class="diff-test-container">
      <div class="diff-test-header">
        <h3>DiffViewer组件测试</h3>
        <button @click="closeDiffViewer">关闭</button>
      </div>
      
      <DiffViewer
        :filePath="testFile"
        :diffType="diffType"
        @close="closeDiffViewer"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import DiffViewer from './DiffViewer.vue'

/**
 * DiffViewer修复验证组件
 * 作者：Evilek
 * 编写日期：2025-07-22
 */

interface TestResult {
  timestamp: string
  status: 'success' | 'warning' | 'error'
  title: string
  details?: string
}

// 响应式数据
const testFile = ref('backend/pom.xml')
const diffType = ref<'WorkingTree' | 'Staged' | 'HeadToWorking'>('WorkingTree')
const testing = ref(false)
const testResults = ref<TestResult[]>([])
const showDiffViewer = ref(false)

// 添加测试结果
const addResult = (status: 'success' | 'warning' | 'error', title: string, details?: string) => {
  testResults.value.push({
    timestamp: new Date().toLocaleTimeString(),
    status,
    title,
    details
  })
}

// 运行测试
const runTest = async () => {
  testing.value = true
  testResults.value = []
  
  try {
    addResult('success', '开始测试', `文件: ${testFile.value}, 类型: ${diffType.value}`)
    
    // 步骤1: 测试后端数据获取
    addResult('success', '步骤1: 测试后端数据获取')
    
    const result = await invoke('get_file_diff', {
      request: {
        file_path: testFile.value,
        diff_type: diffType.value
      }
    })
    
    if (!result) {
      addResult('error', '后端返回空数据')
      return
    }
    
    addResult('success', '后端数据获取成功', JSON.stringify({
      file_path: result.file_path,
      is_binary: result.is_binary,
      is_new_file: result.is_new_file,
      is_deleted_file: result.is_deleted_file,
      hunks_count: result.hunks?.length || 0,
      old_content_length: result.old_content?.length || 0,
      new_content_length: result.new_content?.length || 0
    }, null, 2))
    
    // 步骤2: 验证数据结构
    addResult('success', '步骤2: 验证数据结构')
    
    if (result.is_binary) {
      addResult('warning', '检测到二进制文件，跳过差异测试')
      return
    }
    
    if (!result.hunks || result.hunks.length === 0) {
      addResult('warning', '没有找到差异hunks')
      return
    }
    
    addResult('success', `找到 ${result.hunks.length} 个差异块`)
    
    // 步骤3: 验证hunks结构
    addResult('success', '步骤3: 验证hunks结构')
    
    let totalLines = 0
    let hasValidLines = true
    
    result.hunks.forEach((hunk, index) => {
      if (!hunk.lines || hunk.lines.length === 0) {
        addResult('warning', `Hunk ${index + 1} 没有lines数据`)
        hasValidLines = false
      } else {
        totalLines += hunk.lines.length
        
        // 检查前几行的内容
        const sampleLines = hunk.lines.slice(0, 3).map(line => ({
          type: line.line_type,
          content: line.content,
          contentLength: line.content?.length || 0
        }))
        
        addResult('success', `Hunk ${index + 1}: ${hunk.lines.length} 行`, JSON.stringify({
          old_start: hunk.old_start,
          old_lines: hunk.old_lines,
          new_start: hunk.new_start,
          new_lines: hunk.new_lines,
          sample_lines: sampleLines
        }, null, 2))
      }
    })
    
    if (!hasValidLines) {
      addResult('error', '部分hunks缺少有效的lines数据')
      return
    }
    
    addResult('success', `总共 ${totalLines} 行差异数据`)
    
    // 步骤4: 测试数据转换
    addResult('success', '步骤4: 测试数据转换')
    
    // 模拟DiffViewer的数据转换逻辑
    const hunks: string[] = []
    
    result.hunks.forEach((hunk, hunkIndex) => {
      const hunkHeader = `@@ -${hunk.old_start},${hunk.old_lines} +${hunk.new_start},${hunk.new_lines} @@`
      hunks.push(hunkHeader)
      
      hunk.lines.forEach((line) => {
        let prefix = ' '
        if (line.line_type === 'Delete') {
          prefix = '-'
        } else if (line.line_type === 'Insert') {
          prefix = '+'
        }
        
        const content = line.content ?? ''
        const diffLine = prefix + content
        hunks.push(diffLine)
      })
    })
    
    addResult('success', `转换生成 ${hunks.length} 行Git diff格式数据`)
    
    // 检查是否有异常行
    const emptyLines = hunks.filter(line => line.length <= 1)
    if (emptyLines.length > 0) {
      addResult('warning', `发现 ${emptyLines.length} 个可能的异常行（只有前缀字符）`)
    }
    
    // 步骤5: 测试组件渲染
    addResult('success', '步骤5: 准备组件渲染测试')
    
    const diffViewData = {
      oldFile: {
        fileName: result.old_file_name || result.file_path,
        content: result.old_content || '',
        fileLang: result.file_language || ''
      },
      newFile: {
        fileName: result.new_file_name || result.file_path,
        content: result.new_content || '',
        fileLang: result.file_language || ''
      },
      hunks
    }
    
    addResult('success', '数据转换完成，准备渲染DiffViewer组件')
    addResult('success', '测试完成！点击下方按钮查看实际渲染效果')
    
  } catch (error) {
    addResult('error', '测试失败', error instanceof Error ? error.message : String(error))
  } finally {
    testing.value = false
  }
}

// 显示DiffViewer
const showDiffViewerComponent = () => {
  showDiffViewer.value = true
}

// 关闭DiffViewer
const closeDiffViewer = () => {
  showDiffViewer.value = false
}

// 清空结果
const clearResults = () => {
  testResults.value = []
}
</script>

<style scoped>
.diff-viewer-test {
  padding: 20px;
  max-width: 1200px;
  margin: 0 auto;
}

.test-controls {
  display: flex;
  gap: 16px;
  align-items: end;
  margin-bottom: 30px;
  padding: 16px;
  background: #f6f8fa;
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

.test-results {
  margin-bottom: 30px;
}

.result-list {
  border: 1px solid #d1d5da;
  border-radius: 8px;
  overflow: hidden;
}

.result-item {
  border-bottom: 1px solid #e1e4e8;
}

.result-item:last-child {
  border-bottom: none;
}

.result-item.success {
  background: #f0fff4;
  border-left: 4px solid #28a745;
}

.result-item.warning {
  background: #fffbf0;
  border-left: 4px solid #ffc107;
}

.result-item.error {
  background: #ffeaea;
  border-left: 4px solid #d73a49;
}

.result-header {
  padding: 12px 16px;
  display: flex;
  gap: 12px;
  align-items: center;
}

.result-time {
  font-size: 12px;
  color: #586069;
  min-width: 80px;
}

.result-status {
  font-size: 12px;
  font-weight: bold;
  min-width: 60px;
}

.result-status.SUCCESS {
  color: #28a745;
}

.result-status.WARNING {
  color: #ffc107;
}

.result-status.ERROR {
  color: #d73a49;
}

.result-title {
  flex: 1;
  font-weight: 500;
}

.result-details {
  padding: 0 16px 12px 16px;
}

.result-details pre {
  background: #f6f8fa;
  padding: 8px;
  border-radius: 4px;
  font-size: 12px;
  overflow-x: auto;
  margin: 0;
}

.diff-test-container {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  z-index: 1000;
  background: white;
}

.diff-test-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px;
  border-bottom: 1px solid #d1d5da;
  background: #f6f8fa;
}

.diff-test-header h3 {
  margin: 0;
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
}
</style>
