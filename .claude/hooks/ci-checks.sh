#!/usr/bin/env bash
# Runs CI's format, lint and type checks after a Claude response that changed files.
#   ci-checks.sh baseline  (UserPromptSubmit) records the working tree's state
#   ci-checks.sh check     (Stop) runs the checks if the tree changed since then
set -uo pipefail

cd "$CLAUDE_PROJECT_DIR" || exit 0
input=$(cat)
state_file="$(git rev-parse --git-dir)/claude-ci-baseline"

tree_fingerprint() {
  {
    git status --porcelain=v1 --untracked-files=all
    git diff HEAD
    git ls-files --others --exclude-standard -z | xargs -0 -r sha1sum
  } | sha1sum | cut -d' ' -f1
}

if [ "${1:-}" = "baseline" ]; then
  tree_fingerprint >"$state_file"
  exit 0
fi

[ "$(tree_fingerprint)" = "$(cat "$state_file" 2>/dev/null)" ] && exit 0

report=""
run_check() {
  local name=$1
  shift
  local output
  if ! output=$("$@" 2>&1); then
    report+="### $name failed: $*"$'\n'"$(tail -n 60 <<<"$output")"$'\n\n'
  fi
}

run_check "Rust format" cargo fmt --all -- --check
run_check "Clippy" cargo clippy --workspace --all-targets --all-features -- -D warnings
run_check "Prettier" npm run --silent format:check
run_check "ESLint" npm run --silent lint
run_check "TypeScript" npm run --silent typecheck

[ -z "$report" ] && exit 0

# A second failure after Claude already tried to fix it goes to the user instead,
# so a check Claude can't fix doesn't loop forever.
if [ "$(jq -r '.stop_hook_active // false' <<<"$input")" = "true" ]; then
  jq -n --arg msg "CI checks still failing:"$'\n'"$report" '{systemMessage: $msg}'
else
  jq -n --arg reason "CI checks failed after your changes. Fix these before finishing:"$'\n\n'"$report" \
    '{decision: "block", reason: $reason}'
fi
