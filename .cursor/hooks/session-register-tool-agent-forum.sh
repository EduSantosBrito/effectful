#!/usr/bin/env bash
# Cursor sessionStart: register this workspace session with tool-agent-forum when possible.
#
# Root issue with "message-only" hooks: Cursor only *suggests* work via agent_message; the LLM
# may skip it. This script therefore runs the CLI when a prebuilt binary exists (see hooks.json
# timeout). stdin: hook JSON. stdout: hook response JSON only.

set -euo pipefail

input=$(cat)
workspace_root=$(echo "$input" | jq -r '.workspace_roots[0] // ""')

manual='Tool agent forum — register this Cursor session with the workspace store (if not already done automatically above):

1) From workspace root: `devenv shell -- cargo run -p tool-agent-forum -- init`

2) Display name: `TOOL_AGENT_FORUM_DISPLAY_NAME` env if set; else a unique name (hostname, etc.). Optionally `devenv shell -- cargo run -p tool-agent-forum -- agent list --json` first.

3) `devenv shell -- cargo run -p tool-agent-forum -- agent register --name "<display name>"`

4) Exit code 2 = duplicate active name — pick a new name and retry once.

**Parallel / swarm agents:** Registration is not the end state. Load `.cursor/commands/agent-forum-worker.md` — keep claiming and *finishing* tool-tasks (close after work), loop until `tool-tasks status --json` shows `ready` and `in_progress` both 0, and use forum posts + `@AgentId` mentions + `mentions unanswered` to coordinate.

See `history/2026-04-05-tool-agent-forum-plan.md` and `.cursor/rules/tool-tasks.mdc` for single-writer JSONL expectations.'

forum_bin=""
if [[ -n "$workspace_root" ]]; then
    for cand in "$workspace_root/target/debug/tool-agent-forum" "$workspace_root/target/release/tool-agent-forum"; do
        if [[ -x "$cand" ]]; then
            forum_bin=$cand
            break
        fi
    done
fi

if [[ -z "$workspace_root" || ! -d "$workspace_root" ]]; then
    jq -n --arg msg "tool-agent-forum: hook had no workspace_roots[0]; $manual" '{continue: true, agent_message: $msg, agentMessage: $msg}'
    exit 0
fi

if [[ -z "$forum_bin" ]]; then
    jq -n --arg msg "tool-agent-forum: no executable at target/debug|release/tool-agent-forum — run once: devenv shell -- cargo build -p tool-agent-forum. Then reopen the agent or rely on manual steps. $manual" '{continue: true, agent_message: $msg, agentMessage: $msg}'
    exit 0
fi

"$forum_bin" --root "$workspace_root" init 2>/dev/null || true

base_name="${TOOL_AGENT_FORUM_DISPLAY_NAME:-cursor-$(hostname)-$$}"
name="$base_name"
set +e
reg_out=$("$forum_bin" --root "$workspace_root" agent register --name "$name" 2>/dev/null)
reg_ec=$?
set -e

if [[ "$reg_ec" -eq 2 ]]; then
    name="${base_name}-${RANDOM}"
    set +e
    reg_out=$("$forum_bin" --root "$workspace_root" agent register --name "$name" 2>/dev/null)
    reg_ec=$?
    set -e
fi

if [[ "$reg_ec" -eq 0 && -n "$reg_out" ]]; then
    jq -n --arg msg "tool-agent-forum: registered this session (display name \"$name\", agent id \"$reg_out\"). Use @${reg_out} in forum content when relevant. No need to register again in this session." '{continue: true, agent_message: $msg, agentMessage: $msg}'
else
    jq -n --arg msg "tool-agent-forum: automatic register failed (exit $reg_ec). $manual" '{continue: true, agent_message: $msg, agentMessage: $msg}'
fi
