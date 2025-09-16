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

        <!-- 语言设置卡片 -->
        <div class="language-settings-card">
          <div class="section-card">
            <h3>🌐 语言设置</h3>
            <div class="setting-item">
              <label for="language">提交信息语言</label>
              <select id="language" v-model="globalLanguage" @change="saveLanguageSettings" class="setting-select">
                <option value="Simplified Chinese">简体中文</option>
                <option value="Traditional Chinese">繁体中文</option>
                <option value="English">English</option>
                <option value="Japanese">日本語</option>
                <option value="Korean">한국어</option>
                <option value="French">Français</option>
                <option value="German">Deutsch</option>
                <option value="Spanish">Español</option>
                <option value="Russian">Русский</option>
                <option value="Portuguese">Português</option>
                <option value="Italian">Italiano</option>
                <option value="Dutch">Nederlands</option>
                <option value="Swedish">Svenska</option>
                <option value="Czech">Čeština</option>
                <option value="Polish">Polski</option>
                <option value="Turkish">Türkçe</option>
                <option value="Vietnamese">Tiếng Việt</option>
                <option value="Thai">ไทย</option>
                <option value="Indonesian">Bahasa Indonesia</option>
              </select>
            </div>
            <div class="setting-description">
              此设置将影响所有模板的默认语言配置，可在单个模板中覆盖
            </div>
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
                <span class="template-language">{{ getLanguageDisplayName(template.language) }}</span>
                <span class="template-config">
                  {{ template.enable_emoji ? '🎨' : '' }}
                  {{ template.enable_body ? '📄' : '' }}
                  {{ template.enable_merge_commit ? '🔗' : '' }}
                </span>
              </div>
            </div>
          </div>
        </div>

        <!-- AI分析模板管理 -->
        <div v-if="selectedMenu === 'ai_analysis'" class="template-section">
          <div class="ai-template-header">
            <div class="ai-description">
              <h4>🤖 AI分析模板配置</h4>
              <p>配置单体提交分析和日报汇总的AI提示模板</p>
            </div>
            <div class="ai-actions">
              <button @click="refreshAITemplates" class="refresh-btn" title="刷新模板">
                🔄 刷新
              </button>
              <button @click="resetToDefaultTemplates" class="reset-btn" title="重置为默认">
                🔄 重置默认
              </button>
            </div>
          </div>
          
          <!-- 单体分析模板 -->
          <div class="ai-template-group">
            <h5>📝 单体提交分析模板</h5>
            <div class="ai-template-list">
              <div v-for="template in commitAnalysisTemplates" :key="template.id" class="ai-template-item">
                <div class="ai-template-info">
                  <h6>{{ template.name }}</h6>
                  <p>{{ template.description }}</p>
                  <div class="ai-template-meta">
                    <span class="template-type">{{ getTemplateTypeName(template.template_type) }}</span>
                    <span class="template-version">v{{ template.version }}</span>
                  </div>
                </div>
                <div class="ai-template-actions">
                  <button @click="viewAITemplate(template)" class="view-btn" title="查看模板">
                    👁️ 查看
                  </button>
                  <button @click="editAITemplate(template)" class="edit-btn" title="编辑模板">
                    ✏️ 编辑
                  </button>
                </div>
              </div>
            </div>
          </div>
          
          <!-- 日报汇总模板 -->
          <div class="ai-template-group">
            <h5>📊 日报汇总模板</h5>
            <div class="ai-template-list">
              <div v-for="template in summaryTemplates" :key="template.id" class="ai-template-item">
                <div class="ai-template-info">
                  <h6>{{ template.name }}</h6>
                  <p>{{ template.description }}</p>
                  <div class="ai-template-meta">
                    <span class="template-type">{{ getTemplateTypeName(template.template_type) }}</span>
                    <span class="template-version">v{{ template.version }}</span>
                  </div>
                </div>
                <div class="ai-template-actions">
                  <button @click="viewAITemplate(template)" class="view-btn" title="查看模板">
                    👁️ 查看
                  </button>
                  <button @click="editAITemplate(template)" class="edit-btn" title="编辑模板">
                    ✏️ 编辑
                  </button>
                </div>
              </div>
            </div>
          </div>
          
          <!-- AI分析配置 -->
          <div class="ai-config-section">
            <h5>⚙️ AI分析配置</h5>
            <div class="config-form">
              <div class="config-item">
                <label>分析深度</label>
                <select v-model="aiConfig.depth" @change="saveAIConfig">
                  <option value="Simple">简单分析</option>
                  <option value="Detailed">详细分析</option>
                  <option value="Deep">深度分析</option>
                </select>
              </div>
              <div class="config-item">
                <label>启用代码审查</label>
                <label class="switch">
                  <input type="checkbox" v-model="aiConfig.enable_code_review" @change="saveAIConfig">
                  <span class="slider"></span>
                </label>
              </div>
              <div class="config-item">
                <label>最大代码长度</label>
                <input type="number" v-model="aiConfig.max_code_length" @change="saveAIConfig" min="1000" max="100000">
              </div>
              <div class="config-item">
                <label>超时时间（秒）</label>
                <input type="number" v-model="aiConfig.timeout_seconds" @change="saveAIConfig" min="10" max="300">
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
                <span class="template-language">{{ getLanguageDisplayName(template.language) }}</span>
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
          <!-- 基本信息区域 -->
          <div class="basic-info-section">
            <div class="form-row">
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
            </div>

            <div class="form-row">
              <div class="form-group">
                <label for="template-language">语言</label>
                <select id="template-language" v-model="editingTemplate.language" class="form-select">
                  <option value="FOLLOW_GLOBAL">跟随全局</option>
                  <option value="Simplified Chinese">简体中文</option>
                  <option value="Traditional Chinese">繁体中文</option>
                  <option value="English">English</option>
                  <option value="Japanese">日本語</option>
                  <option value="Korean">한국어</option>
                  <option value="French">Français</option>
                  <option value="German">Deutsch</option>
                  <option value="Spanish">Español</option>
                  <option value="Russian">Русский</option>
                  <option value="Portuguese">Português</option>
                  <option value="Italian">Italiano</option>
                  <option value="Dutch">Nederlands</option>
                  <option value="Swedish">Svenska</option>
                  <option value="Czech">Čeština</option>
                  <option value="Polish">Polski</option>
                  <option value="Turkish">Türkçe</option>
                  <option value="Vietnamese">Tiếng Việt</option>
                  <option value="Thai">ไทย</option>
                  <option value="Indonesian">Bahasa Indonesia</option>
                </select>
              </div>
              <div class="form-group">
                <label>配置选项</label>
                <div class="checkbox-group">
                  <label class="checkbox-item">
                    <input type="checkbox" v-model="editingTemplate.enable_emoji">
                    <span>启用Emoji</span>
                  </label>
                  <label class="checkbox-item">
                    <input type="checkbox" v-model="editingTemplate.enable_body">
                    <span>详细描述</span>
                  </label>
                  <label class="checkbox-item">
                    <input type="checkbox" v-model="editingTemplate.enable_merge_commit">
                    <span>合并提交</span>
                  </label>
                  <label class="checkbox-item">
                    <input type="checkbox" v-model="editingTemplate.use_recent_commits">
                    <span>参考历史</span>
                  </label>
                </div>
              </div>
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

          <!-- 两段式提示词编辑区域 -->
          <div class="two-phase-editor">
            <div class="phase-section">
              <h4>🔍 单文件分析阶段</h4>
              <p class="phase-description">用于分析单个文件的变更内容和意图</p>

              <div class="form-group">
                <label for="file-analysis-system">系统提示词</label>
                <textarea id="file-analysis-system" v-model="editingTemplate.file_analysis_system_prompt"
                  class="form-textarea" rows="6" placeholder="输入单文件分析的系统提示词..."></textarea>
              </div>

              <div class="form-group">
                <label for="file-analysis-user">用户提示词模板</label>
                <textarea id="file-analysis-user" v-model="editingTemplate.file_analysis_user_prompt"
                  class="form-textarea" rows="4" placeholder="输入单文件分析的用户提示词模板，可使用变量：{diff}, {staged_files}"></textarea>
              </div>
            </div>

            <div class="phase-section">
              <h4>📝 总结阶段</h4>
              <p class="phase-description">基于所有文件分析结果生成最终提交消息</p>

              <div class="form-group">
                <label for="summary-system">系统提示词</label>
                <textarea id="summary-system" v-model="editingTemplate.summary_system_prompt" class="form-textarea"
                  rows="6" placeholder="输入总结阶段的系统提示词..."></textarea>
              </div>

              <div class="form-group">
                <label for="summary-user">用户提示词模板</label>
                <textarea id="summary-user" v-model="editingTemplate.summary_user_prompt" class="form-textarea" rows="4"
                  placeholder="输入总结阶段的用户提示词模板，可使用变量：{diff}"></textarea>
              </div>
            </div>
          </div>

          <!-- 兼容性提示 -->
          <div class="compatibility-note">
            <p><strong>💡 提示：</strong>两段式提示词为新功能，如果留空将自动使用原有的系统提示词和用户提示词作为后备。</p>
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

