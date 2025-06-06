<template>
  <div class="analysis-results">
    <div class="page-header">
      <h1>分析结果</h1>
      <p>查看AI Agent双重审核的分析结果</p>
    </div>

    <!-- 统计概览 -->
    <div class="statistics-overview">
      <el-row :gutter="20">
        <el-col :span="6">
          <el-card class="stat-card">
            <div class="stat-content">
              <div class="stat-value">{{ statistics.total_tasks }}</div>
              <div class="stat-label">总任务数</div>
            </div>
          </el-card>
        </el-col>
        <el-col :span="6">
          <el-card class="stat-card">
            <div class="stat-content">
              <div class="stat-value">{{ statistics.approved_count }}</div>
              <div class="stat-label">通过审核</div>
            </div>
          </el-card>
        </el-col>
        <el-col :span="6">
          <el-card class="stat-card">
            <div class="stat-content">
              <div class="stat-value">{{ (statistics.approval_rate * 100).toFixed(1) }}%</div>
              <div class="stat-label">通过率</div>
            </div>
          </el-card>
        </el-col>
        <el-col :span="6">
          <el-card class="stat-card">
            <div class="stat-content">
              <div class="stat-value">{{ statistics.average_processing_time.toFixed(1) }}s</div>
              <div class="stat-label">平均处理时间</div>
            </div>
          </el-card>
        </el-col>
      </el-row>
    </div>

    <!-- 筛选和操作 -->
    <el-card class="filter-card">
      <el-row :gutter="20" align="middle">
        <el-col :span="6">
          <el-select v-model="filters.status" placeholder="筛选状态" clearable @change="loadResults">
            <el-option label="全部" value="" />
            <el-option label="已通过" value="approved" />
            <el-option label="已拒绝" value="rejected" />
            <el-option label="处理中" value="pending" />
            <el-option label="错误" value="error" />
          </el-select>
        </el-col>
        <el-col :span="6">
          <el-input 
            v-model="filters.search" 
            placeholder="搜索提交哈希或消息"
            clearable
            @input="debounceSearch"
          >
            <template #prefix>
              <el-icon><Search /></el-icon>
            </template>
          </el-input>
        </el-col>
        <el-col :span="12">
          <div class="filter-actions">
            <el-button @click="loadResults">
              <el-icon><Refresh /></el-icon>
              刷新
            </el-button>
            <el-button type="primary" @click="showQualityTrends = true">
              <el-icon><TrendCharts /></el-icon>
              质量趋势
            </el-button>
          </div>
        </el-col>
      </el-row>
    </el-card>

    <!-- 结果列表 -->
    <el-card class="results-card">
      <el-table :data="results" v-loading="loading" @row-click="viewResult">
        <el-table-column prop="task_id" label="任务ID" width="120">
          <template #default="scope">
            <span class="task-id">{{ scope.row.task_id.slice(0, 8) }}...</span>
          </template>
        </el-table-column>
        <el-table-column prop="commit_hash" label="提交哈希" width="120">
          <template #default="scope">
            <code class="commit-hash">{{ scope.row.commit_hash }}</code>
          </template>
        </el-table-column>
        <el-table-column prop="commit_message" label="提交消息" min-width="200">
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
        <el-table-column label="质量分数" width="120">
          <template #default="scope">
            <div v-if="scope.row.review_result">
              <el-progress 
                :percentage="Math.round(scope.row.review_result.overall_score * 100)"
                :color="getScoreColor(scope.row.review_result.overall_score)"
                :stroke-width="8"
              />
              <div class="score-text">{{ scope.row.review_result.overall_score.toFixed(3) }}</div>
            </div>
            <span v-else class="no-score">-</span>
          </template>
        </el-table-column>
        <el-table-column prop="created_at" label="创建时间" width="160">
          <template #default="scope">
            {{ formatTime(scope.row.created_at) }}
          </template>
        </el-table-column>
        <el-table-column label="操作" width="150">
          <template #default="scope">
            <el-button size="small" @click.stop="viewResult(scope.row)">
              查看
            </el-button>
            <el-button 
              v-if="scope.row.status === 'rejected' || scope.row.status === 'error'"
              size="small" 
              type="warning" 
              @click.stop="retryAnalysis(scope.row)"
            >
              重试
            </el-button>
          </template>
        </el-table-column>
      </el-table>

      <!-- 分页 -->
      <div class="pagination-wrapper">
        <el-pagination
          v-model:current-page="pagination.page"
          v-model:page-size="pagination.size"
          :page-sizes="[10, 20, 50, 100]"
          :total="pagination.total"
          layout="total, sizes, prev, pager, next, jumper"
          @size-change="loadResults"
          @current-change="loadResults"
        />
      </div>
    </el-card>

    <!-- 结果详情对话框 -->
    <el-dialog v-model="showDetailDialog" title="分析结果详情" width="80%" top="5vh">
      <div v-if="selectedResult" class="result-detail">
        <!-- 基本信息 -->
        <el-descriptions title="基本信息" :column="2" border>
          <el-descriptions-item label="任务ID">
            {{ selectedResult.task_id }}
          </el-descriptions-item>
          <el-descriptions-item label="提交哈希">
            <code>{{ selectedResult.commit_hash }}</code>
          </el-descriptions-item>
          <el-descriptions-item label="状态">
            <el-tag :type="getStatusType(selectedResult.status)">
              {{ getStatusText(selectedResult.status) }}
            </el-tag>
          </el-descriptions-item>
          <el-descriptions-item label="创建时间">
            {{ formatTime(selectedResult.created_at) }}
          </el-descriptions-item>
        </el-descriptions>

        <!-- 提交消息 -->
        <h3>提交消息</h3>
        <el-card class="commit-message-card">
          <pre>{{ selectedResult.commit_message }}</pre>
        </el-card>

        <!-- 分析结果 -->
        <div v-if="selectedResult.analysis_result">
          <h3>Analyzer Agent 分析结果</h3>
          <el-card>
            <el-descriptions :column="2" border>
              <el-descriptions-item label="总结">
                {{ selectedResult.analysis_result.summary }}
              </el-descriptions-item>
              <el-descriptions-item label="类型">
                <el-tag>{{ selectedResult.analysis_result.category }}</el-tag>
              </el-descriptions-item>
              <el-descriptions-item label="影响级别">
                <el-tag :type="getImpactType(selectedResult.analysis_result.impact_level)">
                  {{ selectedResult.analysis_result.impact_level }}
                </el-tag>
              </el-descriptions-item>
              <el-descriptions-item label="置信度">
                {{ selectedResult.analysis_result.confidence_score?.toFixed(3) }}
              </el-descriptions-item>
            </el-descriptions>
            <div v-if="selectedResult.analysis_result.description" style="margin-top: 15px;">
              <strong>详细描述：</strong>
              <p>{{ selectedResult.analysis_result.description }}</p>
            </div>
          </el-card>
        </div>

        <!-- 审核结果 -->
        <div v-if="selectedResult.review_result">
          <h3>Reviewer Agent 审核结果</h3>
          <el-card>
            <el-row :gutter="20">
              <el-col :span="12">
                <h4>质量评分</h4>
                <div class="quality-scores">
                  <div class="overall-score">
                    <span>综合评分：</span>
                    <el-progress 
                      :percentage="Math.round(selectedResult.review_result.overall_score * 100)"
                      :color="getScoreColor(selectedResult.review_result.overall_score)"
                    />
                  </div>
                  <div class="dimension-scores">
                    <div v-for="(score, dimension) in selectedResult.review_result.dimension_scores" :key="dimension">
                      <span>{{ getDimensionName(dimension) }}：</span>
                      <el-progress 
                        :percentage="Math.round(score * 100)"
                        :stroke-width="6"
                        :show-text="false"
                      />
                      <span class="score-value">{{ score.toFixed(3) }}</span>
                    </div>
                  </div>
                </div>
              </el-col>
              <el-col :span="12">
                <h4>审核意见</h4>
                <p>{{ selectedResult.review_result.feedback }}</p>
                
                <div v-if="selectedResult.review_result.suggestions?.length">
                  <h4>改进建议</h4>
                  <ul>
                    <li v-for="suggestion in selectedResult.review_result.suggestions" :key="suggestion">
                      {{ suggestion }}
                    </li>
                  </ul>
                </div>
              </el-col>
            </el-row>
          </el-card>
        </div>

        <!-- 错误信息 -->
        <div v-if="selectedResult.error_message">
          <h3>错误信息</h3>
          <el-alert type="error" :closable="false">
            {{ selectedResult.error_message }}
          </el-alert>
        </div>
      </div>
    </el-dialog>

    <!-- 质量趋势对话框 -->
    <el-dialog v-model="showQualityTrends" title="质量趋势" width="60%">
      <div class="quality-trends">
        <p>质量趋势图表功能开发中...</p>
        <!-- TODO: 添加图表组件 -->
      </div>
    </el-dialog>
  </div>
