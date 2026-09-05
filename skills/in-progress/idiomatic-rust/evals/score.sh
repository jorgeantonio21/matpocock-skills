#!/usr/bin/env bash
# Score bare, merged, and revised skill trees with shared lint exceptions.
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
exec python3 "$here/score.py" "$@"
