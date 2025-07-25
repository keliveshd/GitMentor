<template>
  <div class="diff-viewer-page">
    <!-- 参数错误提示 -->
    <div v-if="paramError" class="param-error">
      <div class="error-content">
        <span class="error-icon">❌</span>
        <h4>参数错误</h4>
        <p>{{ paramError }}</p>
        <button @click="closeWindow" class="close-btn">关闭窗口</button>
      </div>
    </div>

    <!-- 正常的差异查看器 -->
    <DiffViewer
      v-else-if="filePath"
      :file-path="filePath"
      :diff-type="diffType"
      @close="closeWindow"
    />

    <!-- 加载状态 -->
    <div v-else class="loading-page">
      <div class="loading-content">
        <span class="loading-spinner">⏳</span>
        <p>正在初始化差异查看器...</p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRoute } from 'vue-router'
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow'
import DiffViewer from '../components/DiffViewer.vue'

/**
 * 差异查看器页面组件
 * 作者：Evilek
 * 编写日期：2025-07-23
 */

// 响应式数据
const route = useRoute()
const filePath = ref<string>('')
const diffType = ref<'WorkingTree' | 'Staged' | 'HeadToWorking'>('WorkingTree')
const paramError = ref<string>('')

/**
 * 关闭当前窗口
 */
const closeWindow = async () => {
  try {
    const window = getCurrentWebviewWindow()
    await window.close()
  } catch (error) {
    console.error('❌ [DiffViewerPage] 关闭窗口失败:', error)
  }
}

/**
 * 解析URL参数
 */
const parseParams = () => {
  try {
    console.log('🔍 [DiffViewerPage] 开始解析URL参数')
    console.log('🔍 [DiffViewerPage] 当前路由信息:', {
      path: route.path,
      query: route.query,
      fullPath: route.fullPath
    })

    // 从路由查询参数获取数据
    const routeFilePath = route.query.filePath as string
    const routeDiffType = route.query.diffType as string

    console.log('🔍 [DiffViewerPage] 提取的参数:', {
      routeFilePath,
      routeDiffType
    })

    if (!routeFilePath) {
      console.error('❌ [DiffViewerPage] 缺少文件路径参数')
      paramError.value = '缺少必需的文件路径参数'
      return
    }

    filePath.value = routeFilePath

    // 验证差异类型
    const validDiffTypes = ['WorkingTree', 'Staged', 'HeadToWorking']
    if (routeDiffType && validDiffTypes.includes(routeDiffType)) {
      diffType.value = routeDiffType as 'WorkingTree' | 'Staged' | 'HeadToWorking'
    } else {
      console.warn('⚠️ [DiffViewerPage] 无效的差异类型，使用默认值:', routeDiffType)
      diffType.value = 'WorkingTree' // 默认值
    }

    console.log('✅ [DiffViewerPage] 参数解析成功:', {
      filePath: filePath.value,
      diffType: diffType.value
    })
  } catch (error) {
    console.error('❌ [DiffViewerPage] 参数解析失败:', error)
    paramError.value = '参数解析失败: ' + (error instanceof Error ? error.message : '未知错误')
  }
}

/**
 * 设置窗口标题
 */
const setWindowTitle = async () => {
  if (filePath.value) {
    try {
      const window = getCurrentWebviewWindow()
      const fileName = filePath.value.split(/[/\\]/).pop() || filePath.value
      await window.setTitle(`差异查看 - ${fileName}`)
    } catch (error) {
      console.error('❌ [DiffViewerPage] 设置窗口标题失败:', error)
    }
  }
}

// 生命周期
onMounted(async () => {
  parseParams()
  await setWindowTitle()
})
</script>

<style scoped>
.diff-viewer-page {
  height: 100vh;
  width: 100vw;
  overflow: hidden;
  background: #ffffff;
}

.param-error,
.loading-page {
  height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 40px;
}

.error-content,
.loading-content {
  text-align: center;
  max-width: 400px;
}

.error-icon,
.loading-spinner {
  font-size: 48px;
  display: block;
  margin-bottom: 16px;
}

.loading-spinner {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}

.error-content h4 {
  margin: 0 0 8px 0;
  font-size: 18px;
  color: #24292e;
}

.error-content p,
.loading-content p {
  margin: 0 0 16px 0;
  color: #586069;
  line-height: 1.5;
}

.close-btn {
  padding: 8px 16px;
  border: 1px solid #d73a49;
  border-radius: 4px;
  background: #d73a49;
  color: white;
  cursor: pointer;
  font-size: 14px;
  transition: background 0.2s;
}

.close-btn:hover {
  background: #cb2431;
}

/* 深色主题支持 */
@media (prefers-color-scheme: dark) {
  .diff-viewer-page {
    background: #0d1117;
  }

  .error-content h4 {
    color: #e6edf3;
  }

  .error-content p,
  .loading-content p {
    color: #8b949e;
  }
}
</style>
