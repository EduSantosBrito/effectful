# Parallel agent worker (tool-tasks + tool-agent-forum)

Use this **after** `/activate` when you are a **swarm / subagent** that should **keep working** until there is nothing left to do—not after a single registration or claim.

## Anti-patterns (do not do this)

- Stopping after **only** `tool-agent-forum` register, or after **`task claim`**, without **executing** the task (run the command in the title, fix code, re-run until green).
- Treating “I claimed a task” as “done.” **Claiming is the start**, not the finish.
- Ignoring **`tool-tasks status --toon`** for the whole session. Re-check after **every** `task close`.
- **Fixing lint or code without a matching `task close`.** If you change the tree to satisfy `moon run …:lint` (or workspace lint), **claim** the corresponding open tool-tasks leaf first (or immediately after starting), then **`task close`** when green—keep the board honest.
- **Exiting before the stop condition** (§3 step 5) because you are “tired” or “handing off.” Subagents must **loop until** `ready == 0` **and** `in_progress == 0` (and no personal open claims), unless the parent explicitly revokes the run.

## 1) Activate (full session bootstrap)

Do **`.cursor/commands/activate.md`**: roam, `tool-tasks init` + `status --toon`, `gh` if needed, `rust.md`, `effect.rs-fundamentals` skill.

## 2) Forum: register once per session

1. `devenv shell -- cargo build -q -p tool-agent-forum` if `target/debug/tool-agent-forum` is missing.
2. `devenv shell -- cargo run -p tool-agent-forum -- init`
3. `devenv shell -- cargo run -p tool-agent-forum -- agent register --name "<unique display name>"`  
   - Stdout = your **`AgentId`** (e.g. `agent-…`). Save it.  
   - Exit **2** = duplicate active name → change name and retry.

## 3) Work loop (mandatory — repeat until stop condition below)

Each **iteration**:

1. **`devenv shell -- cargo run -p tool-tasks -- status --toon`**
2. **If you have any task you claimed** (`list --toon --claimed-by "<your display name>"` or inspect titles):  
   - **Do the work** to completion (e.g. `moon run <project>:lint`, fix findings **without** new `allow()` silencing, run tests if relevant).  
   - **`devenv shell -- cargo run -p tool-tasks -- task close --id <id>`** when finished.
3. **Else if `ready` > 0:** pick an **open** task (prefer unclaimed), then  
   `devenv shell -- cargo run -p tool-tasks -- task claim --id <id> --by "<your display name>"`  
   → next iteration must **execute** it (step 2).
4. **Else if `ready` == 0** but **`in_progress` > 0:** you are **not** done globally—other agents hold work. Use the **forum** (§4): status post, `@mention` active agents by **`AgentId`**, check your **unanswered mentions**, offer help or ask for unblock. Then **sleep/poll** and repeat from step 1 (do not exit immediately).
5. **Stop condition (required to exit):** `ready == 0` **and** `in_progress == 0` **and** you hold **no** non-closed claimed tasks. Only then may you exit (optional: forum sign-off post, `agent deregister`). Until then, keep iterating steps 1–4.

**Epic / parent tasks:** If the orchestrator assigned a **parent** epic, still **close leaf tasks** you finish; do not “complete” the epic until all children are closed unless instructed otherwise.

## 4) Forum: coordinate with other agents

Mentions use **`@<AgentId>`** only (exact id from `agent list --toon`), not display names.

| Goal | Command |
|------|---------|
| Who is active? | `devenv shell -- cargo run -p tool-agent-forum -- agent list --toon` |
| Start a coordination thread | `devenv shell -- cargo run -p tool-agent-forum -- post create --author <your AgentId> --body "… @agent-other-id …"` |
| Reply on a thread | `devenv shell -- cargo run -p tool-agent-forum -- comment create --post <post-id> --author <your AgentId> --body "…"` |
| Your inbox | `devenv shell -- cargo run -p tool-agent-forum -- mentions unanswered --agent <your AgentId> --toon` |
| Clear an inbox item | `devenv shell -- cargo run -p tool-agent-forum -- mentions ack --id <mention_id from JSON>` |
| Read a thread | `devenv shell -- cargo run -p tool-agent-forum -- thread show --post <post-id> --toon` |

**When to post:** overlap on files, unclear ownership, `ready == 0` but work stuck in `in_progress`, blocker deps, or handoff. **Poll mentions** at least when idle or waiting.

## 5) Concurrency note

`tool-tasks` and `tool-agent-forum` v1 assume **single-writer** friendly use; parallel agents should still **retry** on odd failures. Do not run **two processes** appending the same log recklessly—coordinate via forum if you see corruption/errors.

## 6) Root path

If not cwd, pass `--root <repo>` to both CLIs consistently.
