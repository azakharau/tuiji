# AGENTS.md

## Purpose

This repository is the Rust codebase for `tuiji`, a terminal-first Jira workflow client.

Use this file as the repo-local operating guide for agents working in this worktree:
- understand the module layout
- preserve architectural boundaries
- use the standard build/test commands
- optionally consult the external project knowledge base and memory system when they exist

## Project Structure

Main runtime areas:

- `src/main.rs`
  - terminal bootstrap and app startup
- `src/app`
  - event loop, command routing, state transitions, worker orchestration
- `src/ui`
  - screens, components, rendering, interaction contracts
- `src/data`
  - repository abstractions, SQLite cache, sync/conflict logic
- `src/client`
  - Jira HTTP client abstractions
- `src/config`
  - config model, env overrides, user settings

Important architectural contracts:

- UI interaction contracts: `src/ui/interaction.rs`
- Layout primitives: `src/ui/layout.rs`
- Shared contracts: `src/contracts`
- Repository hub / traits: `src/data/repository`

## Architectural Rules

1. Keep input/routing/state orchestration in `src/app`.
2. Keep rendering and screen composition in `src/ui`.
3. Keep transport-specific Jira logic in `src/client`.
4. Keep persistence and cache/sync behavior in `src/data`.
5. Keep config- and keybinding-driven behavior in config modules rather than hardcoded shortcuts.

Prefer preserving existing explicit contracts over introducing implicit coupling.

## Standard Commands

Prefer the standard Rust commands:

- `cargo check`
- `cargo test`
- `cargo run`
- `cargo fmt`
- `cargo clippy -- -D warnings`

Before finishing non-trivial code changes, prefer running:

1. `cargo fmt`
2. `cargo check`
3. the narrowest relevant tests, or `cargo test` if practical

If you could not run checks, say so explicitly.

## Implementation Guidance

When working in this repo:

1. Inspect the affected module boundary before editing.
2. Prefer config-driven key/action behavior over hardcoded values.
3. Preserve local-cache compatibility unless intentionally changing sync semantics.
4. Avoid blocking operations on Tokio runtime threads.
5. Keep screen-specific behavior inside screen/UI modules unless it is truly cross-cutting.
6. Prefer explicit typed contracts for routing, notifications, sync, and errors.

## Testing Guidance

- Prefer deterministic tests.
- Stub Jira/network interactions in tests; do not depend on live network behavior.
- For debugging failures, `cargo test -- --nocapture` is acceptable.

## Optional External Knowledge Base And Memory

The existence of these external sources is optional.
Their usage is not optional when they are available.

If the following paths exist, read and use them before making non-trivial decisions:

- project notes root:
  - `/Users/aliakseizakharau/obsidian/Projects/tuiji`
- agent operating context:
  - `/Users/aliakseizakharau/obsidian/Projects/tuiji/agent-context`

If they are missing or unavailable:
- continue from repository-local evidence
- do not block work
- note briefly that the external knowledge source was unavailable

## How To Use The External Notes

If the notes root exists, you must start with:

1. `/Users/aliakseizakharau/obsidian/Projects/tuiji/README.md`
2. `/Users/aliakseizakharau/obsidian/Projects/tuiji/project-description.md`
3. `/Users/aliakseizakharau/obsidian/Projects/tuiji/architecture.md`

Treat those files as the durable project knowledge layer and do not skip them for non-trivial work.

## How To Use Agent Context

If `agent-context/` exists, you must treat it as an agent operating layer with three distinct parts:

1. Core memory layer
- `/Users/aliakseizakharau/obsidian/Projects/tuiji/agent-context/MEMORY.md`
- `/Users/aliakseizakharau/obsidian/Projects/tuiji/agent-context/memory/**/*.md`
- This is the closest equivalent to OpenClaw-style memory.

2. Durable project knowledge layer
- `/Users/aliakseizakharau/obsidian/Projects/tuiji/agent-context/PROJECT-KNOWLEDGE.md`
- complements root docs, but does not replace repository code

3. Operational workflow layer
- `sessions/`
- `todos/`
- `decisions/`
- `indexes/`
- useful for handoff and execution tracking, but not the same thing as memory

Required read order:

1. `/Users/aliakseizakharau/obsidian/Projects/tuiji/agent-context/README.md`
2. `/Users/aliakseizakharau/obsidian/Projects/tuiji/agent-context/AGENT-RUNBOOK.md`
3. `/Users/aliakseizakharau/obsidian/Projects/tuiji/agent-context/PROJECT-KNOWLEDGE.md`
4. `/Users/aliakseizakharau/obsidian/Projects/tuiji/agent-context/MEMORY.md`
5. relevant recent files under `memory/`, then `sessions/`, `todos/`, and `decisions/`

Do not let workflow notes override repository code or root project docs.
But do not skip the available memory/context layer when preparing non-trivial changes.
