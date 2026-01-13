# Tuiji

> Terminal User Interface for Jira with offline-first synchronization

Tuiji is a fast, keyboard-driven TUI client for Jira Cloud built with Rust. It features vim-style navigation, offline-first architecture with bidirectional sync, and a customizable interface.

<!-- TODO: Add screenshot/GIF here -->
<!-- ![Tuiji Demo](docs/demo.gif) -->

## Features

- **Vim-style navigation** — `j`/`k` movement, modal editing, customizable keybindings
- **Offline-first** — Work without internet, changes sync when connected
- **Multiple profiles** — Switch between different Jira instances
- **Sprint board view** — See current sprint issues organized by status
- **Issue management** — Create, edit, and view issues with full field support
- **Search** — Find issues across your project
- **Conflict resolution** — Handle sync conflicts with a dedicated UI
- **Custom themes** — Built-in themes plus full customization support
- **Fast** — Native Rust performance, instant startup

## Requirements

- Rust 1.85+ (edition 2024)
- Jira Cloud account with API token
- Terminal with true color support (recommended)

## Installation

### From source

```bash
git clone https://github.com/username/tuiji.git
cd tuiji
cargo install --path .
```

### From crates.io

```bash
cargo install tuiji
```

## Quick Start

1. **Create a Jira API token**

   Go to [Atlassian Account Settings](https://id.atlassian.com/manage-profile/security/api-tokens) and create a new API token.

2. **Run Tuiji**

   ```bash
   tuiji
   ```

3. **Create a profile**

   On first run, you'll be prompted to create a profile with:
   - Profile name
   - Jira base URL (e.g., `https://yourcompany.atlassian.net`)
   - Email address
   - API token

4. **Select a board** and start working!

## Configuration

Configuration is stored in `~/.config/tuiji/config.toml` (Linux/macOS) or `%APPDATA%\tuiji\config.toml` (Windows).

### Example configuration

```toml
active_profile_id = "work"

[[profiles]]
id = "work"
name = "Work Jira"
sync_mode = "cache"  # "cache" (offline-first) or "online"

[profiles.jira]
base_url = "https://company.atlassian.net"
username = "your.email@company.com"
api_token = "your-api-token"

[ui]
theme = "default"  # "default", "dark", "light", or custom theme name
screen_cache_ttl_seconds = 60
notification_ttl_seconds = 5
```

### Environment variables

All settings can be overridden via environment variables:

| Variable | Description |
|----------|-------------|
| `TUIJI_JIRA_BASE_URL` | Jira instance URL |
| `TUIJI_JIRA_USERNAME` | Jira username/email |
| `TUIJI_JIRA_API_TOKEN` | Jira API token |
| `TUIJI_UI_THEME` | Theme name |

## Keybindings

Tuiji uses vim-style keybindings by default.

### Global

| Key | Action |
|-----|--------|
| `q` | Quit / Go back |
| `r` | Refresh current view |
| `Enter` | Confirm / Select |
| `?` | Show help |
| `Esc` | Cancel / Normal mode |

### Navigation

| Key | Action |
|-----|--------|
| `j` / `↓` | Move down |
| `k` / `↑` | Move up |
| `gg` | Go to top |
| `G` | Go to bottom |
| `gh` | Go home |

### Screens

| Key | Action |
|-----|--------|
| `b` | Open board selection |
| `c` | Current sprint |
| `i` | My issues |
| `s` | Search issues |
| `n` | New issue |
| `t` | Sync status |
| `,` | Settings |
| `o` | Open in browser |

### Modes

| Key | Mode |
|-----|------|
| `Esc` | Normal mode |
| `i` | Insert mode (in forms) |
| `v` | Visual mode |
| `:` | Command mode |

### Custom keybindings

Add custom keybindings in `config.toml`:

```toml
[keybindings]
global = [
    { action = "quit", key = "q" },
    { action = "refresh", key = "R" },
]
```

## Screens

| Screen | Description |
|--------|-------------|
| **Home** | Dashboard with quick actions |
| **Board Selection** | Choose active Jira board |
| **Current Sprint** | View sprint issues by status column |
| **My Issues** | Issues assigned to you |
| **Search Issues** | Search across all issues |
| **Issue Detail** | View/edit single issue |
| **New Issue** | Create new issue form |
| **Sync Status** | View sync history and pending changes |
| **Conflicts** | Resolve sync conflicts |
| **Settings** | App configuration |
| **Profiles** | Manage Jira profiles |

## Architecture

Tuiji follows an offline-first architecture:

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   Jira API  │◄───►│   SQLite    │◄───►│     TUI     │
│   (Cloud)   │     │   (Local)   │     │  (ratatui)  │
└─────────────┘     └─────────────┘     └─────────────┘
       ▲                   ▲
       │                   │
       └───── Sync ────────┘
```

- **Local SQLite database** stores all data for offline access
- **Background sync** keeps local data fresh
- **Outbox pattern** for reliable change propagation
- **Conflict detection** when remote changes conflict with local edits

## Development

```bash
# Run in development mode
cargo run

# Run tests
cargo test

# Run clippy
cargo clippy --all-targets --all-features

# Format code
cargo fmt
```

### Project structure

```
src/
├── app/           # Application logic, event loop, state
├── client/        # Jira API client (async)
├── data/          # Repository pattern, SQLite, models
├── ui/
│   ├── components/  # Reusable UI widgets
│   ├── screens/     # Screen implementations
│   └── theme/       # Theme system
├── config.rs      # Configuration management
├── lib.rs         # Library exports
└── main.rs        # Entry point
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [ratatui](https://github.com/ratatui/ratatui) — Terminal UI framework
- [gouqi](https://github.com/softprops/gouqi) — Jira API client
- [tokio](https://tokio.rs/) — Async runtime
- [sqlx](https://github.com/launchbadge/sqlx) — SQL toolkit

---

**Note:** This project is in early development. Expect breaking changes.
