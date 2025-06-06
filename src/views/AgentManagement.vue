<template>
  <div class="agent-management">
    <div class="page-header">
      <h1>Agent 管理</h1>
      <p>管理AI Agent的配置和状态</p>
    </div>

    <!-- Agent状态概览 -->
    <div class="agent-overview">
      <el-row :gutter="20">
        <el-col :span="6">
          <el-card class="stat-card">
            <div class="stat-content">
              <div class="stat-value">{{ totalAgents }}</div>
              <div class="stat-label">总Agent数</div>
            </div>
          </el-card>
        </el-col>
        <el-col :span="6">
          <el-card class="stat-card">
            <div class="stat-content">
              <div class="stat-value">{{ healthyAgents }}</div>
              <div class="stat-label">健康Agent</div>
            </div>
          </el-card>
        </el-col>
        <el-col :span="6">
          <el-card class="stat-card">
            <div class="stat-content">
              <div class="stat-value">{{ totalTasks }}</div>
              <div class="stat-label">处理任务数</div>
            </div>
          </el-card>
        </el-col>
        <el-col :span="6">
          <el-card class="stat-card">
            <div class="stat-content">
              <div class="stat-value">{{ successRate }}%</div>
              <div class="stat-label">成功率</div>
            </div>
          </el-card>
        </el-col>
      </el-row>
    </div>

    <!-- Agent列表 -->
    <el-card class="agent-list-card">
      <template #header>
        <div class="card-header">
          <span>Agent 列表</span>
          <div class="header-actions">
            <el-button type="primary" @click="refreshAgents">
              <el-icon><Refresh /></el-icon>
              刷新
            </el-button>
            <el-button type="success" @click="showCreateDialog = true">
              <el-icon><Plus /></el-icon>
              创建Agent
            </el-button>
          </div>
        </div>
      </template>

      <el-table :data="agents" v-loading="loading">
        <el-table-column prop="agent_id" label="Agent ID" width="200" />
        <el-table-column prop="status" label="状态" width="120">
          <template #default="scope">
            <el-tag :type="getStatusType(scope.row.status)">
              {{ getStatusText(scope.row.status) }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="capabilities" label="能力" width="300">
          <template #default="scope">
            <el-tag 
              v-for="capability in scope.row.capabilities.slice(0, 3)" 
              :key="capability"
              size="small"
              style="margin-right: 5px;"
            >
              {{ capability }}
            </el-tag>
            <span v-if="scope.row.capabilities.length > 3">
              +{{ scope.row.capabilities.length - 3 }}
            </span>
          </template>
        </el-table-column>
        <el-table-column prop="metrics" label="指标" width="200">
          <template #default="scope">
            <div class="metrics-cell">
              <div>任务: {{ scope.row.metrics.total_tasks }}</div>
              <div>成功率: {{ (scope.row.metrics.success_rate * 100).toFixed(1) }}%</div>
            </div>
          </template>
        </el-table-column>
        <el-table-column prop="config_version" label="配置版本" width="120" />
        <el-table-column label="操作" width="200">
          <template #default="scope">
            <el-button size="small" @click="viewAgent(scope.row)">
              查看
            </el-button>
            <el-button size="small" type="primary" @click="configureAgent(scope.row)">
              配置
            </el-button>
            <el-button size="small" type="danger" @click="removeAgent(scope.row)">
              删除
            </el-button>
          </template>
        </el-table-column>
      </el-table>
    </el-card>

    <!-- Agent详情对话框 -->
    <el-dialog v-model="showDetailDialog" title="Agent 详情" width="60%">
      <div v-if="selectedAgent" class="agent-detail">
        <el-descriptions :column="2" border>
          <el-descriptions-item label="Agent ID">
            {{ selectedAgent.agent_id }}
          </el-descriptions-item>
          <el-descriptions-item label="状态">
            <el-tag :type="getStatusType(selectedAgent.status)">
              {{ getStatusText(selectedAgent.status) }}
            </el-tag>
          </el-descriptions-item>
          <el-descriptions-item label="配置版本">
            {{ selectedAgent.config_version }}
          </el-descriptions-item>
          <el-descriptions-item label="总任务数">
            {{ selectedAgent.metrics.total_tasks }}
          </el-descriptions-item>
          <el-descriptions-item label="成功率">
            {{ (selectedAgent.metrics.success_rate * 100).toFixed(2) }}%
          </el-descriptions-item>
          <el-descriptions-item label="平均处理时间">
            {{ selectedAgent.metrics.average_processing_time.toFixed(2) }}s
          </el-descriptions-item>
        </el-descriptions>

        <h3 style="margin-top: 20px;">能力列表</h3>
        <el-tag 
          v-for="capability in selectedAgent.capabilities" 
          :key="capability"
          style="margin-right: 10px; margin-bottom: 10px;"
        >
          {{ capability }}
        </el-tag>
      </div>
    </el-dialog>

    <!-- 创建Agent对话框 -->
    <el-dialog v-model="showCreateDialog" title="创建 Agent" width="50%">
      <el-form :model="createForm" label-width="120px">
        <el-form-item label="Agent类型">
          <el-select v-model="createForm.agent_type" placeholder="选择Agent类型">
            <el-option label="分析器 (Analyzer)" value="analyzer" />
            <el-option label="审核器 (Reviewer)" value="reviewer" />
          </el-select>
        </el-form-item>
        <el-form-item label="LLM客户端">
          <el-select v-model="createForm.config.llm_client" placeholder="选择LLM客户端">
            <el-option 
              v-for="client in availableClients" 
              :key="client" 
              :label="client" 
              :value="client" 
            />
          </el-select>
        </el-form-item>
        <el-form-item label="最大Token数">
          <el-input-number 
            v-model="createForm.config.max_tokens" 
            :min="100" 
            :max="4000" 
            :step="100"
          />
        </el-form-item>
        <el-form-item label="温度参数">
          <el-input-number 
            v-model="createForm.config.temperature" 
            :min="0" 
            :max="2" 
            :step="0.1"
            :precision="1"
          />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="showCreateDialog = false">取消</el-button>
        <el-button type="primary" @click="createAgent" :loading="creating">
          创建
        </el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup>
import { ref, reactive, onMounted, computed } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { Refresh, Plus } from '@element-plus/icons-vue'
import { agentApi } from '@/api/agents'

// 响应式数据
const loading = ref(false)
const creating = ref(false)
const agents = ref([])
const selectedAgent = ref(null)
const showDetailDialog = ref(false)
const showCreateDialog = ref(false)
const availableClients = ref([])

// 创建表单
const createForm = reactive({
  agent_type: '',
  config: {
    llm_client: '',
    max_tokens: 1000,
    temperature: 0.3
  }
})

// 计算属性
const totalAgents = computed(() => agents.value.length)
const healthyAgents = computed(() => 
  agents.value.filter(agent => agent.status === 'idle').length
)
const totalTasks = computed(() => 
  agents.value.reduce((sum, agent) => sum + agent.metrics.total_tasks, 0)
)
const successRate = computed(() => {
  const total = agents.value.reduce((sum, agent) => sum + agent.metrics.total_tasks, 0)
  const successful = agents.value.reduce((sum, agent) => 
    sum + (agent.metrics.total_tasks * agent.metrics.success_rate), 0
  )
  return total > 0 ? (successful / total * 100).toFixed(1) : 0
})

// 方法
const loadAgents = async () => {
  loading.value = true
  try {
    const response = await agentApi.getAgents()
    agents.value = response.data
  } catch (error) {
    ElMessage.error('加载Agent列表失败: ' + error.message)
  } finally {
    loading.value = false
  }
}

const loadAvailableClients = async () => {
  try {
    const response = await agentApi.getLLMClients()
    availableClients.value = response.data.available_clients
  } catch (error) {
    console.error('加载LLM客户端失败:', error)
  }
}

const refreshAgents = () => {
  loadAgents()
}

const viewAgent = (agent) => {
  selectedAgent.value = agent
  showDetailDialog.value = true
}

const configureAgent = (agent) => {
  // TODO: 实现配置编辑功能
  ElMessage.info('配置功能开发中...')
}

const removeAgent = async (agent) => {
  try {
    await ElMessageBox.confirm(
      `确定要删除Agent "${agent.agent_id}" 吗？`,
      '确认删除',
      {
        confirmButtonText: '确定',
        cancelButtonText: '取消',
        type: 'warning'
      }
    )
    
    await agentApi.removeAgent(agent.agent_id)
    ElMessage.success('Agent删除成功')
    loadAgents()
  } catch (error) {
    if (error !== 'cancel') {
      ElMessage.error('删除Agent失败: ' + error.message)
    }
  }
}

const createAgent = async () => {
  creating.value = true
  try {
    await agentApi.createAgent(createForm)
    ElMessage.success('Agent创建成功')
    showCreateDialog.value = false
    loadAgents()
    
    // 重置表单
    createForm.agent_type = ''
    createForm.config.llm_client = ''
    createForm.config.max_tokens = 1000
    createForm.config.temperature = 0.3
  } catch (error) {
    ElMessage.error('创建Agent失败: ' + error.message)
  } finally {
    creating.value = false
  }
}

const getStatusType = (status) => {
  const statusMap = {
    'idle': 'success',
    'processing': 'warning',
    'error': 'danger',
    'stopped': 'info'
  }
  return statusMap[status] || 'info'
}

const getStatusText = (status) => {
  const statusMap = {
    'idle': '空闲',
    'processing': '处理中',
    'error': '错误',
    'stopped': '已停止'
  }
  return statusMap[status] || status
}

// 生命周期
onMounted(() => {
  loadAgents()
  loadAvailableClients()
})
</script>

<style scoped>
.agent-management {
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

.agent-overview {
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

.agent-list-card {
  margin-bottom: 20px;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.header-actions {
  display: flex;
  gap: 10px;
}

.metrics-cell {
  font-size: 12px;
  line-height: 1.4;
}

.agent-detail {
  padding: 10px 0;
}
</style>