</template>

<script setup>
import { ref, reactive, onMounted, computed } from 'vue'
import { ElMessage } from 'element-plus'
import { Search, Refresh, TrendCharts } from '@element-plus/icons-vue'
import { analysisApi } from '@/api/analysis'

// 响应式数据
const loading = ref(false)
const results = ref([])
const selectedResult = ref(null)
const showDetailDialog = ref(false)
const showQualityTrends = ref(false)

// 统计数据
const statistics = ref({
  total_tasks: 0,
  approved_count: 0,
  rejected_count: 0,
  error_count: 0,
  pending_count: 0,
  approval_rate: 0,
  average_processing_time: 0
})

// 筛选条件
const filters = reactive({
  status: '',
  search: ''
})

// 分页
const pagination = reactive({
  page: 1,
  size: 20,
  total: 0
})

// 方法
const loadResults = async () => {
  loading.value = true
  try {
    const params = {
      status: filters.status,
      limit: pagination.size,
      offset: (pagination.page - 1) * pagination.size
    }
    
    const response = await analysisApi.getResults(params)
    results.value = response.data
    pagination.total = response.total || response.data.length
  } catch (error) {
    ElMessage.error('加载分析结果失败: ' + error.message)
  } finally {
    loading.value = false
  }
}

const loadStatistics = async () => {
  try {
    const response = await analysisApi.getStatistics()
    statistics.value = response.data
  } catch (error) {
    console.error('加载统计数据失败:', error)
  }
}

