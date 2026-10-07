#!/usr/bin/env bash
# SSR concurrency stress: bursts of client-aborted requests at SSR routes, then a
# liveness check after every round.
#
# WHY: leptos_i18n 0.6.x's server-side isomorphic effect (context.rs:213) can read a
# disposed signal in a detached tokio task once the request Owner is disposed. With
# panic = "abort" that kills the server; with unwind the panic stays in the task.
# This is the fast repro (orchestration_log/recon/2026-10-04/readiness/n-panic.md).
#
# Usage: scripts/ssr-stress.sh <port> <server-log-file> [rounds=80]
# Exit 0 = server alive after all rounds; exit 1 = server died.
set -uo pipefail

port="${1:?usage: scripts/ssr-stress.sh <port> <server-log-file> [rounds]}"
log="${2:?usage: scripts/ssr-stress.sh <port> <server-log-file> [rounds]}"
rounds="${3:-80}"

panics() { grep -c "panicked" "$log" || true; }

for round in $(seq 1 "$rounds"); do
    pids=()
    for path in / /admin /onboarding /login; do
        for timeout in 0.003 0.006 0.01 0.02 0.04 0.08; do
            curl -s -o /dev/null -b session=stresstoken123 --max-time "$timeout" "http://127.0.0.1:${port}${path}" & pids+=($!)
            curl -s -o /dev/null --max-time "$timeout" "http://127.0.0.1:${port}${path}" & pids+=($!)
        done
    done
    wait "${pids[@]}" 2>/dev/null
    if ! curl -sf -o /dev/null --max-time 5 "http://127.0.0.1:${port}/login"; then
        echo "DEAD at round ${round}; panics logged: $(panics)"
        exit 1
    fi
done
echo "ALIVE after ${rounds} rounds; panics logged: $(panics)"
