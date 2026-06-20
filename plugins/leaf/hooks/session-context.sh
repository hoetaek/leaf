#!/usr/bin/env bash
# Shared SessionStart context builder for the leaf plugin.
# Sourced by hooks/session-start (Claude Code / Cursor / SDK) and
# hooks/session-start-codex (Codex). Not executed directly.

# Escape a string for embedding inside a JSON string value.
# Each ${s//old/new} is a single pass — fast and dependency-free.
escape_for_json() {
  local s="$1"
  s="${s//\\/\\\\}"
  s="${s//\"/\\\"}"
  s="${s//$'\n'/\\n}"
  s="${s//$'\r'/\\r}"
  s="${s//$'\t'/\\t}"
  printf '%s' "$s"
}

# Build the escaped SessionStart context: the using-leaf skill, plus a CLI
# install note when the `leaf` binary is missing. Arg 1 is the hooks dir.
build_leaf_context() {
  local script_dir="$1"
  local plugin_root using_leaf content
  plugin_root="$(cd "${script_dir}/.." && pwd)"
  using_leaf="$(cat "${plugin_root}/skills/using-leaf/SKILL.md" 2>&1 || echo "Error reading using-leaf skill")"

  content="<EXTREMELY_IMPORTANT>
You have LEAF skills.

**Below is your 'leaf:using-leaf' skill — your entry to the LEAF workflow. For all other leaf skills, use the Skill tool.**

${using_leaf}"

  if ! command -v leaf >/dev/null 2>&1; then
    content="${content}

⚠️  The \`leaf\` CLI is not on PATH. The skills drive a repo-local \`.leaf/\` workspace through it. Install it: \`brew install hoetaek/tap/leaf\` (or \`cargo install --git https://github.com/hoetaek/leaf\`)."
  fi

  content="${content}
</EXTREMELY_IMPORTANT>"

  escape_for_json "$content"
}
