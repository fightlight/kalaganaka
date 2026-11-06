# kalaganaka

Desktop time tracker built with Rust and iced.

Type in the task name field to search tasks. Use the arrow keys and Enter to
select a match, or press Enter to add the entered name if there are no matches.
Escape or a click outside dismisses the results. Settings controls how many
tasks appear (1–30, default 5); press Enter to save a typed limit or use the
arrows. Results show the newest matching tasks in creation order.

To rename a task, click the pencil beside it in either selector.
Edit the name and press Enter to save. Press Escape or click outside the selector
to cancel.

## Install from source

Until application packaging is available, install with Cargo. These instructions
assume a current stable Rust toolchain and Cargo are already installed. You also
need Git and your platform's native build tools, including a C compiler for the
bundled SQLite dependency.

```sh
git clone https://github.com/fightlight/kalaganaka.git
cd kalaganaka
cargo install --path crates/desktop --locked
kalaganaka
```

If you already have a checkout, run the `cargo install --path crates/desktop
--locked` command from its root. It builds the release binary and installs it
into Cargo's binary directory. Make sure that directory is on your `PATH`; it is usually
`~/.cargo/bin` on Linux/macOS or `%USERPROFILE%\.cargo\bin` on Windows. See the
[Cargo installation reference](https://doc.rust-lang.org/cargo/commands/cargo-install.html)
for custom installation locations.

To run directly from the checkout without installing:

```sh
cargo run --release --locked -p kalaganaka
```

## Development and coding agents

Agents working on this repository must read [AGENTS.md](AGENTS.md) before making
changes. It describes the workspace, approval workflow, checks, and behavioral
constraints. Use the standard plural filename `AGENTS.md` for agent discovery.
Also follow `~/.agents/AGENTS.md` when that file exists on the host.

Enable the local Conventional Commit hook once per clone:

```sh
git config --local core.hooksPath .githooks
```

For development, run `cargo run -p kalaganaka`. After implementation approval,
validate changes with:

```sh
cargo fmt --all -- --check
cargo test --workspace --locked
```

See [CHANGELOG.md](CHANGELOG.md) for changes and
[CONTRIBUTING.md](CONTRIBUTING.md) for commit and release instructions.
