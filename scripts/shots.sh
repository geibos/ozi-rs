#!/usr/bin/env bash
# The screenshot matrix, run where its pixels are reproducible.
#
# Inside the Playwright image, on any machine: font rendering on a Mac and on a
# Linux CI runner differ, and a baseline that only matches on one of them is a
# baseline nobody else can update. The repository is mounted; node_modules is
# a volume of its own, because the host's are built for the host (esbuild,
# rollup) and must not be overwritten by Linux ones. The stand runs inside the
# container too, so nothing depends on the host's network.
#
#   scripts/shots.sh [--compare|--update|--render] [--screen <id>] [--slice <name>]
set -euo pipefail

IMAGE="mcr.microsoft.com/playwright:v1.63.0-noble"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"

exec docker run --rm --init --ipc=host \
  -v "$ROOT":/work \
  -v ozi-rs-shots-node-modules:/work/node_modules \
  -w /work \
  -e CI="${CI:-}" \
  "$IMAGE" \
  bash -c '
    set -euo pipefail
    # npm ci empties node_modules every time; the lockfile stamp skips it when
    # nothing changed since the last run.
    if ! cmp -s package-lock.json node_modules/.ozi-lock-stamp; then
      npm ci --no-audit --no-fund --loglevel=error
      cp package-lock.json node_modules/.ozi-lock-stamp
    fi
    npx vite --config vite.stand.config.ts --host 127.0.0.1 --port 5273 --strictPort \
      > /tmp/stand.log 2>&1 &
    stand=$!
    trap "kill $stand 2>/dev/null || true" EXIT
    for _ in $(seq 1 120); do
      if curl -fs http://127.0.0.1:5273/ > /dev/null; then break; fi
      sleep 0.5
    done
    curl -fs http://127.0.0.1:5273/ > /dev/null || { cat /tmp/stand.log; exit 2; }
    node --experimental-strip-types src/test/stand/shots.ts "$@"
  ' shots "$@"
