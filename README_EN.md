# GitMentor Lite

GitMentor Lite is a cross‑platform desktop app built with Tauri + Vue 3 + Rust. It generates high‑quality Git commit messages using multiple AI providers and configurable templates, and integrates essential Git operations including "Layered Commit" workflow, Gitflow management, and remote repository support.

---

## Overview

GitMentor Lite focuses on "Commit Message Automation + Git Panel + Gitflow Workflow." The frontend is built with Vue 3 + Element Plus, and the backend uses Tauri v2 + Rust. With git2 and multiple AI providers, it enables:

- AI‑generated standardized commit messages (template‑driven, parameterized)
- Layered commit session orchestration
- Complete Gitflow workflow management
- Remote repository cloning and management
- Intelligent branch switching and repository operations
- Common Git operations: view changes, stage, commit, revert

### Latest Feature Updates (October 2025)

- ✨ **Complete Gitflow Support**: Full lifecycle management for feature, release, and hotfix branches
- 🌐 **Remote Repository Management**: Repository cloning, remote configuration, branch push/pull operations
- 🧠 **Intelligent Branch Switching**: Smart branch management and switching based on branch ownership
- 📊 **Enhanced Git Panel**: More comprehensive branch history and operation interface
- 🔧 **Git Engine Optimization**: More powerful Git operations with improved error handling

## Features

### Commit Message Generation
- Template‑driven two‑stage processing (language, max tokens, temperature, emoji, types, etc.)
- Layered Commit workflow support
- Intelligent commit message analysis and optimization suggestions

### Git Panel
- Diff / Stage / Commit / Revert operations
- Branch history and visualization
- Staging area management and rollback operations

### Gitflow Workflow Management
- Feature branch creation and management
- Release branch and version management
- Hotfix branch and emergency fixes
- Gitflow visualization dashboard and progress tracking

### Remote Repository Support
- Git repository cloning
- Remote repository configuration and management
- Dynamic repository name resolution
- Branch push and pull operations

### Intelligent Branch Operations
- Smart branch switching
- Branch ownership management
- Context‑based branch suggestions

### Providers (Implemented & Registered)
- OpenAI, Ollama, Zhipu, Anthropic, DashScope, Doubao, Gemini, Deepseek
- SiliconCloud, Together, OpenRouter, and more

### Conversation Logging & Progress Streaming
- Records AI requests/responses and sessions for debugging/audit
- Real‑time streaming output display
- Detailed operation logs and error tracking

### Template Version Management
- Commit template version control
- Template customization and management
- Template history and rollback features

### Desktop‑Native Experience
- Tauri v2, lightweight and secure; Windows MSI packaging by default
- Optional Git sidecar bundling
- Cross‑platform support (Windows/Mac/Linux)

## Tech Stack

- **Desktop**: Tauri v2 (opener, dialog, shell plugins)
- **Frontend**: Vue 3 + TypeScript + Vite + Element Plus
- **Backend**: Rust (git2, reqwest, tokio, serde, handlebars, uuid, regex, etc.)
- **Others**: @git-diff-view, vue-diff, jsdiff, pinia, vue-router
- **AI Integration**: Multi‑provider support with unified interface design

## Requirements

- Node.js 16+
- Rust 1.70+
- Git 2.30+
- Windows/Mac/Linux (per Tauri support; default packaging target is Windows MSI)

## Install & Run

1. **Clone the repository**

```bash
git clone <repo-url>
cd GitMentor
```

2. **Install dependencies**

```bash
cd GitMentor-Lite
npm install
```

3. **Development**

```bash
npm run tauri:dev
```

