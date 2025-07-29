<template>
  <div class="template-config-page">
    <div class="config-layout">
      <!-- 左侧菜单 -->
      <div class="config-sidebar">
        <div class="sidebar-header">
          <h2>📝 模板配置</h2>
        </div>
        <nav class="config-menu">
          <div v-for="item in menuItems" :key="item.key" :class="['menu-item', { active: selectedMenu === item.key }]"
            @click="selectedMenu = item.key">
            <span class="menu-icon">{{ item.icon }}</span>
            <span class="menu-label">{{ item.label }}</span>
          </div>
        </nav>
      </div>

      <!-- 右侧内容区域 -->
      <div class="config-content">
        <div class="content-header">
          <h3>{{ getCurrentMenuTitle() }}</h3>
          <div class="header-actions">
            <button v-if="selectedMenu === 'custom'" @click="showCreateDialog = true" class="create-btn">
              ➕ 新建模板
            </button>
          </div>
        </div>

        <!-- 默认模板管理 -->
        <div v-if="selectedMenu === 'default'" class="template-section">
          <div class="template-grid">
            <div v-for="template in defaultTemplates" :key="template.id" class="template-card">
              <div class="template-header">
                <h4>{{ template.name }}</h4>
                <div class="template-actions">
                  <button @click="editTemplate(template)" class="edit-btn" title="编辑">
                    ✏️
                  </button>
                </div>
              </div>
              <p class="template-description">{{ template.description }}</p>
              <div class="template-meta">
                <span class="template-language">{{ template.language === 'zh' ? '中文' : '英文' }}</span>
                <span class="template-config">
                  {{ template.enable_emoji ? '🎨' : '' }}
                  {{ template.enable_body ? '📄' : '' }}
                  {{ template.enable_merge_commit ? '🔗' : '' }}
                </span>
              </div>
            </div>
          </div>
        </div>

        <!-- 自定义模板管理 -->
        <div v-if="selectedMenu === 'custom'" class="template-section">
          <div v-if="customTemplates.length === 0" class="empty-state">
            <div class="empty-icon">📝</div>
            <h3>暂无自定义模板</h3>
            <p>点击"新建模板"创建您的第一个自定义模板</p>
          </div>
          <div v-else class="template-grid">
            <div v-for="template in customTemplates" :key="template.id" class="template-card custom">
              <div class="template-header">
                <h4>{{ template.name }}</h4>
                <div class="template-actions">
                  <button @click="editTemplate(template)" class="edit-btn" title="编辑">
                    ✏️
                  </button>
                  <button @click="deleteTemplate(template.id)" class="delete-btn" title="删除">
                    🗑️
                  </button>
                </div>
              </div>
              <p class="template-description">{{ template.description }}</p>
              <div class="template-meta">
                <span class="template-language">{{ template.language === 'zh' ? '中文' : '英文' }}</span>
                <span class="template-config">
                  {{ template.enable_emoji ? '🎨' : '' }}
                  {{ template.enable_body ? '📄' : '' }}
                  {{ template.enable_merge_commit ? '🔗' : '' }}
                </span>
                <span class="template-date">
                  {{ formatDate(template.created_at) }}
                </span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 创建/编辑模板对话框 -->
    <div v-if="showCreateDialog || showEditDialog" class="dialog-overlay" @click="closeDialogs">
      <div class="dialog-content" @click.stop>
        <div class="dialog-header">
          <h3>{{ showCreateDialog ? '新建模板' : '编辑模板' }}</h3>
          <button @click="closeDialogs" class="close-btn">✕</button>
        </div>

        <div class="dialog-body">
          <div class="form-group">
            <label for="template-name">模板名称</label>
            <input id="template-name" v-model="editingTemplate.name" type="text" class="form-input"
              placeholder="输入模板名称">
          </div>

          <div class="form-group">
            <label for="template-description">模板描述</label>
            <input id="template-description" v-model="editingTemplate.description" type="text" class="form-input"
              placeholder="输入模板描述">
          </div>

          <div class="form-group">
            <label for="template-language">语言</label>
            <select id="template-language" v-model="editingTemplate.language" class="form-select">
              <option value="zh">中文</option>
              <option value="en">英文</option>
            </select>
          </div>

          <div class="form-group">
            <label>配置选项</label>
            <div class="checkbox-group">
              <label class="checkbox-item">
                <input type="checkbox" v-model="editingTemplate.enable_emoji">
                <span>启用Emoji表情</span>
              </label>
              <label class="checkbox-item">
                <input type="checkbox" v-model="editingTemplate.enable_body">
                <span>启用详细描述</span>
              </label>
              <label class="checkbox-item">
                <input type="checkbox" v-model="editingTemplate.enable_merge_commit">
                <span>合并提交模式</span>
              </label>
              <label class="checkbox-item">
                <input type="checkbox" v-model="editingTemplate.use_recent_commits">
                <span>参考最近提交</span>
              </label>
            </div>
          </div>

          <div class="form-group">
            <label for="system-prompt">系统提示词</label>
            <textarea id="system-prompt" v-model="editingTemplate.system_prompt" class="form-textarea" rows="8"
              placeholder="输入系统提示词..."></textarea>
          </div>

          <div class="form-group">
            <label for="user-prompt">用户提示词模板</label>
            <textarea id="user-prompt" v-model="editingTemplate.user_prompt_template" class="form-textarea" rows="6"
              placeholder="输入用户提示词模板，可使用变量：{diff}, {staged_files}, {branch_name}"></textarea>
          </div>

          <div class="form-row">
            <div class="form-group">
              <label for="max-tokens">最大Token数</label>
              <input id="max-tokens" v-model.number="editingTemplate.max_tokens" type="number" class="form-input"
                min="50" max="4000">
            </div>
            <div class="form-group">
              <label for="temperature">温度值</label>
              <input id="temperature" v-model.number="editingTemplate.temperature" type="number" class="form-input"
                min="0" max="2" step="0.1">
            </div>
          </div>
        </div>

        <div class="dialog-footer">
          <button @click="closeDialogs" class="cancel-btn">取消</button>
          <button @click="saveTemplate" class="save-btn" :disabled="saving">
            {{ saving ? '保存中...' : '保存' }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

/**
 * 模板配置页面组件
 * 作者：Evilek
 * 编写日期：2025-01-29
 */

// 接口定义
interface PromptTemplate {
  id: string
  name: string
  description: string
  system_prompt: string
  user_prompt_template: string
  language: string
  max_tokens?: number
  temperature?: number
  enable_emoji?: boolean
  enable_body?: boolean
  enable_merge_commit?: boolean
  use_recent_commits?: boolean
  is_custom?: boolean
  created_at?: string
  updated_at?: string
}

// 响应式数据
const selectedMenu = ref('default')
const defaultTemplates = ref<PromptTemplate[]>([])
const customTemplates = ref<PromptTemplate[]>([])
const showCreateDialog = ref(false)
const showEditDialog = ref(false)
const saving = ref(false)

// 编辑中的模板
const editingTemplate = ref<PromptTemplate>({
  id: '',
  name: '',
  description: '',
  system_prompt: '',
  user_prompt_template: '',
  language: 'zh',
  max_tokens: 200,
  temperature: 0.3,
  enable_emoji: false,
  enable_body: true,
  enable_merge_commit: false,
  use_recent_commits: false,
})

// 菜单项配置
const menuItems = ref([
  { key: 'default', icon: '🏠', label: '默认模板' },
  { key: 'custom', icon: '🎨', label: '自定义模板' }
])

// 计算属性和方法
const getCurrentMenuTitle = () => {
  const item = menuItems.value.find(item => item.key === selectedMenu.value)
  return item ? item.label : '模板配置'
}

const formatDate = (dateStr?: string) => {
  if (!dateStr) return ''
  return new Date(dateStr).toLocaleDateString('zh-CN')
}

// 加载模板数据
const loadTemplates = async () => {
  try {
    const [defaultList, customList] = await Promise.all([
      invoke('get_default_templates') as Promise<PromptTemplate[]>,
      invoke('get_custom_templates') as Promise<PromptTemplate[]>
    ])

    defaultTemplates.value = defaultList
    customTemplates.value = customList
  } catch (error) {
    console.error('加载模板失败:', error)
  }
}

// 编辑模板
const editTemplate = (template: PromptTemplate) => {
  editingTemplate.value = { ...template }
  showEditDialog.value = true
}

// 删除模板
const deleteTemplate = async (templateId: string) => {
  if (!confirm('确定要删除这个模板吗？')) return

  try {
    await invoke('delete_template', { templateId })
    await loadTemplates()
  } catch (error) {
    console.error('删除模板失败:', error)
    alert('删除模板失败: ' + error)
  }
}

// 保存模板
const saveTemplate = async () => {
  if (!editingTemplate.value.name.trim()) {
    alert('请输入模板名称')
    return
  }

  try {
    saving.value = true

    if (showCreateDialog.value) {
      // 创建新模板
      editingTemplate.value.id = Date.now().toString()
      await invoke('create_custom_template', { template: editingTemplate.value })
    } else {
      // 更新现有模板
      await invoke('update_template', { template: editingTemplate.value })
    }

    await loadTemplates()
    closeDialogs()
  } catch (error) {
    console.error('保存模板失败:', error)
    alert('保存模板失败: ' + error)
  } finally {
    saving.value = false
  }
}

// 关闭对话框
const closeDialogs = () => {
  showCreateDialog.value = false
  showEditDialog.value = false
  editingTemplate.value = {
    id: '',
    name: '',
    description: '',
    system_prompt: '',
    user_prompt_template: '',
    language: 'zh',
    max_tokens: 200,
    temperature: 0.3,
    enable_emoji: false,
    enable_body: true,
    enable_merge_commit: false,
    use_recent_commits: false,
  }
}

// 生命周期
onMounted(() => {
  loadTemplates()
})
</script>

<style scoped>
.template-config-page {
  height: 100vh;
  background: #f5f5f5;
}

.config-layout {
  display: flex;
  height: 100%;
}

/* 左侧菜单样式 */
.config-sidebar {
  width: 250px;
  background: white;
  border-right: 1px solid #e0e0e0;
  display: flex;
  flex-direction: column;
}

.sidebar-header {
  padding: 20px;
  border-bottom: 1px solid #e0e0e0;
}

.sidebar-header h2 {
  margin: 0;
  font-size: 18px;
  color: #333;
}

.config-menu {
  flex: 1;
  padding: 10px 0;
}

.menu-item {
  display: flex;
  align-items: center;
  padding: 12px 20px;
  cursor: pointer;
  transition: all 0.2s;
  border-left: 3px solid transparent;
}

.menu-item:hover {
  background: #f8f9fa;
}

.menu-item.active {
  background: #e3f2fd;
  border-left-color: #2196f3;
  color: #1976d2;
}

.menu-icon {
  margin-right: 10px;
  font-size: 16px;
}

.menu-label {
  font-size: 14px;
  font-weight: 500;
}

/* 右侧内容区域样式 */
.config-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.content-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px 30px;
  background: white;
  border-bottom: 1px solid #e0e0e0;
}

