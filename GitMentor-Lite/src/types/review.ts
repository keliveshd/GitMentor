/**
 * Code Review 模块类型定义
 * 作者：Evilek
 * 日期：2025-01-04
 */

// ==================== 审查记录相关类型 ====================

/**
 * 审查记录
 */
export interface ReviewRecord {
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

/**
 * 审查文件
 */
export interface ReviewFile {
  path: string;                  // 文件路径
  oldContent?: string;           // 原内容（如果是提交审查）
  newContent?: string;           // 新内容
  diff?: string;                 // Diff 信息
  language: string;              // 编程语言
  size: number;                  // 文件大小（字节）
}

/**
 * 审查结果
 */
export interface ReviewResults {
  ai?: AIReviewResult;           // AI 审查结果
  lint?: LintResult;             // Lint 结果
  manual?: ManualReview;         // 人工审查结果
}

/**
 * 审查类型
 */
export enum ReviewType {
  AI = 'AI',
  MANUAL = 'MANUAL',
  LINT = 'LINT',
  MIXED = 'MIXED'
}

/**
 * 审查范围
 */
export enum ReviewScope {
  SINGLE_FILE = 'SINGLE_FILE',
  MULTIPLE_FILES = 'MULTIPLE_FILES',
  COMMIT = 'COMMIT',
  BRANCH = 'BRANCH'
}

/**
 * 审查状态
 */
export enum ReviewStatus {
  PENDING = 'PENDING',
  IN_PROGRESS = 'IN_PROGRESS',
  COMPLETED = 'COMPLETED',
  APPROVED = 'APPROVED',
  CHANGES_REQUESTED = 'CHANGES_REQUESTED',
  REJECTED = 'REJECTED'
}

/**
 * 审查深度
 */
export enum ReviewDepth {
  BASIC = 'BASIC',               // 基础：语法、明显错误
  STANDARD = 'STANDARD',         // 标准：质量、性能
  DEEP = 'DEEP'                  // 深度：架构、安全
}

// ==================== AI 审查结果类型 ====================

/**
 * AI 审查结果
 */
export interface AIReviewResult {
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

/**
 * 代码问题
 */
export interface CodeIssue {
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

/**
 * 代码建议
 */
export interface CodeSuggestion {
  id: string;
  type: SuggestionType;
  title: string;
  description: string;
  impact: string;
  effort: 'LOW' | 'MEDIUM' | 'HIGH';
  filesAffected: string[];
}

/**
 * 问题严重性
 */
export enum IssueSeverity {
  CRITICAL = 'CRITICAL',         // 严重：必须修复
  HIGH = 'HIGH',                // 高：强烈建议修复
  MEDIUM = 'MEDIUM',            // 中：建议修复
  LOW = 'LOW',                  // 低：可选修复
  INFO = 'INFO'                 // 信息：提示
}

/**
 * 问题类型
 */
export enum IssueType {
  // 代码质量
  CODE_SMELL = 'CODE_SMELL',    // 代码异味
  COMPLEXITY = 'COMPLEXITY',    // 复杂度

  // 性能
  PERFORMANCE = 'PERFORMANCE',  // 性能问题
  INEFFICIENT = 'INEFFICIENT',  // 低效实现

  // 安全
  SECURITY = 'SECURITY',        // 安全漏洞
  VULNERABILITY = 'VULNERABILITY',

  // 最佳实践
  BEST_PRACTICE = 'BEST_PRACTICE',
  STYLE = 'STYLE',              // 风格问题

  // 错误
  BUG = 'BUG',                  // 潜在 Bug
  ERROR = 'ERROR',              // 错误

  // 可维护性
  MAINTAINABILITY = 'MAINTAINABILITY',
  READABILITY = 'READABILITY',

