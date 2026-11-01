# Agent instructions

This is a personal Helix fork for custom features and builds. Preserve intentional
fork behavior; keep changes focused and easy to maintain when merging upstream.

## Working conventions

- Read nearby code and follow existing patterns before introducing abstractions.
- Avoid unrelated refactors, dependency upgrades, or formatting changes.
- Use the pinned toolchain in `rust-toolchain.toml`; respect the workspace MSRV.
- Update user documentation for changes to commands, keybindings, or configuration.
- See `docs/architecture.md` for internals and `docs/CONTRIBUTING.md` for workflows.

## Code map

- `helix-core/`: text, selections, transactions, syntax, and editing primitives.
- `helix-view/`: editor state, documents, views, and configuration.
- `helix-term/`: terminal application, commands, keymaps, and UI; produces `hx`.
- `helix-lsp/`, `helix-dap/`: language server and debugger clients.
- `runtime/`: grammars, queries, themes; `languages.toml`: language definitions.
- `book/src/`: user documentation; `xtask/`: validation and documentation tooling.

## Build and validation

Run commands from the repository root:

- Develop: `cargo run -- <file>`; build: `cargo build --release --locked`.
- Point `HELIX_RUNTIME` at this checkout's `runtime/` when testing custom builds.
- Format: `cargo fmt --all --check`.
- Lint: `cargo clippy --workspace --all-targets -- -D warnings`.
- Test affected crates with `cargo test -p <crate>`; full suite: `cargo test --workspace`.
- For editing behavior, add regression coverage in `helix-term/tests/` and run
  `cargo integration-test` (accepts a test filter). Cover multiple selections and
  Unicode where relevant. On macOS, increase `ulimit -n` if tests exhaust file handles.
- For runtime changes, use the relevant `cargo xtask` task: `query-check`,
  `indent-check`, `highlight-check`, or `theme-check`.
- Regenerate generated documentation with `cargo xtask docgen` when its inputs change.

Run checks appropriate to the change. Report what changed, checks run, and any
checks that could not be completed.