const viewResult = (result) => {
  selectedResult.value = result
  showDetailDialog.value = true
}

const retryAnalysis = async (result) => {
  try {
    await analysisApi.retryAnalysis(result.task_id)
    ElMessage.success('重试任务已提交')
    loadResults()
  } catch (error) {
    ElMessage.error('重试失败: ' + error.message)
  }
}

const debounceSearch = (() => {
  let timeout
  return () => {
    clearTimeout(timeout)
    timeout = setTimeout(() => {
      loadResults()
    }, 500)
  }
})()

// 辅助方法
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

const getScoreColor = (score) => {
  if (score >= 0.9) return '#67c23a'
  if (score >= 0.8) return '#e6a23c'
  if (score >= 0.7) return '#f56c6c'
  return '#909399'
}

const getImpactType = (level) => {
  const typeMap = {
    'low': 'success',
    'medium': 'warning',
    'high': 'danger'
  }
  return typeMap[level] || 'info'
}

const getDimensionName = (dimension) => {
  const nameMap = {
    'accuracy': '准确性',
    'completeness': '完整性',
    'consistency': '一致性',
    'clarity': '清晰度'
  }
  return nameMap[dimension] || dimension
}

const formatTime = (timestamp) => {
  return new Date(timestamp * 1000).toLocaleString()
}

// 生命周期
onMounted(() => {
  loadResults()
  loadStatistics()
})
</script>

<style scoped>
.analysis-results {
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

.statistics-overview {
  margin-bottom: 20px;
}

.stat-card {
  text-align: center;
}

.stat-content {
  padding: 10px;
}

.stat-value {
  font-size: 28px;
  font-weight: bold;
  color: #409eff;
  margin-bottom: 5px;
}

.stat-label {
  font-size: 14px;
  color: #666;
}

.filter-card {
  margin-bottom: 20px;
}

.filter-actions {
  text-align: right;
}

.results-card {
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

.pagination-wrapper {
  margin-top: 20px;
  text-align: center;
}

.result-detail h3 {
  margin: 20px 0 10px 0;
  color: #333;
}

.commit-message-card {
  margin-bottom: 20px;
}

.commit-message-card pre {
  margin: 0;
  white-space: pre-wrap;
  word-break: break-word;
}

.quality-scores {
  padding: 10px 0;
}

.overall-score {
  margin-bottom: 15px;
}

.dimension-scores > div {
  display: flex;
  align-items: center;
  margin-bottom: 8px;
}

.dimension-scores span:first-child {
  width: 80px;
  font-size: 14px;
}

.dimension-scores .el-progress {
  flex: 1;
  margin: 0 10px;
}

.score-value {
  width: 50px;
  text-align: right;
  font-size: 12px;
}
</style>
