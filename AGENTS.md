# Repository Guidelines

## Project Structure & Modules
- Core app logic: `src/app.rs` (event loop, screen manager) plus submodules `src/app/{event,input,key_handlers,screen_manager,state}.rs`.
- UI: `src/ui` (screens, components). Screens live in `src/ui/screens/*`, shared widgets in `src/ui/components/*`.
- Jira client: `src/client/jira.rs` (async gouqi wrapper).
- Repository hub: `src/data/repository/local.rs` (`RepositoryHub`) plus `AppRepository` trait in `src/data/repository/mod.rs`.
- Config and types: `src/config.rs`.
- Entry point: `src/main.rs`.
- Build artifacts: `target/`; configuration files under `tuiji/config.toml` in your config dir.

## Build, Test, Run
- `cargo check` — fast validation of the workspace.
- `cargo test` — run the test suite (none yet; add tests here).
- `cargo run` — launch the TUI with fullscreen viewport.
- Optional: set config via env (`TUIJI_JIRA_*`, `TUIJI_CFG_FILE_PATH`) before running.

## Coding Style & Naming
- Rust 2024 edition; follow `rustfmt` defaults (4-space indent). Run `cargo fmt` before submitting.
- Prefer `Arc` for shared data in async contexts; avoid `Rc` in new code.
- Screen interfaces: implement `Screen` + `KeyHandler`; avoid long-lived borrows of `AppState`.
- Config-driven key bindings: use `KeyBindings` instead of hardcoded keys.
## Configuration Notes
- UI settings live under `[ui]` in `config.toml` (e.g., `screen_cache_ttl_seconds`).
- Key bindings live under `[keybindings]` in `config.toml`; defaults mirror the built-in vim-style bindings.

## Testing Guidelines
- Add unit tests alongside modules (e.g., `src/app/state.rs` → `state.rs` tests in the same file or `state_tests.rs`).
- Prefer deterministic tests; stub Jira calls (do not hit network in CI).
- Use `cargo test -- --nocapture` when debugging failures.

## Commit & PR Guidelines
- Commits: present-tense, concise summary (e.g., `Add async Jira client`, `Fix key binding hints`).
- PRs should include: brief description, key changes, manual test notes (`cargo check`, `cargo run`), and screenshots/GIFs for UI tweaks.
- Link relevant issues/tickets; note any breaking changes or config migrations.

## Security & Configuration Tips
- Never commit real Jira credentials. Use env vars or `tuiji/config.toml` in your local config dir.
- Validate that new async tasks are cancel-safe and avoid blocking calls on Tokio threads.
