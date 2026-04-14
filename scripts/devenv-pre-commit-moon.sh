#!/usr/bin/env bash
# Pre-commit hook: format + check + test (fast gate before each commit).
# The pre-push hook runs the full suite (build, coverage, audit, check-docs).
# Unset hook-injected GIT_* vars so nested git commands work normally.
unset GIT_DIR GIT_INDEX_FILE GIT_WORK_TREE

set -euo pipefail
mkdir -p tmp
# MOON_CONCURRENCY=1 prevents parallel nightly cargo builds (Dylint rules crate UI tests)
# from racing on the shared target directory — same guard as the pre-push hook.
exec env TMPDIR="$(pwd)/tmp" MOON_CONCURRENCY=1 devenv shell -- moon run :format :check :test
