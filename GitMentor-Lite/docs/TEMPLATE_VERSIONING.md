# GitMentor 模板版本管理功能指南

## 概述

GitMentor 现在支持完整的模板版本管理功能，允许用户安全地编辑、更新和回滚所有类型的模板（包括提交消息模板、日报模板和AI分析模板）。

## 核心特性

### 1. 版本控制
- 每个模板都有独立的版本历史
- 支持创建新版本而不丢失旧版本
- 可以随时切换到任意历史版本
- 系统模板更新不会覆盖用户自定义版本

### 2. 模板类型
- **提交消息模板**：用于生成Git提交消息
- **版本化模板**：包括日报模板和AI分析模板

### 3. 更新机制
- 系统模板更新检测
- 批量更新支持
- 回滚到系统默认版本

## 使用方法

### 获取模板列表

```javascript
// 获取所有提交模板
const commitTemplates = await window.__TAURI__.invoke('get_all_commit_templates');

// 获取所有版本化模板（日报和AI分析模板）
const versionedTemplates = await window.__TAURI__.invoke('get_all_unified_templates');

// 获取统一的所有模板
const allTemplates = await window.__TAURI__.invoke('get_all_templates_unified');
```

### 查看版本历史

```javascript
// 获取提交模板的版本历史
const versions = await window.__TAURI__.invoke('get_commit_template_version_history', {
  templateId: 'commit_standard'
});

// 输出版本信息
versions.forEach(v => {
  console.log(`${v.version}: ${v.name} (${v.description})`);
  console.log(`  创建时间: ${v.created_at}`);
  console.log(`  是否系统版本: ${v.is_builtin}`);
});
```

### 更新模板并创建版本

```javascript
// 更新提交模板并创建新版本
const versionId = await window.__TAURI__.invoke('update_commit_template_with_version', {
  templateId: 'commit_chinese',
  content: `这是更新后的模板内容

变更的文件：{{staged_files}}
代码差异：{{diff}}

请根据以上变更生成提交消息。`,
  versionName: 'v2.0 - 优化版',
  versionDescription: '优化了提示词结构，提高了生成质量'
});
```

### 切换模板版本

```javascript
// 切换到指定版本
await window.__TAURI__.invoke('switch_commit_template_version', {
  templateId: 'commit_chinese',
  versionId: '550e8400-e29b-41d4-a716-446655440000' // 版本ID
});
```

### 检查系统更新

```javascript
// 检查提交模板更新
const commitUpdates = await window.__TAURI__.invoke('check_commit_template_updates');

// 检查所有系统模板更新
const allUpdates = await window.__TAURI__.invoke('check_unified_system_updates');

// 处理更新
for (const update of allUpdates) {
  console.log(`模板 ${update.system_template_id} 有新版本 ${update.new_version}`);
  console.log(`更新描述: ${update.update_description}`);
}
```

### 应用系统更新

```javascript
// 应用单个模板更新
await window.__TAURI__.invoke('apply_commit_template_update', {
  templateId: 'commit_standard'
});

// 批量更新多个模板
const updatedIds = await window.__TAURI__.invoke('batch_update_system_templates', {
  templateIds: ['commit_standard', 'commit_chinese']
});

// 重置所有系统模板到最新版本
const count = await window.__TAURI__.invoke('reset_all_system_templates');
console.log(`已更新 ${count} 个系统模板`);
```

### 创建自定义模板

```javascript
// 创建自定义版本化模板
const templateId = await window.__TAURI__.invoke('create_unified_custom_template', {
  name: '我的自定义模板',
  description: '个人定制的提交消息模板',
  templateType: 'commit_message',
  content: `请生成符合以下规范的提交消息：

1. 第一行必须是50字符以内的摘要
2. 空一行后添加详细说明
3. 使用emoji表示变更类型

变更：
{{staged_files}}

差异：
{{diff}}`,
  baseTemplateId: null // 或者基于现有模板 'commit_standard'
});
```

### 删除自定义模板

```javascript
// 只能删除自定义模板，系统模板无法删除
await window.__TAURI__.invoke('delete_unified_custom_template', {
  templateId: 'your-custom-template-id'
});
```

## 模板配置选项

所有模板都支持以下配置选项：

- `language`: 模板语言（如 "Simplified Chinese", "English"）
- `max_tokens`: 最大令牌数
- `temperature`: 创造性参数（0.0-1.0）
- `enable_emoji`: 是否启用表情符号
- `enable_body`: 是否包含详细描述
- `enable_merge_commit`: 是否合并多个提交
- `use_recent_commits`: 是否使用最近的提交记录

## 数据结构

### PromptTemplate（提交模板）
```typescript
interface PromptTemplate {
  id: string;
  name: string;
  description: string;
  system_prompt: string;
  user_prompt_template: string;
  language: string;
  max_tokens?: number;
  temperature?: number;
  enable_emoji?: boolean;
  enable_body?: boolean;
  // ... 其他配置
}
```

### TemplateVersion（版本信息）
```typescript
interface TemplateVersion {
  id: string;
  version: string;
  name: string;
  description: string;
  content: string;
  created_at: string;
  is_builtin: boolean;
  parent_id?: string;
}
```

## 最佳实践

1. **定期备份**：重要修改后建议创建版本备份
2. **命名规范**：使用清晰的版本名称，如 "v1.0 - 初始版本"
3. **描述完整**：为每个版本添加详细描述，方便后续理解
4. **测试验证**：更新模板后进行实际测试
5. **谨慎更新**：系统更新前先查看更新说明

## 注意事项

1. 系统内置模板无法删除，只能修改或创建新版本
2. 版本切换会立即生效，影响后续所有使用该模板的操作
3. 自定义模板删除后无法恢复，请谨慎操作
4. 建议定期检查系统更新，以获取最新的模板改进

## 故障排除

### 常见问题

**Q: 如何恢复到默认模板？**
```javascript
// 使用 revert_to_builtin_version 命令
await window.__TAURI__.invoke('revert_to_builtin_version', {
  templateId: 'your-template-id'
});
```

**Q: 为什么无法删除模板？**
A: 系统内置模板无法删除，只有用户创建的自定义模板才能删除。

**Q: 如何迁移旧的模板配置？**
A: 系统会自动检测并迁移旧的模板配置到新的版本化系统。

**Q: 版本太多如何清理？**
A: 当前版本保留所有历史版本，建议定期检查并删除不需要的版本。

## 示例代码

完整的示例请参考项目中的 `test_template_versioning.js` 文件，可以在浏览器控制台中运行测试所有功能。