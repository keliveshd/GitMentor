# GitMentor 模板版本管理功能

## 概述

GitMentor 现已支持完整的模板版本管理功能，为所有类型的模板（提交消息模板、日报模板、AI分析模板）提供了强大的版本控制能力。

## 🎯 核心功能

### 1. 版本控制
- ✅ **版本历史**：每个模板都保留完整的修改历史
- ✅ **版本切换**：可随时切换到任意历史版本
- ✅ **版本比较**：查看不同版本之间的差异
- ✅ **安全更新**：系统更新不会覆盖用户自定义版本

### 2. 模板类型
- **提交消息模板**：用于生成Git提交消息
  - `commit_standard` - 标准提交消息
  - `commit_chinese` - 简洁中文提交
  - `commit_detailed` - 详细提交消息
  - `commit_conventional` - 约定式提交
- **版本化模板**：日报模板和AI分析模板

### 3. 管理功能
- ✅ **模板编辑**：支持在线编辑和保存
- ✅ **系统更新**：自动检测并应用系统模板更新
- ✅ **自定义模板**：创建完全自定义的模板
- ✅ **批量操作**：支持批量更新和重置

## 🚀 快速开始

### 1. 查看当前模板

```javascript
// 获取所有提交模板
const templates = await window.__TAURI__.invoke('get_all_commit_templates');
console.log(templates);
// 输出示例：
// [
//   {
//     id: "commit_standard",
//     name: "标准提交消息",
//     version: "1.0.0",
//     description: "生成符合常规规范的提交消息",
//     is_custom: false
//   },
//   ...
// ]
```

### 2. 查看版本历史

```javascript
// 获取特定模板的版本历史
const versions = await window.__TAURI__.invoke('get_commit_template_version_history', {
  templateId: 'commit_standard'
});
console.log(versions);
// 输出示例：
// [
//   {
//     id: "uuid-v1",
//     version: "1.0.0",
//     name: "初始版本",
//     description: "系统默认版本",
//     created_at: "2024-01-01T00:00:00Z",
//     is_builtin: true
//   },
//   {
//     id: "uuid-v2",
//     version: "1.0.1",
//     name: "优化版本",
//     description: "改进了提示词结构",
//     created_at: "2024-01-02T10:30:00Z",
//     is_builtin: false,
//     parent_id: "uuid-v1"
//   }
// ]
```

### 3. 更新模板并创建版本

```javascript
// 更新模板内容并创建新版本
const versionId = await window.__TAURI__.invoke('update_commit_template_with_version', {
  templateId: 'commit_chinese',
  content: `这是优化后的中文提交模板

请根据以下变更生成简洁的中文提交消息：

变更文件：{{staged_files}}
代码差异：{{diff}}

要求：
1. 第一行不超过25个字
2. 使用动词开头
3. 语言简洁明了`,
  versionName: 'v1.1.0 - 优化提示词',
  versionDescription: '添加了更明确的格式要求'
});
```

### 4. 切换模板版本

```javascript
// 切换到指定的历史版本
await window.__TAURI__.invoke('switch_commit_template_version', {
  templateId: 'commit_chinese',
  versionId: versionId // 从版本历史中获取的版本ID
});
```

### 5. 检查系统更新

```javascript
// 检查是否有系统模板更新
const updates = await window.__TAURI__.invoke('check_commit_template_updates');
if (updates.length > 0) {
  console.log('发现以下更新：');
  updates.forEach(update => {
    console.log(`- ${update.system_template_id}: ${update.new_version}`);
    console.log(`  ${update.update_description}`);
  });
}
```

## 📖 使用场景

### 场景1：自定义提交消息格式

```javascript
// 创建符合团队规范的提交消息模板
await window.__TAURI__.invoke('update_commit_template_with_version', {
  templateId: 'commit_standard',
  content: `请根据以下变更生成符合团队规范的提交消息：

格式要求：
<类型>(<模块>): <描述>

类型说明：
- feat: 新功能
- fix: 修复bug
- docs: 文档变更
- style: 代码格式
- refactor: 重构
- test: 测试相关
- chore: 构建工具

