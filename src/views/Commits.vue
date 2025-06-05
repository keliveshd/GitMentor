<template>
  <div class="commits">
    <div class="page-header">
      <h1>提交历史</h1>
      <div class="header-actions">
        <el-select v-model="selectedRepo" placeholder="选择仓库" style="width: 200px;">
          <el-option
            v-for="repo in repositories"
            :key="repo.id"
            :label="repo.name"
            :value="repo.id"
          />
        </el-select>
        <el-button :icon="Refresh" @click="loadCommits">刷新</el-button>
      </div>
    </div>

    <el-card>
      <el-table 
        :data="commits" 
        v-loading="loading"
        empty-text="请先选择仓库查看提交历史"
      >
        <el-table-column prop="hash" label="提交哈希" width="120">
          <template #default="{ row }">
            <el-tag size="small">{{ row.hash.substring(0, 8) }}</el-tag>
          </template>
        </el-table-column>
        
        <el-table-column prop="message" label="提交信息" min-width="300" show-overflow-tooltip />
        
        <el-table-column prop="author_name" label="作者" width="120" />
        
        <el-table-column prop="commit_date" label="提交时间" width="180">
          <template #default="{ row }">
            {{ formatDate(row.commit_date) }}
          </template>
        </el-table-column>
        
        <el-table-column prop="files_changed" label="文件变更" width="100" align="center">
          <template #default="{ row }">
            <el-tag type="info">{{ row.files_changed }}</el-tag>
          </template>
        </el-table-column>
        
        <el-table-column label="代码变更" width="150" align="center">
          <template #default="{ row }">
            <span class="code-changes">
              <span class="additions">+{{ row.insertions }}</span>
              <span class="deletions">-{{ row.deletions }}</span>
            </span>
          </template>
        </el-table-column>
        
        <el-table-column label="操作" width="120" fixed="right">
          <template #default="{ row }">
            <el-button size="small" @click="viewCommitDetail(row)">
              详情
            </el-button>
          </template>
        </el-table-column>
      </el-table>
      
      <!-- 分页 -->
      <div class="pagination-wrapper">
        <el-pagination
          v-model:current-page="currentPage"
          v-model:page-size="pageSize"
          :page-sizes="[20, 50, 100]"
          :total="total"
          layout="total, sizes, prev, pager, next, jumper"
          @size-change="handleSizeChange"
          @current-change="handleCurrentChange"
        />
      </div>
    </el-card>
  </div>
</template>

<script setup>
import { ref, watch, onMounted } from 'vue'
import { useAppStore } from '../stores/app'
import { Refresh } from '@element-plus/icons-vue'
import { ElMessage } from 'element-plus'

const appStore = useAppStore()
const { repositories } = appStore

const loading = ref(false)
const selectedRepo = ref(null)
const commits = ref([])
const currentPage = ref(1)
const pageSize = ref(20)
const total = ref(0)

// 监听仓库选择变化
watch(selectedRepo, (newRepo) => {
  if (newRepo) {
    loadCommits()
  }
})

// 加载提交历史
const loadCommits = async () => {
  if (!selectedRepo.value) {
    ElMessage.warning('请先选择仓库')
    return
  }
  
  try {
    loading.value = true
    
    // 这里将来会调用后端API
    // 模拟数据
    const mockCommits = Array.from({ length: pageSize.value }, (_, index) => ({
      id: index + 1,
      hash: Math.random().toString(36).substring(2, 42),
      message: `提交信息 ${index + 1}`,
      author_name: `开发者${index % 3 + 1}`,
      author_email: `dev${index % 3 + 1}@example.com`,
      commit_date: new Date(Date.now() - Math.random() * 30 * 24 * 60 * 60 * 1000).toISOString(),
      files_changed: Math.floor(Math.random() * 10) + 1,
      insertions: Math.floor(Math.random() * 100) + 1,
      deletions: Math.floor(Math.random() * 50),
      ai_analysis: null,
      category: null
    }))
    
    commits.value = mockCommits
    total.value = 500 // 模拟总数
    
  } catch (error) {
    ElMessage.error('加载提交历史失败: ' + error.message)
  } finally {
    loading.value = false
  }
}

// 查看提交详情
const viewCommitDetail = (commit) => {
  ElMessage.info(`查看提交详情: ${commit.hash.substring(0, 8)}`)
  // 这里将来会打开提交详情对话框或跳转到详情页
}

// 分页处理
const handleSizeChange = (val) => {
  pageSize.value = val
  loadCommits()
}

const handleCurrentChange = (val) => {
  currentPage.value = val
  loadCommits()
}

// 格式化日期
const formatDate = (dateString) => {
  return new Date(dateString).toLocaleString('zh-CN')
}

onMounted(() => {
  // 如果有当前仓库，自动选择
  if (appStore.currentRepository) {
    selectedRepo.value = appStore.currentRepository.id
  }
})
</script>

<style scoped>
.commits {
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

.header-actions {
  display: flex;
  gap: 12px;
  align-items: center;
}

.code-changes {
  display: flex;
  gap: 8px;
}

.additions {
  color: #67c23a;
  font-weight: bold;
}

.deletions {
  color: #f56c6c;
  font-weight: bold;
}

.pagination-wrapper {
  margin-top: 20px;
  display: flex;
  justify-content: center;
}
</style>
