<template>
  <div class="settings">
    <div class="page-header">
      <h1>设置</h1>
    </div>

    <el-row :gutter="20">
      <el-col :span="16">
        <el-card>
          <template #header>
            <span>应用设置</span>
          </template>
          
          <el-form :model="config" label-width="120px">
            <el-form-item label="主题">
              <el-radio-group v-model="config.theme">
                <el-radio value="light">浅色主题</el-radio>
                <el-radio value="dark">深色主题</el-radio>
              </el-radio-group>
            </el-form-item>
            
            <el-form-item label="语言">
              <el-select v-model="config.language" style="width: 200px;">
                <el-option label="简体中文" value="zh-CN" />
                <el-option label="English" value="en-US" />
              </el-select>
            </el-form-item>
            
            <el-form-item label="自动保存">
              <el-switch v-model="config.autoSave" />
            </el-form-item>
            
            <el-form-item label="API端点">
              <el-input 
                v-model="config.apiEndpoint" 
                placeholder="后端API地址"
                style="width: 300px;"
              />
            </el-form-item>
            
            <el-form-item>
              <el-button type="primary" @click="saveSettings">保存设置</el-button>
              <el-button @click="resetSettings">重置</el-button>
            </el-form-item>
          </el-form>
        </el-card>
      </el-col>
      
      <el-col :span="8">
        <el-card>
          <template #header>
            <span>关于</span>
          </template>
          
          <div class="about-content">
            <div class="app-info">
              <h3>GitMentor</h3>
              <p>版本: 0.1.0</p>
              <p>基于AI技术的Git提交分析工具</p>
            </div>
            
            <div class="tech-stack">
              <h4>技术栈</h4>
              <ul>
                <li>Vue 3 + TypeScript</li>
                <li>Element Plus</li>
                <li>Tauri</li>
                <li>Python FastAPI</li>
              </ul>
            </div>
          </div>
        </el-card>
      </el-col>
    </el-row>
  </div>
</template>

<script setup>
import { reactive } from 'vue'
import { useAppStore } from '../stores/app'
import { ElMessage } from 'element-plus'

const appStore = useAppStore()
const config = reactive({ ...appStore.config })

const saveSettings = () => {
  try {
    Object.assign(appStore.config, config)
    appStore.saveConfig()
    ElMessage.success('设置保存成功')
  } catch (error) {
    ElMessage.error('保存设置失败')
  }
}

const resetSettings = () => {
  config.theme = 'light'
  config.language = 'zh-CN'
  config.autoSave = true
  config.apiEndpoint = 'http://localhost:8000'
  ElMessage.info('设置已重置')
}
</script>

<style scoped>
.settings {
  padding: 0;
}

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
}

.page-header h1 {
  margin: 0;
  color: #303133;
}

.about-content {
  text-align: center;
}

.app-info {
  margin-bottom: 30px;
}

.app-info h3 {
  color: #409eff;
  margin-bottom: 10px;
}

.app-info p {
  color: #606266;
  margin: 5px 0;
}

.tech-stack {
  text-align: left;
}

.tech-stack h4 {
  color: #303133;
  margin-bottom: 10px;
}

.tech-stack ul {
  list-style: none;
  padding: 0;
}

.tech-stack li {
  color: #606266;
  margin: 5px 0;
  padding-left: 16px;
  position: relative;
}

.tech-stack li::before {
  content: '•';
  color: #409eff;
  position: absolute;
  left: 0;
}
</style>
