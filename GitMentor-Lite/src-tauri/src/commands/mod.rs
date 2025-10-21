// 模块路由：集中导出所有 Tauri 命令模块
// Author: Evilek, Date: 2025-08-11
// 这个SB文件只是把命令模块拼在一起，别tm在这里写业务逻辑

pub mod ai_analysis_commands;
pub mod ai_commands;
pub mod daily_report_commands;
pub mod debug_commands;
pub mod git_commands;
pub mod git_config_commands;
pub mod gitflow_commands;
pub mod system_commands;
pub mod template_commands;
pub mod unified_template_commands;
pub mod update_commands;
