# Use the right skill or rule for this task

You are using the **/skill** command. The user has provided a task or topic. Your job is to pick the best-matching **skill or rule** and apply it, while **always** respecting project rules that are already in your context.

## Mandatory: Always-applied project rules

**You MUST follow all always-applied project rules** that appear in your context (e.g. under "always_applied_workspace_rules" or similar). These typically include things like:
- How to run shell commands (e.g. `devenv shell --`)
- Issue tracking (**tool-tasks** only via `cargo run -p tool-tasks`, no markdown TODOs)
- MCP/server usage (e.g. Context7, Postgres, Debugger) and roam (CLI via devenv)

**Never override or ignore these rules** when following a skill or rule. If a skill suggests something that conflicts (e.g. running a raw shell command when the project requires `devenv shell --`), follow the project rule.

## Where to find skills and rules

**Consider both sources when choosing what to apply:**

1. **`.cursor/skills/`**  
   Each subdirectory (e.g. `code-reviewer/`, `rust-pro/`) has a **`SKILL.md`**. Use the **Read** tool to load the chosen skill. You can use the `agent_skills` list as a shortcut for names and paths, but always load the full file from disk.

2. **`.cursor/rules/`**  
   **`.mdc`** files here are first-class: role definitions, coding standards, and agent personas (e.g. from agency-agents). Many have `alwaysApply: false`, so they are **not** auto-injected—you must explicitly consider and load them when they match the task.
   - To find a matching rule: list the directory or search by keyword; read the **frontmatter** (first ~10 lines) of candidate `.mdc` files to get `description`, then **Read** the full file for the one that best matches the task.
   - If a rule’s description matches the task (e.g. "code review", "security audit", "frontend developer"), **read that .mdc and follow it** (alone or together with a skill).

Do not rely only on `agent_skills`—rules in `.cursor/rules/` are not listed there; you must look in `.cursor/rules/` and load the right `.mdc` when it fits.

## What to do

1. **Identify the task**  
   Use the user’s full message as the task (e.g. "review this PR", "add tests", "optimize this query").

2. **Choose the best skill or rule**  
   Look in **both** `.cursor/skills/` and `.cursor/rules/` for the best match. Prefer one primary source; add another only if the task clearly needs both. For rules, use directory listing + frontmatter to pick, then read the full `.mdc`.

3. **Read the chosen file**  
   Use the **Read** tool on the chosen `SKILL.md` or `.mdc` before answering or writing code.

4. **Follow it while respecting project rules**  
   Apply the skill’s or rule’s instructions. Where they conflict with always-applied project rules (shell, issue tracking, MCP, etc.), **follow the project rules**.

5. **Respond**  
   Say what you’re using (e.g. "I’ll use the code-reviewer rule and…"), then do the work.

## Rules

- **Always** read the chosen skill or rule from `.cursor/skills/` or `.cursor/rules/` before acting; do not rely only on short descriptions.
- **Always** keep following always-applied project rules; never drop them when applying a skill or rule.
- If nothing fits well, say so and either do the task with your best judgment or suggest adding a skill under `.cursor/skills/` or a rule under `.cursor/rules/`.
- If several fit, pick the most specific or the one that best matches the main goal.
