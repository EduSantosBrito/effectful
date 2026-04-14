#!/usr/bin/env bash
# Appends hook stdin JSON to vault/Logs/{UTC_YYYYMMDDHHMMSS}-{conversation_id}.jsonl
set -euo pipefail

# Resolve monorepo root from this script's location (Cursor cwd is not guaranteed).
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(git -C "${SCRIPT_DIR}/../.." rev-parse --show-toplevel 2>/dev/null || true)"
[[ -n "${REPO_ROOT}" ]] || exit 0

# Cursor hooks do not run inside devenv; `tool-llm-git-context` is usually not on PATH.
BIN=""
if [[ -n "${LLM_GIT_CONTEXT_BIN:-}" && -x "${LLM_GIT_CONTEXT_BIN}" ]]; then
    BIN="${LLM_GIT_CONTEXT_BIN}"
elif [[ -x "${REPO_ROOT}/target/debug/tool-llm-git-context" ]]; then
    BIN="${REPO_ROOT}/target/debug/tool-llm-git-context"
elif [[ -x "${REPO_ROOT}/target/release/tool-llm-git-context" ]]; then
    BIN="${REPO_ROOT}/target/release/tool-llm-git-context"
else
    TMP="$(command -v tool-llm-git-context 2>/dev/null || true)"
    [[ -n "${TMP}" ]] && BIN="${TMP}"
fi

if [[ -z "${BIN}" ]]; then
    echo "llm-git-context-append: no tool-llm-git-context binary (run: cargo build -p tool-llm-git-context, or set LLM_GIT_CONTEXT_BIN)" >&2
    exit 0
fi

exec "${BIN}" append-event --repo "${REPO_ROOT}"
