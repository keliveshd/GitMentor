# GitMentor MVP Demo 实现指南

## 项目概述

GitMentor MVP是一个专注于Git提交消息生成的最小可行版本，采用Tauri + Vue 3 + Rust架构。

## 核心功能

1. **文件夹选择器** - 选择Git仓库目录
2. **Git状态检测** - 显示文件变更状态
3. **文件选择界面** - 选择要提交的文件
4. **AI生成按钮** - 触发提交消息生成
5. **LLM服务集成** - 支持Ollama和OpenAI兼容API
6. **可配置提示词模板** - 外部配置文件管理

## 技术栈

- **前端**: Vue 3 + TypeScript + Element Plus
- **后端**: Rust + Tauri + git2 + reqwest + handlebars
- **配置**: TOML格式配置文件
- **模板引擎**: handlebars-rust

## 项目结构

```
GitMentor-MVP/
├── src-tauri/
│   ├── src/
│   │   ├── main.rs
│   │   ├── lib.rs
│   │   ├── commands/
│   │   │   ├── mod.rs
│   │   │   ├── git_commands.rs
│   │   │   └── llm_commands.rs
│   │   ├── core/
│   │   │   ├── mod.rs
│   │   │   ├── git_engine.rs
│   │   │   ├── llm_client.rs
│   │   │   └── template_engine.rs
│   │   └── types/
│   │       ├── mod.rs
│   │       └── git_types.rs
│   ├── Cargo.toml
│   └── tauri.conf.json
├── src/
│   ├── main.ts
│   ├── App.vue
│   ├── components/
│   │   ├── FolderSelector.vue
│   │   ├── GitStatus.vue
│   │   ├── FileSelector.vue
│   │   └── CommitGenerator.vue
│   ├── stores/
│   │   └── git.ts
│   └── types/
│       └── git.ts
├── config/
│   ├── app.toml
│   └── prompts.toml
└── package.json
```

## 实现步骤

### 步骤1: 项目初始化

```bash
# 创建Tauri项目
npm create tauri-app@latest GitMentor-MVP --template vue-ts
cd GitMentor-MVP

# 安装前端依赖
npm install element-plus @element-plus/icons-vue pinia

# 配置Rust依赖
```

### 步骤2: Rust后端核心实现

#### Cargo.toml配置
```toml
[package]
name = "gitmentor-mvp"
version = "0.1.0"
edition = "2021"

[dependencies]
tauri = { version = "2.0", features = ["shell-open", "dialog-open"] }
git2 = "0.18"
reqwest = { version = "0.11", features = ["json"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.0", features = ["full"] }
anyhow = "1.0"
handlebars = "4.5"
toml = "0.8"
```