- Starts Vite dev server (http://localhost:1420) and the Tauri window
- If the port is occupied, run `GitMentor-Lite/kill-port-1420.bat`

4. **Build for production**

```bash
npm run tauri:build
```

- Produces Windows MSI by default
- For the Git sidecar, see `GitMentor-Lite/src-tauri/binaries/README.md`

**First‑time setup**: Choose provider → Set API key → Test connection → Select repository

## Usage

### Basic Workflow

1. **Select or Clone Repository**
   - Choose existing local repository
   - Or clone remote repository to local

2. **Generate Commit Messages**
   - Review change contents
   - Select templates and parameters
   - Click "Generate," preview message, then commit

3. **Gitflow Workflow**
   - Launch Gitflow wizard
   - Create feature branches
   - Track branch status and progress
   - Complete branch merges

### Advanced Features

- **Layered Commit**: AI proposes messages per layer; edit and confirm
- **Remote Sync**: Push/pull branches to remote repository
- **Template Management**: Create and manage custom commit templates
- **Conversation History**: View AI generation history and debug information

## API (Tauri Commands)

Invoked from the frontend via `@tauri-apps/api.invoke`:

### Basic Commands
- `greet`

### Git Operations
- `select_repository`, `get_git_status`, `stage_files`, `commit_changes`, `revert_files`, `generate_commit_message`
- `clone_repository`, `configure_remote`, `push_branch`, `pull_branch`

### Branch Management
- `create_feature_branch`, `merge_branch`, `switch_branch`
- `list_branches`, `get_branch_info`

### Gitflow Workflow
- `init_gitflow`, `create_feature`, `create_release`, `create_hotfix`
- `get_gitflow_status`, `complete_gitflow_operation`

### Template Management
- `list_templates`, `create_template`, `update_template`
- `generate_from_template`

### AI / Layered Commit
- `list_providers`, `update_provider_config`, `test_ai_connection`
- `get_layered_sessions`, `cancel_layered_commit`
- `get_conversation_records_by_session`

### Debug & Configuration
- `get_debug_settings`, `set_debug_logs_enabled`, `update_debug_settings`
- `get_git_config`, `update_git_config`

Full parameter and response specifications are recommended to be maintained in `docs/API.md`.

## Structure

```
GitMentor-Lite/
├── src/                          # Frontend (Vue 3 + TS + Element Plus)
│   ├── components/               # Vue components
│   │   ├── gitflow/              # Gitflow workflow components
│   │   ├── LayeredCommitProgress.vue
│   │   └── ...
│   ├── pages/                    # Page components
│   ├── types/                    # TypeScript type definitions
│   └── utils/                    # Utility functions
├── src-tauri/                    # Rust backend & Tauri config
│   ├── src/
│   │   ├── commands/             # Tauri command implementations
│   │   │   ├── gitflow_commands.rs
│   │   │   ├── repository_commands.rs
│   │   │   └── ...
│   │   ├── core/                 # Core business logic
│   │   │   ├── git_engine.rs     # Git operation engine
│   │   │   ├── ai_manager.rs     # AI management
│   │   │   ├── gitflow_manager.rs # Gitflow management
│   │   │   └── ...
│   │   ├── providers/            # AI provider implementations
│   │   └── templates/            # Template system
│   ├── capabilities/             # Tauri permissions config
│   └── icons/                    # Application icons
├── docs/                         # Project documentation
├── public/icons/                 # AI provider icons
└── package.json                  # Build scripts
```

## Architecture & Flow

- **Frontend (Vue)** invokes Tauri commands via `invoke`
- **Backend (Tauri + Rust)** dispatches to core modules:
  - `GitEngine` (git2) for repository operations
  - `GitflowManager` for Gitflow workflow management
  - `RepositoryManager` for remote repository operations
  - `AIManager` via provider factory
  - `LLMClient/PromptManager` for unified parameters & templating
  - `ConversationLogger` for requests/responses and session auditing
  - `LayeredCommitManager` for session orchestration and cancellation
- **Results returned** to frontend for preview and confirmation

## Configuration

- **Location**: `src-tauri/.config` (gitignored)
- **Contains**: Provider keys, template settings, runtime logs
- **Security**: Keys are not committed; do not expose `.config` publicly

## Common Scripts

- `GitMentor-Lite/kill-port-1420.bat`: Clean up port 1420 occupation
- `GitMentor-Lite/build-backend.bat`: Build backend
- `GitMentor-Lite/diagnose_and_start.bat`: Diagnose and start application

## Development Documentation

- [Git Repository Management Design Document](GitMentor-Lite/docs/Git仓库管理功能设计文档.md)
- [TEMPLATE_VERSIONING](GitMentor-Lite/docs/TEMPLATE_VERSIONING.md)
- [Daily Report Enhancement](GitMentor-Lite/DAILY_REPORT_ENHANCEMENT.md)
- [Streaming Implementation](GitMentor-Lite/streaming-implementation-summary.md)

## Contributing

Contributions via Issues/PRs are welcome. Please ensure:

- Follow existing code style and structure
- Rust builds successfully; frontend type‑checks and builds
- New features (Gitflow, remote repositories, etc.) require corresponding documentation/examples
- For new providers/commands, register them in the factory and update API documentation

## Changelog

### v0.2.7 (2025-10-21)
- ✨ Added complete Gitflow workflow support
- 🌐 Added remote repository cloning and management functionality
- 🧠 Added intelligent branch switching and ownership management
- 🔧 Optimized Git engine functionality and updated dependencies
- 📊 Enhanced log file path handling
- 🛠️ Dynamic Git remote repository name resolution

### v0.2.6 and earlier
- Basic Git operation panel
- AI commit message generation
- Layered commit workflow
- Multi‑provider support

## License

GPL-3.0 license

## Contact

- Author: Evilek
- Project: [GitHub Repository]
- Support: Please submit an Issue

---

*Last Updated: October 30, 2025*
