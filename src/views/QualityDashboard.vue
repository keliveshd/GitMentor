<template>
  <div class="quality-dashboard">
    <div class="page-header">
      <h1>质量仪表板</h1>
      <p>实时监控代码质量和分析趋势</p>
    </div>

    <!-- 关键指标卡片 -->
    <div class="metrics-overview">
      <el-row :gutter="20">
        <el-col :span="6">
          <el-card class="metric-card">
            <div class="metric-content">
              <div class="metric-icon">
                <el-icon size="32" color="#409eff"><TrendCharts /></el-icon>
              </div>
              <div class="metric-info">
                <div class="metric-value">{{ dashboardData.statistics?.total_tasks || 0 }}</div>
                <div class="metric-label">总分析任务</div>
              </div>
            </div>
          </el-card>
        </el-col>
        <el-col :span="6">
          <el-card class="metric-card">
            <div class="metric-content">
              <div class="metric-icon">
                <el-icon size="32" color="#67c23a"><SuccessFilled /></el-icon>
              </div>
              <div class="metric-info">
                <div class="metric-value">{{ (dashboardData.statistics?.approval_rate * 100).toFixed(1) }}%</div>
                <div class="metric-label">通过率</div>
              </div>
            </div>
          </el-card>
        </el-col>
        <el-col :span="6">
          <el-card class="metric-card">
            <div class="metric-content">
              <div class="metric-icon">
                <el-icon size="32" color="#e6a23c"><Star /></el-icon>
              </div>
              <div class="metric-info">
                <div class="metric-value">{{ dashboardData.quality_trends?.average_quality_score?.toFixed(3) || '0.000' }}</div>
                <div class="metric-label">平均质量分</div>
              </div>
            </div>
          </el-card>
        </el-col>
        <el-col :span="6">
          <el-card class="metric-card">
            <div class="metric-content">
              <div class="metric-icon">
                <el-icon size="32" color="#f56c6c"><Timer /></el-icon>
              </div>
              <div class="metric-info">
                <div class="metric-value">{{ dashboardData.statistics?.average_processing_time?.toFixed(1) || '0.0' }}s</div>
                <div class="metric-label">平均处理时间</div>
              </div>
            </div>
          </el-card>
        </el-col>
      </el-row>
    </div>

    <!-- 图表区域 -->
    <el-row :gutter="20">
      <!-- 质量趋势图 -->
      <el-col :span="12">
        <el-card class="chart-card">
          <template #header>
            <div class="card-header">
              <span>质量趋势</span>
              <el-select v-model="trendPeriod" size="small" @change="loadQualityTrends">
                <el-option label="24小时" value="24" />
                <el-option label="7天" value="168" />
                <el-option label="30天" value="720" />
              </el-select>
            </div>
          </template>
          <div class="chart-container">
            <div v-if="!qualityTrends.length" class="chart-placeholder">
              <el-empty description="暂无数据" />
            </div>
            <div v-else class="quality-trends-chart">
              <!-- 这里应该集成图表库，如ECharts -->
              <div class="trend-summary">
                <p><strong>样本数量:</strong> {{ dashboardData.quality_trends?.sample_count || 0 }}</p>
                <p><strong>平均质量分:</strong> {{ dashboardData.quality_trends?.average_quality_score?.toFixed(3) || '0.000' }}</p>
              </div>
              <div class="dimension-scores">
                <h4>维度评分</h4>
                <div v-for="(score, dimension) in dashboardData.quality_trends?.dimension_averages" :key="dimension">
                  <div class="dimension-item">
                    <span class="dimension-name">{{ getDimensionName(dimension) }}</span>
                    <el-progress 
                      :percentage="Math.round(score * 100)"
                      :color="getScoreColor(score)"
                    />
                    <span class="dimension-score">{{ score.toFixed(3) }}</span>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </el-card>
      </el-col>

      <!-- 状态分布图 -->
      <el-col :span="12">
        <el-card class="chart-card">
          <template #header>
            <span>任务状态分布</span>
          </template>
          <div class="chart-container">
            <div class="status-distribution">
              <div class="status-item">
                <div class="status-label">
                  <el-tag type="success">已通过</el-tag>
                  <span class="status-count">{{ dashboardData.statistics?.approved_count || 0 }}</span>
                </div>
                <el-progress 
                  :percentage="getStatusPercentage('approved')"
                  color="#67c23a"
                  :show-text="false"
                />
              </div>
              <div class="status-item">
                <div class="status-label">
                  <el-tag type="danger">已拒绝</el-tag>
                  <span class="status-count">{{ dashboardData.statistics?.rejected_count || 0 }}</span>
                </div>
                <el-progress 
                  :percentage="getStatusPercentage('rejected')"
                  color="#f56c6c"
                  :show-text="false"
                />
              </div>
              <div class="status-item">
                <div class="status-label">
                  <el-tag type="warning">处理中</el-tag>
                  <span class="status-count">{{ dashboardData.statistics?.pending_count || 0 }}</span>
                </div>
                <el-progress 
                  :percentage="getStatusPercentage('pending')"
                  color="#e6a23c"
                  :show-text="false"
                />
              </div>
              <div class="status-item">
                <div class="status-label">
                  <el-tag type="info">错误</el-tag>
                  <span class="status-count">{{ dashboardData.statistics?.error_count || 0 }}</span>
                </div>
                <el-progress 
                  :percentage="getStatusPercentage('error')"
                  color="#909399"
                  :show-text="false"
                />
              </div>
            </div>
          </div>
        </el-card>
      </el-col>
    </el-row>

    <!-- 最近分析结果 -->
    <el-card class="recent-results-card">
      <template #header>
        <div class="card-header">
          <span>最近分析结果</span>
          <el-button size="small" @click="loadDashboard">
            <el-icon><Refresh /></el-icon>
            刷新
          </el-button>
        </div>
      </template>
      <el-table :data="dashboardData.recent_results || []" v-loading="loading">
        <el-table-column prop="task_id" label="任务ID" width="120">
          <template #default="scope">
            <span class="task-id">{{ scope.row.task_id?.slice(0, 8) }}...</span>
          </template>
        </el-table-column>
        <el-table-column prop="commit_hash" label="提交" width="100">
          <template #default="scope">
            <code class="commit-hash">{{ scope.row.commit_hash }}</code>
          </template>
        </el-table-column>
        <el-table-column prop="commit_message" label="消息" min-width="200">
          <template #default="scope">
            <div class="commit-message">{{ scope.row.commit_message }}</div>
          </template>
        </el-table-column>
        <el-table-column prop="status" label="状态" width="100">
          <template #default="scope">
            <el-tag :type="getStatusType(scope.row.status)">
              {{ getStatusText(scope.row.status) }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column label="质量分" width="120">
          <template #default="scope">
            <div v-if="scope.row.quality_score !== null">
              <el-progress 
                :percentage="Math.round(scope.row.quality_score * 100)"
                :color="getScoreColor(scope.row.quality_score)"
                :stroke-width="8"
              />
              <div class="score-text">{{ scope.row.quality_score.toFixed(3) }}</div>
            </div>
            <span v-else class="no-score">-</span>
          </template>
        </el-table-column>
        <el-table-column prop="created_at" label="时间" width="160">
          <template #default="scope">
            {{ formatTime(scope.row.created_at) }}
          </template>
        </el-table-column>
      </el-table>
    </el-card>
  </div>
</template>

<script setup>
import { ref, reactive, onMounted, computed } from 'vue'
import { ElMessage } from 'element-plus'
import { 
  TrendCharts, 
  SuccessFilled, 
  Star, 
  Timer, 
  Refresh 
} from '@element-plus/icons-vue'
import { analysisApi } from '@/api/analysis'

// 响应式数据
const loading = ref(false)
const trendPeriod = ref('24')
const qualityTrends = ref([])
const dashboardData = ref({
  statistics: {},
  quality_trends: {},
  recent_results: []
})

// 方法
const loadDashboard = async () => {
  loading.value = true
  try {
    const response = await analysisApi.getDashboard()
    dashboardData.value = response
  } catch (error) {
    ElMessage.error('加载仪表板数据失败: ' + error.message)
  } finally {
    loading.value = false
  }
}

const loadQualityTrends = async () => {
  try {
    const response = await analysisApi.getQualityTrends(parseInt(trendPeriod.value))
    qualityTrends.value = response
  } catch (error) {
    console.error('加载质量趋势失败:', error)
  }
}

// 辅助方法
const getDimensionName = (dimension) => {
  const nameMap = {
    'accuracy': '准确性',
    'completeness': '完整性',
    'consistency': '一致性',
    'clarity': '清晰度'
  }
  return nameMap[dimension] || dimension
}

const getScoreColor = (score) => {
  if (score >= 0.9) return '#67c23a'
  if (score >= 0.8) return '#e6a23c'
  if (score >= 0.7) return '#f56c6c'
  return '#909399'
}

const getStatusType = (status) => {
  const statusMap = {
    'approved': 'success',
    'rejected': 'danger',
    'pending': 'warning',
    'error': 'danger'
  }
  return statusMap[status] || 'info'
}

const getStatusText = (status) => {
  const statusMap = {
    'approved': '已通过',
    'rejected': '已拒绝',
    'pending': '处理中',
    'error': '错误'
  }
  return statusMap[status] || status
}

const getStatusPercentage = (status) => {
  const stats = dashboardData.value.statistics
  if (!stats || stats.total_tasks === 0) return 0
  
  const count = stats[`${status}_count`] || 0
  return Math.round((count / stats.total_tasks) * 100)
}

const formatTime = (timestamp) => {
  return new Date(timestamp * 1000).toLocaleString()
}

// 生命周期
onMounted(() => {
  loadDashboard()
  loadQualityTrends()
  
  // 设置定时刷新
  const interval = setInterval(() => {
    loadDashboard()
  }, 30000) // 30秒刷新一次
  
  // 组件卸载时清除定时器
  onUnmounted(() => {
    clearInterval(interval)
  })
})
</script>

<style scoped>
.quality-dashboard {
  padding: 20px;
}

.page-header {
  margin-bottom: 20px;
}

.page-header h1 {
  margin: 0 0 8px 0;
  font-size: 24px;
  font-weight: 600;
}

.page-header p {
  margin: 0;
  color: #666;
}

.metrics-overview {
  margin-bottom: 20px;
}

.metric-card {
  height: 120px;
}

.metric-content {
  display: flex;
  align-items: center;
  height: 100%;
}

.metric-icon {
  margin-right: 15px;
}

.metric-info {
  flex: 1;
}

.metric-value {
  font-size: 28px;
  font-weight: bold;
  color: #333;
  margin-bottom: 5px;
}

.metric-label {
  font-size: 14px;
  color: #666;
}

.chart-card {
  margin-bottom: 20px;
  min-height: 400px;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.chart-container {
  height: 320px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.chart-placeholder {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
}

.quality-trends-chart {
  width: 100%;
  padding: 20px;
}

.trend-summary {
  margin-bottom: 20px;
  padding: 15px;
  background: #f5f7fa;
  border-radius: 4px;
}

.trend-summary p {
  margin: 5px 0;
}

.dimension-scores h4 {
  margin-bottom: 15px;
  color: #333;
}

.dimension-item {
  display: flex;
  align-items: center;
  margin-bottom: 10px;
}

.dimension-name {
  width: 80px;
  font-size: 14px;
}

.dimension-item .el-progress {
  flex: 1;
  margin: 0 15px;
}

.dimension-score {
  width: 60px;
  text-align: right;
  font-size: 12px;
  font-family: monospace;
}

.status-distribution {
  width: 100%;
  padding: 20px;
}

.status-item {
  margin-bottom: 20px;
}

.status-label {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.status-count {
  font-weight: bold;
  font-size: 16px;
}

.recent-results-card {
  margin-bottom: 20px;
}

.task-id {
  font-family: monospace;
  font-size: 12px;
}

.commit-hash {
  font-family: monospace;
  font-size: 12px;
  background: #f5f5f5;
  padding: 2px 4px;
  border-radius: 3px;
}

.commit-message {
  max-width: 300px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.score-text {
  text-align: center;
  font-size: 12px;
  margin-top: 5px;
}

.no-score {
  color: #999;
}
</style>
