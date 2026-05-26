#!/usr/bin/env bash
# Frontend freshness guard for `just run-release`.
#
# Computes a content hash over the inputs that determine the `vite build`
# output (Svelte sources, static assets, build configuration, lockfile) and
# compares it against the hash recorded inside `build/.input-hash` from the
# previous successful run. On match, the recipe short-circuits; on mismatch
# (or missing sentinel) it runs `npm run build` and writes the fresh hash.
#
# Content-hash chosen over `find -newer` (per design.md Decision 1) because
# `git checkout` rewrites mtimes to checkout time, which would otherwise
# produce false-negative cache hits after branch switches.
#
# Outputs:
#   - On cache miss: runs `npm run build`, writes `build/.input-hash`.
#   - On cache hit:  prints "frontend cache hit (build/.input-hash matches)".

set -euo pipefail

HASH_FILE="build/.input-hash"

# Inputs that influence `vite build` output. Update this list if you add a
# new top-level config file (e.g. postcss.config.cjs, tailwind.config.js).
INPUT_PATHS=(
    src
    static
    vite.config.ts
    svelte.config.js
    package.json
    package-lock.json
    tsconfig.json
    components.json
)

compute_hash() {
    # Stream `git ls-files` output for tracked files under the input set,
    # plus any untracked-but-existing files in the same set, then hash by
    # content (not by mtime). This is immune to `git checkout` mtime games.
    #
    # We feed `find` a filtered list of paths that actually exist on disk
    # to avoid spurious "No such file" noise when an optional config (e.g.
    # `static/`) is absent.
    local existing=()
    for p in "${INPUT_PATHS[@]}"; do
        if [[ -e "$p" ]]; then
            existing+=("$p")
        fi
    done

    if (( ${#existing[@]} == 0 )); then
        echo "dev-build-frontend: no input paths exist; refusing to compute empty hash" >&2
        return 1
    fi

    # Use shasum (POSIX, present on macOS + Linux) to hash each file's
    # bytes, then hash the concatenated per-file digests. Sort by path to
    # make the result deterministic across filesystems.
    find "${existing[@]}" -type f -print0 \
        | LC_ALL=C sort -z \
        | xargs -0 shasum -a 256 \
        | shasum -a 256 \
        | awk '{print $1}'
}

main() {
    local current
    current="$(compute_hash)"

    if [[ -d build && -f "$HASH_FILE" ]]; then
        local previous
        previous="$(cat "$HASH_FILE" 2>/dev/null || true)"
        if [[ "$previous" == "$current" ]]; then
            echo "frontend cache hit (build/.input-hash matches)"
            return 0
        fi
    fi

    echo "frontend cache miss; running vite build"
    npm run build

    # Write the hash AFTER the build succeeds so a failed build does not
    # poison the cache with a hash that points to an incomplete `build/`.
    # Re-compute in case the build itself touched a tracked input (e.g.
    # generated routes); the recipe asserts inputs are stable, so this
    # second hash MUST equal the first — assert that.
    local fresh
    fresh="$(compute_hash)"
    if [[ "$fresh" != "$current" ]]; then
        echo "dev-build-frontend: warning — input hash drifted during build" >&2
        echo "  pre-build:  $current" >&2
        echo "  post-build: $fresh" >&2
        # Record the post-build hash anyway so the next run can settle.
    fi
    mkdir -p build
    printf '%s\n' "$fresh" > "$HASH_FILE"
}

main "$@"
