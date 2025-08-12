# GitMentor Lite (English)

GitMentor Lite is a cross‑platform desktop app built with Tauri + Vue 3 + Rust. It generates high‑quality Git commit messages using multiple AI providers and configurable templates, and integrates essential Git operations including a “Layered Commit” workflow.

For Chinese documentation, see README.md.

---

## Overview

GitMentor Lite focuses on “Commit Message Automation + Git Panel.” The frontend is built with Vue 3 + Element Plus, and the backend uses Tauri v2 + Rust. With git2 and multiple AI providers, it enables:

- One‑click generation of standardized commit messages (template‑driven, parameterized)
- Layered commit session orchestration
- Common Git operations: view changes, stage, commit, revert

## Features

- Commit message generation
  - Template‑driven two‑stage processing (language, max tokens, temperature, emoji, types, etc.)
  - Layered Commit workflow
- Git panel
  - Diff / Stage / Commit / Revert
  - History / Branches (as shown in the UI; evolving)
- Providers (implemented & registered)
  - OpenAI, Ollama, Zhipu, Anthropic, DashScope, Doubao, Gemini, Deepseek
- Conversation logging & progress streaming
  - Records AI requests/responses and sessions for debugging/audit
- Desktop‑native experience
  - Tauri v2, lightweight and secure; Windows MSI packaging by default; optional Git sidecar

## Tech Stack

- Desktop: Tauri v2 (opener, dialog, shell plugins)
- Frontend: Vue 3 + TypeScript + Vite + Element Plus
- Backend: Rust (git2, reqwest, tokio, serde, handlebars, uuid, regex, etc.)
- Others: @git-diff-view, vue-diff, jsdiff, pinia, vue-router

## Requirements

- Node.js 16+
- Rust 1.70+
- Git 2.30+
- Windows/Mac/Linux (per Tauri support; default packaging target is Windows MSI)

## Install & Run

1. Clone

   - git clone <repo-url>
   - cd GitMentor

2. Install deps

   - cd GitMentor-Lite
   - npm install

3. Dev

   - npm run tauri:dev
   - Starts Vite dev server (http://localhost:1420) and the Tauri window
   - If the port is occupied, run GitMentor-Lite/kill-port-1420.bat

4. Build
   - npm run tauri:build
   - Produces Windows MSI by default
   - For the Git sidecar, see GitMentor-Lite/src-tauri/binaries/README.md

First‑time setup: Choose provider → Set API key → Test connection → Select repository

## Usage

- Generate a commit message

  1. Select a repository and review changes
  2. Choose a template and parameters in the generator view
  3. Click “Generate,” preview the message, then commit

- Layered Commit

  1. Enter the “Layered Commit” mode
  2. The AI proposes messages per layer; edit and confirm
  3. Cancel the session at any time

- Git operations
  - Use the panel to Stage / Revert / Commit
  - History/Branches depend on the UI (evolving)

## Configuration

- Location: src-tauri/.config (gitignored)
- Contains: Provider keys, template settings, runtime logs
- Security: Keys are not committed; do not expose .config publicly

## API (Tauri commands, partial)

Invoked from the frontend via `@tauri-apps/api.invoke`:

- Basic
  - `greet`
- Git
  - `select_repository`, `get_git_status`, `stage_files`, `commit_changes`, `revert_files`, `generate_commit_message`
- Debug
  - `get_debug_settings`, `set_debug_logs_enabled`, `update_debug_settings`
- Git config
  - `get_git_config`, `update_git_config`
- AI / Layered Commit
  - `list_providers`, `update_provider_config`, `remove_provider_config`
  - `get_layered_sessions`, `get_conversation_records_by_session`
  - `check_and_process_file_tokens`, `cancel_layered_commit`
  - `check_first_time_setup`, `test_ai_connection`

A dedicated docs/API.md is recommended for full parameter/response specifications.

## Structure

```text
GitMentor-Lite/
  src/                # Frontend (Vue 3 + TS + Element Plus; icons under public/icons)
  src-tauri/          # Rust backend & Tauri config
    src/core/         # AI manager, templates, Git engine, layered commits, logging
    tauri.conf.json   # Build/packaging config (devUrl, externalBin, etc.)
  package.json        # Scripts (dev/build/tauri:dev/tauri:build)
```

## Architecture & Flow

- The frontend (Vue) invokes Tauri commands via `invoke`
- The backend (Tauri + Rust) dispatches to core modules:
  - GitEngine (git2) for repository operations
  - AIManager via provider factory
  - LLMClient/PromptManager for unified parameters & templating
  - ConversationLogger for requests/responses and session auditing
  - LayeredCommitManager for session orchestration and cancellation
- Results are returned to the frontend for preview and confirmation

## Contributing

Contributions via Issues/PRs are welcome. Please ensure:

- Follow existing code style and structure
- Rust builds successfully; frontend type‑checks and builds
- For new providers/commands, add docs/examples and register them in the factory

## FAQ / Troubleshooting

- Port 1420 in use: run GitMentor-Lite/kill-port-1420.bat

## License

MIT
