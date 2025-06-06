<template>
  <div class="repository-config">
    <div class="page-header">
      <h1>仓库配置</h1>
      <p>管理多个Git仓库的配置和Agent分配</p>
    </div>

    <!-- 操作栏 -->
    <el-card class="action-card">
      <el-row :gutter="20" align="middle">
        <el-col :span="12">
          <el-input 
            v-model="searchText" 
            placeholder="搜索仓库名称或路径"
            clearable
          >
            <template #prefix>
              <el-icon><Search /></el-icon>
            </template>
          </el-input>
        </el-col>
        <el-col :span="12">
          <div class="action-buttons">
            <el-button @click="loadRepositories">
              <el-icon><Refresh /></el-icon>
              刷新
            </el-button>
            <el-button type="primary" @click="showCreateDialog = true">
              <el-icon><Plus /></el-icon>
              添加仓库
            </el-button>
          </div>
        </el-col>
      </el-row>
    </el-card>

    <!-- 仓库列表 -->
    <el-card class="repositories-card">
      <el-table :data="filteredRepositories" v-loading="loading">
        <el-table-column prop="name" label="仓库名称" width="200" />
        <el-table-column prop="path" label="路径" min-width="300">
          <template #default="scope">
            <code class="repo-path">{{ scope.row.path }}</code>
          </template>
        </el-table-column>
        <el-table-column prop="type" label="类型" width="100">
          <template #default="scope">
            <el-tag size="small">{{ scope.row.type }}</el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="enabled" label="状态" width="100">
          <template #default="scope">
            <el-switch 
              v-model="scope.row.enabled"
              @change="toggleRepository(scope.row)"
            />
          </template>
        </el-table-column>
        <el-table-column label="Agent配置" width="200">
          <template #default="scope">
            <div class="agent-config">
              <el-tag size="small" type="success">
                分析: {{ scope.row.agents?.analyzer || 'default' }}
              </el-tag>
              <el-tag size="small" type="warning" style="margin-top: 5px;">
                审核: {{ scope.row.agents?.reviewer || 'default' }}
              </el-tag>
            </div>
          </template>
        </el-table-column>
        <el-table-column label="操作" width="200">
          <template #default="scope">
            <el-button size="small" @click="editRepository(scope.row)">
              编辑
            </el-button>
            <el-button size="small" type="primary" @click="configureAgents(scope.row)">
              Agent
            </el-button>
            <el-button size="small" type="danger" @click="deleteRepository(scope.row)">
              删除
            </el-button>
          </template>
        </el-table-column>
      </el-table>
    </el-card>

    <!-- 创建/编辑仓库对话框 -->
    <el-dialog 
      v-model="showCreateDialog" 
      :title="editingRepository ? '编辑仓库' : '添加仓库'"
      width="60%"
    >
      <el-form :model="repositoryForm" :rules="repositoryRules" ref="repositoryFormRef" label-width="120px">
        <el-form-item label="仓库名称" prop="name">
          <el-input v-model="repositoryForm.name" placeholder="输入仓库名称" />
        </el-form-item>
        <el-form-item label="仓库路径" prop="path">
          <el-input v-model="repositoryForm.path" placeholder="输入Git仓库路径">
            <template #append>
              <el-button @click="validatePath" :loading="validatingPath">
                验证
              </el-button>
            </template>
          </el-input>
          <div v-if="pathValidation.message" class="path-validation">
            <el-alert 
              :type="pathValidation.valid ? 'success' : 'error'"
              :title="pathValidation.message"
              :closable="false"
              show-icon
            />
          </div>
        </el-form-item>
        <el-form-item label="仓库类型" prop="type">
          <el-select v-model="repositoryForm.type" placeholder="选择仓库类型">
            <el-option label="Git仓库" value="git" />
            <el-option label="本地目录" value="local" />
          </el-select>
        </el-form-item>
        <el-form-item label="启用状态">
          <el-switch v-model="repositoryForm.enabled" />
        </el-form-item>
        <el-form-item label="分析设置">
          <el-row :gutter="20">
            <el-col :span="12">
              <el-form-item label="自动分析">
                <el-switch v-model="repositoryForm.analysis_settings.auto_analysis" />
              </el-form-item>
            </el-col>
            <el-col :span="12">
              <el-form-item label="批处理大小">
                <el-input-number 
                  v-model="repositoryForm.analysis_settings.batch_size"
                  :min="1"
                  :max="100"
                />
              </el-form-item>
            </el-col>
          </el-row>
          <el-row :gutter="20">
            <el-col :span="12">
              <el-form-item label="重试限制">
                <el-input-number 
                  v-model="repositoryForm.analysis_settings.retry_limit"
                  :min="1"
                  :max="10"
                />
              </el-form-item>
            </el-col>
            <el-col :span="12">
              <el-form-item label="质量阈值">
                <el-input-number 
                  v-model="repositoryForm.analysis_settings.quality_threshold"
                  :min="0.5"
                  :max="1.0"
                  :step="0.05"
                  :precision="2"
                />
              </el-form-item>
            </el-col>
          </el-row>
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="showCreateDialog = false">取消</el-button>
        <el-button type="primary" @click="saveRepository" :loading="saving">
          {{ editingRepository ? '更新' : '创建' }}
        </el-button>
      </template>
    </el-dialog>

    <!-- Agent配置对话框 -->
    <el-dialog v-model="showAgentDialog" title="Agent配置" width="50%">
      <div v-if="selectedRepository">
        <h3>{{ selectedRepository.name }} - Agent配置</h3>
        <el-form :model="agentForm" label-width="120px">
          <el-form-item label="分析Agent">
            <el-select v-model="agentForm.analyzer" placeholder="选择分析Agent">
              <el-option 
                v-for="agent in availableAgents.analyzer" 
                :key="agent" 
                :label="agent" 
                :value="agent" 
              />
            </el-select>
          </el-form-item>
          <el-form-item label="审核Agent">
            <el-select v-model="agentForm.reviewer" placeholder="选择审核Agent">
              <el-option 
                v-for="agent in availableAgents.reviewer" 
                :key="agent" 
                :label="agent" 
                :value="agent" 
              />
            </el-select>
          </el-form-item>
        </el-form>
      </div>
      <template #footer>
        <el-button @click="showAgentDialog = false">取消</el-button>
        <el-button type="primary" @click="saveAgentConfig" :loading="savingAgent">
          保存
        </el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup>
