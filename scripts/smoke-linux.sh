#!/usr/bin/env bash
# Run under an isolated X display; require a visible application window.
set -euo pipefail
binary=$(realpath "$1")
log=$(mktemp)
"$binary" >"$log" 2>&1 &
app_pid=$!
# Invoked by the EXIT trap below.
# shellcheck disable=SC2329
cleanup() {
    kill "$app_pid" 2>/dev/null || true
    wait "$app_pid" 2>/dev/null || true
    cat "$log"
    rm -f "$log"
}
trap cleanup EXIT
for ((attempt = 0; attempt < 40; attempt++)); do
    if ! kill -0 "$app_pid" 2>/dev/null; then
        echo "Application exited before its window was ready" >&2
        exit 1
    fi
    if xdotool search --onlyvisible --name '^IDE_C--$' >/dev/null 2>&1; then
        echo "Visible IDE_C-- window detected"
        exit 0
    fi
    sleep 0.25
done
echo "No visible IDE_C-- window after 10 seconds" >&2
exit 1
