#!/usr/bin/env bash
# Shared SessionStart context builder for the leaf plugin.
# Sourced by hooks/session-start. Not executed directly. Since the single-folder
# dual-manifest layout (0.4.3), Codex also discovers hooks.json and runs this
# hook, so the using-leaf context is injected on Claude Code, Cursor, Codex, and
# the SDK alike.

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

⚠️ REQUIRED — the \`leaf\` CLI is not on PATH.

The skills above drive a repo-local \`.leaf/\` workspace through this binary;
every \`leaf\` command (\`leaf init\`, \`leaf new\`, \`leaf doctor\`) WILL FAIL
until it is installed. Before doing any LEAF work, you MUST tell the user to run:

    /leaf:install

Surface this command to the user explicitly and wait for them to run it. Do not
silently skip it, and do not install the binary yourself. \"The user probably
already knows\" or \"I'll mention it later\" is the wrong instinct — state it now."
  fi

  content="${content}
</EXTREMELY_IMPORTANT>"

  escape_for_json "$content"
}
