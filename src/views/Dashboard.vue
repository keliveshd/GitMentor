<template>
  <div class="dashboard">
    <el-row :gutter="20">
      <!-- 统计卡片 -->
      <el-col :span="6">
        <el-card class="stat-card">
          <div class="stat-content">
            <div class="stat-icon repositories">
              <el-icon><Folder /></el-icon>
            </div>
            <div class="stat-info">
              <div class="stat-number">{{ repositories.length }}</div>
              <div class="stat-label">仓库总数</div>
            </div>
          </div>
        </el-card>
      </el-col>
      
      <el-col :span="6">
        <el-card class="stat-card">
          <div class="stat-content">
            <div class="stat-icon commits">
              <el-icon><List /></el-icon>
            </div>
            <div class="stat-info">
              <div class="stat-number">0</div>
              <div class="stat-label">总提交数</div>
            </div>
          </div>
        </el-card>
      </el-col>
      
      <el-col :span="6">
        <el-card class="stat-card">
          <div class="stat-content">
            <div class="stat-icon contributors">
              <el-icon><User /></el-icon>
            </div>
            <div class="stat-info">
              <div class="stat-number">0</div>
              <div class="stat-label">贡献者数</div>
            </div>
          </div>
        </el-card>
      </el-col>
      
      <el-col :span="6">
        <el-card class="stat-card">
          <div class="stat-content">
            <div class="stat-icon reports">
              <el-icon><Document /></el-icon>
            </div>
            <div class="stat-info">
              <div class="stat-number">0</div>
              <div class="stat-label">生成报告</div>
            </div>
          </div>
        </el-card>
      </el-col>
    </el-row>

    <el-row :gutter="20" style="margin-top: 20px;">
      <!-- 快速操作 -->
      <el-col :span="12">
        <el-card>
          <template #header>
            <div class="card-header">
              <span>快速操作</span>
            </div>
          </template>
          <div class="quick-actions">
            <el-button 
              type="primary" 
              :icon="Plus" 
              @click="$router.push('/repositories')"
              size="large"
            >
              添加仓库
            </el-button>
            <el-button 
              :icon="Refresh" 
              @click="refreshData"
              size="large"
            >
              刷新数据
            </el-button>
            <el-button 
              :icon="Download" 
              @click="$router.push('/reports')"
              size="large"
            >
              生成报告
            </el-button>
          </div>
        </el-card>
      </el-col>
      
      <!-- 最近活动 -->
      <el-col :span="12">
        <el-card>
          <template #header>
            <div class="card-header">
              <span>最近活动</span>
            </div>
          </template>
          <div class="recent-activities">
            <el-empty 
              v-if="recentActivities.length === 0"
              description="暂无活动记录"
              :image-size="100"
            />
            <div 
              v-else
              v-for="activity in recentActivities" 
              :key="activity.id"
              class="activity-item"
            >
              <el-icon class="activity-icon"><Clock /></el-icon>
              <div class="activity-content">
                <div class="activity-title">{{ activity.title }}</div>
                <div class="activity-time">{{ activity.time }}</div>
              </div>
            </div>
          </div>
        </el-card>
      </el-col>
    </el-row>

    <!-- 欢迎信息 -->
    <el-row style="margin-top: 20px;" v-if="repositories.length === 0">
      <el-col :span="24">
        <el-card>
          <div class="welcome-content">
            <el-icon class="welcome-icon"><Star /></el-icon>
            <h2>欢迎使用 GitMentor</h2>
            <p>GitMentor 是一个基于AI技术的Git提交分析工具，帮助您深入了解代码仓库的贡献情况和工作效率。</p>
            <div class="welcome-actions">
              <el-button 
                type="primary" 
                size="large"
                @click="$router.push('/repositories')"
              >
                开始使用 - 添加第一个仓库
              </el-button>
            </div>
          </div>
        </el-card>
      </el-col>
    </el-row>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'
import { useAppStore } from '../stores/app'
import { 
  Folder, 
  List, 
  User, 
  Document, 
  Plus, 
  Refresh, 
  Download, 
  Clock,
  Star
} from '@element-plus/icons-vue'
import { ElMessage } from 'element-plus'

const appStore = useAppStore()
const { repositories } = appStore

const recentActivities = ref([])

const refreshData = async () => {
  try {
    ElMessage.success('数据刷新成功')
    await appStore.loadRepositories()
  } catch (error) {
    ElMessage.error('数据刷新失败')
  }
}

onMounted(() => {
  // 加载最近活动
  loadRecentActivities()
})

const loadRecentActivities = () => {
  // 这里将来会从后端API加载
  recentActivities.value = []
}
</script>

<style scoped>
.dashboard {
  padding: 0;
}

.stat-card {
  margin-bottom: 20px;
}

.stat-content {
  display: flex;
  align-items: center;
}

.stat-icon {
  width: 60px;
  height: 60px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  margin-right: 16px;
  font-size: 24px;
  color: white;
}

.stat-icon.repositories {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
}

.stat-icon.commits {
  background: linear-gradient(135deg, #f093fb 0%, #f5576c 100%);
}

.stat-icon.contributors {
  background: linear-gradient(135deg, #4facfe 0%, #00f2fe 100%);
}

.stat-icon.reports {
  background: linear-gradient(135deg, #43e97b 0%, #38f9d7 100%);
}

.stat-info {
  flex: 1;
}

.stat-number {
  font-size: 32px;
  font-weight: bold;
  color: #303133;
  line-height: 1;
}

.stat-label {
  font-size: 14px;
  color: #909399;
  margin-top: 4px;
}

.card-header {
  font-weight: bold;
  color: #303133;
}

.quick-actions {
  display: flex;
  gap: 12px;
  flex-wrap: wrap;
}

.recent-activities {
  max-height: 300px;
  overflow-y: auto;
}

.activity-item {
  display: flex;
  align-items: center;
  padding: 12px 0;
  border-bottom: 1px solid #f0f0f0;
}

.activity-item:last-child {
  border-bottom: none;
}

.activity-icon {
  margin-right: 12px;
  color: #909399;
}

.activity-content {
  flex: 1;
}

.activity-title {
  font-size: 14px;
  color: #303133;
  margin-bottom: 4px;
}

.activity-time {
  font-size: 12px;
  color: #909399;
}

.welcome-content {
  text-align: center;
  padding: 40px 20px;
}

.welcome-icon {
  font-size: 64px;
  color: #409eff;
  margin-bottom: 20px;
}

.welcome-content h2 {
  color: #303133;
  margin-bottom: 16px;
}

.welcome-content p {
  color: #606266;
  margin-bottom: 32px;
  max-width: 600px;
  margin-left: auto;
  margin-right: auto;
  line-height: 1.6;
}

.welcome-actions {
  margin-top: 20px;
}
</style>
