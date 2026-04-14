Use **tool-tasks** for tracking fixes. As you run `bin/pre-push`, for each problem found create simple small tasks:

`devenv shell -- cargo run -p tool-tasks -- task create --title "…"`

Include problems, type errors, and failures from anywhere in the monorepo. Finish those tasks one by one (`task claim` → fix → `task close`). See `.cursor/rules/tool-tasks.mdc` for the full CLI.
