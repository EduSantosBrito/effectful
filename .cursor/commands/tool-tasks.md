GOAL THIS RUN: Complete exactly ONE **tool-tasks** task end-to-end: implement/verify, green tests, close task, commit.

- Use **only** `tool-tasks` for task state (no markdown TODOs, no `todo_write`). Run via:

  `devenv shell -- cargo run -p tool-tasks -- …`

- Prefer **`--toon`** on **`status`**, **`list`**, and **`tree`** when you parse output.

## Steps

1. **Store / overview**
   - Run: `devenv shell -- cargo run -p tool-tasks -- init` (idempotent).
   - Run: `devenv shell -- cargo run -p tool-tasks -- status --toon`
   - If `ready` is 0: run `devenv shell -- cargo run -p tool-tasks -- list --toon` (and/or `tree --toon`) and explain blocked/claimed/closed work; STOP (do not invent work).

2. **Claim one task**
   - Pick an `open` task id (when `ready` > 0).
   - Run: `devenv shell -- cargo run -p tool-tasks -- task claim --id <ID> --by <agent-or-user>`
   - Use **`list --toon`** for full fields (title, parent, status, `claimed_by`).

3. **Implement**
   - Match repo conventions; run the right tests/formatters.

4. **Follow-ups**
   - If you need a separate trackable item:  
     `devenv shell -- cargo run -p tool-tasks -- task create --title "…"`  
     then optional `task set-parent` / `task dep add` (do not expand current task scope).

5. **Close**
   - Run: `devenv shell -- cargo run -p tool-tasks -- task close --id <ID>`

6. **Commit**
   - Conventional commit; body may reference the task id, e.g. `Completes tool-tasks <id>.`

7. **One task per run** — do not start a second task in the same run.

CLI reference: `devenv shell -- cargo run -p tool-tasks -- --help` and `.cursor/rules/tool-tasks.mdc`.

Parallel agents must not append to the **same** `events.jsonl` concurrently; options (coordinator, `--root`, locks, merge) are in `history/2026-04-05-tool-tasks-concurrency-parallel-agents.md`.