.content-header h3 {
  margin: 0;
  font-size: 20px;
  color: #333;
}

.create-btn {
  background: #4caf50;
  color: white;
  border: none;
  padding: 8px 16px;
  border-radius: 6px;
  cursor: pointer;
  font-size: 14px;
  transition: background 0.2s;
}

.create-btn:hover {
  background: #45a049;
}

/* 模板区域样式 */
.template-section {
  flex: 1;
  padding: 20px 30px;
  overflow-y: auto;
}

.template-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(350px, 1fr));
  gap: 20px;
}

.template-card {
  background: white;
  border-radius: 8px;
  padding: 20px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  transition: transform 0.2s, box-shadow 0.2s;
}

.template-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.15);
}

.template-card.custom {
  border-left: 4px solid #ff9800;
}

.template-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 10px;
}

.template-header h4 {
  margin: 0;
  font-size: 16px;
  color: #333;
}

.template-actions {
  display: flex;
  gap: 8px;
}

.edit-btn,
.delete-btn {
  background: none;
  border: none;
  cursor: pointer;
  padding: 4px;
  border-radius: 4px;
  transition: background 0.2s;
}

.edit-btn:hover {
  background: #e3f2fd;
}

.delete-btn:hover {
  background: #ffebee;
}

.template-description {
  color: #666;
  font-size: 14px;
  margin-bottom: 15px;
  line-height: 1.4;
}

