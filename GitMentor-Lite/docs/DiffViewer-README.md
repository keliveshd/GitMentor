# GitMentor 差异查看器组件

## 概述

GitMentor项目提供了两个功能强大的差异查看器组件，用于显示Git文件差异：

1. **DiffViewer** - 基础差异查看器
2. **EnhancedDiffViewer** - 增强差异查看器

两个组件都基于`@git-diff-view/vue`库构建，提供类似VSCode Git差异查看器的功能。

## 功能特性

### 基础功能 (两个组件都支持)

- ✅ **并排差异显示** - 左侧显示原始代码，右侧显示修改后代码
- ✅ **统一差异显示** - 在单个视图中显示差异
- ✅ **语法高亮** - 根据文件类型进行代码语法着色
- ✅ **行号显示** - 显示原始文件和修改后文件的行号
- ✅ **差异高亮** - 删除行(红色)、新增行(绿色)、修改行标记
- ✅ **代码换行控制** - 可以启用/禁用代码行自动换行
- ✅ **二进制文件支持** - 检测并提示二进制文件
- ✅ **错误处理** - 完善的错误处理和重试机制

### 增强功能 (仅EnhancedDiffViewer支持)

- 🚀 **差异统计** - 显示新增/删除行数统计
- 🚀 **差异导航** - 上一个/下一个差异跳转按钮
- 🚀 **空白字符显示** - 可选显示制表符、空格等空白字符
- 🚀 **复制差异** - 一键复制Git diff格式的差异内容
- 🚀 **下载差异文件** - 将差异保存为.diff文件
- 🚀 **键盘快捷键** - 支持Alt+↑/↓导航，Esc关闭
- 🚀 **详细二进制文件信息** - 显示文件大小变化等详细信息
- 🚀 **响应式设计** - 完整的移动端适配
- 🚀 **主题切换** - 支持明亮/暗黑主题

## 使用方法

### 基础用法

```vue
<template>
  <DiffViewer
    :filePath="'src/main.rs'"
    :diffType="'WorkingTree'"
    @close="handleClose"
  />
</template>

<script setup>
import DiffViewer from '@/components/DiffViewer.vue'

const handleClose = () => {
  // 处理关闭事件
}
</script>
```

### 增强用法

```vue
<template>
  <EnhancedDiffViewer
    :filePath="'src/main.rs'"
    :diffType="'WorkingTree'"
    @close="handleClose"
  />
</template>

<script setup>
import EnhancedDiffViewer from '@/components/EnhancedDiffViewer.vue'

const handleClose = () => {
  // 处理关闭事件
}
</script>
```

## Props 参数

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `filePath` | `string` | - | 要查看差异的文件路径 |
| `diffType` | `'WorkingTree' \| 'Staged' \| 'HeadToWorking'` | `'WorkingTree'` | 差异类型 |

### diffType 说明

- `WorkingTree` - 工作区与暂存区的差异
- `Staged` - 暂存区与HEAD的差异  
- `HeadToWorking` - HEAD与工作区的差异

## Events 事件

| 事件名 | 参数 | 说明 |
|--------|------|------|
| `close` | - | 用户点击关闭按钮时触发 |

## 键盘快捷键 (仅增强版本)

| 快捷键 | 功能 |
|--------|------|
| `Alt + ↑` | 跳转到上一个差异 |
| `Alt + ↓` | 跳转到下一个差异 |
| `Esc` | 关闭差异查看器 |

## 技术实现

### 依赖库

- `@git-diff-view/vue` - Vue差异显示组件
- `@git-diff-view/core` - 核心差异处理逻辑
- `@git-diff-view/file` - 文件差异生成工具

### 数据流

1. **后端数据获取** - 通过Tauri调用`get_file_diff`命令
2. **数据转换** - 将后端返回的差异数据转换为组件所需格式
3. **渲染显示** - 使用@git-diff-view/vue组件渲染差异

### 后端接口

```rust
// Tauri命令
#[tauri::command]
pub async fn get_file_diff(
    request: FileDiffRequest,
    git_engine: State<'_, Mutex<GitEngine>>,
) -> Result<FileDiffResult, String>
```

## 样式定制

组件使用Scoped CSS，可以通过CSS变量进行主题定制：

```css
:root {
  --diff-background-color: #ffffff;
  --diff-text-color: #24292e;
  --diff-font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
  --diff-gutter-insert-background-color: #d6fedb;
  --diff-gutter-delete-background-color: #fadde0;
  --diff-code-insert-background-color: #eaffee;
  --diff-code-delete-background-color: #fdeff0;
}
```

## 最佳实践

### 1. 组件选择

- **基础项目** - 使用`DiffViewer`，功能简洁，性能更好
- **专业工具** - 使用`EnhancedDiffViewer`，功能完整，用户体验更佳

### 2. 性能优化

- 大文件差异时建议使用分页或虚拟滚动
- 二进制文件自动跳过差异计算
- 合理使用`v-if`控制组件渲染

### 3. 用户体验

- 提供加载状态指示
- 实现错误重试机制
- 支持键盘导航
- 响应式设计适配移动端

## 故障排除

### 常见问题

1. **差异不显示**
   - 检查文件路径是否正确
   - 确认Git仓库状态
   - 查看浏览器控制台错误信息

2. **性能问题**
   - 大文件建议分块处理
   - 检查内存使用情况
   - 考虑使用虚拟滚动

3. **样式问题**
   - 确认CSS文件正确导入
   - 检查CSS变量设置
   - 验证响应式断点

### 调试技巧

```javascript
// 启用详细日志
console.log('DiffViewer Debug Mode')

// 检查数据结构
console.log('Diff Data:', diffData.value)

// 监控性能
console.time('DiffViewer Render')
// ... 渲染逻辑
console.timeEnd('DiffViewer Render')
```

## 更新日志

### v1.0.0 (2025-07-22)
- ✨ 初始版本发布
- ✨ 基础差异查看器实现
- ✨ 增强差异查看器实现
- ✨ 完整的功能特性支持
- ✨ 响应式设计
- ✨ 键盘快捷键支持

## 贡献指南

欢迎提交Issue和Pull Request来改进这些组件！

### 开发环境

```bash
# 安装依赖
npm install

# 开发模式
npm run dev

# 构建
npm run build
```

## 许可证

MIT License

---

**作者**: Evilek  
**日期**: 2025-07-22  
**项目**: GitMentor
