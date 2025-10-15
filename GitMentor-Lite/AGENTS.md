# Repository Guidelines

## Project Structure & Module Organization
- App UI: `src/` (Vue 3 + TS). Key folders: `components/`, `pages/`, `composables/`, `router/`, `utils/`.
- Desktop backend: `src-tauri/` (Rust + Tauri). Key folders: `src/commands/`, `src/core/`, `src/templates/`, `src/types/`.
- Static assets: `public/` (icons, index.html). Build output: `dist/`.
- Scripts and helpers: `scripts/`, batch files in repo root.

## Build, Test, and Development Commands
- `npm run dev` — Vite dev server for the web UI.
- `npm run tauri:dev` (Windows) / `npm run tauri:dev:unix` (Unix) — run the desktop app.
- `npm run build` — type-check (`vue-tsc`) and Vite build.
- `npm run tauri:build` — package the desktop app.
- `npm run type-check` — TypeScript + Vue SFC type checks.
- Rust (optional): from `src-tauri/`, `cargo build` or `cargo test`.

## Coding Style & Naming Conventions
- TypeScript/Vue: 2-space indent, single quotes, semicolons. Composition API in `.vue` SFCs.
- Naming: Vue components/pages in `PascalCase.vue` (e.g., `DiffViewer.vue`); composables `useX.ts` in `src/composables/`; variables/functions `camelCase`; types `PascalCase`.
- Rust: follow rustfmt defaults; modules/files `snake_case`, types `CamelCase`, functions `snake_case`. Use `anyhow` for errors where appropriate.

## Testing Guidelines
- No formal JS test suite yet. If adding, prefer Vitest + Vue Test Utils: place tests under `src/__tests__/` with `*.spec.ts`; add `npm test` script.
- Rust: add unit tests with `#[cfg(test)]` in modules under `src-tauri/src/` and run `cargo test`.
- Manual checks: `npm run tauri:dev`, verify Git panel actions, diff viewer rendering, and dialogs.

## Commit & Pull Request Guidelines
- Use Conventional Commit prefixes when possible: `feat:`, `fix:`, `chore:`, `docs:`, `refactor:`. English or Chinese is fine; keep subjects imperative and concise.
- PRs must include: clear description, linked issues, screenshots/GIFs for UI changes, steps to reproduce/validate, and note of breaking changes.
- Before pushing: `npm run type-check` and ensure the app starts via `tauri:dev`.

## Security & Configuration Tips
- Do not commit API keys or personal data. Provider settings live under `src-tauri/.config/*.json`; consider ignoring machine-specific files locally.
- Keep embedded binaries under `src-tauri/binaries/` intact; avoid renames without updating references.

## Agent-Specific Notes
- Prefer minimal diffs and keep existing structure. Update templates in `src-tauri/templates/` instead of hardcoding strings. Avoid adding new build tools unless discussed.
