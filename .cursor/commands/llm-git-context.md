# LLM git context (vault sessions)

Use this when the user wants **session logging**, **vault paths**, **hook setup**, or **tool-llm-git-context** behavior.

1. **Read** `.cursor/rules/llm-git-context.mdc` in full (use the Read tool) and follow it for the rest of the task.
2. If they need a **new** persisted session, run:  
   `devenv shell -- cargo run -p tool-llm-git-context -- session-init`  
   and report the printed `branch`, `vault`, and `events_log` paths.
3. If hooks are not appending, ensure the binary exists (`devenv shell -- cargo build -p tool-llm-git-context`) and that `setup-cursor` has been run at least once after clone.
4. To **sync the session log into vault notes**, run `session-handoff --log <session.jsonl>` (path from `current-session.json` → `events_log`, typically `vault/Logs/<stem>.jsonl`), paste stdout to the agent, then have it update Obsidian pages under the vault directory.
5. **Ending a session** is manual: there is no `session-finish` in this tool. Run **`session-init`** again when you want a new log file.
6. To **validate a hook JSON line without writing** (fixtures / CI): `echo '<json>' | devenv shell -- bash -c 'cargo run -p tool-llm-git-context -- verify-event'` (stdin must be one JSON value; same rules as `append-event`).

Do not hand-edit the session JSONL or `current-session.json`.
