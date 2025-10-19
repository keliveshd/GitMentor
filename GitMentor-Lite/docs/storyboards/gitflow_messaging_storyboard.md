# Gitflow 消息故事板

## 目标
- 帮助缺乏 Git 专业背景的成员在 GitMentor 内管理 feature、release、bugfix、hotfix 分支。
- 在每个 Gitflow 节点展示匹配的 AI 生成文案（状态播报、发布说明、复盘等）。
- 保留既有 GitPanel 与 Layered Commit 的操作体验，仅在其之上叠加 Gitflow 指南。

## 角色与前置条件
- **主要角色：** 兼顾多项需求并依赖 AI 记录状态的产品向开发者。
- **次要角色：** 需要统一发布/热修复叙事的版本发布负责人。
- 仓库已配置 `main`/`develop`（可自定义）；GitMentor 已接入仓库并启用 AI 提示。

## 视觉图例
- `UI`：界面关键画面或布局。
- `Action`：用户执行的动作。
- `System`：GitMentor/Tauri 在后台执行的逻辑。
- `AI`：向用户呈现的自动生成文案或提示语。

## 场景 1 - 进入 Gitflow 仪表盘
- **UI：** 顶部沿用现有 `menu-bar` 与 `tab-navigation`，在 tab 列表中新增带图标的 “Gitflow” 选项；`tab-content` 区域保持浅灰背景，左侧列使用与历史面板相同的卡片栅格展示四类分支概览，右侧列沿用 `repo-header` 状态条显示基线分支、同步提示与告警。
- **Action：** 用户在 GitPanel 中点击新的 `Gitflow` 标签。
- **System：** 加载 Gitflow 数据仓，读取各类分支并标注过期或待合并状态。
- **AI：** 在右侧状态列上方给出下一步建议（示例：`当前没有活跃 feature，建议为工单 ABC-123 创建 feature 分支`）。

## 场景 2 - 启动 Feature 分支向导
- **UI：** 复用 `modal-overlay` 与 `modal-content` 样式，以侧滑抽屉覆盖 `tab-content` 的右半部；抽屉内左侧为三步进度条（类型选择→命名与元数据→操作预览），右侧为表单区域并在底部预留命名规则提示条。
- **Action：** 用户选择 `feature`，填写工单编号，可勾选模板检查清单。
- **System：** 校验命名规则，从 `develop` 派生分支，可选是否立即推送远端。
- **AI：** 侧栏生成“Feature 开工稿”，列出目标、风险、下一步计划，可直接复制。

## 场景 3 - Feature 工作面板引导
- **UI：** `tab-content` 内顶部保持 `repo-header`，下方区域拆分为左侧进度区与右侧操作区；左侧嵌入现有 `LayeredCommitProgress` 组件，阶段标签改为 Gitflow 节点，右侧使用与提交历史一致的卡片列表展示差异、提醒和快捷按钮。
- **Action：** 用户完成提交后返回 Gitflow 仪表盘查看分支卡片。
- **System：** 展示最新提交摘要、与 `develop` 的差异、快捷操作按钮（`Update from develop`、`Open PR`、`Generate status`）。
- **AI：** 自动生成每日状态草稿，汇总提交信息、未完成任务与阻塞点。

## 场景 4 - Bugfix 监控
- **UI：** 继续使用卡片栅格样式，但为 bugfix 卡片添加黄色边框与 `status-chip`；卡片右上显示 SLA 倒计时，与现有同步提示的徽章风格保持一致；右侧详情列提供高优先级标识与提醒开关。
- **Action：** 用户将某 feature 任务转为 bugfix 并在仪表盘查看。
- **System：** `git_engine` 从 `develop` 派生 bugfix 分支，标记高优先级并计划提醒。
- **AI：** 生成“Bugfix 事件记录”模板，包含复现步骤、假设、验证清单。

