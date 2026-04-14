#!/usr/bin/env bash
# Pre-commit hook: formatting only (fast gate).
# Heavy tasks (check, test, build, audit) run in the pre-push hook instead.
# Unset hook-injected GIT_* vars so nested git commands work normally.
unset GIT_DIR GIT_INDEX_FILE GIT_WORK_TREE

set -euo pipefail
mkdir -p tmp
exec env TMPDIR="$(pwd)/tmp" devenv shell -- moon run :format