  // 其他
  DOCUMENTATION = 'DOCUMENTATION',
  DEPRECATED = 'DEPRECATED',
  UNUSED = 'UNUSED'
}

/**
 * 建议类型
 */
export enum SuggestionType {
  REFACTOR = 'REFACTOR',         // 重构
  OPTIMIZE = 'OPTIMIZE',         // 优化
  FIX = 'FIX',                   // 修复
  DOCUMENT = 'DOCUMENT',         // 文档
  TEST = 'TEST',                 // 测试
  REMOVE = 'REMOVE',             // 移除
  EXTRACT = 'EXTRACT'            // 提取
}

/**
 * 审查指标
 */
export interface ReviewMetrics {
  totalFiles: number;           // 文件总数
  totalLines: number;           // 代码行数
  issuesFound: number;          // 发现的问题数
  issuesBySeverity: IssuesBySeverity;
  issuesByType: IssuesByType;
  complexityScore?: number;     // 复杂度评分
  maintainabilityIndex?: number; // 可维护性指数
}

/**
 * 按严重性分类的问题数量
 */
export interface IssuesBySeverity {
  critical: number;
  high: number;
  medium: number;
  low: number;
  info: number;
}

/**
 * 按类型分类的问题数量
 */
export interface IssuesByType {
  [IssueType.CODE_SMELL]?: number;
  [IssueType.PERFORMANCE]?: number;
  [IssueType.SECURITY]?: number;
  [IssueType.BUG]?: number;
  [IssueType.BEST_PRACTICE]?: number;
  [IssueType.STYLE]?: number;
  [IssueType.ERROR]?: number;
  [IssueType.MAINTAINABILITY]?: number;
  [IssueType.READABILITY]?: number;
  [IssueType.DOCUMENTATION]?: number;
  [IssueType.DEPRECATED]?: number;
  [IssueType.UNUSED]?: number;
  [IssueType.COMPLEXITY]?: number;
  [IssueType.INEFFICIENT]?: number;
  [IssueType.VULNERABILITY]?: number;
}

// ==================== 配置相关类型 ====================

/**
 * 审查配置
 */
export interface ReviewConfig {
  version: string;              // 配置版本
  ai: AIConfig;                 // AI 审查配置
  lint: LintConfig;             // Lint 配置
  rules: ReviewRule[];          // 审查规则
  autoRules: AutoRule[];        // 自动规则
}

/**
 * AI 配置
 */
export interface AIConfig {
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

/**
 * Lint 配置
 */
export interface LintConfig {
  enabled: boolean;
  tools: {
    eslint?: ESLintConfig;
    clippy?: ClippyConfig;
    pylint?: PylintConfig;
  };
  autoFix: boolean;
  ignorePatterns: string[];
}

/**
 * ESLint 配置
 */
export interface ESLintConfig {
  enabled: boolean;
  configFile?: string;
  useConfig?: any;
}

/**
 * Clippy 配置
 */
export interface ClippyConfig {
  enabled: boolean;
  level: 'warn' | 'deny';
}

/**
 * Pylint 配置
 */
export interface PylintConfig {
  enabled: boolean;
  configFile?: string;
}

/**
 * 审查规则
 */
export interface ReviewRule {
  id: string;
  name: string;
  description: string;
  enabled: boolean;
  severity: IssueSeverity;
  patterns: string[];           // 匹配模式
  actions: RuleAction[];
}

/**
 * 自动规则
 */
export interface AutoRule {
  id: string;
  condition: string;            // 触发条件
  action: string;               // 自动操作
  enabled: boolean;
}

/**
 * 规则动作
 */
export enum RuleAction {
  WARN = 'WARN',
  ERROR = 'ERROR',
  SUGGEST = 'SUGGEST',
  IGNORE = 'IGNORE'
}

// ==================== Lint 结果类型 ====================

/**
 * Lint 结果
 */
export interface LintResult {
  tool: string;                 // Lint 工具名称 (ESLint/Clippy/Pylint)
  version: string;              // 工具版本
  executionTime: number;        // 执行时间（毫秒）
  errors: LintError[];          // 错误列表
  warnings: LintWarning[];      // 警告列表
  summary: LintSummary;         // 摘要
  config: LintConfig;           // 使用的配置
}

/**
 * Lint 错误
 */
export interface LintError {
  file: string;
  line: number;
  column: number;
  ruleId: string;
  severity: 'error' | 'warning';
  message: string;
  suggestion?: string;
}

/**
 * Lint 警告
 */
export interface LintWarning {
  file: string;
  line: number;
  ruleId: string;
  message: string;
}

/**
 * Lint 摘要
 */
export interface LintSummary {
  totalFiles: number;
  totalErrors: number;
  totalWarnings: number;
}

// ==================== 人工审查类型 ====================

/**
 * 人工审查
 */
export interface ManualReview {
  workflowId: string;           // 工作流 ID
  reviewers: Reviewer[];        // 审查者
  decision?: ReviewDecision;    // 最终决策
  discussions: Discussion[];    // 讨论线程
  createdAt: string;
  completedAt?: string;
  status: ReviewStatus;
}

/**
 * 审查者
 */
export interface Reviewer {
  id: string;
  name: string;
  email: string;
  role: ReviewerRole;
  status: ReviewerStatus;
  joinedAt: string;
}

/**
 * 审查者角色
 */
export enum ReviewerRole {
  REVIEWER = 'REVIEWER',
  LEAD_REVIEWER = 'LEAD_REVIEWER',
  OBSERVER = 'OBSERVER'
}

/**
 * 审查者状态
 */
export enum ReviewerStatus {
  PENDING = 'PENDING',
  APPROVED = 'APPROVED',
  REQUESTED_CHANGES = 'REQUESTED_CHANGES'
}

/**
 * 审查决策
 */
export interface ReviewDecision {
  reviewer: string;
  decision: ReviewerStatus;
  comment?: string;
  timestamp: string;
}

/**
 * 讨论
 */
export interface Discussion {
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

/**
 * 回复
 */
export interface Reply {
  id: string;
  author: string;
  content: string;
  timestamp: string;
}

/**
 * 评论
 */
export interface ReviewComment {
  id: string;
  author: string;
  content: string;
  timestamp: string;
  resolved: boolean;
}

// ==================== 统计数据类型 ====================

/**
 * 统计数据
 */
export interface ReviewStatistics {
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

/**
 * 统计周期
 */
export enum StatisticsPeriod {
  LAST_WEEK = 'LAST_WEEK',
  LAST_MONTH = 'LAST_MONTH',
  LAST_QUARTER = 'LAST_QUARTER',
  LAST_YEAR = 'LAST_YEAR',
  ALL_TIME = 'ALL_TIME'
}

/**
 * 按类型分类的审查数量
 */
export interface ReviewsByType {
  ai: number;
  manual: number;
  lint: number;
  mixed: number;
}

/**
 * Token 使用统计
 */
export interface TokenUsage {
  total: number;
  byProvider: {
    [provider: string]: number;
  };
  byModel: {
    [model: string]: number;
  };
  cost: number;                 // 估算成本（美元）
}

/**
 * 常见问题
 */
export interface TopIssue {
  type: IssueType;
  count: number;
  percentage: number;
}

/**
 * 贡献者统计
 */
export interface ContributorStats {
  name: string;
  reviews: number;
  issues: number;
  approved: number;
}

/**
 * 趋势数据
 */
export interface TrendData {
  reviews: number[];
  issues: number[];
  labels: string[];
}

// ==================== 筛选和查询类型 ====================

/**
 * 审查筛选条件
 */
export interface ReviewFilters {
  dateFrom?: string;
  dateTo?: string;
  reviewTypes?: ReviewType[];
  statuses?: ReviewStatus[];
  files?: string[];
  reviewers?: string[];
  query?: string;
}

/**
 * 分页选项
 */
export interface PaginationOptions {
  page: number;
  pageSize: number;
  sortBy?: string;
  sortOrder?: 'asc' | 'desc';
}

/**
 * 分页结果
 */
export interface PaginatedResult<T> {
  total: number;
  page: number;
  pageSize: number;
  data: T[];
}

// ==================== 导出和统计类型 ====================

/**
 * 导出格式
 */
export enum ExportFormat {
  JSON = 'JSON',
  MARKDOWN = 'MARKDOWN',
  CSV = 'CSV',
  HTML = 'HTML'
}

/**
 * 导出选项
 */
export interface ExportOptions {
  format: ExportFormat;
  includeComments: boolean;
  includeMetrics: boolean;
  template?: string;
}

// ==================== 错误类型 ====================

/**
 * 审查错误
 */
export enum ReviewError {
  REPOSITORY_NOT_SELECTED = 'REPOSITORY_NOT_SELECTED',
  FILES_NOT_FOUND = 'FILES_NOT_FOUND',
  AI_PROVIDER_ERROR = 'AI_PROVIDER_ERROR',
  TOKEN_LIMIT_EXCEEDED = 'TOKEN_LIMIT_EXCEEDED',
  INVALID_CONFIG = 'INVALID_CONFIG',
  PERMISSION_DENIED = 'PERMISSION_DENIED',
  CACHE_ERROR = 'CACHE_ERROR',
  SERIALIZATION_ERROR = 'SERIALIZATION_ERROR',
  FILESYSTEM_ERROR = 'FILESYSTEM_ERROR',
  TIMEOUT = 'TIMEOUT',
  INTERRUPTED = 'INTERRUPTED',
  LINT_TOOL_ERROR = 'LINT_TOOL_ERROR'
}

/**
 * 审查错误信息
 */
export interface ReviewErrorInfo {
  code: ReviewError;
  message: string;
  details?: any;
  timestamp: string;
}