#### Git类型定义 (src-tauri/src/types/git_types.rs)
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileStatus {
    pub path: String,
    pub status: String, // "modified", "added", "deleted", "untracked"
    pub selected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitStatusResult {
    pub files: Vec<FileStatus>,
    pub branch: String,
    pub has_changes: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitRequest {
    pub selected_files: Vec<String>,
    pub additional_context: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitMessageResult {
    pub message: String,
    pub confidence: f32,
    pub processing_time_ms: u64,
}
```

#### Git引擎实现 (src-tauri/src/core/git_engine.rs)
```rust
use git2::{Repository, Status, StatusOptions};
use anyhow::{Result, anyhow};
use crate::types::git_types::{FileStatus, GitStatusResult};

pub struct GitEngine {
    repo_path: Option<String>,
}

impl GitEngine {
    pub fn new() -> Self {
        Self { repo_path: None }
    }

    pub fn open_repository(&mut self, path: &str) -> Result<()> {
        let repo = Repository::open(path)?;
        self.repo_path = Some(path.to_string());
        Ok(())
    }

    pub fn get_status(&self) -> Result<GitStatusResult> {
        let repo_path = self.repo_path.as_ref()
            .ok_or_else(|| anyhow!("No repository opened"))?;
        
        let repo = Repository::open(repo_path)?;
        
        // 获取当前分支
        let head = repo.head()?;
        let branch = head.shorthand().unwrap_or("unknown").to_string();
        
        // 获取文件状态
        let mut opts = StatusOptions::new();
        opts.include_untracked(true);
        opts.include_ignored(false);
        
        let statuses = repo.statuses(Some(&mut opts))?;
        let mut files = Vec::new();
        
        for entry in statuses.iter() {
            let path = entry.path().unwrap_or("").to_string();
            let status_flags = entry.status();
            
            let status = if status_flags.is_wt_modified() || status_flags.is_index_modified() {
                "modified"
            } else if status_flags.is_wt_new() {
                "untracked"
            } else if status_flags.is_index_new() {
                "added"
            } else if status_flags.is_wt_deleted() || status_flags.is_index_deleted() {
                "deleted"
            } else {
                "unknown"
            };
            
            files.push(FileStatus {
                path,
                status: status.to_string(),
                selected: false,
            });
        }
        
        Ok(GitStatusResult {
            has_changes: !files.is_empty(),
            branch,
            files,
        })
    }

    pub fn get_diff_summary(&self, file_paths: &[String]) -> Result<String> {
        let repo_path = self.repo_path.as_ref()
            .ok_or_else(|| anyhow!("No repository opened"))?;
        
        let repo = Repository::open(repo_path)?;
        
        // 获取工作目录和索引的差异
        let mut diff_output = String::new();
        
        for file_path in file_paths {
            // 这里简化处理，实际应该获取详细的diff信息
            diff_output.push_str(&format!("File: {}\n", file_path));
        }
        
        Ok(diff_output)
    }
}
```

#### LLM客户端实现 (src-tauri/src/core/llm_client.rs)
```rust
use reqwest::Client;
use serde::{Deserialize, Serialize};
use anyhow::Result;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMConfig {
    pub provider: String, // "ollama" or "openai"
    pub base_url: String,
    pub model: String,
    pub api_key: Option<String>,
    pub timeout_seconds: u64,
}

#[derive(Debug, Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    stream: bool,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: Message,
}

#[derive(Debug, Deserialize)]
struct Message {
    content: String,
}

pub struct LLMClient {
    client: Client,
    config: LLMConfig,
}

impl LLMClient {
    pub fn new(config: LLMConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .build()
            .expect("Failed to create HTTP client");
        
        Self { client, config }
    }

    pub async fn generate_commit_message(&self, prompt: &str) -> Result<String> {
        let request = ChatRequest {
            model: self.config.model.clone(),
            messages: vec![
                ChatMessage {
                    role: "user".to_string(),
                    content: prompt.to_string(),
                }
            ],
            stream: false,
        };

        let mut request_builder = self.client
            .post(&format!("{}/v1/chat/completions", self.config.base_url))
            .header("Content-Type", "application/json")
            .json(&request);

        // 如果有API密钥，添加Authorization头
        if let Some(api_key) = &self.config.api_key {
            request_builder = request_builder.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = request_builder.send().await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("LLM API error: {}", error_text));
        }

        let chat_response: ChatResponse = response.json().await?;
        
        if let Some(choice) = chat_response.choices.first() {
            Ok(choice.message.content.clone())
        } else {
            Err(anyhow::anyhow!("No response from LLM"))
        }
    }
}
```

#### 模板引擎实现 (src-tauri/src/core/template_engine.rs)
```rust
use handlebars::Handlebars;
use serde_json::Value;
use anyhow::Result;
use std::collections::HashMap;

pub struct TemplateEngine {
    handlebars: Handlebars<'static>,
}

impl TemplateEngine {
    pub fn new() -> Result<Self> {
        let mut handlebars = Handlebars::new();

        // 注册默认的提交消息模板
        let default_template = r#"
请根据以下Git变更信息生成一个简洁、清晰的提交消息：

分支: {{branch}}
变更文件数量: {{file_count}}

文件变更详情:
{{#each files}}
- {{this.status}}: {{this.path}}
{{/each}}

{{#if diff_summary}}
变更摘要:
{{diff_summary}}
{{/if}}

请生成一个符合以下格式的提交消息：
- 第一行：简洁的标题（不超过50字符）
- 空行
- 详细描述（如果需要）

提交消息应该：
1. 使用动词开头（如：Add, Fix, Update, Remove等）
2. 简洁明了地描述变更内容
3. 如果是bug修复，说明修复了什么问题
4. 如果是新功能，说明添加了什么功能
"#;

        handlebars.register_template_string("commit_message", default_template)?;

        Ok(Self { handlebars })
    }

    pub fn load_template_from_file(&mut self, template_path: &str) -> Result<()> {
        let template_content = std::fs::read_to_string(template_path)?;
        self.handlebars.register_template_string("commit_message", template_content)?;
        Ok(())
    }

    pub fn render_commit_prompt(&self, data: &CommitPromptData) -> Result<String> {
        let mut context = HashMap::new();
        context.insert("branch", &data.branch);
        context.insert("file_count", &data.files.len().to_string());
        context.insert("files", &data.files);

        if let Some(diff) = &data.diff_summary {
            context.insert("diff_summary", diff);
        }

        self.handlebars.render("commit_message", &context)
            .map_err(Into::into)
    }
}

#[derive(Debug)]
pub struct CommitPromptData {
    pub branch: String,
    pub files: Vec<FileChangeInfo>,
    pub diff_summary: Option<String>,
}

#[derive(Debug)]
pub struct FileChangeInfo {
    pub path: String,
    pub status: String,
}
```

#### Tauri命令实现 (src-tauri/src/commands/git_commands.rs)
```rust
use tauri::State;
use tokio::sync::Mutex;
use crate::core::git_engine::GitEngine;
use crate::types::git_types::{GitStatusResult, CommitRequest, CommitMessageResult};
use crate::core::llm_client::LLMClient;
use crate::core::template_engine::{TemplateEngine, CommitPromptData, FileChangeInfo};
use std::time::Instant;

#[tauri::command]
pub async fn select_repository(
    path: String,
    git_engine: State<'_, Mutex<GitEngine>>,
) -> Result<String, String> {
    let mut engine = git_engine.lock().await;
    engine.open_repository(&path)
        .map_err(|e| format!("Failed to open repository: {}", e))?;
    Ok("Repository opened successfully".to_string())
}

#[tauri::command]
pub async fn get_git_status(
    git_engine: State<'_, Mutex<GitEngine>>,
) -> Result<GitStatusResult, String> {
    let engine = git_engine.lock().await;
    engine.get_status()
        .map_err(|e| format!("Failed to get git status: {}", e))
}

#[tauri::command]
pub async fn generate_commit_message(
    request: CommitRequest,
    git_engine: State<'_, Mutex<GitEngine>>,
    llm_client: State<'_, LLMClient>,
    template_engine: State<'_, TemplateEngine>,
) -> Result<CommitMessageResult, String> {
    let start_time = Instant::now();

    // 获取Git状态和差异信息
    let git_status = {
        let engine = git_engine.lock().await;
        engine.get_status()
            .map_err(|e| format!("Failed to get git status: {}", e))?
    };

    let diff_summary = {
        let engine = git_engine.lock().await;
        engine.get_diff_summary(&request.selected_files)
            .map_err(|e| format!("Failed to get diff summary: {}", e))?
    };

    // 准备模板数据
    let file_changes: Vec<FileChangeInfo> = git_status.files
        .into_iter()
        .filter(|f| request.selected_files.contains(&f.path))
        .map(|f| FileChangeInfo {
            path: f.path,
            status: f.status,
        })
        .collect();

    let prompt_data = CommitPromptData {
        branch: git_status.branch,
        files: file_changes,
        diff_summary: Some(diff_summary),
    };

    // 渲染提示词
    let prompt = template_engine.render_commit_prompt(&prompt_data)
        .map_err(|e| format!("Failed to render prompt: {}", e))?;

    // 调用LLM生成提交消息
    let commit_message = llm_client.generate_commit_message(&prompt).await
        .map_err(|e| format!("Failed to generate commit message: {}", e))?;

    let processing_time = start_time.elapsed().as_millis() as u64;

    Ok(CommitMessageResult {
        message: commit_message,
        confidence: 0.85, // 简化的置信度
        processing_time_ms: processing_time,
    })
}
```

#### 主程序入口 (src-tauri/src/main.rs)
```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod core;
mod types;

use commands::{git_commands, config_commands};
use core::{git_engine::GitEngine, llm_client::LLMClient, template_engine::TemplateEngine};
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    // 加载配置
    let config = config_commands::load_config()
        .expect("Failed to load configuration");

    // 初始化组件
    let git_engine = Mutex::new(GitEngine::new());
    let llm_client = LLMClient::new(config.llm);
    let template_engine = TemplateEngine::new()
        .expect("Failed to initialize template engine");

    tauri::Builder::default()
        .manage(git_engine)
        .manage(llm_client)
        .manage(template_engine)
        .invoke_handler(tauri::generate_handler![
            git_commands::select_repository,
            git_commands::get_git_status,
            git_commands::generate_commit_message,
            config_commands::get_config,
            config_commands::update_config,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### 步骤3: 前端Vue组件实现

#### 主应用组件 (src/App.vue)
```vue
<template>
  <div id="app">
    <el-container>
      <el-header>
        <h1>GitMentor MVP - AI提交消息生成器</h1>
      </el-header>
      <el-main>
        <el-steps :active="currentStep" finish-status="success">
          <el-step title="选择仓库" />
          <el-step title="选择文件" />
          <el-step title="生成消息" />
        </el-steps>

        <div class="step-content">
          <FolderSelector
            v-if="currentStep === 0"
            @repository-selected="onRepositorySelected"
          />

          <FileSelector
            v-if="currentStep === 1"
            :git-status="gitStatus"
            @files-selected="onFilesSelected"
            @back="currentStep = 0"
          />

          <CommitGenerator
            v-if="currentStep === 2"
            :selected-files="selectedFiles"
            @back="currentStep = 1"
            @commit-generated="onCommitGenerated"
          />
        </div>
      </el-main>
    </el-container>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import FolderSelector from './components/FolderSelector.vue'
import FileSelector from './components/FileSelector.vue'
import CommitGenerator from './components/CommitGenerator.vue'
import type { GitStatusResult } from './types/git'

const currentStep = ref(0)
const gitStatus = ref<GitStatusResult | null>(null)
const selectedFiles = ref<string[]>([])

const onRepositorySelected = (status: GitStatusResult) => {
  gitStatus.value = status
  currentStep.value = 1
}

const onFilesSelected = (files: string[]) => {
  selectedFiles.value = files
  currentStep.value = 2
}

const onCommitGenerated = () => {
  // 可以添加成功提示或重置流程
  console.log('Commit message generated successfully')
}
</script>

<style>
#app {
  font-family: Avenir, Helvetica, Arial, sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  color: #2c3e50;
}

.step-content {
  margin-top: 20px;
  padding: 20px;
}
</style>
```

#### 文件夹选择器组件 (src/components/FolderSelector.vue)
```vue
<template>
  <el-card>
    <template #header>
      <span>选择Git仓库</span>
    </template>

    <div class="folder-selector">
      <el-button
        type="primary"
        @click="selectFolder"
        :loading="loading"
        size="large"
      >
        <el-icon><Folder /></el-icon>
        选择仓库文件夹
      </el-button>

      <div v-if="selectedPath" class="selected-path">
        <p><strong>已选择:</strong> {{ selectedPath }}</p>
      </div>

      <el-alert
        v-if="error"
        :title="error"
        type="error"
        :closable="false"
        style="margin-top: 10px"
      />
    </div>
  </el-card>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import { open } from '@tauri-apps/api/dialog'
import { Folder } from '@element-plus/icons-vue'
import type { GitStatusResult } from '../types/git'

const emit = defineEmits<{
  repositorySelected: [status: GitStatusResult]
}>()

const loading = ref(false)
const selectedPath = ref('')
const error = ref('')

const selectFolder = async () => {
  try {
    loading.value = true
    error.value = ''

    // 打开文件夹选择对话框
    const selected = await open({
      directory: true,
      multiple: false,
    })

    if (selected && typeof selected === 'string') {
      selectedPath.value = selected

      // 调用Rust后端选择仓库
      await invoke('select_repository', { path: selected })

      // 获取Git状态
      const status = await invoke<GitStatusResult>('get_git_status')

      if (status.has_changes) {
        emit('repositorySelected', status)
      } else {
        error.value = '该仓库没有待提交的变更'
      }
    }
  } catch (err) {
    error.value = `错误: ${err}`
  } finally {
    loading.value = false
  }
}
</script>

<style scoped>
.folder-selector {
  text-align: center;
  padding: 20px;
}

.selected-path {
  margin-top: 15px;
  padding: 10px;
  background-color: #f5f7fa;
  border-radius: 4px;
}
</style>
```

#### 文件选择器组件 (src/components/FileSelector.vue)
```vue
<template>
  <el-card>
    <template #header>
      <div class="card-header">
        <span>选择要提交的文件</span>
        <div>
          <el-button @click="$emit('back')" size="small">返回</el-button>
          <el-button
            type="primary"
            @click="confirmSelection"
            :disabled="selectedFiles.length === 0"
            size="small"
          >
            下一步 ({{ selectedFiles.length }} 个文件)
          </el-button>
        </div>
      </div>
    </template>

    <div v-if="gitStatus">
      <div class="repo-info">
        <el-tag type="info">分支: {{ gitStatus.branch }}</el-tag>
        <el-tag type="warning">{{ gitStatus.files.length }} 个变更文件</el-tag>
      </div>

      <div class="file-list">
        <el-checkbox-group v-model="selectedFiles">
          <div
            v-for="file in gitStatus.files"
            :key="file.path"
            class="file-item"
          >
            <el-checkbox :label="file.path">
              <div class="file-info">
                <el-tag
                  :type="getStatusType(file.status)"
                  size="small"
                >
                  {{ getStatusText(file.status) }}
                </el-tag>
                <span class="file-path">{{ file.path }}</span>
              </div>
            </el-checkbox>
          </div>
        </el-checkbox-group>
      </div>

      <div class="actions">
        <el-button @click="selectAll" size="small">全选</el-button>
        <el-button @click="selectNone" size="small">全不选</el-button>
      </div>
    </div>
  </el-card>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import type { GitStatusResult } from '../types/git'

const props = defineProps<{
  gitStatus: GitStatusResult | null
}>()

const emit = defineEmits<{
  back: []
  filesSelected: [files: string[]]
}>()

const selectedFiles = ref<string[]>([])

const confirmSelection = () => {
  emit('filesSelected', selectedFiles.value)
}

const selectAll = () => {
  if (props.gitStatus) {
    selectedFiles.value = props.gitStatus.files.map(f => f.path)
  }
}

const selectNone = () => {
  selectedFiles.value = []
}

const getStatusType = (status: string) => {
  switch (status) {
    case 'modified': return 'warning'
    case 'added': return 'success'
    case 'deleted': return 'danger'
    case 'untracked': return 'info'
    default: return ''
  }
}

const getStatusText = (status: string) => {
  switch (status) {
    case 'modified': return '修改'
    case 'added': return '新增'
    case 'deleted': return '删除'
    case 'untracked': return '未跟踪'
    default: return status
  }
}
</script>

<style scoped>
.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.repo-info {
  margin-bottom: 15px;
}

.repo-info .el-tag {
  margin-right: 10px;
}

.file-list {
  max-height: 400px;
  overflow-y: auto;
  border: 1px solid #dcdfe6;
  border-radius: 4px;
  padding: 10px;
}

.file-item {
  margin-bottom: 8px;
  padding: 5px;
  border-radius: 4px;
}

.file-item:hover {
  background-color: #f5f7fa;
}

.file-info {
  display: flex;
  align-items: center;
  gap: 10px;
}

.file-path {
  font-family: monospace;
  font-size: 14px;
}

.actions {
  margin-top: 15px;
  text-align: center;
}

.actions .el-button {
  margin: 0 5px;
}
</style>
```

#### 提交消息生成器组件 (src/components/CommitGenerator.vue)
```vue
<template>
  <el-card>
    <template #header>
      <div class="card-header">
        <span>生成提交消息</span>
        <el-button @click="$emit('back')" size="small">返回</el-button>
      </div>
    </template>

    <div class="generator-content">
      <div class="selected-files">
        <h3>已选择的文件 ({{ selectedFiles.length }})</h3>
        <el-tag
          v-for="file in selectedFiles"
          :key="file"
          style="margin: 2px;"
          size="small"
        >
          {{ file }}
        </el-tag>
      </div>

      <div class="generate-section">
        <el-button
          type="primary"
          size="large"
          @click="generateMessage"
          :loading="generating"
        >
          <el-icon><Magic /></el-icon>
          {{ generating ? '生成中...' : '生成提交消息' }}
        </el-button>
      </div>

      <div v-if="result" class="result-section">
        <h3>生成的提交消息:</h3>
        <el-input
          v-model="result.message"
          type="textarea"
          :rows="6"
          placeholder="提交消息将在这里显示..."
          class="commit-message"
        />

        <div class="result-info">
          <el-tag type="success">
            置信度: {{ (result.confidence * 100).toFixed(1) }}%
          </el-tag>
          <el-tag type="info">
            处理时间: {{ result.processing_time_ms }}ms
          </el-tag>
        </div>

        <div class="actions">
          <el-button @click="copyToClipboard" type="success">
            <el-icon><CopyDocument /></el-icon>
            复制到剪贴板
          </el-button>
          <el-button @click="generateMessage" type="primary">
            <el-icon><Refresh /></el-icon>
            重新生成
          </el-button>
        </div>
      </div>

      <el-alert
        v-if="error"
        :title="error"
        type="error"
        :closable="false"
        style="margin-top: 15px"
      />
    </div>
  </el-card>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import { writeText } from '@tauri-apps/api/clipboard'
import { ElMessage } from 'element-plus'
import { Magic, CopyDocument, Refresh } from '@element-plus/icons-vue'
import type { CommitMessageResult } from '../types/git'

const props = defineProps<{
  selectedFiles: string[]
}>()

const emit = defineEmits<{
  back: []
  commitGenerated: [result: CommitMessageResult]
}>()

const generating = ref(false)
const result = ref<CommitMessageResult | null>(null)
const error = ref('')

const generateMessage = async () => {
  try {
    generating.value = true
    error.value = ''

    const commitResult = await invoke<CommitMessageResult>('generate_commit_message', {
      request: {
        selected_files: props.selectedFiles,
        additional_context: null
      }
    })

    result.value = commitResult
    emit('commitGenerated', commitResult)

  } catch (err) {
    error.value = `生成失败: ${err}`
  } finally {
    generating.value = false
  }
}

const copyToClipboard = async () => {
  if (result.value) {
    try {
      await writeText(result.value.message)
      ElMessage.success('已复制到剪贴板')
    } catch (err) {
      ElMessage.error('复制失败')
    }
  }
}
</script>

<style scoped>
.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.generator-content {
  padding: 20px 0;
}

.selected-files {
  margin-bottom: 20px;
  padding: 15px;
  background-color: #f5f7fa;
  border-radius: 4px;
}

.selected-files h3 {
  margin-bottom: 10px;
  color: #606266;
}

.generate-section {
  text-align: center;
  margin: 30px 0;
}

.result-section {
  margin-top: 20px;
}

.result-section h3 {
  margin-bottom: 10px;
  color: #606266;
}

.commit-message {
  margin-bottom: 15px;
}

.result-info {
  margin-bottom: 15px;
}

.result-info .el-tag {
  margin-right: 10px;
}

.actions {
  text-align: center;
}

.actions .el-button {
  margin: 0 5px;
}
</style>
```

### 步骤4: 配置文件示例

#### 应用配置文件 (config/app.toml)
```toml
[llm]
provider = "ollama"  # "ollama" 或 "openai"
base_url = "http://localhost:11434"
model = "llama2"
api_key = ""  # OpenAI API密钥（如果使用OpenAI）
timeout_seconds = 30

[template]
template_file = "config/prompts.toml"  # 可选的自定义模板文件路径
```

#### 提示词模板文件 (config/prompts.toml)
```toml
[commit_message_template]
content = """
请根据以下Git变更信息生成一个简洁、清晰的提交消息：

分支: {{branch}}
变更文件数量: {{file_count}}

文件变更详情:
{{#each files}}
- {{this.status}}: {{this.path}}
{{/each}}

{{#if diff_summary}}
变更摘要:
{{diff_summary}}
{{/if}}

请生成一个符合以下格式的提交消息：
- 第一行：简洁的标题（不超过50字符）
- 空行
- 详细描述（如果需要）

提交消息应该：
1. 使用动词开头（如：Add, Fix, Update, Remove等）
2. 简洁明了地描述变更内容
3. 如果是bug修复，说明修复了什么问题
4. 如果是新功能，说明添加了什么功能
5. 使用中文描述

示例格式：
Add: 新增用户登录功能

实现了基于JWT的用户认证系统，包括登录、注册和密码重置功能。
添加了用户会话管理和权限验证中间件。
"""
```

### 步骤5: 构建和运行

#### package.json配置
```json
{
  "name": "gitmentor-mvp",
  "version": "0.1.0",
  "scripts": {
    "dev": "vite",
    "build": "vite build",
    "tauri": "tauri",
    "tauri:dev": "tauri dev",
    "tauri:build": "tauri build"
  },
  "dependencies": {
    "vue": "^3.3.0",
    "element-plus": "^2.4.0",
    "@element-plus/icons-vue": "^2.1.0",
    "pinia": "^2.1.0",
    "@tauri-apps/api": "^2.0.0"
  },
  "devDependencies": {
    "@vitejs/plugin-vue": "^4.4.0",
    "typescript": "^5.0.0",
    "vite": "^4.4.0"
  }
}
```

#### 运行命令
```bash
# 开发模式
npm run tauri:dev

# 构建生产版本
npm run tauri:build
```

## 使用说明

1. **启动应用**: 运行 `npm run tauri:dev`
2. **选择仓库**: 点击"选择仓库文件夹"按钮，选择一个Git仓库
3. **选择文件**: 在文件列表中勾选要提交的文件
4. **生成消息**: 点击"生成提交消息"按钮，等待AI生成结果
5. **复制使用**: 复制生成的提交消息到剪贴板，在Git客户端中使用

## 扩展功能

- 支持自定义提示词模板
- 支持多种LLM服务提供商
- 可配置的API参数
- 提交消息历史记录
- 批量文件处理优化
```
```
```
