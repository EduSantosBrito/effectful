We are going to program you to behave the way we want.

- **Roam:** Load navigation workflow with `devenv shell -- cat .cursor/rules/roam.mdc` (or rely on always-applied `roam.mdc`). Run `roam` via `devenv shell -- roam …`.
- **Tool-tasks:** Run `devenv shell -- cargo run -p tool-tasks -- init` then `devenv shell -- cargo run -p tool-tasks -- status --toon` (session task context; see `.cursor/rules/tool-tasks.mdc`). For git + tasks + LLM session path, use `git status`, `tool-tasks status --toon`, and read `.tool-llm-git-context/current-session.json` as needed.
- **Rust gate:** `moon run :format` and `moon run :check` (see root `README.md`), or `cargo fmt --all -- --check` and `cargo clippy --workspace --all-targets --all-features -- -D warnings`.
- **GitHub CLI:** Use the `gh` cli for GitHub when needed.
- **Rust** Load rust workflow with `devenv shell -- cat .cursor/commands/rust.md`.
- **Effect** Load effect skill with `devenv shell -- cat .cursor/skills/effect.rs-fundamentals/SKILL.md`.
- **LLM git context** (vault session worktrees + hook JSONL): Read `.cursor/rules/llm-git-context.mdc` when working under `.tool-llm-git-context/` or changing hook logging. For a **new** persisted session run `devenv shell -- cargo run -p tool-llm-git-context -- session-init`. Hooks require `cargo build -p tool-llm-git-context` (or `LLM_GIT_CONTEXT_BIN`). Slash helper: `/llm-git-context`.
