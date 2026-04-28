# Task 2 chunk-size rationale

- Source warning: Task 1 captured Vite's chunk-size advisory for generated `App-BwVPn97T.js` at 841.95 kB after minification.
- Fix applied: `vite.config.ts` now uses targeted Rollup `manualChunks` for `node_modules/maplibre-gl`, splitting the core map engine out of the application chunk.
- Evidence after split: `npm run build` produced `App-BO4NQSKQ.js` at 36.78 kB and a separate `maplibre-BDOosUHv.js` vendor chunk at 802.88 kB.
- Rationale for `chunkSizeWarningLimit: 850`: the remaining oversized payload is the intentionally included MapLibre engine required by the core map view. Further app-level code splitting would not reduce that required vendor payload, so the limit is set just above the measured vendor chunk while keeping future chunks above that size visible.