import { ref, reactive, computed, onMounted } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { Search, Refresh, Plus } from '@element-plus/icons-vue'
import { repositoryConfigApi } from '@/api/repositoryConfig'

// 响应式数据
const loading = ref(false)
const saving = ref(false)
const savingAgent = ref(false)
const validatingPath = ref(false)
const repositories = ref([])
const searchText = ref('')
const showCreateDialog = ref(false)
const showAgentDialog = ref(false)
const editingRepository = ref(null)
const selectedRepository = ref(null)
const repositoryFormRef = ref(null)

// 表单数据
const repositoryForm = reactive({
  name: '',
  path: '',
  type: 'git',
  enabled: true,
  analysis_settings: {
    auto_analysis: true,
    batch_size: 10,
    retry_limit: 3,
    quality_threshold: 0.85
  }
})

const agentForm = reactive({
  analyzer: '',
  reviewer: ''
})

const availableAgents = ref({
  analyzer: [],
  reviewer: []
})

const pathValidation = reactive({
  valid: false,
  message: ''
})

// 表单验证规则
const repositoryRules = {
  name: [
    { required: true, message: '请输入仓库名称', trigger: 'blur' }
  ],
  path: [
    { required: true, message: '请输入仓库路径', trigger: 'blur' }
  ],
  type: [
    { required: true, message: '请选择仓库类型', trigger: 'change' }
  ]
}

// 计算属性
const filteredRepositories = computed(() => {
  if (!searchText.value) return repositories.value
  
  return repositories.value.filter(repo => 
    repo.name.toLowerCase().includes(searchText.value.toLowerCase()) ||
    repo.path.toLowerCase().includes(searchText.value.toLowerCase())
  )
})

// 方法
const loadRepositories = async () => {
  loading.value = true
  try {
    const response = await repositoryConfigApi.getRepositories()
    repositories.value = response.repositories
  } catch (error) {
    ElMessage.error('加载仓库配置失败: ' + error.message)
  } finally {
    loading.value = false
  }
}

