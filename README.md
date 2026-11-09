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

Click a calendar frame to view its task and elapsed time. Use the pencil beside
Start or End to edit the local time; Enter or the checkmark saves, Tab switches
between hours and minutes, and Escape cancels. Running frames can have their
start edited. Edits retain the existing date and seconds and cannot overlap
another frame. Invalid or ambiguous local times are rejected.

Use the plus beside Time Frame to add a completed frame. Choose a date and task,
then enter the local start and end times. Open the task selector to search; use
the arrow keys and Enter to choose a result, or create a task with the plus when
there are no matches. Escape closes the open selector or date picker, then the
window. Tab and Shift+Tab cycle through the task, time inputs, and action buttons.
The new frame cannot overlap an existing frame or end in the future. After saving,
the calendar moves to the new frame while the current frame selection stays intact.

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

### Linux application launcher

To install the icon for your user, run these commands from the checkout root.
The second command requires ImageMagick:

```sh
install -Dm644 assets/app-icon.png \
  "$HOME/.local/share/icons/hicolor/256x256/apps/kalaganaka.png"
magick "$HOME/.local/share/icons/hicolor/256x256/apps/kalaganaka.png" \
  -resize 256x256 \
  "$HOME/.local/share/icons/hicolor/256x256/apps/kalaganaka.png"
```

Create a desktop entry so the application appears in your desktop's application
overview. This example uses Cargo's default installation path; adjust `Exec` if
you installed the binary elsewhere:

```sh
mkdir -p "$HOME/.local/share/applications"
cat > "$HOME/.local/share/applications/kalaganaka.desktop" <<EOF
[Desktop Entry]
Type=Application
Name=Kalaganaka
Comment=Manual task time tracking
Exec="$HOME/.cargo/bin/kalaganaka"
Icon=kalaganaka
Terminal=false
Categories=Office;Utility;
EOF
```

If GNOME still shows an older icon, log out and back in to refresh the shell.

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
