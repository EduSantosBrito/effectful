# Cursor setup

## Rules vs skills: how the AI uses them

- **Rules** (`.cursor/rules/*.mdc`): Cursor injects rules into the AI context in two ways:
  - **Always-applied**: frontmatter `alwaysApply: true` (and optional `globs`) — these are always in context (e.g. `shell.mdc`, `tool-tasks.mdc`, `mcp-servers.mdc`). The AI must never ignore them.
  - **Opt-in**: `alwaysApply: false` and empty `globs` — Cursor does *not* auto-inject these. They only apply when the user **@-mentions** the rule (e.g. `@code-reviewer`) or when a slash command tells the AI to **explicitly load** a matching rule (e.g. `/skill`).
- **Skills** (`.cursor/skills/*/SKILL.md`): Listed in the agent’s “available skills”; the AI is expected to read and follow them when chosen (e.g. via `/skill`).

**Getting the AI to use rules and pick the right one:** Use the **`/skill`** command. Its instructions tell the AI to:
1. Always respect always-applied project rules (never override them).
2. Consider *both* `.cursor/skills/` and `.cursor/rules/` when picking the best match — and to explicitly read the chosen `.mdc` when a rule fits the task.

So “pick the right skill” with `/skill` now means “pick the right skill **or rule**” and to load the relevant rule file from `.cursor/rules/` when it matches. Agency-agents–generated `.mdc` files are opt-in by design; they become usable when the AI is instructed (via `/skill`) to consider and load them, or when you @-mention them (e.g. `@code-reviewer`).

### Agent CLI tools (canonical entrypoints)

| Area | Crate | Notes |
|------|--------|------|
| Task tracking | `crates/tool-tasks` | `.cursor/rules/tool-tasks.mdc` |
| Agent forum | `crates/tool-agent-forum` | |
| LLM session JSONL, hooks, `verify-event` | `crates/tool-llm-git-context` | See crate README |

**`CLAUDE.md`** documents these; **`/activate`** lists tasks + Rust gate commands.

### LLM session logging (`tool-llm-git-context`)

- **Rule:** `.cursor/rules/llm-git-context.mdc` — auto-attached when editing `.tool-llm-git-context/**`, `crates/tool-llm-git-context/**`, `llm-git-context-append.sh`, or `.cursor/hooks.json`.
- **Skill:** `.cursor/skills/tool-llm-git-context/SKILL.md` — listed for agents; points at the rule + crate README.
- **Command:** `.cursor/commands/llm-git-context.md` — use **`/llm-git-context`** to load the workflow explicitly.
- **Bootstrap:** `.cursor/commands/activate.md` includes a short checklist item for new sessions and hook binaries.

## Agency agents (submodule)

The [agency-agents](https://github.com/msitarzewski/agency-agents) repo is included as a git submodule at `.cursor/agency-agents`. It provides agent definitions that can be converted into Cursor rules.

**Note:** The install script generates every agent as a `.mdc` with `alwaysApply: false` and `globs: ""`, so Cursor never auto-applies them. To use an agent, either reference it in the prompt (e.g. `@frontend-developer`) or use `/skill` so the AI considers and loads the matching rule from `.cursor/rules/`. To make a specific agent always-on or file-scoped, edit that rule’s frontmatter (e.g. set `alwaysApply: true` or `globs: "**/*.tsx"`).

### Fresh clone

After cloning the repo, init the submodule once:

```bash
git submodule update --init --recursive
```

### Auto-updates

**Dependabot** is configured to open a PR whenever the upstream `agency-agents` repo has new commits (weekly check). Merge the Dependabot PR to update the submodule; no manual fetching needed.

### Regenerating Cursor rules from agency-agents

To (re)generate `.cursor/rules/*.mdc` from the submodule:

```bash
.cursor/agency-agents/scripts/install.sh --tool cursor
```

Run from the repo root. See `.cursor/agency-agents/integrations/cursor/README.md` for details.

## Microsoft RustTraining (submodule)

The [microsoft/RustTraining](https://github.com/microsoft/RustTraining) repo is included as a git submodule at `.cursor/microsoft-rust-training`. It contains seven mdBook-style courses (bridge books from C/C++, C#, and Python; deep dives on async, patterns, type-driven correctness; and engineering practices).

**For AI assistants:** Use the Cursor skills **rust-microsoft-training** and **rust-production-engineering** (under `.cursor/skills/`) so tasks are grounded in the right book and chapter paths inside the submodule.

After a fresh clone, submodules are initialized the same way as for agency-agents:

```bash
git submodule update --init --recursive
```