.template-meta {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 12px;
  color: #999;
}

.template-language {
  background: #e8f5e8;
  color: #2e7d32;
  padding: 2px 8px;
  border-radius: 12px;
}

.template-config {
  font-size: 14px;
}

.template-date {
  font-size: 11px;
}

/* 空状态样式 */
.empty-state {
  text-align: center;
  padding: 60px 20px;
  color: #666;
}

.empty-icon {
  font-size: 48px;
  margin-bottom: 20px;
}

.empty-state h3 {
  margin: 0 0 10px 0;
  font-size: 18px;
}

.empty-state p {
  margin: 0;
  font-size: 14px;
}

/* 对话框样式 */
.dialog-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.dialog-content {
  background: white;
  border-radius: 8px;
  width: 90%;
  max-width: 600px;
  max-height: 90vh;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.dialog-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px;
  border-bottom: 1px solid #e0e0e0;
}

.dialog-header h3 {
  margin: 0;
  font-size: 18px;
}

.close-btn {
  background: none;
  border: none;
  font-size: 18px;
  cursor: pointer;
  padding: 4px;
  border-radius: 4px;
  transition: background 0.2s;
}

.close-btn:hover {
  background: #f5f5f5;
}

.dialog-body {
  flex: 1;
  padding: 20px;
  overflow-y: auto;
}

.form-group {
  margin-bottom: 20px;
}

.form-group label {
  display: block;
  margin-bottom: 6px;
  font-weight: 500;
  color: #333;
}

.form-input,
.form-select,
.form-textarea {
  width: 100%;
  padding: 8px 12px;
  border: 1px solid #ddd;
  border-radius: 4px;
  font-size: 14px;
  transition: border-color 0.2s;
}

.form-input:focus,
.form-select:focus,
.form-textarea:focus {
  outline: none;
  border-color: #2196f3;
}

.form-textarea {
  resize: vertical;
  font-family: 'Courier New', monospace;
}

.form-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 15px;
}

.checkbox-group {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 10px;
}

.checkbox-item {
  display: flex;
  align-items: center;
  cursor: pointer;
}

.checkbox-item input {
  margin-right: 8px;
}

.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
  padding: 20px;
  border-top: 1px solid #e0e0e0;
}

.cancel-btn,
.save-btn {
  padding: 8px 20px;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 14px;
  transition: background 0.2s;
}

.cancel-btn {
  background: #f5f5f5;
  color: #666;
}

.cancel-btn:hover {
  background: #e0e0e0;
}

.save-btn {
  background: #2196f3;
  color: white;
}

.save-btn:hover:not(:disabled) {
  background: #1976d2;
}

.save-btn:disabled {
  background: #ccc;
  cursor: not-allowed;
}
</style>