const loadAvailableAgents = async () => {
  try {
    // 这里应该调用API获取可用的Agent列表
    // 暂时使用模拟数据
    availableAgents.value = {
      analyzer: ['default_analyzer', 'advanced_analyzer'],
      reviewer: ['default_reviewer', 'strict_reviewer']
    }
  } catch (error) {
    console.error('加载可用Agent失败:', error)
  }
}

const editRepository = (repository) => {
  editingRepository.value = repository
  Object.assign(repositoryForm, repository)
  showCreateDialog.value = true
}

const saveRepository = async () => {
  try {
    await repositoryFormRef.value.validate()
    
    saving.value = true
    
    if (editingRepository.value) {
      await repositoryConfigApi.updateRepository(editingRepository.value.name, repositoryForm)
      ElMessage.success('仓库配置更新成功')
    } else {
      await repositoryConfigApi.createRepository(repositoryForm)
      ElMessage.success('仓库配置创建成功')
    }
    
    showCreateDialog.value = false
    resetForm()
    loadRepositories()
  } catch (error) {
    ElMessage.error('保存仓库配置失败: ' + error.message)
  } finally {
    saving.value = false
  }
}

const deleteRepository = async (repository) => {
  try {
    await ElMessageBox.confirm(
      `确定要删除仓库配置 "${repository.name}" 吗？`,
      '确认删除',
      {
        confirmButtonText: '确定',
        cancelButtonText: '取消',
        type: 'warning'
      }
    )
    
    await repositoryConfigApi.deleteRepository(repository.name)
    ElMessage.success('仓库配置删除成功')
    loadRepositories()
  } catch (error) {
    if (error !== 'cancel') {
      ElMessage.error('删除仓库配置失败: ' + error.message)
    }
  }
}

const toggleRepository = async (repository) => {
  try {
    if (repository.enabled) {
      await repositoryConfigApi.enableRepository(repository.name)
      ElMessage.success('仓库已启用')
    } else {
      await repositoryConfigApi.disableRepository(repository.name)
      ElMessage.success('仓库已禁用')
    }
  } catch (error) {
    // 回滚状态
    repository.enabled = !repository.enabled
    ElMessage.error('切换仓库状态失败: ' + error.message)
  }
}

const configureAgents = (repository) => {
  selectedRepository.value = repository
  agentForm.analyzer = repository.agents?.analyzer || ''
  agentForm.reviewer = repository.agents?.reviewer || ''
  showAgentDialog.value = true
}

const saveAgentConfig = async () => {
  try {
    savingAgent.value = true
    
    await repositoryConfigApi.updateAgents(selectedRepository.value.name, {
      analyzer: agentForm.analyzer,
      reviewer: agentForm.reviewer
    })
    
    ElMessage.success('Agent配置保存成功')
    showAgentDialog.value = false
    loadRepositories()
  } catch (error) {
    ElMessage.error('保存Agent配置失败: ' + error.message)
  } finally {
    savingAgent.value = false
  }
}

const validatePath = async () => {
  if (!repositoryForm.path) {
    ElMessage.warning('请先输入仓库路径')
    return
  }
  
  validatingPath.value = true
  try {
    const response = await repositoryConfigApi.validatePath(repositoryForm.name || 'temp', repositoryForm.path)
    pathValidation.valid = response.valid
    pathValidation.message = response.message
  } catch (error) {
    pathValidation.valid = false
    pathValidation.message = '路径验证失败: ' + error.message
  } finally {
    validatingPath.value = false
  }
}

const resetForm = () => {
  editingRepository.value = null
  Object.assign(repositoryForm, {
    name: '',
    path: '',
    type: 'git',
    enabled: true,
    analysis_settings: {
      auto_analysis: true,
      batch_size: 10,
      retry_limit: 3,
      quality_threshold: 0.85
    }
  })
  pathValidation.valid = false
  pathValidation.message = ''
}

// 生命周期
onMounted(() => {
  loadRepositories()
  loadAvailableAgents()
})
</script>

<style scoped>
.repository-config {
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

.action-card {
  margin-bottom: 20px;
}

.action-buttons {
  text-align: right;
}

.repositories-card {
  margin-bottom: 20px;
}

.repo-path {
  font-family: monospace;
  font-size: 12px;
  background: #f5f5f5;
  padding: 2px 4px;
  border-radius: 3px;
}

.agent-config {
  display: flex;
  flex-direction: column;
  gap: 5px;
}

.path-validation {
  margin-top: 10px;
}
</style>