## 场景 5 - 创建 Release 分支
- **UI：** 进入某 release 卡片后，在右侧详情列展开 `DrawerDetail` 区域：顶部沿用 `section-header` 显示版本号与 CI 状态，中部列表列出纳入的 feature 分支（复用历史记录卡的两列布局），右上角保持 prompts 面板用于触发 AI。
- **Action：** 发布负责人勾选待发布的 feature 并点击 `Create release branch`。
- **System：** 从 `develop` 创建 `release/x.y.z`，合并选定 feature，按配置触发 CI 钩子。
- **AI：** 输出按功能模块划分的发布说明草稿，并标记待完成的 QA 事项。

## 场景 6 - 完成 Release 并生成对外沟通
- **UI：** 使用与调试设置相同的模态框尺寸，顶部 `modal-header` 显示完成选项，`modal-body` 左侧为复选项列表（合并到 main、创建标签、删除分支），右侧延伸出 AI 预览区，采用与消息生成面板一致的对话气泡风格。
- **Action：** 用户点击 `Finish release`。
- **System：** 执行合并与打标签流程，提供清理选项并刷新分支列表。
- **AI：** 输出最终发布总结与回滚检查清单，并自动归档到文档库。

## 场景 7 - Hotfix 紧急通道
- **UI：** 在 `tab-content` 顶部固定一条红色强调的告警栏（复用当前 `alert-banner` 样式），点击后打开与 feature 向导相同的抽屉；仪表盘中的 hotfix 卡片采用全宽布局，在卡片内部使用时间轴组件（与 LayeredCommitProgress 相同的纵向节点）展示状态。
- **Action：** 用户在告警横幅中点击 `New hotfix` 应对线上事故。
- **System：** 从 `main` 派生 hotfix，强制合并后回流 `develop`，提醒加速 CI。
- **AI：** 生成事故播报模板与复盘提纲，根据严重度自动调整语气。

## 结束后的效果
- 分支卡片在完成后折叠为 “Archived” 视图，并附带 AI 文案链接。
- Gitflow 上下文（分支类型、状态、未完成任务）会缓存并传递给后续 AI 指令。
- 核心指标：Gitflow 分支处理量、发布用时、AI 文案采用率。

## 边界情况与缓解
- **基线分支缺失：** 阻止创建并提示更新 Git 配置，AI 提供修复步骤。
- **完成流程遇到冲突：** 流程暂停，展示差异提示，附上 AI 生成的解决建议。
- **离线模式：** 仪表盘提示功能受限，AI 草稿待联网后展示，同时排队保存。

## 未决问题
- Gitflow 引导是否需要通过 Debug 开关灰度推出？
- 是否要在 Gitflow 卡片内接入代码所有者审批，或交由外部工具处理？
- 提醒通知应通过哪些渠道发送（站内、邮件、外部 webhook）？

## 开发进度（2025-10-19）
- 已创建 `feature/gitflow-ui` 开发分支并提交初始实现。
- 新增 `useGitflow` 组合式函数，提供静态分支数据与向导状态。
- 搭建 `GitflowDashboard`、`GitflowBranchCard`、`GitflowWizard` 组件骨架，与现有 `GitPanel` Tab 导航集成。
- 在 UI 中落地仪表盘布局、分支卡片、详情侧栏与创建向导占位内容，验证故事板的核心交互路径。
- 第二阶段补充：仪表盘新增 Hotfix 警报横幅、Bugfix SLA 倒计时、分支详情抽屉、AI 草稿卡片与快捷操作占位；向导按照“基线选择→信息填写→操作预览”流程实现表单与校验。
- 组件层面加入 `GitflowBranchDetail`、`GitflowStageTimeline`，支持按分支类型展示阶段进度、QA 检查项、热修时间线及待办列表，AI 提示内容与故事板保持一致。
- 后端新增 `list_gitflow_branches`、`create_gitflow_branch` Tauri 命令，从真实仓库读取分支、计算 ahead/behind 并支持创建+可选推送；前端 `useGitflow` 改为实时调用并处理加载/错误状态。
