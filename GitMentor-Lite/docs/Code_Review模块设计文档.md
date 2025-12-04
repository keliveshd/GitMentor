# Code Review 模块设计文档

## 文档信息

- **文档名称**: Code Review 模块设计文档
- **项目**: GitMentor-Lite
- **版本**: v1.0
- **作者**: Evilek
- **创建日期**: 2025-01-04
- **最后更新**: 2025-01-04

---

## 目录

1. [项目概述](#1-项目概述)
2. [需求分析](#2-需求分析)
3. [系统设计](#3-系统设计)
4. [技术架构](#4-技术架构)
5. [数据库设计](#5-数据库设计)
6. [API 设计](#6-api-设计)
7. [界面设计](#7-界面设计)
8. [实现方案](#8-实现方案)
9. [测试策略](#9-测试策略)
10. [部署方案](#10-部署方案)
11. [风险评估](#11-风险评估)
12. [时间规划](#12-时间规划)

---

## 1. 项目概述

### 1.1 背景

GitMentor-Lite 是一个基于 Tauri + Vue 3 + Rust 的桌面 Git 客户端，已具备以下核心功能：

- Git 仓库管理（克隆、状态、提交、分支）
- AI 提交信息生成（支持 8 个 AI 提供商）
- Gitflow 工作流支持
- 模板管理与版本控制
- 日报生成与分析

现有代码库架构清晰，AI 集成完善，为新增 Code Review 模块提供了良好的基础。

### 1.2 目标

为 GitMentor-Lite 新增 Code Review 模块，提供：

- **AI 智能代码审查**: 利用现有 AI 基础设施，自动分析代码质量、安全性、性能
- **人工审查流程**: 支持团队协作的代码审查工作流
- **代码规范检查**: 集成静态代码分析工具
- **历史记录管理**: 完整的审查历史记录、查询和统计

### 1.3 范围

**包含范围**:
- 新增 Code Review Tab 页面
- AI 代码审查引擎
- 人工审查工作流
- 审查结果存储和查询
- 与现有 Git 操作集成
- Lint 工具集成（可选）

**不包含范围**:
- 代码编辑功能（使用外部编辑器）
- 与远程代码托管平台（GitHub/GitLab）同步
- 邮件通知系统
- 性能基准测试工具

---

## 2. 需求分析

### 2.1 功能需求

#### 2.1.1 AI 代码审查

**功能描述**: 利用 AI 自动分析代码质量、安全性和性能问题

**详细需求**:
- 支持三级审查深度：基础、标准、深度
- 支持多种触发方式：手动触发、提交前触发
- 支持审查粒度：单文件、多文件、整个提交
- 支持流式响应，实时显示审查进度
- 审查结果结构化展示
- 支持审查历史和对比

**审查维度**:
- 基础级：语法错误、明显逻辑错误
- 标准级：代码质量、性能问题、常见漏洞
- 深度级：架构设计、安全漏洞、最佳实践

**输入**:
- 代码内容（文件内容或 diff 内容）
- 审查深度配置
- 审查规则配置（可选）

**输出**:
- 问题列表（问题类型、位置、描述、严重性）
- 建议和改进方案
- 审查报告（Markdown/JSON）
- Token 使用统计

#### 2.1.2 人工审查流程

**功能描述**: 支持团队成员之间的代码审查协作

**详细需求**:
- 创建审查请求
- 分配审查者
- 多轮评论和回复
- 审查状态跟踪
- 审批决策（通过/需修改/拒绝）

**工作流**:
1. 创建审查请求 → 选择文件/提交 → 填写说明
2. 分配审查者 → 系统通知或手动指定
3. 审查者审查 → 查看代码 → 添加评论
4. 讨论和修改 → 多轮评论 → 解决问题
5. 最终决策 → 通过/需修改/拒绝
6. 记录总结 → 保存审查历史

**审查状态**:
- `PENDING`: 待审查
- `IN_PROGRESS`: 审查中
- `APPROVED`: 已批准
- `CHANGES_REQUESTED`: 需修改
- `REJECTED`: 已拒绝
- `MERGED`: 已合并

#### 2.1.3 代码规范检查

**功能描述**: 自动检查代码规范问题

**详细需求**:
- 集成多种 Lint 工具
- 支持多语言：JavaScript/TypeScript、Rust、Python
- 自动检测项目配置文件
- 结果可视化展示
- 支持忽略规则

**集成工具**:
- JavaScript/TypeScript: ESLint
- Rust: Clippy
- Python: Pylint/Flake8
- 通用: 自定义规则

**检查内容**:
- 代码格式
- 命名规范
- 最佳实践
- 潜在错误
- 性能问题

#### 2.1.4 历史记录管理

**功能描述**: 完整的审查历史记录、查询和统计

**详细需求**:
- 存储所有审查记录
- 高级搜索和过滤
- 统计分析
- 数据导出

**存储信息**:
- 审查 ID、时间戳
- 审查类型（AI/人工/Lint）
- 文件列表
- 审查结果
- 评论记录
- 审查者信息
- Token 使用统计

**搜索维度**:
- 时间范围
- 审查类型
- 文件类型
- 审查者
- 审查状态
- 关键词

**统计指标**:
- 审查通过率
- 常见问题统计
- 审查时长分析
- Token 使用统计
- 贡献者活跃度

### 2.2 非功能需求

#### 2.2.1 性能需求

- AI 审查响应时间：基础级 < 5s，标准级 < 10s，深度级 < 30s
- Lint 检查时间：文件 < 2s
- 数据查询响应时间：< 1s
- 支持最大文件大小：1MB
- 并发审查数：最多 3 个

#### 2.2.2 可用性需求

- 用户界面简洁直观
- 操作流程清晰
- 错误提示友好
- 支持键盘快捷键
- 提供操作指南

#### 2.2.3 可维护性需求

- 代码模块化设计
- 清晰的文档
- 单元测试覆盖 > 80%
- 配置驱动
- 支持插件扩展

#### 2.2.4 兼容性需求

- 兼容现有 GitMentor-Lite 功能
- 兼容不同操作系统：Windows、macOS、Linux
- 兼容不同 Git 版本
- 支持多语言界面

---

## 3. 系统设计

### 3.1 设计原则

1. **模块化设计**: 各模块独立开发，低耦合高内聚
2. **配置驱动**: 通过配置文件控制功能开关和行为
3. **渐进式演进**: 分阶段实施，逐步完善功能
4. **用户体验优先**: 简化操作流程，提供直观界面
5. **可扩展性**: 预留扩展点，支持未来新功能

### 3.2 整体架构

```
┌─────────────────────────────────────────────────────────────┐
│                      Vue 3 前端                             │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────────────────┐ │
│  │ CodeReview  │ │ReviewDetail │ │   ReviewHistory        │ │
│  │ Panel       │ │View         │ │   & Statistics         │ │
│  └─────────────┘ └─────────────┘ └─────────────────────────┘ │
└─────────────────────┬───────────────────────────────────────┘
                      │ Tauri IPC
┌─────────────────────▼───────────────────────────────────────┐
│                     Rust 后端                               │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────────────┐ │
│  │ Code Review  │ │ Review       │ │   Review Storage    │ │
│  │ Commands     │ │ Engine       │ │   & History         │ │
│  └──────────────┘ └──────────────┘ └──────────────────────┘ │
│  ┌──────────────┐ ┌──────────────┐                        │
│  │ AI Analyzer  │ │ Lint         │                        │
│  │ (复用现有)    │ │ Integrator   │                        │
│  └──────────────┘ └──────────────┘                        │
└─────────────────────────────────────────────────────────────┘
                      │
                      ▼
          ┌─────────────────────┐
          │     配置与存储      │
          │  - ai_config.json   │
          │  - templates.json   │
          │  - review_history   │
          └─────────────────────┘
```

### 3.3 模块划分

#### 3.3.1 前端模块

**CodeReviewPanel.vue**
- 作用：主面板，展示当前审查状态和操作入口
- 功能：创建审查、查看结果、管理历史
- 状态：当前仓库、审查配置、审查历史

**ReviewDetailView.vue**
- 作用：详细审查视图
- 功能：查看 AI 审查结果、Lint 结果、人工评论
- 交互：展开/折叠问题、标记已解决、查看建议

**ReviewHistoryList.vue**
- 作用：历史记录列表
- 功能：搜索、过滤、分页、排序
- 数据：历史审查记录、统计数据

**ReviewStatistics.vue**
- 作用：统计分析视图
- 功能：图表展示各类指标
- 图表：审查通过率、常见问题、时长分析

#### 3.3.2 后端模块

**CodeReviewEngine**
- 作用：审查引擎，协调 AI、Lint 和人工审查
- 功能：执行审查流程、合并结果、生成报告

**ReviewAnalyzer**
- 作用：AI 分析器（复用现有 AIManager）
- 功能：调用 AI 模型、分析代码、生成建议

**ReviewWorkflowManager**
- 作用：审查工作流管理
- 功能：状态跟踪、通知机制、决策记录

**ReviewStorage**
- 作用：审查历史存储
- 功能：数据持久化、查询优化、索引管理

**LintIntegrator**
- 作用：Lint 工具集成
- 功能：调用外部工具、解析结果、格式转换

#### 3.3.3 存储模块

**配置存储**
- `.config/code_review_config.json` - 审查配置
- `.config/review_templates.json` - 审查模板
- `.config/review_rules.json` - 审查规则

**数据存储**
- `.config/code_review_history.json` - 审查历史记录
- `.config/review_cache/` - 审查结果缓存

### 3.4 数据流设计

#### 3.4.1 AI 审查数据流

```
用户选择文件 → CodeReviewEngine → 读取文件内容
                                   ↓
生成审查请求 → ReviewAnalyzer → 调用 AIManager
                                   ↓
接收流式响应 → 解析并格式化 → 返回给前端 → 实时显示
                                   ↓
保存结果 → ReviewStorage → 写入历史记录
```

#### 3.4.2 人工审查数据流

```
创建审查请求 → ReviewWorkflowManager → 分配审查者
                                    ↓
审查者操作 → 添加评论 → 更新状态
                                    ↓
讨论和修改 → 多轮迭代 → 决策记录
                                    ↓
完成审查 → ReviewStorage → 保存流程
```

---

## 4. 技术架构

### 4.1 技术栈选择

**前端技术栈**
- Vue 3.5.13 (Composition API)
- TypeScript 5.6.2
- Element Plus 2.10.1 (UI 组件)
- Vue Router 4.5.1
- Pinia 3.0.3 (状态管理)

**后端技术栈**
- Rust (Tauri 2)
- git2 (Git 操作)
- tokio (异步运行时)
- reqwest (HTTP 客户端)
- serde/serde_json (序列化)

**工具链**
- ESLint (代码检查)
- Prettier (代码格式化)
- Vite (构建工具)
- Cargo (Rust 包管理)

### 4.2 AI 集成方案

**复用现有 AI 基础设施**
- 使用现有 `AIManager` 和 `LLMClient`
- 保持多提供商支持（OpenAI、Anthropic 等）
- 继承流式响应能力
- 利用现有缓存机制

**审查提示词设计**
- 基础级提示词：专注语法和明显错误
- 标准级提示词：关注质量和性能
- 深度级提示词：分析架构和安全
- 模板化设计，支持自定义

**流式响应优化**
- 实时显示审查进度
- 增量展示问题和建议
- 支持中断和重新开始
- Token 使用量监控

### 4.3 Lint 集成方案

**工具选择**
- JavaScript/TypeScript: ESLint（内置项目已有）
- Rust: Clippy（内置 cargo 工具）
- Python: Pylint/Flake8（需安装）
- 通用: 自定义规则引擎

**集成方式**
- 通过 `std::process::Command` 调用外部工具
- 解析输出格式（JSON/XML）
- 转换为统一的数据结构
- 前端可视化展示

**配置检测**
- 自动检测项目根目录配置文件
- `.eslintrc.js/.json`
- `pyproject.toml`
- `Cargo.toml`

### 4.4 数据存储方案

**文件结构**
```
.config/
├── code_review_config.json      # 全局配置
├── code_review_templates.json   # 审查模板
├── code_review_history.json     # 历史记录
└── review_cache/                # 缓存目录
    ├── {review_id}.json        # 单次审查结果
    └── {commit_hash}/          # 按提交分组
```

**数据结构设计**
```rust
#[derive(Serialize, Deserialize)]
pub struct ReviewRecord {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub review_type: ReviewType,
    pub scope: ReviewScope,
    pub files: Vec<String>,
    pub ai_result: Option<AIReviewResult>,
    pub lint_result: Option<LintResult>,
    pub manual_review: Option<ManualReview>,
    pub status: ReviewStatus,
}

#[derive(Serialize, Deserialize)]
pub struct AIReviewResult {
    pub provider: String,
    pub model: String,
    pub depth: ReviewDepth,
    pub token_count: u32,
    pub issues: Vec<CodeIssue>,
    pub suggestions: Vec<CodeSuggestion>,
    pub summary: String,
}

#[derive(Serialize, Deserialize)]
pub struct CodeIssue {
    pub id: String,
    pub severity: IssueSeverity,
    pub file: String,
    pub line: Option<u32>,
    pub issue_type: IssueType,
    pub title: String,
    pub description: String,
    pub code_snippet: String,
}
```

---

## 5. 数据库设计

### 5.1 存储方案

由于使用 Tauri 桌面应用，不使用传统数据库，采用 JSON 文件存储。

### 5.2 数据模型

#### 5.2.1 审查记录模型

```typescript
interface ReviewRecord {
  id: string;                    // 审查 ID (UUID)
  timestamp: string;             // 创建时间 (ISO 8601)
  reviewType: ReviewType;        // 审查类型 (AI/MANUAL/LINT/MIXED)
  scope: ReviewScope;            // 审查范围 (FILE/COMMIT/MULTIPLE)
  branch: string;                // 当前分支
  commitHash?: string;           // 提交哈希（如果是提交审查）
  files: ReviewFile[];           // 审查的文件列表
  config: ReviewConfig;          // 审查配置
  status: ReviewStatus;          // 审查状态
  duration: number;              // 审查耗时（毫秒）
  results: ReviewResults;        // 审查结果
  comments?: ReviewComment[];    // 评论（人工审查）
  reviewers?: string[];          // 审查者列表
  creator: string;               // 创建者
  summary?: string;              // 总结
}

interface ReviewFile {
  path: string;                  // 文件路径
  oldContent?: string;           // 原内容（如果是提交审查）
  newContent?: string;           // 新内容
  diff?: string;                 // Diff 信息
  language: string;              // 编程语言
  size: number;                  // 文件大小（字节）
}

interface ReviewResults {
  ai?: AIReviewResult;           // AI 审查结果
  lint?: LintResult;             // Lint 结果
  manual?: ManualReview;         // 人工审查结果
}

enum ReviewType {
  AI = 'AI',
  MANUAL = 'MANUAL',
  LINT = 'LINT',
  MIXED = 'MIXED'
}

enum ReviewScope {
  SINGLE_FILE = 'SINGLE_FILE',
  MULTIPLE_FILES = 'MULTIPLE_FILES',
  COMMIT = 'COMMIT',
  BRANCH = 'BRANCH'
}

enum ReviewStatus {
  PENDING = 'PENDING',
  IN_PROGRESS = 'IN_PROGRESS',
  COMPLETED = 'COMPLETED',
  APPROVED = 'APPROVED',
  CHANGES_REQUESTED = 'CHANGES_REQUESTED',
  REJECTED = 'REJECTED'
}

enum ReviewDepth {
  BASIC = 'BASIC',     // 基础：语法、明显错误
  STANDARD = 'STANDARD', // 标准：质量、性能
  DEEP = 'DEEP'        // 深度：架构、安全
}
```

#### 5.2.2 AI 审查结果模型

```typescript
interface AIReviewResult {
  provider: string;              // AI 提供商名称
  model: string;                 // 使用的模型
  depth: ReviewDepth;            // 审查深度
  tokenCount: number;            // Token 使用量
  responseTime: number;          // 响应时间（毫秒）
  summary: string;               // 总结
  issues: CodeIssue[];           // 问题列表
  suggestions: CodeSuggestion[]; // 建议列表
  metrics: ReviewMetrics;        // 审查指标
}

interface CodeIssue {
  id: string;
  severity: IssueSeverity;
  file: string;
  line?: number;
  column?: number;
  issueType: IssueType;
  title: string;
  description: string;
  codeSnippet: string;
  suggestion?: string;
  references?: string[];         // 相关链接
  cweId?: string;               // 安全漏洞编号
}

enum IssueSeverity {
  CRITICAL = 'CRITICAL',        // 严重：必须修复
  HIGH = 'HIGH',               // 高：强烈建议修复
  MEDIUM = 'MEDIUM',           // 中：建议修复
  LOW = 'LOW',                 // 低：可选修复
  INFO = 'INFO'                // 信息：提示
}

enum IssueType {
  // 代码质量
  CODE_SMELL = 'CODE_SMELL',   // 代码异味
  COMPLEXITY = 'COMPLEXITY',   // 复杂度

  // 性能
  PERFORMANCE = 'PERFORMANCE', // 性能问题
  INEFFICIENT = 'INEFFICIENT', // 低效实现

  // 安全
  SECURITY = 'SECURITY',       // 安全漏洞
  VULNERABILITY = 'VULNERABILITY',

  // 最佳实践
  BEST_PRACTICE = 'BEST_PRACTICE',
  STYLE = 'STYLE',             // 风格问题

  // 错误
  BUG = 'BUG',                 // 潜在 Bug
  ERROR = 'ERROR',             // 错误

  // 可维护性
  MAINTAINABILITY = 'MAINTAINABILITY',
  READABILITY = 'READABILITY',

  // 其他
  DOCUMENTATION = 'DOCUMENTATION',
  DEPRECATED = 'DEPRECATED',
  UNUSED = 'UNUSED'
}
```

#### 5.2.3 Lint 结果模型

```typescript
interface LintResult {
  tool: string;                 // Lint 工具名称 (ESLint/Clippy/Pylint)
  version: string;              // 工具版本
  executionTime: number;        // 执行时间（毫秒）
  errors: LintError[];          // 错误列表
  warnings: LintWarning[];      // 警告列表
  summary: LintSummary;         // 摘要
  config: LintConfig;           // 使用的配置
}

interface LintError {
  file: string;
  line: number;
  column: number;
  ruleId: string;
  severity: 'error' | 'warning';
  message: string;
  suggestion?: string;
}
```

#### 5.2.4 人工审查模型

```typescript
interface ManualReview {
  workflowId: string;           // 工作流 ID
  reviewers: Reviewer[];        // 审查者
  decision?: ReviewDecision;    // 最终决策
  discussions: Discussion[];    // 讨论线程
  createdAt: string;
  completedAt?: string;
  status: ReviewStatus;
}

interface Reviewer {
  id: string;
  name: string;
  email: string;
  role: ReviewerRole;
  status: ReviewerStatus;
  joinedAt: string;
}

enum ReviewerRole {
  REVIEWER = 'REVIEWER',
  LEAD_REVIEWER = 'LEAD_REVIEWER',
  OBSERVER = 'OBSERVER'
}

enum ReviewerStatus {
  PENDING = 'PENDING',
  APPROVED = 'APPROVED',
  REQUESTED_CHANGES = 'REQUESTED_CHANGES'
}

interface Discussion {
  id: string;
  file: string;
  line?: number;
  author: string;
  content: string;
  timestamp: string;
  replies: Reply[];
  resolved: boolean;
  resolvedBy?: string;
  resolvedAt?: string;
}

interface Reply {
  id: string;
  author: string;
  content: string;
  timestamp: string;
}
```

#### 5.2.5 审查配置模型

```typescript
interface ReviewConfig {
  version: string;              // 配置版本
  ai: AIConfig;                 // AI 审查配置
  lint: LintConfig;             // Lint 配置
  manual: ManualConfig;         // 人工审查配置
  rules: ReviewRule[];          // 审查规则
  autoRules: AutoRule[];        // 自动规则
  notifications: NotificationConfig; // 通知配置
}

interface AIConfig {
  provider: string;             // 默认提供商
  model: string;                // 默认模型
  defaultDepth: ReviewDepth;    // 默认深度
  timeout: number;              // 超时时间（秒）
  retries: number;              // 重试次数
  streaming: boolean;           // 是否启用流式响应
  maxTokens: number;            // 最大 Token 数
  temperature: number;          // 创造性参数
  customPrompts: {
    [key in ReviewDepth]?: string;
  };
}

interface LintConfig {
  enabled: boolean;
  tools: {
    eslint?: ESLintConfig;
    clippy?: ClippyConfig;
    pylint?: PylintConfig;
  };
  autoFix: boolean;
  ignorePatterns: string[];
}

interface ReviewRule {
  id: string;
  name: string;
  description: string;
  enabled: boolean;
  severity: IssueSeverity;
  patterns: string[];           // 匹配模式
  actions: RuleAction[];
}

enum RuleAction {
  WARN = 'WARN',
  ERROR = 'ERROR',
  SUGGEST = 'SUGGEST',
  IGNORE = 'IGNORE'
}
```

#### 5.2.6 统计模型

```typescript
interface ReviewStatistics {
  period: StatisticsPeriod;     // 统计周期
  totalReviews: number;         // 总审查数
  totalFiles: number;           // 总文件数
  totalIssues: number;          // 总问题数
  issuesBySeverity: IssuesBySeverity;
  issuesByType: IssuesByType;
  reviewsByType: ReviewsByType;
  averageReviewTime: number;    // 平均审查时间（分钟）
  tokenUsage: TokenUsage;       // Token 使用统计
  topIssues: TopIssue[];        // 常见问题排行
  contributors: ContributorStats[]; // 贡献者统计
  trend: TrendData;             // 趋势数据
}

interface IssuesBySeverity {
  critical: number;
  high: number;
  medium: number;
  low: number;
  info: number;
}

interface TokenUsage {
  total: number;
  byProvider: {
    [provider: string]: number;
  };
  byModel: {
    [model: string]: number;
  };
  cost: number;                 // 估算成本（美元）
}
```

### 5.3 索引设计

为提高查询性能，在 JSON 文件中建立索引：

```typescript
interface ReviewIndex {
  reviewsById: { [id: string]: number }; // 记录索引
  reviewsByDate: { [date: string]: string[] }; // 按日期索引
  reviewsByType: { [type: string]: string[] }; // 按类型索引
  reviewsByStatus: { [status: string]: string[] }; // 按状态索引
  reviewsByFile: { [file: string]: string[] }; // 按文件索引
  reviewsByCommit: { [commit: string]: string }; // 按提交索引
}
```

---

## 6. API 设计

### 6.1 Tauri 命令列表

#### 6.1.1 审查创建

```rust
#[tauri::command]
pub async fn create_code_review(
    files: Vec<String>,
    config: ReviewConfig
) -> Result<ReviewRecord, String>

#[tauri::command]
pub async fn create_ai_review(
    files: Vec<String>,
    depth: ReviewDepth,
    config: Option<AIConfig>
) -> Result<AIReviewResult, String>

#[tauri::command]
pub async fn create_lint_review(
    files: Vec<String>,
    config: Option<LintConfig>
) -> Result<LintResult, String>

#[tauri::command]
pub async fn start_manual_review(
    files: Vec<String>,
    reviewers: Vec<String>,
    config: ManualConfig
) -> Result<ManualReview, String>
```

#### 6.1.2 审查查询

```rust
#[tauri::command]
pub async fn get_review_record(
    review_id: String
) -> Result<ReviewRecord, String>

#[tauri::command]
pub async fn get_review_list(
    filters: ReviewFilters,
    pagination: PaginationOptions
) -> Result<Vec<ReviewRecord>, String>

#[tauri::command]
pub async fn search_reviews(
    query: String,
    filters: Option<ReviewFilters>
) -> Result<Vec<ReviewRecord>, String>

#[tauri::command]
pub async fn get_review_statistics(
    period: StatisticsPeriod
) -> Result<ReviewStatistics, String>
```

#### 6.1.3 审查操作

```rust
#[tauri::command]
pub async fn update_review_status(
    review_id: String,
    status: ReviewStatus
) -> Result<(), String>

#[tauri::command]
pub async fn add_review_comment(
    review_id: String,
    comment: ReviewComment
) -> Result<(), String>

#[tauri::command]
pub async fn resolve_review_issue(
    review_id: String,
    issue_id: String,
    action: IssueAction
) -> Result<(), String>

#[tauri::command]
pub async fn export_review_report(
    review_id: String,
    format: ExportFormat
) -> Result<String, String>
```

#### 6.1.4 配置管理

```rust
#[tauri::command]
pub async fn get_review_config() -> Result<ReviewConfig, String>

#[tauri::command]
pub async fn update_review_config(
    config: ReviewConfig
) -> Result<(), String>

#[tauri::command]
pub async fn reset_review_config() -> Result<(), String>
```

#### 6.1.5 缓存管理

```rust
#[tauri::command]
pub async fn clear_review_cache() -> Result<(), String>

#[tauri::command]
pub async fn get_cache_size() -> Result<u64, String>

#[tauri::command]
pub async fn export_review_history(
    format: ExportFormat,
    filters: Option<ReviewFilters>
) -> Result<String, String>
```

### 6.2 请求/响应示例

#### 6.2.1 AI 审查请求

**请求示例**:
```json
{
  "files": [
    "src/components/HelloWorld.vue",
    "src/utils/helper.ts"
  ],
  "depth": "STANDARD",
  "config": {
    "provider": "openai",
    "model": "gpt-4",
    "customPrompt": "请专注于代码质量和性能问题"
  }
}
```

**响应示例**:
```json
{
  "id": "rev-123456",
  "timestamp": "2025-01-04T10:30:00Z",
  "status": "COMPLETED",
  "aiResult": {
    "provider": "openai",
    "model": "gpt-4",
    "depth": "STANDARD",
    "tokenCount": 1250,
    "responseTime": 3500,
    "summary": "共发现 3 个问题：1 个性能问题，2 个代码质量问题",
    "issues": [
      {
        "id": "iss-001",
        "severity": "HIGH",
        "file": "src/utils/helper.ts",
        "line": 42,
        "issueType": "PERFORMANCE",
        "title": "低效的数组遍历",
        "description": "在循环中使用 splice 会导致 O(n²) 复杂度",
        "codeSnippet": "array.splice(i, 1)",
        "suggestion": "使用 filter 方法替代：array = array.filter(item => item !== target)"
      }
    ],
    "suggestions": [
      {
        "type": "REFACTOR",
        "description": "考虑将 utils/helper.ts 中的重复代码提取为公共函数",
        "impact": "提高代码复用性和可维护性"
      }
    ]
  }
}
```

#### 6.2.2 审查列表查询

**请求示例**:
```json
{
  "filters": {
    "dateFrom": "2025-01-01",
    "dateTo": "2025-01-31",
    "reviewTypes": ["AI", "MANUAL"],
    "statuses": ["COMPLETED", "APPROVED"]
  },
  "pagination": {
    "page": 1,
    "pageSize": 20,
    "sortBy": "timestamp",
    "sortOrder": "desc"
  }
}
```

**响应示例**:
```json
{
  "total": 45,
  "page": 1,
  "pageSize": 20,
  "data": [
    {
      "id": "rev-123456",
      "timestamp": "2025-01-04T10:30:00Z",
      "reviewType": "AI",
      "files": ["src/components/HelloWorld.vue"],
      "status": "COMPLETED",
      "summary": "代码质量良好，仅发现 1 个小问题"
    }
    // ... 更多记录
  ]
}
```

### 6.3 错误处理

#### 6.3.1 错误码定义

```rust
pub enum ReviewError {
    RepositoryNotSelected,       // 未选择仓库
    FilesNotFound,              // 文件不存在
    AIProviderError(String),    // AI 提供商错误
    TokenLimitExceeded,         // Token 超限
    InvalidConfig,              // 配置无效
    PermissionDenied,           // 权限不足
    CacheError,                 // 缓存错误
    SerializationError,         // 序列化错误
    FileSystemError,            // 文件系统错误
    Timeout,                    // 超时
    Interrupted,                // 用户中断
    LintToolError(String),      // Lint 工具错误
}
```

#### 6.3.2 错误响应格式

```json
{
  "success": false,
  "error": {
    "code": "AI_PROVIDER_ERROR",
    "message": "OpenAI API 返回错误",
    "details": {
      "provider": "openai",
      "statusCode": 429,
      "message": "Rate limit exceeded"
    }
  },
  "timestamp": "2025-01-04T10:30:00Z"
}
```

---

## 7. 界面设计

### 7.1 整体布局

#### 7.1.1 CodeReviewPanel 主界面

```
┌─────────────────────────────────────────────────────────────┐
│ Code Review                                [设置] [历史]  │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌───────────────────────────────────────────────────────┐ │
│  │ 当前状态                                            │ │
│  │ 📁 repo-name (main)                                │ │
│  │ 文件已暂存: 3 个                                    │ │
│  │ 未提交: 5 个                                        │ │
│  └───────────────────────────────────────────────────────┘ │
│                                                             │
│  ┌───────────────────────────────────────────────────────┐ │
│  │ 选择审查范围                                        │ │
│  │ ○ 暂存文件 (3 个)  ○ 提交记录  ○ 自定义文件         │ │
│  │                                                     │ │
│  │ 选择的文件:                                        │ │
│  │ ☑ src/components/HelloWorld.vue                    │ │
│  │ ☑ src/utils/helper.ts                              │ │
│  │ ☐ src/api/service.js (未暂存)                      │ │
│  └───────────────────────────────────────────────────────┘ │
│                                                             │
│  ┌───────────────────────────────────────────────────────┐ │
│  │ 审查配置                                            │ │
│  │ 类型: [AI ▼]  深度: [标准 ▼]                        │ │
│  │ ⚙️ 高级设置...                                      │ │
│  └───────────────────────────────────────────────────────┘ │
│                                                             │
│  [ 开始 AI 审查 ]         [ 开始 Lint 检查 ]                │
│                                                             │
│  ┌───────────────────────────────────────────────────────┐ │
│  │ 最近审查                                            │ │
│  │ 2025-01-04 10:30 - AI 审查 - 2 个问题                │ │
│  │ 2025-01-03 15:20 - 人工审查 - 已批准                 │ │
│  └───────────────────────────────────────────────────────┘ │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

#### 7.1.2 审查结果详情视图

```
┌─────────────────────────────────────────────────────────────┐
│ ← 返回    审查结果详情                          [导出] [分享] │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  📊 审查摘要                                                │
│  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ │
│  审查类型: AI 审查            深度: 标准                     │
│  文件数: 2 个                问题数: 3 个                   │
│  耗时: 3.5 秒                Token: 1,250                   │
│                                                             │
│  问题分布: 🔴 严重 1  🟡 中等 2  🟢 低 0                   │
│                                                             │
│  ┌───────────────────────────────────────────────────────┐ │
│  │ 🔴 严重 - PERFORMANCE                                │ │
│  │ 文件: src/utils/helper.ts (第 42 行)                  │ │
│  │ 标题: 低效的数组遍历                                  │ │
│  │                                                      │ │
│  │ 描述: 在循环中使用 splice 会导致 O(n²) 复杂度          │ │
│  │                                                      │ │
│  │ 问题代码:                                             │ │
│  │   for (let i = 0; i < array.length; i++) {          │ │
│  │     if (array[i] === target) {                      │ │
│  │ ❌   array.splice(i, 1);    // O(n²) 复杂度          │ │
│  │     }                                                │ │
│  │   }                                                  │ │
│  │                                                      │ │
│  │ ✅ 建议:                                              │ │
│  │   array = array.filter(item => item !== target);    │ │
│  │                                                      │ │
│  │ [标记已解决] [查看详情]                               │ │
│  └───────────────────────────────────────────────────────┘ │
│                                                             │
│  ┌───────────────────────────────────────────────────────┐ │
│  │ 🟡 中等 - CODE_SMELL                                  │ │
│  │ 文件: src/components/HelloWorld.vue (第 15 行)         │ │
│  │ 标题: 过长的函数                                      │ │
│  │ ...                                                  │ │
│  └───────────────────────────────────────────────────────┘ │
│                                                             │
│  ┌───────────────────────────────────────────────────────┐ │
│  │ 审查建议                                              │ │
│  │ 1. 考虑重构 utils/helper.ts 中的性能问题               │ │
│  │ 2. 拆分过长的 Vue 组件                                │ │
│  └───────────────────────────────────────────────────────┘ │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

#### 7.1.3 审查历史视图

```
┌─────────────────────────────────────────────────────────────┐
│ Code Review 历史                          [统计] [导出]     │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  🔍 筛选: [全部类型 ▼] [全部状态 ▼] [📅 最近 30 天 ▼] [🔍 搜索] │
│                                                             │
│  ┌───────────────────────────────────────────────────────┐ │
│  │ 2025-01-04 10:30    AI 审查    ✅ 通过    2 个问题     │ │
│  │ 提交: a3f5d8       文件: 2 个    时长: 3.5s            │ │
│  │ 查看详情 →                                            │ │
│  ├───────────────────────────────────────────────────────┤ │
│  │ 2025-01-03 15:20   人工审查    ✅ 已批准   5 个评论     │ │
│  │ 提交: b7c2e1       审查者: 张三, 李四                  │ │
│  │ 查看详情 →                                            │ │
│  ├───────────────────────────────────────────────────────┤ │
│  │ 2025-01-02 09:15   Lint 检查   ⚠️ 警告    12 个问题     │ │
│  │ 提交: d9e8f3       文件: 15 个   Token: 0              │ │
│  │ 查看详情 →                                            │ │
│  └───────────────────────────────────────────────────────┘ │
│                                                             │
│  ┌───────────────────────────────────────────────────────┐ │
│  │ 分页: ← 上一页  1/5  下一页 →  共 45 条记录            │ │
│  └───────────────────────────────────────────────────────┘ │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

#### 7.1.4 审查统计视图

```
┌─────────────────────────────────────────────────────────────┐
│ Code Review 统计分析                            [返回]     │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  📊 总体统计 (2024-01-01 ~ 2024-12-31)                       │
│  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ │
│  总审查次数: 128      总问题数: 456      平均审查时间: 4.2 分钟 │
│                                                             │
│  ┌───────────────────┐  ┌───────────────────┐              │
│  │   问题严重性分布   │  │   审查类型分布     │              │
│  │                   │  │                   │              │
│  │  🔴 严重: 45     │  │  🤖 AI: 89      │              │
│  │  🟡 中等: 210    │  │  👥 人工: 28     │              │
│  │  🟢 低等: 180    │  │  🔧 Lint: 11     │              │
│  │  ℹ️ 信息: 21     │  │                   │              │
│  │                   │  │                   │              │
│  └───────────────────┘  └───────────────────┘              │
│                                                             │
│  ┌───────────────────────────────────────────────────────┐ │
│  │              月度审查趋势                             │ │
│  │   140 ┤                                         ╭──╮   │ │
│  │   120 ┤                                    ╭───╱   ╰─╮│ │
│  │   100 ┤                               ╭───╱          │ │
│  │    80 ┤                          ╭───╱                │ │
│  │    60 ┤                     ╭───╱                     │ │
│  │    40 ┤                ╭───╱                          │ │
│  │    20 ┤           ╭───╱                               │ │
│  │     0 ┤ ╭───╱───╱───╱                                │ │
│  │       └─────────────────────────────────────────────  │ │
│  │         1  2  3  4  5  6  7  8  9 10 11 12           │ │
│  └───────────────────────────────────────────────────────┘ │
│                                                             │
│  ┌───────────────────────────────────────────────────────┐ │
│  │ Top 5 常见问题                                       │ │
│  │ 1. CODE_SMELL - 89 次                                │ │
│  │ 2. PERFORMANCE - 67 次                               │ │
│  │ 3. STYLE - 54 次                                     │ │
│  │ 4. SECURITY - 32 次                                  │ │
│  │ 5. BUG - 28 次                                       │ │
│  └───────────────────────────────────────────────────────┘ │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### 7.2 交互设计

#### 7.2.1 操作流程

1. **开始审查**
   - 用户选择审查范围（暂存文件/提交/自定义）
   - 选择审查类型（AI/Lint/人工）
   - 配置审查深度和参数
   - 点击"开始审查"按钮

2. **查看结果**
   - 实时显示审查进度
   - 问题列表可展开/折叠
   - 点击问题查看详情和建议
   - 支持标记问题为"已解决"

3. **人工审查**
   - 创建审查请求
   - 分配审查者
   - 添加评论和讨论
   - 跟踪审查状态

4. **查看历史**
   - 搜索和过滤历史记录
   - 点击记录查看详情
   - 导出审查报告

#### 7.2.2 快捷键设计

| 快捷键 | 功能 |
|--------|------|
| `Ctrl + R` | 开始 AI 审查 |
| `Ctrl + L` | 开始 Lint 检查 |
| `Ctrl + H` | 查看审查历史 |
| `Ctrl + S` | 保存审查结果 |
| `Ctrl + E` | 导出审查报告 |
| `F5` | 刷新当前页面 |
| `Esc` | 关闭当前对话框 |

#### 7.2.3 响应式设计

- 适配不同屏幕尺寸
- 主界面 1200px 以上最佳
- 移动端友好（触摸操作优化）

### 7.3 状态管理

使用 Pinia 管理前端状态：

```typescript
interface CodeReviewState {
  currentRepo: string | null;
  selectedFiles: string[];
  reviewConfig: ReviewConfig;
  currentReview: ReviewRecord | null;
  reviewHistory: ReviewRecord[];
  reviewStatistics: ReviewStatistics | null;
  loading: boolean;
  error: string | null;
}
```

### 7.4 组件设计

#### 7.4.1 组件层次结构

```
CodeReviewPanel.vue          # 主面板
├── ReviewScopeSelector.vue  # 范围选择器
├── ReviewConfigPanel.vue    # 配置面板
├── ReviewActions.vue        # 操作按钮
└── RecentReviews.vue        # 最近审查

ReviewDetailView.vue         # 详情视图
├── ReviewSummary.vue        # 摘要信息
├── IssueList.vue            # 问题列表
│   ├── IssueItem.vue        # 单个问题
│   └── IssueDetail.vue      # 问题详情
├── SuggestionPanel.vue      # 建议面板
└── ActionButtons.vue        # 操作按钮

ReviewHistoryList.vue        # 历史列表
├── SearchBar.vue            # 搜索栏
├── FilterPanel.vue          # 过滤面板
└── HistoryItem.vue          # 历史记录项

ReviewStatistics.vue         # 统计分析
├── StatisticsCards.vue      # 统计卡片
├── ChartsPanel.vue          # 图表面板
└── TopIssues.vue            # 常见问题
```

#### 7.4.2 组件接口

```typescript
// IssueItem.vue
interface Props {
  issue: CodeIssue;
  showFile?: boolean;
  compact?: boolean;
}

interface Emits {
  (e: 'expand', issueId: string): void;
  (e: 'resolve', issueId: string): void;
  (e: 'comment', issueId: string, comment: string): void;
}

// ReviewSummary.vue
interface Props {
  review: ReviewRecord;
}

interface Emits {
  (e: 'export', format: ExportFormat): void;
  (e: 'share', reviewId: string): void;
}
```

---

## 8. 实现方案

### 8.1 开发策略

采用 **渐进式演进方案**，分四个阶段实施：

#### 阶段 1：MVP（最小可行产品）- 1 周

**目标**: 快速交付核心价值，验证技术可行性

**功能清单**:
- [ ] AI 代码审查（基础/标准/深度）
- [ ] 单文件/多文件审查
- [ ] 审查结果展示和保存
- [ ] 简单的历史记录查看
- [ ] 审查配置管理

**交付物**:
- CodeReviewPanel.vue 主界面
- ReviewDetailView.vue 详情页
- CodeReviewEngine 后端引擎
- ReviewStorage 存储模块
- 基础类型定义

**验收标准**:
- 可以选择文件进行 AI 审查
- 审查结果正确显示
- 历史记录可以查询
- 配置可以保存和加载

#### 阶段 2：增强功能 - 1 周

**目标**: 完善基础功能，增加人工审查

**功能清单**:
- [ ] 历史 commit 审查
- [ ] 人工审查流程（简化版）
- [ ] 评论功能
- [ ] 审查状态管理
- [ ] 高级搜索和过滤

**交付物**:
- ReviewHistoryList.vue 列表页
- ReviewWorkflowManager 工作流管理
- 评论系统
- 搜索过滤组件

**验收标准**:
- 可以审查历史 commit
- 可以创建和跟踪审查请求
- 支持评论和讨论
- 搜索过滤功能正常

#### 阶段 3：专业功能 - 1-2 周

**目标**: 增加专业功能，提升审查质量

**功能清单**:
- [ ] Lint 工具集成
- [ ] 高级搜索和过滤
- [ ] 统计分析
- [ ] 审查模板和规则配置
- [ ] 数据导出

**交付物**:
- LintIntegrator 模块
- ReviewStatistics.vue 统计页
- 规则配置界面
- 导出功能

**验收标准**:
- Lint 检查结果正确显示
- 统计数据准确
- 可以导出报告
- 规则配置生效

#### 阶段 4：协作功能 - 可选

**目标**: 支持团队协作和复杂工作流

**功能清单**:
- [ ] 多审查者支持
- [ ] 审批工作流
- [ ] 通知机制
- [ ] 审查模板
- [ ] 集成 CI/CD

**交付物**:
- 完整的工作流系统
- 通知系统
- 模板管理
- CI/CD 集成

**验收标准**:
- 支持多审查者
- 工作流可配置
- 通知及时准确
- CI/CD 集成稳定

### 8.2 技术实现细节

#### 8.2.1 前端实现

**目录结构**:
```
src/
├── components/codereview/
│   ├── CodeReviewPanel.vue         # 主面板 [阶段1]
│   ├── ReviewDetailView.vue        # 详情视图 [阶段1]
│   ├── ReviewHistoryList.vue       # 历史列表 [阶段2]
│   ├── ReviewStatistics.vue        # 统计分析 [阶段3]
│   ├── common/
│   │   ├── IssueItem.vue           # 问题项
│   │   ├── ReviewSummary.vue       # 摘要
│   │   └── CommentThread.vue       # 评论线程 [阶段2]
│   └── config/
│       ├── ReviewConfigPanel.vue   # 配置面板 [阶段1]
│       └── RuleConfig.vue          # 规则配置 [阶段3]
├── composables/
│   ├── useCodeReview.ts            # 审查逻辑 [阶段1]
│   ├── useReviewWorkflow.ts        # 工作流 [阶段2]
│   └── useReviewSearch.ts          # 搜索 [阶段2]
├── stores/
│   └── codeReview.ts               # Pinia 状态 [阶段1]
└── types/
    └── review.ts                   # 类型定义 [阶段1]
```

**关键代码示例**:

```typescript
// composables/useCodeReview.ts
import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { ReviewRecord, ReviewConfig } from '../types/review';

export function useCodeReview() {
  const loading = ref(false);
  const currentReview = ref<ReviewRecord | null>(null);
  const error = ref<string | null>(null);

  const startAIReview = async (
    files: string[],
    config: ReviewConfig
  ): Promise<ReviewRecord> => {
    loading.value = true;
    error.value = null;

    try {
      const result = await invoke<ReviewRecord>('create_ai_review', {
        files,
        config
      });
      currentReview.value = result;
      return result;
    } catch (err) {
      error.value = err as string;
      throw err;
    } finally {
      loading.value = false;
    }
  };

  return {
    loading,
    currentReview,
    error,
    startAIReview
  };
}
```

#### 8.2.2 后端实现

**目录结构**:
```
src-tauri/src/
├── commands/
│   ├── code_review_commands.rs     # Tauri 命令 [阶段1]
│   ├── review_workflow_commands.rs # 工作流命令 [阶段2]
│   └── review_lint_commands.rs     # Lint 命令 [阶段3]
├── core/
│   ├── code_review_engine.rs       # 审查引擎 [阶段1]
│   ├── review_analyzer.rs          # 分析器 [阶段1]
│   ├── review_storage.rs           # 存储 [阶段1]
│   ├── review_workflow_manager.rs  # 工作流 [阶段2]
│   ├── lint_integrator.rs          # Lint 集成 [阶段3]
│   └── review_statistics.rs        # 统计 [阶段3]
└── types/
    └── review_types.rs             # 类型定义 [阶段1]
```

**关键代码示例**:

```rust
// commands/code_review_commands.rs
use crate::core::code_review_engine::CodeReviewEngine;
use crate::types::review_types::*;

#[tauri::command]
pub async fn create_ai_review(
    files: Vec<String>,
    depth: ReviewDepth,
    config: Option<AIConfig>,
) -> Result<AIReviewResult, String> {
    let engine = CodeReviewEngine::new();
    engine
        .analyze_with_ai(files, depth, config.unwrap_or_default())
        .await
        .map_err(|e| e.to_string())
}

// core/code_review_engine.rs
pub struct CodeReviewEngine {
    ai_manager: Arc<RwLock<AIManager>>,
    storage: Arc<RwLock<ReviewStorage>>,
}

impl CodeReviewEngine {
    pub async fn analyze_with_ai(
        &self,
        files: Vec<String>,
        depth: ReviewDepth,
        config: AIConfig,
    ) -> Result<AIReviewResult> {
        // 读取文件内容
        let file_contents = self.read_files(&files)?;

        // 构建提示词
        let prompt = self.build_prompt(&file_contents, depth, &config)?;

        // 调用 AI
        let ai_manager = self.ai_manager.read().await;
        let response = ai_manager
            .generate_response(prompt, config.clone())
            .await?;

        // 解析结果
        let result = self.parse_ai_response(response, &files)?;

        // 保存结果
        {
            let mut storage = self.storage.write().await;
            storage.save_review_result(&result)?;
        }

        Ok(result)
    }

    fn build_prompt(
        &self,
        contents: &[(String, String)],
        depth: ReviewDepth,
        config: &AIConfig,
    ) -> Result<String> {
        // 根据深度选择模板
        let template = match depth {
            ReviewDepth::BASIC => BASIC_PROMPT,
            ReviewDepth::STANDARD => STANDARD_PROMPT,
            ReviewDepth::DEEP => DEEP_PROMPT,
        };

        // 格式化文件内容
        let formatted_content = contents
            .iter()
            .map(|(path, content)| format!("文件: {}\n内容:\n{}", path, content))
            .collect::<Vec<_>>()
            .join("\n\n");

        Ok(format!("{}\n\n待审查的代码:\n{}", template, formatted_content))
    }
}
```

#### 8.2.3 数据存储实现

```rust
// core/review_storage.rs
#[derive(Serialize, Deserialize)]
pub struct ReviewStorage {
    config_path: PathBuf,
    data_path: PathBuf,
    cache_dir: PathBuf,
}

impl ReviewStorage {
    pub fn new(config_dir: PathBuf) -> Result<Self> {
        let data_path = config_dir.join("code_review_history.json");
        let cache_dir = config_dir.join("review_cache");
        std::fs::create_dir_all(&cache_dir)?;

        Ok(Self {
            config_path: config_dir.join("code_review_config.json"),
            data_path,
            cache_dir,
        })
    }

    pub async fn save_review(&self, review: &ReviewRecord) -> Result<()> {
        // 读取现有数据
        let mut reviews = self.load_all_reviews().await?;

        // 添加新记录
        reviews.push(review.clone());

        // 写入文件
        let json = serde_json::to_string_pretty(&reviews)?;
        tokio::fs::write(&self.data_path, json).await?;

        // 写入缓存
        let cache_path = self.cache_dir.join(format!("{}.json", review.id));
        let cache_json = serde_json::to_string_pretty(review)?;
        tokio::fs::write(cache_path, cache_json).await?;

        Ok(())
    }

    pub async fn load_review(&self, id: &str) -> Result<Option<ReviewRecord>> {
        // 先尝试从缓存读取
        let cache_path = self.cache_dir.join(format!("{}.json", id));
        if cache_path.exists() {
            let content = tokio::fs::read_to_string(cache_path).await?;
            let review = serde_json::from_str(&content)?;
            return Ok(Some(review));
        }

        // 从主文件读取
        let reviews = self.load_all_reviews().await?;
        Ok(reviews.into_iter().find(|r| r.id == id))
    }

    pub async fn search_reviews(
        &self,
        filters: &ReviewFilters,
    ) -> Result<Vec<ReviewRecord>> {
        let reviews = self.load_all_reviews().await?;

        let filtered: Vec<ReviewRecord> = reviews
            .into_iter()
            .filter(|r| self.matches_filters(r, filters))
            .collect();

        Ok(filtered)
    }
}
```

#### 8.2.4 AI 集成实现

```rust
// core/review_analyzer.rs
pub struct ReviewAnalyzer {
    ai_manager: Arc<RwLock<AIManager>>,
}

impl ReviewAnalyzer {
    pub async fn analyze_with_stream(
        &self,
        prompt: String,
        config: AIConfig,
    ) -> Result<impl Stream<Item = Result<Token, Error>>> {
        let ai_manager = self.ai_manager.read().await;

        // 使用现有 AI 管理器的流式响应
        ai_manager
            .stream_generate(prompt, config)
            .await
            .map_err(|e| anyhow!("AI 分析失败: {}", e))
    }

    fn parse_ai_stream(
        &self,
        stream: impl Stream<Item = Result<String, Error>>,
    ) -> impl Stream<Item = Result<AnalysisChunk, Error>> {
        stream
            .map(|line| {
                let line = line?;
                let chunk: AnalysisChunk = serde_json::from_str(&line)
                    .map_err(|e| anyhow!("解析 AI 响应失败: {}", e))?;
                Ok(chunk)
            })
    }
}

// 响应格式定义
#[derive(Serialize, Deserialize)]
struct AnalysisChunk {
    #[serde(rename = "type")]
    chunk_type: String,
    data: ChunkData,
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum ChunkData {
    Issue { issue: CodeIssue },
    Suggestion { suggestion: CodeSuggestion },
    Summary { text: String },
    Progress { progress: u32 },
    Done,
}
```

### 8.3 配置管理

```rust
// core/review_config.rs
#[derive(Clone, Serialize, Deserialize)]
pub struct ReviewConfig {
    pub version: String,
    pub ai: AIConfig,
    pub lint: LintConfig,
    pub rules: Vec<ReviewRule>,
}

impl ReviewConfig {
    pub fn default() -> Self {
        Self {
            version: "1.0".to_string(),
            ai: AIConfig::default(),
            lint: LintConfig::default(),
            rules: vec![],
        }
    }

    pub fn load(path: &Path) -> Result<Self> {
        if path.exists() {
            let content = std::fs::read_to_string(path)?;
            let config = serde_json::from_str(&content)?;
            Ok(config)
        } else {
            let default = Self::default();
            default.save(path)?;
            Ok(default)
        }
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let dir = path.parent().unwrap();
        std::fs::create_dir_all(dir)?;
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }
}
```

### 8.4 Lint 集成实现

```rust
// core/lint_integrator.rs
pub struct LintIntegrator {
    config_dir: PathBuf,
}

impl LintIntegrator {
    pub async fn run_eslint(&self, files: &[String]) -> Result<LintResult> {
        let output = tokio::process::Command::new("npx")
            .args(&["eslint", "--format", "json"])
            .args(files)
            .output()
            .await?;

        if !output.status.success() {
            return Err(anyhow!("ESLint 执行失败: {}", String::from_utf8_lossy(&output.stderr)));
        }

        let json_output = String::from_utf8_lossy(&output.stdout);
        let eslint_results: Vec<ESLintResult> = serde_json::from_str(&json_output)?;

        self.convert_eslint_results(eslint_results)
    }

    fn convert_eslint_results(&self, results: Vec<ESLintResult>) -> Result<LintResult> {
        let mut errors = vec![];
        let mut warnings = vec![];

        for result in results {
            for message in result.messages {
                let lint_error = LintError {
                    file: result.file_path.clone(),
                    line: message.line,
                    column: message.column,
                    rule_id: message.rule_id,
                    severity: match message.severity {
                        2 => "error".to_string(),
                        1 => "warning".to_string(),
                        _ => "info".to_string(),
                    },
                    message: message.message,
                    suggestion: message.suggestions.first().map(|s| s.text.clone()),
                };

                if message.severity >= 2 {
                    errors.push(lint_error);
                } else {
                    warnings.push(lint_error);
                }
            }
        }

        Ok(LintResult {
            tool: "ESLint".to_string(),
            version: self.get_eslint_version().await?,
            execution_time: 0,
            errors,
            warnings,
            summary: LintSummary {
                total_files: results.len() as u32,
                total_errors: errors.len() as u32,
                total_warnings: warnings.len() as u32,
            },
            config: LintConfig::default(),
        })
    }
}
```

### 8.5 统计实现

```rust
// core/review_statistics.rs
pub struct ReviewStatistics {
    storage: Arc<RwLock<ReviewStorage>>,
}

impl ReviewStatistics {
    pub async fn generate(
        &self,
        period: StatisticsPeriod,
    ) -> Result<ReviewStatisticsData> {
        let reviews = self.storage.read().await.get_reviews_in_period(&period).await?;

        let total_reviews = reviews.len() as u32;
        let total_issues = reviews
            .iter()
            .flat_map(|r| &r.results.ai.issues)
            .count() as u32;

        let issues_by_severity = self.calculate_issues_by_severity(&reviews);
        let issues_by_type = self.calculate_issues_by_type(&reviews);
        let reviews_by_type = self.calculate_reviews_by_type(&reviews);

        let average_review_time = self.calculate_average_time(&reviews);
        let token_usage = self.calculate_token_usage(&reviews);

        Ok(ReviewStatisticsData {
            period,
            total_reviews,
            total_issues,
            issues_by_severity,
            issues_by_type,
            reviews_by_type,
            average_review_time,
            token_usage,
            top_issues: self.get_top_issues(&reviews, 5),
            contributors: self.get_contributors(&reviews),
        })
    }
}
```

---

## 9. 测试策略

### 9.1 测试层级

#### 9.1.1 单元测试

**范围**: 核心业务逻辑函数

**测试对象**:
- `CodeReviewEngine` - 审查引擎逻辑
- `ReviewAnalyzer` - AI 分析逻辑
- `ReviewStorage` - 存储逻辑
- `ReviewWorkflowManager` - 工作流逻辑
- `LintIntegrator` - Lint 集成逻辑

**示例**:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ai_review_basic() {
        let engine = CodeReviewEngine::new();
        let files = vec!["test.js".to_string()];
        let result = engine.analyze_with_ai(files, ReviewDepth::BASIC, AIConfig::default()).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_review_storage_save() {
        let temp_dir = tempfile::tempdir().unwrap();
        let storage = ReviewStorage::new(temp_dir.path().to_path_buf()).unwrap();
        let review = ReviewRecord::default();
        let result = tokio::runtime::Runtime::new().unwrap().block_on(async {
            storage.save_review(&review).await
        });
        assert!(result.is_ok());
    }
}
```

**目标**:
- 前端组件 > 80% 覆盖率
- 后端核心模块 > 90% 覆盖率
- 工具函数 100% 覆盖率

#### 9.1.2 集成测试

**范围**: 模块间交互

**测试场景**:
1. AI 审查流程
   - 前端调用 → Tauri 命令 → AI 管理器 → 存储
   - 流式响应处理
   - 错误处理

2. Lint 检查流程
   - 调用外部工具
   - 解析输出
   - 转换格式

3. 工作流管理
   - 创建审查 → 分配审查者 → 评论 → 完成

**示例**:
```rust
#[cfg(test)]
mod integration_tests {
    use tauri::test::mock;

    #[tokio::test]
    async fn test_full_review_flow() {
        let (app, handle) = mock::mock_app();
        tauri::Builder::default()
            .manage(CodeReviewEngine::new())
            .run(app)
            .expect("Failed to run app");

        // 模拟调用
        let result = handle.invoke("create_ai_review", json!({
            "files": ["test.js"],
            "depth": "BASIC"
        })).await;

        assert!(result.is_ok());
    }
}
```

#### 9.1.3 E2E 测试

**范围**: 完整用户流程

**测试场景**:
1. 创建 AI 审查
2. 查看审查结果
3. 保存历史记录
4. 搜索和过滤
5. 配置管理

**工具**: Playwright

**示例**:
```typescript
// tests/e2e/review.spec.ts
import { test, expect } from '@playwright/test';

test('should complete AI review flow', async ({ page }) => {
  await page.goto('/#/code-review');

  // 选择文件
  await page.click('[data-testid="select-files"]');
  await page.check('text=src/components/HelloWorld.vue');

  // 开始审查
  await page.click('[data-testid="start-review"]');

  // 等待结果
  await expect(page.locator('[data-testid="review-results"]')).toBeVisible();

  // 验证结果
  const issueCount = await page.locator('[data-testid="issue-item"]').count();
  expect(issueCount).toBeGreaterThan(0);
});
```

### 9.2 测试数据

#### 9.2.1 示例文件

创建测试用代码文件：
- `test_samples/js/good.js` - 无问题代码
- `test_samples/js/bad.js` - 包含多种问题
- `test_samples/rust/good.rs` - 优质 Rust 代码
- `test_samples/rust/bad.rs` - 包含问题

#### 9.2.2 Mock 数据

```rust
// 测试用的审查记录
fn mock_review_record() -> ReviewRecord {
    ReviewRecord {
        id: "test-123".to_string(),
        timestamp: Utc::now(),
        review_type: ReviewType::AI,
        scope: ReviewScope::SINGLE_FILE,
        branch: "main".to_string(),
        files: vec![ReviewFile {
            path: "test.js".to_string(),
            language: "javascript".to_string(),
            size: 1024,
            ..Default::default()
        }],
        results: ReviewResults {
            ai: Some(mock_ai_result()),
            ..Default::default()
        },
        status: ReviewStatus::COMPLETED,
        ..Default::default()
    }
}
```

### 9.3 测试覆盖

| 测试类型 | 目标覆盖率 | 关键场景 |
|---------|------------|----------|
| 单元测试 | 90%+ | 核心逻辑函数 |
| 集成测试 | 100% | 模块间交互 |
| E2E 测试 | 100% | 完整用户流程 |
| 性能测试 | N/A | 响应时间、并发 |
| 兼容性测试 | N/A | 多 OS、Git 版本 |

### 9.4 持续集成

**.github/workflows/test.yml**:
```yaml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions/setup-node@v3
        with:
          node-version: 18
      - run: npm install
      - run: npm run test:unit

      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - run: cargo test

  e2e:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - run: npm install
      - run: npx playwright install
      - run: npm run test:e2e
```

---

## 10. 部署方案

### 10.1 构建配置

#### 10.1.1 Tauri 配置

**src-tauri/tauri.conf.json**:
```json
{
  "bundle": {
    "identifier": "com.gitmentor.code-review",
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ]
  },
  "allowlist": {
    "all": false,
    "shell": {
      "all": false,
      "open": true
    },
    "protocol": {
      "asset": true,
      "assetScope": ["**"]
    }
  }
}
```

#### 10.1.2 依赖管理

**Cargo.toml**:
```toml
[dependencies]
# Code Review 模块新增
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.0", features = ["full"] }
anyhow = "1.0"
uuid = { version = "1.0", features = ["v4"] }
regex = "1.0"
clap = { version = "4.0", features = ["derive"] }

# 复用现有依赖
git2 = "0.18"
reqwest = { version = "0.11", features = ["stream", "json"] }
```

### 10.2 发布流程

#### 10.2.1 版本管理

**版本号规则**: `vX.Y.Z`
- X: 主版本号（重大功能更新）
- Y: 次版本号（新功能）
- Z: 修订版本号（Bug 修复）

**分支策略**:
- `main` - 主分支，稳定版本
- `dev` - 开发分支（当前）
- `feature/codereview-*` - 功能分支
- `release/vX.Y` - 发布分支

#### 10.2.2 构建脚本

**package.json**:
```json
{
  "scripts": {
    "codereview:build": "npm run type-check && npm run lint && npm run test:unit",
    "codereview:dev": "npm run codereview:build && npm run tauri:dev",
    "codereview:test": "npm run test:unit && npm run test:integration",
    "codereview:release": "npm run codereview:build && npm run tauri:build"
  }
}
```

#### 10.2.3 自动化发布

**.github/workflows/release.yml**:
```yaml
name: Release

on:
  push:
    tags:
      - 'v*'

jobs:
  release:
    permissions:
      contents: write
    strategy:
      fail-fast: false
      matrix:
        platform: [macos-latest, ubuntu-20.04, windows-latest]
    runs-on: ${{ matrix.platform }}
    steps:
      - uses: actions/checkout@v3

      - name: Rust setup
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Sync node version and setup cache
        uses: actions/setup-node@v3
        with:
          node-version: '18'

      - name: Install frontend dependencies
        run: npm ci

      - name: Tauri build
        uses: tauri-apps/tauri-action@v0
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        with:
          tagName: ${{ github.ref_name }}
          releaseName: App Name v__VERSION__
          releaseBody: See CHANGELOG.md for details
          releaseDraft: true
          prerelease: false
```

### 10.3 安装包配置

#### 10.3.1 Windows

**NSIS 脚本** (可选):
```nsis
# 安装 Code Review 功能
Section "CodeReview"
  SetOutPath $INSTDIR
  File /r "codereview\*"
  WriteUninstaller "$INSTDIR\Uninstall-CodeReview.exe"
SectionEnd

# 卸载时清理配置
Section "Uninstall"
  RMDir /r "$APPDATA\GitMentor\code_review*"
SectionEnd
```

#### 10.3.2 macOS

**DMG 背景**:
- 添加 Code Review 功能介绍
- 拖拽安装提示

#### 10.3.3 Linux

**AppImage 配置**:
- 包含所有依赖
- 自解压安装

### 10.4 配置迁移

#### 10.4.1 升级检查

```rust
// 启动时检查版本
pub fn check_and_migrate_config() -> Result<()> {
    let config_dir = get_config_dir();
    let config_path = config_dir.join("code_review_config.json");

    if !config_path.exists() {
        // 首次使用，创建默认配置
        let default_config = ReviewConfig::default();
        default_config.save(&config_path)?;
        return Ok(());
    }

    // 读取配置
    let config = ReviewConfig::load(&config_path)?;

    // 检查版本并迁移
    if config.version != CURRENT_VERSION {
        let migrated_config = migrate_config(config)?;
        migrated_config.save(&config_path)?;
    }

    Ok(())
}
```

#### 10.4.2 数据迁移

```rust
// 数据结构升级
fn migrate_config(old: ReviewConfig) -> Result<ReviewConfig> {
    match old.version.as_str() {
        "1.0" => Ok(migrate_1_0_to_1_1(old)),
        _ => Err(anyhow!("Unsupported config version")),
    }
}
```

### 10.5 更新机制

#### 10.5.1 内置更新

```rust
// 检查 Code Review 模块更新
pub async fn check_module_update() -> Result<Option<UpdateInfo>> {
    let current_version = env!("CARGO_PKG_VERSION");
    let update_url = format!("{}/api/codereview/updates", UPDATE_SERVER);

    let response = reqwest::get(&update_url).await?;
    let updates: Vec<UpdateInfo> = response.json().await?;

    Ok(updates.into_iter().find(|u| u.version > current_version))
}
```

#### 10.5.2 便携版更新

```rust
// 便携版 Code Review 模块更新
pub async fn update_portable_module() -> Result<()> {
    let update_info = check_module_update().await?;
    if let Some(info) = update_info {
        // 下载更新
        let mut file = tokio::fs::File::create("codereview_update.zip").await?;
        let mut response = reqwest::get(&info.download_url).await?;
        tokio::io::copy(&mut response, &mut file).await?;

        // 解压并替换
        unzip_and_replace("codereview_update.zip")?;
        cleanup_update_files()?;
    }

    Ok(())
}
```

---

## 11. 风险评估

### 11.1 技术风险

#### 11.1.1 AI 依赖风险

**风险描述**: 过度依赖外部 AI 服务，可能面临服务中断、成本上升、API 限制等问题

**影响等级**: 高

**缓解措施**:
- ✅ 支持多 AI 提供商（OpenAI、Anthropic 等）
- ✅ 实现本地缓存，避免重复调用
- ✅ 提供离线模式（基础 Lint 检查）
- ✅ 用户可配置 Token 限制和成本预算
- ✅ 失败重试机制和降级策略

**应急预案**:
- 当所有 AI 服务不可用时，自动切换到 Lint 模式
- 提供本地模型支持（Ollama）
- 提示用户等待或手动输入审查建议

#### 11.1.2 性能风险

**风险描述**: AI 审查响应时间过长，大文件或大量文件审查导致 UI 卡顿

**影响等级**: 中

**缓解措施**:
- ✅ 流式响应，实时显示进度
- ✅ 分批处理（一次最多 10 个文件）
- ✅ 后台异步处理，不阻塞 UI
- ✅ 文件大小限制（最大 1MB）
- ✅ 并发数限制（最多 3 个审查任务）

**监控指标**:
- 响应时间 < 30s（深度审查）
- UI 响应性 > 60fps
- 内存使用 < 500MB

#### 11.1.3 数据一致性问题

**风险描述**: 并发写入历史记录，可能导致数据损坏或丢失

**影响等级**: 中

**缓解措施**:
- ✅ 使用 RwLock 确保读写安全
- ✅ 写入前备份，失败时回滚
- ✅ 定期数据完整性检查
- ✅ 版本号机制，检测冲突

**测试场景**:
- 并发 100 次写入
- 写入过程中强制中断
- 磁盘空间不足

#### 11.1.4 Lint 工具集成风险

**风险描述**: 外部 Lint 工具不可用或版本不兼容

**影响等级**: 中

**缓解措施**:
- ✅ 自动检测工具是否安装
- ✅ 提供手动安装指南
- ✅ Lint 为可选功能，不影响核心审查
- ✅ 兼容多个 Lint 版本

**回退方案**:
- Lint 不可用时，仅使用 AI 审查
- 提供错误提示和安装指导

### 11.2 产品风险

#### 11.2.1 用户接受度

**风险描述**: 用户对 AI 审查结果的准确性和实用性不满意

**影响等级**: 高

**缓解措施**:
- ✅ 提供多种审查深度选择
- ✅ 允许用户自定义审查规则
- ✅ 支持人工审查流程
- ✅ 收集用户反馈，持续优化提示词

**验证方法**:
- Beta 测试收集反馈
- A/B 测试对比不同提示词效果
- 用户满意度调查

#### 11.2.2 复杂度增加

**风险描述**: 功能过于复杂，增加学习成本，降低用户体验

**影响等级**: 中

**缓解措施**:
- ✅ 渐进式开发，分阶段交付
- ✅ 简化默认配置，降低使用门槛
- ✅ 提供操作指南和示例
- ✅ 智能默认设置，减少配置工作

**UX 原则**:
- 80/20 原则：80% 用户使用 20% 的功能
- 默认即最佳实践
- 一键操作，减少点击数

#### 11.2.3 与现有功能冲突

**风险描述**: 新功能与现有 Git 操作产生冲突或覆盖

**影响等级**: 低

**缓解措施**:
- ✅ 代码审查独立 Tab 页，不影响主界面
- ✅ 清晰的视觉区分
- ✅ 详细的功能说明
- ✅ 完整的单元测试和集成测试

**测试验证**:
- 回归测试确保现有功能正常
- 多场景测试验证兼容性

### 11.3 资源风险

#### 11.3.1 开发周期

**风险描述**: 实际开发时间超出预期，影响项目整体进度

**影响等级**: 中

**缓解措施**:
- ✅ 采用渐进式开发，先交付 MVP
- ✅ 每周评估进度，及时调整范围
- ✅ 并行开发前端和后端
- ✅ 重用现有基础设施

**时间缓冲**:
- 每个阶段预留 20% 缓冲时间
- 关键路径预留额外缓冲

#### 11.3.2 维护成本

**风险描述**: 新功能增加维护工作量

**影响等级**: 中

**缓解措施**:
- ✅ 模块化设计，降低耦合
- ✅ 完整的文档和注释
- ✅ 自动化测试减少回归成本
- ✅ 配置驱动，减少硬编码

**维护计划**:
- 每月审查代码质量
- 季度重构和优化
- 年度架构评估

### 11.4 风险矩阵

| 风险类别 | 风险项 | 概率 | 影响 | 等级 | 应对策略 |
|---------|--------|------|------|------|----------|
| 技术 | AI 服务中断 | 中 | 高 | 高 | 多提供商 + 本地缓存 |
| 技术 | 性能问题 | 中 | 中 | 中 | 流式响应 + 分批处理 |
| 技术 | 数据一致性问题 | 低 | 中 | 低 | 读写锁 + 备份 |
| 技术 | Lint 集成失败 | 中 | 中 | 中 | 可选功能 + 回退 |
| 产品 | 用户接受度低 | 中 | 高 | 高 | Beta 测试 + 持续优化 |
| 产品 | 功能过于复杂 | 低 | 中 | 低 | 渐进式 + 简化默认 |
| 资源 | 开发周期延长 | 中 | 中 | 中 | MVP + 时间缓冲 |
| 资源 | 维护成本高 | 低 | 中 | 低 | 模块化 + 自动化 |

### 11.5 监控与预警

#### 11.5.1 指标监控

**技术指标**:
- AI 响应时间
- 审查成功率
- 错误率
- 内存/CPU 使用

**产品指标**:
- 功能使用率
- 用户满意度
- 错误报告数

#### 11.5.2 预警机制

```rust
// 监控指标收集
pub struct ReviewMetrics {
    pub response_time: Duration,
    pub token_count: u32,
    pub success: bool,
    pub error_message: Option<String>,
}

impl ReviewMetrics {
    pub fn record(&self) {
        // 发送到监控系统
        send_metrics(self);
    }
}

// 预警规则
pub struct AlertRule {
    pub metric: String,
    pub threshold: f64,
    pub severity: AlertSeverity,
}
```

---

## 12. 时间规划

### 12.1 整体时间线

```
项目周期: 4 周 (可扩展至 6 周)

阶段 1: MVP 核心功能              1 周
  ├─ Day 1-2: 后端核心模块开发
  ├─ Day 3-4: 前端界面开发
  ├─ Day 5: 集成和调试
  └─ Day 6-7: 测试和文档

阶段 2: 增强功能                  1 周
  ├─ Day 1-2: 人工审查流程
  ├─ Day 3-4: 评论系统
  ├─ Day 5-6: 历史记录和搜索
  └─ Day 7: 测试和优化

阶段 3: 专业功能                  1-2 周
  ├─ Day 1-3: Lint 工具集成
  ├─ Day 4-6: 统计和分析
  ├─ Day 7-10: 配置管理
  └─ Day 11-14: 测试和完善

阶段 4: 协作功能 (可选)          1-2 周
  ├─ Day 1-3: 多审查者支持
  ├─ Day 4-6: 审批工作流
  ├─ Day 7-10: 通知机制
  └─ Day 11-14: CI/CD 集成
```

### 12.2 详细任务分解

#### 12.2.1 阶段 1: MVP (第 1 周)

**Day 1-2: 后端核心模块开发**

- [ ] 设计并实现 `ReviewRecord` 等核心类型定义
- [ ] 实现 `ReviewStorage` 存储模块
- [ ] 实现 `CodeReviewEngine` 审查引擎
- [ ] 复用 `AIManager` 实现 AI 审查功能
- [ ] 实现 Tauri 命令接口

**任务卡**:
```
任务: 实现 CodeReviewEngine
负责人: [待分配]
预估: 8 小时
依赖: AIManager
输出:
  - src-tauri/src/core/code_review_engine.rs
  - src-tauri/src/commands/code_review_commands.rs
验收: 通过单元测试
```

**Day 3-4: 前端界面开发**

- [ ] 创建 `CodeReviewPanel.vue` 主面板
- [ ] 创建 `ReviewDetailView.vue` 详情页
- [ ] 实现 `useCodeReview.ts` Composables
- [ ] 设计 UI 组件和样式
- [ ] 实现 Pinia 状态管理

**Day 5: 集成和调试**

- [ ] 前后端联调
- [ ] 修复 bug
- [ ] 性能优化
- [ ] 流式响应测试

**Day 6-7: 测试和文档**

- [ ] 编写单元测试
- [ ] 编写集成测试
- [ ] 编写用户文档
- [ ] 代码审查

**交付物**:
- ✅ AI 代码审查功能（基础/标准/深度）
- ✅ 单文件/多文件审查
- ✅ 审查结果展示
- ✅ 历史记录保存和查询
- ✅ 审查配置管理

#### 12.2.2 阶段 2: 增强功能 (第 2 周)

**Day 1-2: 人工审查流程**

- [ ] 实现 `ReviewWorkflowManager`
- [ ] 创建审查请求界面
- [ ] 实现审查者分配机制
- [ ] 实现状态跟踪

**Day 3-4: 评论系统**

- [ ] 实现评论功能
- [ ] 支持多轮回复
- [ ] 评论历史记录
- [ ] 评论富文本支持

**Day 5-6: 历史记录和搜索**

- [ ] 实现 `ReviewHistoryList.vue`
- [ ] 实现搜索和过滤功能
- [ ] 实现分页和排序
- [ ] 优化查询性能

**Day 7: 测试和优化**

- [ ] E2E 测试
- [ ] 性能测试
- [ ] 修复 bug
- [ ] 用户体验优化

**交付物**:
- ✅ 历史 commit 审查
- ✅ 人工审查流程
- ✅ 评论和讨论功能
- ✅ 审查状态管理
- ✅ 高级搜索和过滤

#### 12.2.3 阶段 3: 专业功能 (第 3-4 周)

**Day 1-3: Lint 工具集成**

- [ ] 实现 `LintIntegrator`
- [ ] 集成 ESLint
- [ ] 集成 Clippy
- [ ] 集成 Pylint（可选）
- [ ] Lint 结果可视化

**Day 4-6: 统计和分析**

- [ ] 实现 `ReviewStatistics`
- [ ] 创建统计图表
- [ ] 实现数据导出
- [ ] 趋势分析

**Day 7-10: 配置管理**

- [ ] 实现规则配置界面
- [ ] 实现模板管理
- [ ] 实现自动规则
- [ ] 配置导入/导出

**Day 11-14: 测试和完善**

- [ ] 全面测试
- [ ] 性能优化
- [ ] 文档完善
- [ ] Beta 发布

**交付物**:
- ✅ Lint 工具集成
- ✅ 统计分析功能
- ✅ 审查规则配置
- ✅ 数据导出
- ✅ 完整的用户文档

#### 12.2.4 阶段 4: 协作功能 (第 5-6 周，可选)

**Day 1-3: 多审查者支持**

- [ ] 审查者管理
- [ ] 审查分配策略
- [ ] 审查进度跟踪
- [ ] 审查权重设置

**Day 4-6: 审批工作流**

- [ ] 审批规则配置
- [ ] 审批决策记录
- [ ] 审批历史追踪
- [ ] 自动审批

**Day 7-10: 通知机制**

- [ ] 通知规则设置
- [ ] 邮件通知（可选）
- [ ] 应用内通知
- [ ] Webhook 支持

**Day 11-14: CI/CD 集成**

- [ ] GitHub Actions 集成
- [ ] GitLab CI 集成
- [ ] 自动审查触发
- [ ] 审查结果反馈

**交付物**:
- ✅ 多审查者协作
- ✅ 复杂审批工作流
- ✅ 通知机制
- ✅ CI/CD 集成

### 12.3 里程碑计划

| 里程碑 | 日期 | 交付物 | 验收标准 |
|--------|------|--------|----------|
| M1: MVP 完成 | 第 1 周末 | AI 审查基础功能 | 可以进行基础/标准/深度审查 |
| M2: Beta 发布 | 第 2 周末 | 完整审查流程 | 支持 AI + 人工审查 |
| M3: v1.0 发布 | 第 4 周末 | 专业级功能 | Lint 集成 + 统计分析 |
| M4: v2.0 发布 | 第 6 周末 | 协作功能 | 多审查者 + 审批工作流 |

### 12.4 资源投入

#### 12.4.1 人力资源

**核心开发人员**:
- 后端开发: 1 人 (Rust + Tauri)
- 前端开发: 1 人 (Vue 3 + TypeScript)
- 测试: 1 人 (兼职)

**时间分配**:
- 后端开发: 60% 时间
- 前端开发: 60% 时间
- 测试: 30% 时间
- 文档: 20% 时间

#### 12.4.2 技术资源

**开发环境**:
- IDE: VS Code / CLion
- 版本控制: Git
- CI/CD: GitHub Actions
- 测试工具: Playwright + Cargo test

**外部服务**:
- AI 提供商 API 费用 (估算: $50/月)
- 云存储 (可选): 备份历史记录

### 12.5 风险缓冲

#### 12.5.1 时间缓冲

- 每个阶段预留 20% 时间缓冲
- 关键任务预留额外 1-2 天
- 集成测试预留 2-3 天

#### 12.5.2 功能裁剪

**优先级排序**:
1. **P0** (必须): AI 审查 + 历史记录
2. **P1** (重要): 人工审查 + 搜索过滤
3. **P2** (一般): Lint 集成 + 统计分析
4. **P3** (可选): 协作功能 + CI/CD

**如果时间不足**:
- 砍掉 P3 功能
- 简化 P2 功能
- 减少 Lint 工具数量（仅 ESLint）

### 12.6 成功指标

#### 12.6.1 技术指标

- [ ] 代码覆盖率 > 80%
- [ ] 单元测试通过率 100%
- [ ] AI 审查响应时间 < 10s (标准级)
- [ ] 内存使用 < 500MB
- [ ] 崩溃率 < 0.1%

#### 12.6.2 产品指标

- [ ] 功能完成度 > 90%
- [ ] 用户满意度 > 4.0/5.0
- [ ] 功能使用率 > 60%
- [ ] Bug 报告 < 10 个/周

#### 12.6.3 业务指标

- [ ] MVP 1 周内交付
- [ ] Beta 测试参与 > 50 人
- [ ] 用户反馈处理时间 < 48 小时
- [ ] 新功能留存率 > 70%

---

## 总结

本文档详细描述了 GitMentor-Lite Code Review 模块的完整设计方案，包括需求分析、系统设计、技术架构、数据库设计、API 设计、界面设计、实现方案、测试策略、部署方案、风险评估和时间规划。

通过采用**渐进式演进方案**，我们可以在 1 周内交付 MVP（核心功能），并在 4 周内完成所有计划功能，最终可扩展至 6 周支持企业级协作功能。

整个设计遵循以下原则：
- ✅ **模块化设计**: 降低耦合，提高可维护性
- ✅ **配置驱动**: 通过配置文件控制功能行为
- ✅ **渐进式实施**: 分阶段交付，降低风险
- ✅ **用户体验优先**: 简化操作，提供直观界面
- ✅ **可扩展性**: 预留扩展点，支持未来发展

通过本设计文档，团队可以：
1. 明确功能范围和目标
2. 了解技术实现方案
3. 规划开发时间和资源
4. 评估风险和应对策略
5. 制定测试和部署计划

下一步将进入实施阶段，按照本设计文档逐步开发 Code Review 模块。
