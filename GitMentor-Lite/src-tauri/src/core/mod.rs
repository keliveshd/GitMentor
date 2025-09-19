// 核心模块聚合：职责边界说明
// Author: Evilek, Date: 2025-08-11
// - ai_manager / ai_provider / providers：AI 提供商与请求调度
// - prompt_manager：提示词模板与两段式处理
// - git_engine / git_config：Git 操作与执行模式管理
// - layered_commit_manager：分层提交编排与取消
// - conversation_logger：AI 请求/响应记录
// - report_engine：日报生成与缓存管理
// 别tm在这里写业务，写清楚模块边界，改就加注释。

pub mod ai_analysis_prompts;
pub mod ai_config;
pub mod ai_manager;
pub mod ai_provider;
pub mod conversation_logger;
pub mod git_config;
pub mod git_engine;
pub mod layered_commit_manager;
pub mod llm_client;
pub mod prompt_manager;
pub mod providers;
pub mod report_engine;
pub mod response_cleaner;
pub mod update_manager;
