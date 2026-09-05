#!/usr/bin/env bash
# Explicit model and per-run budget keep repeated comparisons reproducible.
set -euo pipefail
if (($# < 2)); then
  echo 'usage: compare.sh <model> <budget-usd-per-run> [scenario ...]' >&2
  exit 2
fi
model=$1
export EVAL_MAX_BUDGET_USD=$2
shift 2
here=$(cd "$(dirname "$0")" && pwd)
repeats=${EVAL_REPEATS:-2}
[[ $repeats =~ ^[0-9]+$ && $repeats -ge 2 ]] || { echo 'at least two repeats required' >&2; exit 2; }
scenarios=("$@")
((${#scenarios[@]})) || scenarios=(s6-decoder s7-inference s8-cli s9-review)
for scenario in "${scenarios[@]}"; do
  [[ $scenario =~ ^[a-z0-9-]+$ && -d $here/scenarios/$scenario/start ]] || {
    echo "unknown scenario: $scenario" >&2; exit 2;
  }
done
python3 - "$EVAL_MAX_BUDGET_USD" "$repeats" "${#scenarios[@]}" <<'PYPLAN'
from decimal import Decimal, InvalidOperation
import sys
try:
    budget = Decimal(sys.argv[1])
    if not budget.is_finite() or budget <= 0:
        raise ValueError('budget must be positive and finite')
except (InvalidOperation, ValueError) as error:
    raise SystemExit(str(error))
runs = int(sys.argv[2]) * int(sys.argv[3]) * 3
print(f'{runs} runs; maximum requested model budget ${runs * budget}')
PYPLAN
[[ ${EVAL_DRY_RUN:-0} != 1 ]] || exit 0
root=${EVAL_RESULTS_ROOT:-$here/results/comparison-$(date +%Y%m%d-%H%M%S)}
mkdir -p "$root"
root=$(cd "$root" && pwd)
# Freeze revised guidance and evaluator lint policy for the whole campaign.
export EVAL_SKILL_SOURCE="$root/revision/idiomatic-rust"
[[ ! -e $EVAL_SKILL_SOURCE ]] || { echo 'choose a fresh results root' >&2; exit 2; }
mkdir -p "$EVAL_SKILL_SOURCE"
cp "$here/../"*.md "$EVAL_SKILL_SOURCE/"
export EVAL_LINTS_PATH="$EVAL_SKILL_SOURCE/LINTS.md"
for ((repeat = 1; repeat <= repeats; repeat++)); do
  export EVAL_RESULTS_ROOT="$root/repeat-$repeat"
  arms=(bare merged skill)
  if ((repeat % 2 == 0)); then arms=(skill merged bare); fi
  for scenario in "${scenarios[@]}"; do
    for arm in "${arms[@]}"; do
      "$here/run.sh" "$scenario" "$arm" "$model"
    done
    "$here/score.sh" "$scenario" >"$EVAL_RESULTS_ROOT/$scenario/scores.md"
  done
done
python3 "$here/analyze.py" summary "$root"
