#!/bin/bash
# Run a long command in a sibling Herdr pane, wait for its exit code, print the tail, close the pane.
#   tools/hrun.sh [--keep] [--down] [--lines N] [--timeout MS] -- <command string>
#   tools/hrun.sh --wait <pane-id> [--lines N] [--timeout MS]     # resume waiting after a timeout
# Exit code = the command's exit code; 124 = still running (pane kept, resume with --wait).
# Outside Herdr (HERDR_ENV!=1) the command runs here directly.
set -u
keep=0 dir=right lines=60 timeout=580000 pane=
while [ $# -gt 0 ]; do
  case $1 in
    --keep) keep=1 ;;
    --down) dir=down ;;
    --lines) lines=$2; shift ;;
    --timeout) timeout=$2; shift ;;
    --wait) pane=$2; shift ;;
    --) shift; break ;;
    *) echo "hrun: unknown option $1" >&2; exit 2 ;;
  esac
  shift
done
cmd="$*"
[ -n "$pane" ] || [ -n "$cmd" ] || { echo "hrun: no command" >&2; exit 2; }

if [ -z "$pane" ] && [ "${HERDR_ENV:-}" != 1 ]; then
  echo "hrun: not inside Herdr; running directly" >&2
  bash -c "$cmd" 2>&1 | tail -n "$lines"; exit "${PIPESTATUS[0]}"
fi

if [ -z "$pane" ]; then
  pane=$(herdr pane split --current --direction "$dir" --cwd "$PWD" --no-focus |
    python3 -c 'import json,sys; print(json.load(sys.stdin)["result"]["pane"]["pane_id"])') || exit 1
  # Marker at line start + digits: the echoed command line holds a literal `$r` and cannot match; ( ) keeps `exit` from killing the pane shell.
  herdr pane run "$pane" "( $cmd ); r=\$?; echo; echo HRUN_RC=\$r" >/dev/null || exit 1
fi
echo "hrun: pane $pane" >&2

if ! herdr pane wait-output "$pane" --regex '^HRUN_RC=[0-9]' --timeout "$timeout" >/dev/null 2>&1; then
  herdr pane read "$pane" --source recent-unwrapped --lines "$lines"
  echo "hrun: still running in $pane; resume: tools/hrun.sh --wait $pane (kill own PIDs to stop; ctrl+c misses gurun.sh)" >&2
  exit 124
fi
out=$(herdr pane read "$pane" --source recent-unwrapped --lines "$lines")
printf '%s\n' "$out" | grep -v '^HRUN_RC='
rc=$(printf '%s\n' "$out" | sed -n 's/^HRUN_RC=\([0-9]*\).*/\1/p' | tail -1)
[ "$keep" = 1 ] && echo "hrun: kept pane $pane" >&2 || herdr pane close "$pane" >/dev/null
exit "${rc:-1}"
