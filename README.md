# tuiji

Tuiji is a keyboard-driven terminal client for working with Jira issues from a cached sprint board. It uses a local SQLite cache for responsive reads and an outbox for supported issue changes.

## Support scope

Tuiji supports **Jira Cloud only**. Its write client targets `/rest/api/3` and sends Jira's Atlassian Document Format (ADF). Jira Server and Jira Data Center are not supported.

Current capabilities are deliberately narrow:

- browse the cached current sprint and issue details;
- run the online **My Issues** query;
- run raw JQL searches online;
- create issues with a summary, description, and Jira-provided issue type;
- edit an issue's summary, description, and priority;
- add comments, transition issues, and assign issues to yourself;
- inspect synchronization status and resolve detected conflicts.

## Requirements

- Rust 1.85 or newer when building from source;
- a Jira Cloud site and account;
- a Jira API token.

## Install

Tuiji is not published to crates.io yet. Install the current revision with Cargo:

```bash
cargo install --git https://github.com/tuijiorg/tuiji
```

Or build from source:

```bash
git clone https://github.com/tuijiorg/tuiji.git
cd tuiji
cargo build --release
./target/release/tuiji
```

For local development, `cargo run` builds and starts the application.

## First run and credentials

Run `tuiji`. When no usable profile exists, Tuiji opens the profile-creation screen. Enter a profile name, Jira Cloud base URL (for example, `https://example.atlassian.net`), Jira username/email, and API token. After the profile is saved, Tuiji opens board selection; choose the board whose current sprint you want to cache.

The active profile can obtain its API token from three sources, in this precedence order:

1. `TUIJI_JIRA_API_TOKEN`;
2. a non-empty `api_token` in `config.toml`;
3. `api_token_command`, executed through `sh -c` so a password manager or other credential helper can print the token.

The command must exit successfully and print a non-empty token. Trailing whitespace is removed. A literal `api_token` is stored as clear text, so prefer the environment variable or `api_token_command` when appropriate.

The default configuration path is `$XDG_CONFIG_HOME/tuiji/config.toml`, or `~/.config/tuiji/config.toml` when `XDG_CONFIG_HOME` is unset. Set `TUIJI_CFG_FILE_PATH` to use another file. On Unix, Tuiji sets a saved configuration file's mode to `0600`.

### Configuration example

When using `api_token_command`, keep `api_token` empty; a non-empty value takes precedence over the command.

```toml
active_profile_id = "work"

[[profiles]]
id = "work"
name = "Work Jira"

[profiles.jira]
base_url = "https://example.atlassian.net"
username = "you@example.com"
api_token = ""
api_token_command = "pass show jira/tuiji-token"

[ui]
theme = "default"

[sync]
interval_seconds = 120
```

`sync.interval_seconds` defaults to `120`. Set it to `0` to disable periodic pulls. `TUIJI_SYNC_INTERVAL_SECONDS` overrides the configured value. Jira base URL and username can likewise be overridden with `TUIJI_JIRA_BASE_URL` and `TUIJI_JIRA_USERNAME`.

## Online and offline behavior

The current sprint and issue-detail screens read from the local cache, so previously synchronized issues remain browsable without a connection. Edits and comments are applied optimistically to the cache and queued in the outbox. Transitions and assign-to-me changes use the same outbox once their required online metadata has been loaded. Pending changes are pushed through synchronization when Jira is reachable; transient failures remain queued rather than being discarded.

These operations require an active Jira connection:

- creating an issue;
- loading **My Issues**;
- running a JQL search;
- listing available transitions;
- listing creatable issue types;
- resolving your Jira account identity the first time **Assign to me** is used.

Consequently, a transition can only be selected after its choices have been loaded, and an offline assign-to-me action requires the identity to have already been resolved during the current run.

## Conflicts

A pull that finds a remote change conflicting with a local issue change shows a warning and opens the Conflicts screen. Select an issue, then choose:

- `l` to keep the local version and queue it for another push;
- `j` to accept the stored remote version.

Press `Enter` to confirm the selected resolution. Press `q` while the confirmation is open to cancel it.

## Default keybindings

The table below reflects `src/config/keybinding_defaults.rs`. Key names are case-sensitive.

| Scope | Keys |
|---|---|
| Global | `q` Quit; `r` Refresh; `Enter` Confirm; `gh` Home; `b` Boards; `o` Open issue in browser; `,` Settings |
| Home | `q` Quit; `c` Current Sprint; `i` My Issues; `s` Search Issues; `n` New Issue; `b` Boards; `t` Sync Status; `,` Settings; `k`/`↑` Up; `j`/`↓` Down; `gg` Top; `G` Bottom |
| Board selection | `q` Quit; `k`/`↑` Up; `j`/`↓` Down; `gg` Top; `G` Bottom |
| Current Sprint | `Enter` Open issue; `k`/`↑` Up; `j`/`↓` Down; `gg` Top; `G` Bottom |
| Issue detail | `Enter` Confirm transition or comment; `q` Close; `e` Edit; `m` Transition; `C` Comment; `a` Assign to me; `PageDown`/`PageUp` Scroll by page; `k`/`↑` Up; `j`/`↓` Down; `gg` Top; `G` Bottom |
| Profile creation | `k`/`↑` Up; `j`/`↓` Down; `h`/`←` Left; `l`/`→` Right; `gg` Top; `G` Bottom; `0`/`^` Line start; `$` Line end; `w`/`W` Word forward; `b`/`B` Word backward; `e`/`E` Word end; `i` Insert before; `a` Insert after; `I` Insert at line start; `A` Insert at line end |
| Profiles | `q` Close; `e` Edit; `d` Delete; `n` New; `k`/`↑` Up; `j`/`↓` Down; `gg` Top; `G` Bottom |
| Settings | `Enter` Select; `k`/`↑` Up; `j`/`↓` Down; `gg` Top; `G` Bottom |
| My Issues | `Enter` Open issue; `k`/`↑` Up; `j`/`↓` Down; `gg` Top; `G` Bottom |
| Search Issues | `/` Focus JQL query; `Enter` Submit/open; `k`/`↑` Up; `j`/`↓` Down; `gg` Top; `G` Bottom |
| Conflicts | `l` Keep local; `j` Accept remote; `k`/`↑` Up; `↓` Down; `gg` Top; `G` Bottom |
| Sync Status | `s` Sync now; `p` Pause; `t` Retry; `u` Resume; `A` All jobs; `P` Pull jobs; `U` Push jobs |
| New/Edit Issue form | `Enter` Submit; `k`/`↑` Up; `j`/`↓` Down; `h`/`←` Left; `l`/`→` Right; `gg` Top; `G` Bottom; `0`/`^` Line start; `$` Line end; `w`/`W` Word forward; `b`/`B` Word backward; `e`/`E` Word end; `i` Insert before; `a` Insert after; `I` Insert at line start; `A` Insert at line end |

## License

Tuiji is available under the [MIT License](LICENSE).