示例：
feat(auth): 添加用户登录功能
fix(api): 修复数据解析错误

变更：
{{staged_files}}

差异：
{{diff}}`,
  versionName: 'v2.0.0 - 团队规范',
  versionDescription: '适配团队开发规范的提交格式'
});
```

### 场景2：多语言支持

```javascript
// 创建英文提交模板
await window.__TAURI__.invoke('create_unified_custom_template', {
  name: 'English Commit Template',
  description: 'Generate English commit messages',
  templateType: 'commit_message',
  content: `Generate a concise commit message based on the following changes:

Files changed:
{{staged_files}}

Code diff:
{{diff}}

Requirements:
- First line under 50 characters
- Use imperative mood (Add, Fix, Update...)
- No period at the end
- Be specific about what changed`,
  baseTemplateId: 'commit_standard'
});
```

### 场景3：批量管理

```javascript
// 批量更新所有系统模板
const templateIds = ['commit_standard', 'commit_chinese', 'commit_detailed'];
const updatedIds = await window.__TAURI__.invoke('batch_update_system_templates', {
  templateIds: templateIds
});
console.log(`成功更新 ${updatedIds.length} 个模板`);

// 重置所有模板到最新系统版本
const count = await window.__TAURI__.invoke('reset_all_system_templates');
console.log(`已重置 ${count} 个系统模板`);
```

## 🔧 API 参考

### 提交模板管理

| 命令 | 描述 | 参数 |
|------|------|------|
| `get_all_commit_templates` | 获取所有提交模板 | 无 |
| `get_commit_template` | 获取指定提交模板 | `templateId: string` |
| `update_commit_template_with_version` | 更新模板并创建版本 | `templateId, content, versionName, versionDescription` |
| `switch_commit_template_version` | 切换模板版本 | `templateId, versionId` |
| `get_commit_template_version_history` | 获取版本历史 | `templateId` |

### 版本化模板管理

| 命令 | 描述 | 参数 |
|------|------|------|
| `get_all_unified_templates` | 获取所有模板 | 无 |
| `create_unified_custom_template` | 创建自定义模板 | `name, description, templateType, content, baseTemplateId` |
| `delete_unified_custom_template` | 删除自定义模板 | `templateId` |

### 系统更新管理

| 命令 | 描述 | 参数 |
|------|------|------|
| `check_commit_template_updates` | 检查提交模板更新 | 无 |
| `apply_commit_template_update` | 应用指定更新 | `templateId` |
| `batch_update_system_templates` | 批量更新 | `templateIds[]` |
| `reset_all_system_templates` | 重置所有模板 | 无 |

## 📊 数据结构

### PromptTemplate
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
  version?: string;
  is_custom?: boolean;
  created_at?: string;
  updated_at?: string;
}
```

### TemplateVersion
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

## 🧪 测试

1. **访问测试页面**：打开 `http://localhost:1421` 并导航到测试页面
2. **运行测试脚本**：在浏览器控制台执行 `testTemplateVersioning()`
3. **查看结果**：测试结果会显示在页面上，包括每个操作的详细信息

## ⚠️ 注意事项

1. **系统模板保护**：系统内置模板无法删除，只能修改
2. **版本管理**：每次更新都会创建新版本，建议使用清晰的版本命名
3. **备份重要修改**：重要修改前建议备份
4. **性能考虑**：过多的版本历史可能影响性能，定期清理不需要的版本

## 🐛 故障排除

### Q: 如何恢复默认模板？
```javascript
await window.__TAURI__.invoke('reset_all_system_templates');
```

### Q: 版本切换后没有生效？
A: 请检查模板ID和版本ID是否正确，或重新加载应用

### Q: 如何查看模板的实际内容？
```javascript
const template = await window.__TAURI__.invoke('get_commit_template', {
  templateId: 'commit_standard'
});
console.log(template.system_prompt);
```

## 📝 更新日志

### v2.0.0 (2024-01-01)
- ✨ 新增统一的模板版本管理系统
- ✨ 支持提交模板的版本控制
- ✨ 新增系统模板更新机制
- ✨ 完善的API接口
- 🔧 优化模板存储结构