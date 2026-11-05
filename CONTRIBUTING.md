# Contributing

## Documenting changes

Enable the versioned commit message hook once after cloning:

```sh
git config --local core.hooksPath .githooks
```

The hook rejects subjects that do not match the format and allowed types below.
It checks the first line only; commit bodies remain free-form. Merge and revert
commits must also use this format, for example `revert: undo the calendar change`.
Local hooks can be bypassed with `--no-verify`; this is not a server-side check.

Use Conventional Commits so each change can become a changelog entry:

```text
fix(calendar): preserve the selected date when switching views
feat(statistics): add weekly totals
docs: explain local setup
refactor(storage): simplify task queries
```

These are examples, not a list of implemented changes. Describe the behavior in
the subject. Use the body for the cause, implementation rationale, and issue
references. Keep unrelated fixes in separate commits. When squash merging, use a
Conventional Commit message for the final squash commit.

Use `fix` for bugs, `feat` for features, and `perf` for performance improvements.
Use `docs`, `refactor`, `style`, `test`, `build`, `ci`, or `chore` for other work.
Mark breaking changes with `!`, for example `feat(storage)!: replace the data
format`, and explain the impact in a `BREAKING CHANGE:` footer.

Git history is the detailed record; [CHANGELOG.md](CHANGELOG.md) is the generated
summary, with links to each commit. Existing nonconventional commits appear under
Other changes. Uncommitted changes are not included.

## Generating the changelog

Install the same git-cliff version used by CI:

```sh
cargo install git-cliff --version 2.14.2 --locked
```

Preview changes since the last version tag:

```sh
git cliff --unreleased
```

Regenerate the complete changelog, including unreleased commits:

```sh
git cliff --output CHANGELOG.md
```

Commit the generated file when preparing a release. It is a snapshot, so it is
not automatically updated after every commit. GitHub release notes are generated
automatically when a release tag is pushed.

## Releasing

Use one stable version for the application and its workspace crates. Tags have
the form `vMAJOR.MINOR.PATCH`; prerelease tags are not supported by this workflow.

1. Finish and commit the changes intended for the release.
2. Update `[workspace.package].version` in `Cargo.toml` and refresh `Cargo.lock`
   with `cargo check --workspace`. Run `cargo test --workspace`.
3. Generate the changelog for the intended version before creating its tag:

   ```sh
   git cliff --tag v0.1.1 --output CHANGELOG.md
   ```

4. Review and commit the version files and changelog, preferably through a release
   PR. Substitute your intended version for `0.1.1` in these examples.
5. Once the release commit is on the default branch, tag that commit and push:

   ```sh
   git tag -a v0.1.1 -m "Release v0.1.1"
   git push origin v0.1.1
   ```

The release workflow verifies that the tag matches `Cargo.toml`, generates notes
only for that tag, and creates a GitHub Release using the repository's built-in
`GITHUB_TOKEN`. No additional secret is needed. It publishes release notes only;
packaged application binaries are not built or attached.

There are no existing release tags. Use `v0.1.0` for the first release if the
workspace version remains `0.1.0`; all history will belong to that first release.
