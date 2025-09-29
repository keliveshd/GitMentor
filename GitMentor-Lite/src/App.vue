<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import GitPanel from "./components/GitPanel.vue";
import FirstTimeSetupGuide from "./components/FirstTimeSetupGuide.vue";

const greetMsg = ref("");
const name = ref("");
const showTestSection = ref(false);
const showFirstTimeSetup = ref(false);
const appReady = ref(false);

async function greet() {
  greetMsg.value = await invoke("greet", { name: name.value });
}

/**
 * 检查是否需要首次启动引导
 * Author: Evilek, Date: 2025-01-09
 */
const checkFirstTimeSetup = async () => {
  try {
    const needsSetup = await invoke('check_first_time_setup') as boolean;
    showFirstTimeSetup.value = needsSetup;
    appReady.value = true;
  } catch (error) {
    console.error('检查首次启动状态失败:', error);
    // 如果检查失败，显示错误信息但仍继续
    showFirstTimeSetup.value = false;
    appReady.value = true;

    // 如果是连接错误，添加重试机制
    if (error && error.toString().includes('Tauri')) {
      console.log('Tauri连接失败，将在5秒后重试...');
      setTimeout(async () => {
        try {
          const needsSetup = await invoke('check_first_time_setup') as boolean;
          showFirstTimeSetup.value = needsSetup;
        } catch (retryError) {
          console.error('重试失败:', retryError);
        }
      }, 5000);
    }
  }
}

/**
 * 完成首次设置引导
 * Author: Evilek, Date: 2025-01-09
 */
const completeFirstTimeSetup = () => {
  showFirstTimeSetup.value = false;
}

// 生命周期
onMounted(async () => {
  // 等待Tauri初始化 - 增加等待时间以避免连接问题
  await new Promise(resolve => setTimeout(resolve, 2000));
  await checkFirstTimeSetup();
});
</script>

<template>
  <!-- 首次启动引导 -->
  <FirstTimeSetupGuide
    v-if="showFirstTimeSetup && appReady"
    @complete="completeFirstTimeSetup"
  />

  <!-- 主应用界面 -->
  <div v-if="appReady && !showFirstTimeSetup">
    <!-- 路由视图 - 支持多页面 -->
    <router-view v-slot="{ Component }">
      <component :is="Component" v-if="Component" />
      <!-- 默认主页面 -->
      <div v-else class="app">
        <!-- 主要内容区域 -->
        <main class="main-content">
          <!-- Git面板 -->
          <div class="git-panel-container">
            <GitPanel />
          </div>

          <!-- 测试区域（可选） -->
          <div class="test-section" v-if="showTestSection">
            <h3>🧪 Tauri连接测试</h3>
            <form class="test-form" @submit.prevent="greet">
              <input id="greet-input" v-model="name" placeholder="输入名称进行测试..." class="test-input" />
              <button type="submit" class="test-button">测试连接</button>
            </form>
            <p v-if="greetMsg" class="test-result">{{ greetMsg }}</p>
          </div>
        </main>

        <!-- 页脚 -->
        <footer class="app-footer">
          <p>GitMentor MVP v2.0 - 作者：Evilek | 基于 Tauri + Vue 3 + Rust</p>
          <button @click="showTestSection = !showTestSection" class="toggle-test-btn">
            {{ showTestSection ? '隐藏' : '显示' }}测试区域
          </button>
        </footer>
      </div>
    </router-view>
  </div>

  <!-- 加载状态 -->
  <div v-if="!appReady" class="loading-screen">
    <div class="loading-content">
      <div class="loading-spinner"></div>
      <p>正在初始化 GitMentor...</p>
    </div>
  </div>
</template>

<style scoped>
.app {
  min-height: 100vh;
  display: flex;
  flex-direction: column;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
}

.app-header {
  background: rgba(255, 255, 255, 0.95);
  backdrop-filter: blur(10px);
  padding: 20px;
  text-align: center;
  border-bottom: 1px solid rgba(255, 255, 255, 0.2);
  box-shadow: 0 2px 20px rgba(0, 0, 0, 0.1);
}

.app-header h1 {
  margin: 0;
  color: #2d3748;
  font-size: 2.5rem;
  font-weight: 700;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}

.subtitle {
  margin: 8px 0 0 0;
  color: #4a5568;
  font-size: 1.1rem;
  font-weight: 400;
}

.main-content {
  flex: 1;
  padding: 20px;
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.git-panel-container {
  background: rgba(255, 255, 255, 0.95);
  backdrop-filter: blur(10px);
  border-radius: 12px;
  padding: 20px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.1);
  border: 1px solid rgba(255, 255, 255, 0.2);
}

.test-section {
  background: rgba(255, 255, 255, 0.9);
  backdrop-filter: blur(10px);
  border-radius: 12px;
  padding: 20px;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.1);
  border: 1px solid rgba(255, 255, 255, 0.2);
}

.test-section h3 {
  margin: 0 0 15px 0;
  color: #2d3748;
}

.test-form {
  display: flex;
  gap: 10px;
  margin-bottom: 15px;
}

.test-input {
  flex: 1;
  padding: 10px 15px;
  border: 2px solid #e2e8f0;
  border-radius: 8px;
  font-size: 14px;
  transition: border-color 0.2s ease;
}

.test-input:focus {
  outline: none;
  border-color: #667eea;
}

