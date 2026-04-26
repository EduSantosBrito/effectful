# TASK

Implement one vertical slice.

Issue file: {{ISSUE_FILE}}

Issue title: {{ISSUE_TITLE}}

Branch: {{BRANCH}}

VCS:

{{VCS_INSTRUCTIONS}}

SOURCE:

{{SOURCE_INSTRUCTIONS}}

{{ISSUE_BODY}}

# CONTEXT

Before implementing behavior changes, use the `tdd` skill. Follow red-green-refactor for this issue.

Read relevant repo docs before changing code:

- `README.md`
- `TESTING.md`
- `moon.yml`
- Relevant crate `Cargo.toml` and nearby tests

Use local code search to find the smallest relevant surface area. Pay close attention to tests near changed code.

# EXECUTION

- Make minimal, surgical changes.
- Preserve type safety: no unchecked panics, no unnecessary `unwrap`, no broad dynamic typing.
- Model invalid states explicitly with enums/results where needed.
- Add deterministic tests for acceptance criteria when behavior changes.
- Prefer red-green-refactor for bug fixes and behavior changes.
- Do not modify `.sandcastle`.

# FEEDBACK LOOPS

Run relevant checks before committing. Prefer narrow checks first, then broader checks if practical:

- `cargo test -p <crate>`
- `cargo nextest run -p <crate>`
- `moon run :test`
- `moon run :ci-format :clippy`

# COMMIT

Commit your changes with a concise conventional commit message. If `.jj/` exists, use `jj describe`; otherwise use `git commit`.

If the task cannot be completed, commit only complete safe work and explain blockers in the final output.

When complete, output {{COMPLETION_SIGNAL}}.

# FINAL RULES

Only work on this issue. Do not start adjacent work.
