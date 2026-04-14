---
name: tool-llm-git-context
description: >-
  Vault directory and append-only hook JSONL logs under vault/Logs.
  Use when configuring or debugging session-init, setup-cursor, hooks, or
  .tool-llm-git-context paths. Always read .cursor/rules/llm-git-context.mdc
  and run CLI via devenv shell --.
category: tooling
displayName: tool-llm-git-context
color: gray
---

# tool-llm-git-context

## When to use

- User mentions **session-init**, **session-handoff**, **`vault/Logs/*.jsonl`** session log, **llm-git-context** hooks, or `.tool-llm-git-context/`.
- Editing `crates/tool-llm-git-context` or `.cursor/hooks/llm-git-context-append.sh`.

## What to do

1. Read **`.cursor/rules/llm-git-context.mdc`** in full — it is authoritative.
2. Read **`crates/tool-llm-git-context/README.md`** for command flags and defaults.
3. Run all CLI invocations as **`devenv shell -- cargo run -p tool-llm-git-context -- …`**.

## Hard rules

- **Never** patch **`.tool-llm-git-context/vault/Logs/*.jsonl`** with editor tools (hooks append each line).
- **Never** rewrite `current-session.json` by hand — use `session-init`.

## User shortcuts

- Slash command **`/llm-git-context`** (`.cursor/commands/llm-git-context.md`) loads the same workflow.
- Session **validation only:** subcommand **`verify-event`** (pipe JSON on stdin). For combined git + tasks + session file context, use **`git`**, **`tool-tasks status --toon`**, and **`current-session.json`** as separate steps.