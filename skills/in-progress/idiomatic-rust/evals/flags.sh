#!/usr/bin/env bash
# Shared by score.sh and check.sh. Source it; do not run it. The lint set is written once, in
# LINTS.md, and a scenario's extra flags once, in its check-flags file.

# read_check_flags <LINTS.md>: sets `flags` from the `flags=( ... )` block under "## The check
# command", so the scorer cannot drift from the skill.
read_check_flags() {
  local lints_md=$1 block
  block=$(awk '
    /^## The check command/ { section = 1 }
    section && /^flags=\(/ { inside = 1 }
    inside { print }
    inside && /^\)$/ { exit }
  ' "$lints_md")
  if [[ -z $block ]]; then
    echo "flags.sh: no 'flags=(' block under '## The check command' in $lints_md" >&2
    return 1
  fi
  flags=()
  eval "$block"
  if ((${#flags[@]} == 0)); then
    echo "flags.sh: the flags block in $lints_md is empty" >&2
    return 1
  fi
}

# read_scenario_flags <scenario-dir>: sets `scenario_flags` from the scenario's check-flags file,
# one line of extra clippy arguments applied to both check passes (a CLI relaxes print_stdout, for
# example), or to an empty array when the scenario has none.
read_scenario_flags() {
  local file="$1/check-flags"
  # shellcheck disable=SC2034 # read by the script that sourced this file
  scenario_flags=()
  if [[ -f $file ]]; then
    # shellcheck disable=SC2034 # read by the script that sourced this file
    read -ra scenario_flags <"$file"
  fi
}