// 接口定义（更新为两段式模板）
interface PromptTemplate {
  id: string
  name: string
  description: string

  // 原有字段（保持向后兼容）
  system_prompt: string
  user_prompt_template: string

  // 新增：两段式提示词字段
  file_analysis_system_prompt?: string
  file_analysis_user_prompt?: string
  summary_system_prompt?: string
  summary_user_prompt?: string

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
const globalLanguage = ref('Simplified Chinese')

// AI分析相关数据
const commitAnalysisTemplates = ref<any[]>([])
const summaryTemplates = ref<any[]>([])
const aiConfig = ref({
  model: 'gpt-4',
  depth: 'Detailed',
  enable_code_review: true,
  max_code_length: 50000,
  timeout_seconds: 60
})

// 编辑中的模板（更新为两段式）
const editingTemplate = ref<PromptTemplate>({
  id: '',
  name: '',
  description: '',
  system_prompt: '',
  user_prompt_template: '',

  // 新增：两段式提示词字段
  file_analysis_system_prompt: '',
  file_analysis_user_prompt: '',
  summary_system_prompt: '',
  summary_user_prompt: '',

  language: 'FOLLOW_GLOBAL',
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
  { key: 'custom', icon: '🎨', label: '自定义模板' },
  { key: 'ai_analysis', icon: '🤖', label: 'AI分析模板' }
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

const getLanguageDisplayName = (language: string) => {
  const languageMap: Record<string, string> = {
    'FOLLOW_GLOBAL': '跟随全局',
    'Simplified Chinese': '简体中文',
    'Traditional Chinese': '繁体中文',
    'English': 'English',
    'Japanese': '日本語',
    'Korean': '한국어',
    'French': 'Français',
    'German': 'Deutsch',
    'Spanish': 'Español',
    'Russian': 'Русский',
    'Portuguese': 'Português',
    'Italian': 'Italiano',
    'Dutch': 'Nederlands',
    'Swedish': 'Svenska',
    'Czech': 'Čeština',
    'Polish': 'Polski',
    'Turkish': 'Türkçe',
    'Vietnamese': 'Tiếng Việt',
    'Thai': 'ไทย',
    'Indonesian': 'Bahasa Indonesia',
    // 兼容旧版本
    'zh': '简体中文',
    'en': 'English'
  }
  return languageMap[language] || language
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

// 加载语言设置
const loadLanguageSettings = async () => {
  try {
    const config = await invoke('get_ai_config') as any
    globalLanguage.value = config.base.language || 'Simplified Chinese'
  } catch (error) {
    console.error('加载语言设置失败:', error)
    globalLanguage.value = 'Simplified Chinese'
  }
}

// 保存语言设置
const saveLanguageSettings = async () => {
  try {
    // 获取当前AI配置
    const config = await invoke('get_ai_config') as any
    // 更新语言设置
    config.base.language = globalLanguage.value
    // 保存配置
    await invoke('update_ai_config', { config })
    console.log('语言设置已保存:', globalLanguage.value)
  } catch (error) {
    console.error('保存语言设置失败:', error)
  }
}

// 编辑模板
const editTemplate = (template: PromptTemplate) => {
  // 确保所有两段式提示词字段都有默认值
  editingTemplate.value = {
    ...template,
    // 如果新字段为空或undefined，使用空字符串作为默认值
    file_analysis_system_prompt: template.file_analysis_system_prompt || '',
    file_analysis_user_prompt: template.file_analysis_user_prompt || '',
    summary_system_prompt: template.summary_system_prompt || '',
    summary_user_prompt: template.summary_user_prompt || ''
  }
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

    // 新增：两段式提示词字段的默认值
    file_analysis_system_prompt: '',
    file_analysis_user_prompt: '',
    summary_system_prompt: '',
    summary_user_prompt: '',

    language: 'FOLLOW_GLOBAL',
    max_tokens: 200,
    temperature: 0.3,
    enable_emoji: false,
    enable_body: true,
    enable_merge_commit: false,
    use_recent_commits: false,
  }
}

// AI分析方法
const loadAITemplates = async () => {
  try {
    const templates: any[] = await invoke('get_ai_analysis_templates')
    // 分类模板
    commitAnalysisTemplates.value = templates.filter((t: any) => 
      t.template_type?.CommitAnalysis
    )
    summaryTemplates.value = templates.filter((t: any) => 
      t.template_type?.DailySummary
    )
  } catch (error) {
    console.error('加载AI模板失败:', error)
  }
}

const loadAIConfig = async () => {
  try {
    const config: any = await invoke('get_ai_analysis_config')
    aiConfig.value = { ...aiConfig.value, ...config }
  } catch (error) {
    console.error('加载AI配置失败:', error)
  }
}

const saveAIConfig = async () => {
  try {
    await invoke('set_ai_analysis_config', { config: aiConfig.value })
    // 显示保存成功提示
  } catch (error) {
    console.error('保存AI配置失败:', error)
  }
}

const refreshAITemplates = () => {
  loadAITemplates()
}

const resetToDefaultTemplates = () => {
  // TODO: 实现重置为默认模板
  console.log('重置为默认模板')
}

const getTemplateTypeName = (templateType: any) => {
  if (templateType.CommitAnalysis) {
    const depth = templateType.CommitAnalysis.depth
    return `提交分析-${depth}`
  } else if (templateType.DailySummary) {
    return '日报汇总'
  }
  return '未知类型'
}

const viewAITemplate = (template: any) => {
  // TODO: 实现查看模板详情
  console.log('查看模板:', template)
}

const editAITemplate = (template: any) => {
  // TODO: 实现编辑模板
  console.log('编辑模板:', template)
}

// 生命周期
onMounted(() => {
  loadTemplates()
  loadLanguageSettings()
  loadAITemplates()
  loadAIConfig()
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

/* 语言设置卡片样式 */
.language-settings-card {
  margin: 20px 30px 0 30px;
}

.section-card {
  background: white;
  border-radius: 8px;
  padding: 20px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  border-left: 4px solid #2196f3;
}

.section-card h3 {
  margin: 0 0 15px 0;
  font-size: 16px;
  color: #333;
  display: flex;
  align-items: center;
  gap: 8px;
}

.setting-item {
  margin-bottom: 10px;
}

.setting-item label {
  display: block;
  margin-bottom: 6px;
  font-weight: 500;
  color: #333;
  font-size: 14px;
}

.setting-select {
  width: 200px;
  padding: 8px 12px;
  border: 1px solid #ddd;
  border-radius: 4px;
  font-size: 14px;
  background: white;
  transition: border-color 0.2s;
}

.setting-select:focus {
  outline: none;
  border-color: #2196f3;
}

.setting-description {
  font-size: 12px;
  color: #666;
  margin-top: 8px;
  line-height: 1.4;
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
  width: 95%;
  max-width: 1200px;
  max-height: 95vh;
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
  max-height: 80vh;
}

/* 基本信息区域样式 */
.basic-info-section {
  margin-bottom: 30px;
  padding: 20px;
  background: #f8f9fa;
  border-radius: 8px;
  border: 1px solid #e9ecef;
}

/* 两段式编辑器样式 */
.two-phase-editor {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 20px;
  margin-bottom: 20px;
}

.phase-section {
  padding: 20px;
  background: #ffffff;
  border: 2px solid #e9ecef;
  border-radius: 8px;
  transition: border-color 0.2s;
}

.phase-section:hover {
  border-color: #2196f3;
}

.phase-section h4 {
  margin: 0 0 8px 0;
  font-size: 16px;
  color: #333;
  display: flex;
  align-items: center;
  gap: 8px;
}

.phase-description {
  margin: 0 0 20px 0;
  font-size: 13px;
  color: #666;
  font-style: italic;
}

/* 兼容性提示样式 */
.compatibility-note {
  padding: 15px;
  background: #e3f2fd;
  border: 1px solid #bbdefb;
  border-radius: 6px;
  margin-bottom: 20px;
}

.compatibility-note p {
  margin: 0;
  font-size: 13px;
  color: #1565c0;
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

/* AI分析模板样式 */
.ai-template-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin: 20px 30px 0 30px;
  padding: 20px;
  background: white;
  border-radius: 8px;
  box-shadow: 0 2px 4px rgba(0,0,0,0.1);
}

.ai-description h4 {
  margin: 0 0 5px 0;
  color: #333;
}

.ai-description p {
  margin: 0;
  color: #666;
  font-size: 14px;
}

.ai-actions {
  display: flex;
  gap: 10px;
}

.refresh-btn, .reset-btn {
  padding: 6px 12px;
  border: 1px solid #ddd;
  border-radius: 4px;
  background: white;
  cursor: pointer;
  font-size: 12px;
  transition: all 0.2s;
}

.refresh-btn:hover {
  background: #f5f5f5;
  border-color: #2196f3;
  color: #2196f3;
}

.reset-btn:hover {
  background: #fff3cd;
  border-color: #ffc107;
  color: #856404;
}

.ai-template-group {
  margin: 20px 30px 0 30px;
  background: white;
  border-radius: 8px;
  padding: 20px;
  box-shadow: 0 2px 4px rgba(0,0,0,0.1);
}

.ai-template-group h5 {
  margin: 0 0 15px 0;
  color: #333;
  font-size: 16px;
}

.ai-template-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.ai-template-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 15px;
  background: #f8f9fa;
  border-radius: 6px;
  border: 1px solid #e9ecef;
  transition: all 0.2s;
}

.ai-template-item:hover {
  background: #e9ecef;
  border-color: #dee2e6;
}

.ai-template-info h6 {
  margin: 0 0 5px 0;
  color: #333;
  font-size: 14px;
}

.ai-template-info p {
  margin: 0 0 8px 0;
  color: #666;
  font-size: 13px;
}

.ai-template-meta {
  display: flex;
  gap: 10px;
  font-size: 12px;
}

.template-type {
  background: #e3f2fd;
  color: #1976d2;
  padding: 2px 8px;
  border-radius: 4px;
}

.template-version {
  background: #f3e5f5;
  color: #7b1fa2;
  padding: 2px 8px;
  border-radius: 4px;
}

.ai-template-actions {
  display: flex;
  gap: 8px;
}

.view-btn, .edit-btn {
  padding: 4px 8px;
  border: 1px solid #ddd;
  border-radius: 4px;
  background: white;
  cursor: pointer;
  font-size: 12px;
  transition: all 0.2s;
}

.view-btn:hover {
  background: #e3f2fd;
  border-color: #2196f3;
}

.edit-btn:hover {
  background: #fff3cd;
  border-color: #ffc107;
}

.ai-config-section {
  margin: 20px 30px 0 30px;
  background: white;
  border-radius: 8px;
  padding: 20px;
  box-shadow: 0 2px 4px rgba(0,0,0,0.1);
}

.ai-config-section h5 {
  margin: 0 0 15px 0;
  color: #333;
  font-size: 16px;
}

.config-form {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
  gap: 15px;
}

.config-item {
  display: flex;
  flex-direction: column;
  gap: 5px;
}

.config-item label {
  font-size: 14px;
  color: #555;
  font-weight: 500;
}

.config-item select,
.config-item input {
  padding: 8px;
  border: 1px solid #ddd;
  border-radius: 4px;
  font-size: 14px;
}

.config-item select:focus,
.config-item input:focus {
  outline: none;
  border-color: #2196f3;
}

/* 开关样式 */
.switch {
  position: relative;
  display: inline-block;
  width: 44px;
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
  background-color: #ccc;
  transition: .4s;
  border-radius: 24px;
}

.slider:before {
  position: absolute;
  content: "";
  height: 16px;
  width: 16px;
  left: 4px;
  bottom: 4px;
  background-color: white;
  transition: .4s;
  border-radius: 50%;
}

input:checked + .slider {
  background-color: #2196f3;
}

input:checked + .slider:before {
  transform: translateX(20px);
}
</style>
