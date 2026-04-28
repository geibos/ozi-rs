#!/usr/bin/env bash
set -euo pipefail

evidence_dir=".sisyphus/evidence"
build_log="$evidence_dir/task-6-release-build.log"
startup_log="$evidence_dir/task-6-release-startup.log"
combined_log="$evidence_dir/task-6-final-run-release.log"
gate_log="$evidence_dir/task-6-final-warning-gate.log"
synthetic_log="$evidence_dir/task-6-synthetic-warning-check.log"
startup_output=""
binary="./src-tauri/target/release/ozi-rs"
pattern='warning|warn|deprecated|security|csp|bundle|error'

mkdir -p "$evidence_dir"

scan_combined_output() {
    local input_file="$1"
    local raw_matches
    local filtered_matches
    raw_matches="$(mktemp)"
    filtered_matches="$(mktemp)"
    trap 'rm -f "$raw_matches" "$filtered_matches"' RETURN

    grep -Ein "$pattern" "$input_file" > "$raw_matches" || true

    grep -Ev '^[0-9]+:[[:space:]]+(Compiling|Checking|Building)[[:space:]]+[[:alnum:]_.+-]+[[:space:]]+v[0-9]' "$raw_matches" > "$filtered_matches" || true

    grep -Fv \
        -e '--no-bundle' \
        -e 'BundleLoaderView' \
        -e '.sisyphus/notepads/run-release-warnings/' \
        -e 'task-4-error-variants.txt' \
        "$filtered_matches" > "$gate_log" || true

    if [[ -s "$gate_log" ]]; then
        printf 'Release warning gate failed; unclassified warning-like lines remain. See %s\n' "$gate_log" >&2
        return 1
    fi

    printf 'Release warning gate passed; no unclassified warning-like lines found.\n' > "$gate_log"
}

if [[ "${1:-}" == "--synthetic-warning-check" ]]; then
    printf 'WARNING: synthetic release warning\n' > "$synthetic_log"
    cp "$synthetic_log" "$combined_log"
    scan_combined_output "$combined_log"
    exit $?
fi

build_status=0
npm run tauri build -- --no-bundle > "$build_log" 2>&1 || build_status=$?

startup_status=0
: > "$startup_log"
if [[ -x "$binary" ]]; then
    startup_output="$(mktemp)"
    "$binary" > "$startup_output" 2>&1 &
    app_pid=$!

    cleanup_app() {
        local kill_sent=0
        if kill -0 "$app_pid" 2>/dev/null; then
            kill -TERM "$app_pid" 2>/dev/null || true
            kill_sent=1
            sleep 2
        fi
        if kill -0 "$app_pid" 2>/dev/null; then
            kill -KILL "$app_pid" 2>/dev/null || true
            kill_sent=2
        fi
        wait "$app_pid" 2>/dev/null || true

        {
            printf '[task-6 startup] begin bounded capture: 15s\n'
            cat "$startup_output"
            printf '[task-6 startup] stop status: %s\n' "$kill_sent"
            printf '[task-6 startup] end bounded capture\n'
        } > "$startup_log"
        rm -f "$startup_output"
    }

    trap cleanup_app EXIT
    sleep 15
    cleanup_app
    trap - EXIT
else
    startup_status=1
    printf 'missing release binary: %s\n' "$binary" > "$startup_log"
fi

{
    printf '[task-6 build] begin capture\n'
    cat "$build_log"
    printf '[task-6 build] end capture: status %s\n' "$build_status"
    printf '[task-6 startup] begin capture\n'
    cat "$startup_log"
    printf '[task-6 startup] end capture: status %s\n' "$startup_status"
} > "$combined_log"

scan_status=0
scan_combined_output "$combined_log" || scan_status=$?

if (( build_status != 0 )); then
    printf 'Release build failed with status %s. See %s\n' "$build_status" "$build_log" >&2
fi
if (( startup_status != 0 )); then
    printf 'Release startup failed with status %s. See %s\n' "$startup_status" "$startup_log" >&2
fi

if (( build_status != 0 || startup_status != 0 || scan_status != 0 )); then
    exit 1
fi
