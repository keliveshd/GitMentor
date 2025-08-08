<template>
  <div class="debug-settings">
    <div class="settings-header">
      <h3>🛠️ 开发者设置</h3>
      <p class="settings-description">这些设置主要用于开发和调试</p>
    </div>
    
    <div class="settings-section">
      <div class="setting-item">
        <div class="setting-info">
          <label class="setting-label">调试日志</label>
          <p class="setting-desc">在控制台显示详细的调试信息</p>
        </div>
        <div class="setting-control">
          <label class="switch">
            <input 
              type="checkbox" 
              v-model="debugSettings.debug_logs_enabled"
              @change="updateDebugLogs"
            >
            <span class="slider"></span>
          </label>
        </div>
      </div>
    </div>
    
    <div class="settings-footer">
      <p class="footer-note">
        💡 提示：调试日志会显示Git操作的详细执行过程，有助于诊断性能问题
      </p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import { useToast } from '@/composables/useToast'

const toast = useToast()

interface DebugSettings {
  debug_logs_enabled: boolean
}

const debugSettings = ref<DebugSettings>({
  debug_logs_enabled: false
})

// 加载调试设置
const loadDebugSettings = async () => {
  try {
    const settings = await invoke('get_debug_settings') as DebugSettings
    debugSettings.value = settings
    console.log('[DEBUG] 调试设置加载完成:', settings)
  } catch (error) {
    console.error('加载调试设置失败:', error)
    toast.error('加载调试设置失败: ' + error, '设置错误')
  }
}

// 更新调试日志设置
const updateDebugLogs = async () => {
  try {
    const message = await invoke('set_debug_logs_enabled', { 
      enabled: debugSettings.value.debug_logs_enabled 
    }) as string
    
    toast.success(message, '设置更新')
    console.log('[INFO]', message)
  } catch (error) {
    console.error('更新调试日志设置失败:', error)
    toast.error('更新设置失败: ' + error, '设置错误')
    // 回滚设置
    debugSettings.value.debug_logs_enabled = !debugSettings.value.debug_logs_enabled
  }
}

onMounted(() => {
  loadDebugSettings()
})
</script>

<style scoped>
.debug-settings {
  padding: 20px;
  max-width: 600px;
  margin: 0 auto;
}

.settings-header {
  margin-bottom: 30px;
  text-align: center;
}

.settings-header h3 {
  margin: 0 0 10px 0;
  color: var(--color-text);
  font-size: 1.5rem;
}

.settings-description {
  color: var(--color-text-secondary);
  margin: 0;
  font-size: 0.9rem;
}

.settings-section {
  background: var(--color-bg-secondary);
  border-radius: 8px;
  padding: 20px;
  margin-bottom: 20px;
}

.setting-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 15px 0;
}

.setting-info {
  flex: 1;
}

.setting-label {
  display: block;
  font-weight: 500;
  color: var(--color-text);
  margin-bottom: 5px;
}

.setting-desc {
  color: var(--color-text-secondary);
  font-size: 0.85rem;
  margin: 0;
}

.setting-control {
  margin-left: 20px;
}

/* 开关样式 */
.switch {
  position: relative;
  display: inline-block;
  width: 50px;
  height: 24px;
}

.switch input {
  opacity: 0;
  width: 0;
  height: 0;
}

.slider {
  position: absolute;
  cursor: pointer;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: var(--color-border);
  transition: 0.3s;
  border-radius: 24px;
}

.slider:before {
  position: absolute;
  content: "";
  height: 18px;
  width: 18px;
  left: 3px;
  bottom: 3px;
  background-color: white;
  transition: 0.3s;
  border-radius: 50%;
}

input:checked + .slider {
  background-color: var(--color-primary);
}

input:checked + .slider:before {
  transform: translateX(26px);
}

.settings-footer {
  background: var(--color-bg-tertiary);
  border-radius: 6px;
  padding: 15px;
  border-left: 4px solid var(--color-primary);
}

.footer-note {
  margin: 0;
  color: var(--color-text-secondary);
  font-size: 0.85rem;
  line-height: 1.4;
}
</style>