.test-button {
  padding: 10px 20px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  border: none;
  border-radius: 8px;
  font-weight: 600;
  cursor: pointer;
  transition: transform 0.2s ease;
}

.test-button:hover {
  transform: translateY(-2px);
}

.test-result {
  margin: 0;
  padding: 10px;
  background: #f0fff4;
  border: 1px solid #9ae6b4;
  border-radius: 6px;
  color: #22543d;
}

.app-footer {
  background: rgba(255, 255, 255, 0.95);
  backdrop-filter: blur(10px);
  padding: 15px 20px;
  text-align: center;
  border-top: 1px solid rgba(255, 255, 255, 0.2);
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.app-footer p {
  margin: 0;
  color: #4a5568;
  font-size: 14px;
}

.toggle-test-btn {
  padding: 6px 12px;
  background: transparent;
  color: #667eea;
  border: 1px solid #667eea;
  border-radius: 6px;
  font-size: 12px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.toggle-test-btn:hover {
  background: #667eea;
  color: white;
}

/* 加载屏幕样式 - Author: Evilek, Date: 2025-01-09 */
.loading-screen {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.loading-content {
  text-align: center;
  color: white;
}

.loading-spinner {
  width: 40px;
  height: 40px;
  border: 4px solid rgba(255, 255, 255, 0.3);
  border-top: 4px solid white;
  border-radius: 50%;
  animation: spin 1s linear infinite;
  margin: 0 auto 16px;
}

@keyframes spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}

.loading-content p {
  font-size: 16px;
  margin: 0;
}
</style>

<style>
:root {
  font-family: 'Inter', 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
  font-size: 16px;
  line-height: 1.6;
  font-weight: 400;
  color: #2d3748;
  background-color: #f7fafc;
  font-synthesis: none;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  -webkit-text-size-adjust: 100%;
}

.container {
  margin: 0;
  padding-top: 10vh;
  display: flex;
  flex-direction: column;
  justify-content: center;
  text-align: center;
}

.logo {
  height: 6em;
  padding: 1.5em;
  will-change: filter;
  transition: 0.75s;
}

.logo.tauri:hover {
  filter: drop-shadow(0 0 2em #24c8db);
}

.row {
  display: flex;
  justify-content: center;
}

a {
  font-weight: 500;
  color: #646cff;
  text-decoration: inherit;
}

a:hover {
  color: #535bf2;
}

h1 {
  text-align: center;
}

input,
button {
  border-radius: 8px;
  border: 1px solid transparent;
  padding: 0.6em 1.2em;
  font-size: 1em;
  font-weight: 500;
  font-family: inherit;
  color: #0f0f0f;
  background-color: #ffffff;
  transition: border-color 0.25s;
  box-shadow: 0 2px 2px rgba(0, 0, 0, 0.2);
}

button {
  cursor: pointer;
}

button:hover {
  border-color: #396cd8;
}

button:active {
  border-color: #396cd8;
  background-color: #e8e8e8;
}

input,
button {
  outline: none;
}

#greet-input {
  margin-right: 5px;
}

/* GitMentor MVP Styles */
.section {
  margin: 20px 0;
  padding: 20px;
  border: 1px solid #ddd;
  border-radius: 8px;
}

.repo-path {
  margin: 15px 0;
  padding: 10px;
  background-color: #f8f9fa;
  border: 1px solid #dee2e6;
  border-radius: 4px;
  text-align: left;
}

.repo-path p {
  margin: 0;
  font-family: monospace;
  font-size: 14px;
  word-break: break-all;
}

.git-status {
  margin-top: 20px;
  text-align: left;
}

.file-list {
  margin: 10px 0;
}

.file-item {
  display: flex;
  align-items: center;
  padding: 8px;
  margin: 4px 0;
  border: 1px solid #eee;
  border-radius: 4px;
  cursor: pointer;
  transition: background-color 0.2s;
}

.file-item:hover {
  background-color: #f0f0f0;
}

.file-item.selected {
  background-color: #e3f2fd;
  border-color: #2196f3;
}

.file-status {
  padding: 2px 6px;
  border-radius: 3px;
  font-size: 12px;
  font-weight: bold;
  margin-right: 10px;
  min-width: 60px;
  text-align: center;
}

.file-status.modified {
  background-color: #fff3cd;
  color: #856404;
}

.file-status.added {
  background-color: #d4edda;
  color: #155724;
}

.file-status.deleted {
  background-color: #f8d7da;
  color: #721c24;
}

.file-status.untracked {
  background-color: #d1ecf1;
  color: #0c5460;
}

.file-path {
  font-family: monospace;
  font-size: 14px;
}

.commit-section {
  margin-top: 20px;
  padding: 15px;
  background-color: #f8f9fa;
  border-radius: 4px;
}

.commit-message textarea {
  width: 100%;
  margin-top: 10px;
  padding: 10px;
  border: 1px solid #ddd;
  border-radius: 4px;
  font-family: monospace;
  resize: vertical;
}

@media (prefers-color-scheme: dark) {
  :root {
    color: #f6f6f6;
    background-color: #2f2f2f;
  }

  a:hover {
    color: #24c8db;
  }

  input,
  button {
    color: #ffffff;
    background-color: #0f0f0f98;
  }

  button:active {
    background-color: #0f0f0f69;
  }
}
</style>