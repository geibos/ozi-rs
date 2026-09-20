#!/usr/bin/env node
/**
 * npm audit gate for production dependencies.
 *
 * `npm audit --audit-level=high` has no ignore mechanism, so a single
 * transitive advisory with no non-breaking fix leaves CI permanently red and
 * the gate stops meaning anything. The Rust side already solved this in
 * `.cargo/audit.toml`: advisories that cannot be fixed right now are listed
 * with a reason and a recheck condition. This script gives the npm side the
 * same contract.
 *
 * Rules:
 *   - high and critical advisories fail the build;
 *   - an advisory listed in ALLOWED is reported but does not fail;
 *   - an ALLOWED entry that no longer appears fails the build too, so stale
 *     exemptions cannot outlive the vulnerability they excuse.
 *
 * Every ALLOWED entry MUST say why the advisory cannot be addressed now and
 * what should trigger a recheck.
 */
import { execFileSync } from "node:child_process";

const FAIL_LEVELS = new Set(["high", "critical"]);

const ALLOWED = [
  {
    id: "GHSA-jrc7-96c5-q579",
    package: "maplibre-gl",
    reason:
      "XSS sanitizer bypass in DOM.sanitize(), reachable through setHTML() " +
      "and custom attribution HTML. ozi-rs renders map popups with " +
      "Popup.setText() (MapView.svelte) and uses no innerHTML anywhere, so " +
      "the affected sink is never called. The advisory is only fixed in " +
      "maplibre-gl 6.10, two major versions ahead of the pinned 4.x, and a " +
      "major map-engine upgrade needs its own slice with visual verification.",
    recheck:
      "When the MapLibre 6 upgrade slice lands, or immediately if any code " +
      "starts calling Popup.setHTML(), Marker with custom HTML, or custom " +
      "attribution strings.",
  },
];

function runAudit() {
  try {
    return execFileSync("npm", ["audit", "--omit=dev", "--json"], {
      encoding: "utf8",
      maxBuffer: 32 * 1024 * 1024,
    });
  } catch (error) {
    // npm exits non-zero whenever advisories exist; the JSON is still on stdout.
    if (typeof error.stdout === "string" && error.stdout.length > 0) {
      return error.stdout;
    }
    throw error;
  }
}

const report = JSON.parse(runAudit());
const found = new Map();
for (const entry of Object.values(report.vulnerabilities ?? {})) {
  for (const via of entry.via ?? []) {
    if (typeof via !== "object" || !via.url) continue;
    const id = via.url.split("/").pop();
    if (!FAIL_LEVELS.has(via.severity)) continue;
    found.set(id, { id, severity: via.severity, package: via.name, title: via.title });
  }
}

const allowedIds = new Set(ALLOWED.map((a) => a.id));
const blocking = [...found.values()].filter((a) => !allowedIds.has(a.id));
const waived = [...found.values()].filter((a) => allowedIds.has(a.id));
const stale = ALLOWED.filter((a) => !found.has(a.id));

for (const a of waived) {
  console.log(`waived  ${a.severity.padEnd(8)} ${a.package} ${a.id} — ${a.title}`);
}
for (const a of blocking) {
  console.error(`BLOCKING ${a.severity.padEnd(8)} ${a.package} ${a.id} — ${a.title}`);
}
for (const a of stale) {
  console.error(
    `STALE EXEMPTION ${a.id} (${a.package}) no longer reported — remove it from scripts/npm-audit-gate.mjs`,
  );
}

if (blocking.length > 0 || stale.length > 0) {
  console.error(
    `\nnpm audit gate failed: ${blocking.length} blocking advisory/ies, ${stale.length} stale exemption(s).`,
  );
  process.exit(1);
}

console.log(
  `npm audit gate passed: 0 blocking high/critical advisories, ${waived.length} waived.`,
);
