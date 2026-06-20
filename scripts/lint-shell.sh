#!/usr/bin/env bash
#
# Lint the plugin's shell scripts with ShellCheck plus a bash syntax check.
#
# Usage:
#   scripts/lint-shell.sh            Lint all tracked shell scripts
#   scripts/lint-shell.sh file ...   Lint the given files
#
# Shell files are detected by a `.sh` suffix or a shell shebang. Polyglot
# `.cmd` wrappers (no shebang) are intentionally skipped.
set -euo pipefail

die() {
  echo "error: $*" >&2
  exit 1
}

command -v shellcheck >/dev/null 2>&1 || die "shellcheck is not on PATH"

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$repo_root"

is_shell_file() {
  local path="$1" first=""
  [ -f "$path" ] || return 1
  case "$path" in
    *.sh) return 0 ;;
  esac
  IFS= read -r first <"$path" || true
  [[ "$first" =~ ^#!.*[/[:space:]](bash|dash|ksh|sh)([[:space:]]|$) ]]
}

files=()
if [ "$#" -gt 0 ]; then
  for f in "$@"; do
    is_shell_file "$f" && files+=("$f")
  done
else
  while IFS= read -r path; do
    is_shell_file "$path" && files+=("$path")
  done < <(git ls-files)
fi

if [ "${#files[@]}" -eq 0 ]; then
  echo "No shell files to lint."
  exit 0
fi

echo "Linting ${#files[@]} shell file(s): ${files[*]}"
shellcheck --severity=warning --external-sources --source-path=SCRIPTDIR "${files[@]}"
for f in "${files[@]}"; do
  bash -n "$f"
done
echo "✓ shell lint passed"
