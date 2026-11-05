# Agent instructions

Read `~/.agents/AGENTS.md` if it exists on the host, then follow this repository
guide. See `README.md` for installation and `CONTRIBUTING.md` for commits and
releases.

## Workspace

- `crates/tracking-core`: tracking rules, tasks, sessions, frames, and statistics.
  Keep domain logic independent of the desktop UI and database implementation.
- `crates/desktop`: the `kalaganaka` binary using iced. `app.rs` handles messages
  and application state; `ui/` renders windows and the calendar.
- `crates/desktop/src/storage`: SQLite persistence and asynchronous commands.
- `crates/desktop/migrations`: database schema migrations.
- `TODO.md`: outstanding bugs and features. Read its current contents before
  choosing an item; ordering may change.

## Workflow

1. Inspect the current code and worktree. Preserve unrelated user changes.
2. Explain the proposed implementation and obtain explicit approval before
   changing production code.
3. During implementation review, change production code only. Defer test changes,
   test runs, and documentation updates until the user approves the whole logic.
4. After final logic approval, add relevant regression coverage, run the checks,
   and align documentation. Remove completed TODO entries at this stage.
5. Commit or push only within the user's authorized scope. Do not create release
   tags or publish releases unless requested.

Follow explicit exceptions requested by the user. Keep chat concise and in
English. State uncertainty rather than inventing behavior or validation results.

## Implementation constraints

- Preserve the separation between UI orchestration, domain rules, and storage.
  Storage commands run on blocking workers; the UI serializes mutations.
- Timestamps are stored as UTC; calendar boundaries and labels use local time.
  Statistics include completed frames clipped to the selected period and omit
  tasks with zero completed time.
- Keep the calendar at a stable position in the widget tree when status messages
  appear or disappear. Otherwise iced can recreate its scroll state at 00:00.
- Scroll to the current time on startup, Today, Day/Week changes, successful
  start/pause, and running-task switches when today is visible. Preserve existing
  date-navigation behavior. Ordinary updates must not reset scrolling.
- Protect existing user data when changing SQLite persistence or migrations.
  Do not use the real application database for tests.
- Unsafe Rust is forbidden by the workspace lint configuration.

## Validation and commits

After final logic approval, run the checks appropriate to the change:

```sh
cargo fmt --all -- --check
cargo test --workspace --locked
git diff --check
```

Use the repository's `build/` directory for test working directories and
temporary test artifacts, not `/tmp`. Cargo build artifacts remain in `target/`.
The desktop tests cover widget-state continuity and application messages without
opening a GUI; storage tests use isolated databases under `build/tests/`.
Report any manual UI checks separately from automated tests.

Use Conventional Commits, for example `fix(calendar): prevent scroll resets`.
Enable the versioned hook with `git config --local core.hooksPath .githooks`.
Generate `CHANGELOG.md` with git-cliff when preparing a release; do not edit its
entries by hand. Shared crate versions are defined in the root `Cargo.toml`.
