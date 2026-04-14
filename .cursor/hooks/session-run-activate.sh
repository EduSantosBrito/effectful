#!/usr/bin/env bash
# Cursor sessionStart: inject /activate bootstrap for the agent (see .cursor/commands/activate.md).
# This does not execute activate or shell commands — only adds agent_message text (LLM may skip).
# stdin: hook JSON (discarded). stdout: hook response JSON for Cursor.

set -euo pipefail
cat >/dev/null || true

msg='Session bootstrap — run this before any other work, as if the user had invoked the /activate slash command:

1) Read and follow `.cursor/commands/activate.md` in full (use the Read tool on that path, or `devenv shell -- cat .cursor/commands/activate.md`). Execute the listed setup: roam workflow, tool-tasks init + status --json, GitHub CLI when needed, rust command file, Effect skill path — exactly as that file specifies.

2) Do not skip this step; treat it as the first mandatory context load for the session.'

jq -n --arg msg "$msg" '{continue: true, agent_message: $msg, agentMessage: $msg}'
