<template>
  <div class="repositories">
    <div class="page-header">
      <h1>仓库管理</h1>
      <el-button type="primary" :icon="Plus" @click="showAddDialog = true">
        添加仓库
      </el-button>
    </div>

    <!-- 仓库列表 -->
    <el-card>
      <el-table 
        :data="repositories" 
        v-loading="loading"
        empty-text="暂无仓库，点击上方按钮添加第一个仓库"
      >
        <el-table-column prop="name" label="仓库名称" min-width="200">
          <template #default="{ row }">
            <div class="repo-name">
              <el-icon><Folder /></el-icon>
              <span>{{ row.name }}</span>
            </div>
          </template>
        </el-table-column>
        
        <el-table-column prop="path" label="本地路径" min-width="300" show-overflow-tooltip />
        
        <el-table-column prop="remote_url" label="远程地址" min-width="300" show-overflow-tooltip>
          <template #default="{ row }">
            <span v-if="row.remote_url">{{ row.remote_url }}</span>
            <span v-else class="text-muted">无远程仓库</span>
          </template>
        </el-table-column>
        
        <el-table-column prop="last_analyzed" label="最后分析时间" width="180">
          <template #default="{ row }">
            <span v-if="row.last_analyzed">{{ formatDate(row.last_analyzed) }}</span>
            <span v-else class="text-muted">未分析</span>
          </template>
        </el-table-column>
        
        <el-table-column prop="total_commits" label="提交数" width="100" align="center">
          <template #default="{ row }">
            <el-tag>{{ row.total_commits || 0 }}</el-tag>
          </template>
        </el-table-column>
        
        <el-table-column label="操作" width="200" fixed="right">
          <template #default="{ row }">
            <el-button 
              type="primary" 
              size="small" 
              @click="analyzeRepository(row)"
              :loading="row.analyzing"
            >
              分析
            </el-button>
            <el-button 
              size="small" 
              @click="viewRepository(row)"
            >
              查看
            </el-button>
            <el-button 
              type="danger" 
              size="small" 
              @click="confirmDelete(row)"
            >
              删除
            </el-button>
          </template>
        </el-table-column>
      </el-table>
    </el-card>

    <!-- 添加仓库对话框 -->
    <el-dialog 
      v-model="showAddDialog" 
      title="添加Git仓库" 
      width="600px"
      @close="resetAddForm"
    >
      <el-form 
        ref="addFormRef" 
        :model="addForm" 
        :rules="addFormRules" 
        label-width="100px"
      >
        <el-form-item label="仓库路径" prop="path">
          <el-input 
            v-model="addForm.path" 
            placeholder="请选择Git仓库目录"
            readonly
          >
            <template #append>
              <el-button @click="selectDirectory">选择目录</el-button>
            </template>
          </el-input>
        </el-form-item>
        
        <el-form-item label="仓库名称" prop="name">
          <el-input 
            v-model="addForm.name" 
            placeholder="仓库名称（自动从路径提取）"
          />
        </el-form-item>
      </el-form>
      
      <template #footer>
        <el-button @click="showAddDialog = false">取消</el-button>
        <el-button 
          type="primary" 
          @click="addRepository"
          :loading="adding"
        >
          添加
        </el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup>
import { ref, reactive, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useAppStore } from '../stores/app'
import { Folder, Plus } from '@element-plus/icons-vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { open } from '@tauri-apps/plugin-dialog'

const router = useRouter()
const appStore = useAppStore()
const { repositories } = appStore

const loading = ref(false)
const adding = ref(false)
const showAddDialog = ref(false)
const addFormRef = ref()

const addForm = reactive({
  path: '',
  name: ''
})

const addFormRules = {
  path: [
    { required: true, message: '请选择仓库路径', trigger: 'blur' }
  ],
  name: [
    { required: true, message: '请输入仓库名称', trigger: 'blur' }
  ]
}

// 选择目录
const selectDirectory = async () => {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      title: '选择Git仓库目录'
    })
    
    if (selected) {
      addForm.path = selected
      // 自动提取仓库名称
      const pathParts = selected.split(/[/\\]/)
      addForm.name = pathParts[pathParts.length - 1]
    }
  } catch (error) {
    ElMessage.error('选择目录失败: ' + error.message)
  }
}

// 添加仓库
const addRepository = async () => {
  try {
    const valid = await addFormRef.value.validate()
    if (!valid) return
    
    adding.value = true
    
    // 检查是否为Git仓库
    // 这里将来会调用后端API验证
    
    const newRepo = {
      id: Date.now(),
      name: addForm.name,
      path: addForm.path,
      remote_url: null,
      created_at: new Date().toISOString(),
      last_analyzed: null,
      total_commits: 0
    }
    
    appStore.addRepository(newRepo)
    ElMessage.success('仓库添加成功')
    showAddDialog.value = false
    
  } catch (error) {
    ElMessage.error('添加仓库失败: ' + error.message)
  } finally {
    adding.value = false
  }
}

// 重置添加表单
const resetAddForm = () => {
  addForm.path = ''
  addForm.name = ''
  addFormRef.value?.clearValidate()
}

// 分析仓库
const analyzeRepository = async (repo) => {
  try {
    repo.analyzing = true
    ElMessage.info('开始分析仓库，请稍候...')
    
    // 这里将来会调用后端API进行分析
    await new Promise(resolve => setTimeout(resolve, 2000)) // 模拟分析过程
    
    repo.last_analyzed = new Date().toISOString()
    repo.total_commits = Math.floor(Math.random() * 1000) + 10 // 模拟数据
    
    ElMessage.success('仓库分析完成')
  } catch (error) {
    ElMessage.error('分析失败: ' + error.message)
  } finally {
    repo.analyzing = false
  }
}

// 查看仓库详情
const viewRepository = (repo) => {
  appStore.setCurrentRepository(repo)
  router.push(`/repository/${repo.id}`)
}

// 确认删除
const confirmDelete = async (repo) => {
  try {
    await ElMessageBox.confirm(
      `确定要删除仓库 "${repo.name}" 吗？此操作不可恢复。`,
      '确认删除',
      {
        confirmButtonText: '删除',
        cancelButtonText: '取消',
        type: 'warning'
      }
    )
    
    appStore.removeRepository(repo.id)
    ElMessage.success('仓库删除成功')
  } catch {
    // 用户取消删除
  }
}

// 格式化日期
const formatDate = (dateString) => {
  return new Date(dateString).toLocaleString('zh-CN')
}

onMounted(() => {
  // 页面加载时刷新仓库列表
  appStore.loadRepositories()
})
</script>

<style scoped>
.repositories {
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

.repo-name {
  display: flex;
  align-items: center;
  gap: 8px;
}

.text-muted {
  color: #909399;
}
</style>
