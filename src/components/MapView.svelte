<script lang="ts">
  import { onMount } from "svelte";
  import maplibregl from "maplibre-gl";
  import "maplibre-gl/dist/maplibre-gl.css";
  import { appState, activeMap } from "../lib/stores";
  import { getTracksGeojson } from "../lib/api";
  import { registerSqliteProtocol } from "../lib/maplibre/sqlite-protocol";
  import { registerOziProtocol } from "../lib/maplibre/ozi-protocol";
  import { initTracksLayer, updateTracksLayer } from "../lib/maplibre/tracks-layer";

  let mapEl: HTMLDivElement;
  let map: maplibregl.Map;
  let currentMapSourceId: string | null = null;

  // FPS counter (toggle with F3)
  let fpsVisible = $state(false);
  let fps = $state(0);
  let rafId: number;

  function startFpsCounter() {
    let frames = 0;
    let last = performance.now();

    function tick() {
      frames++;
      const now = performance.now();
      if (now - last >= 1000) {
        fps = Math.round(frames * 1000 / (now - last));
        frames = 0;
        last = now;
      }
      rafId = requestAnimationFrame(tick);
    }

    rafId = requestAnimationFrame(tick);
  }

  function stopFpsCounter() {
    cancelAnimationFrame(rafId);
    fps = 0;
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "F3") {
      e.preventDefault();
      fpsVisible = !fpsVisible;
      fpsVisible ? startFpsCounter() : stopFpsCounter();
    }
  }

  onMount(() => {
    registerSqliteProtocol();
    registerOziProtocol();

    window.addEventListener("keydown", handleKeydown);

    map = new maplibregl.Map({
      container: mapEl,
      style: {
        version: 8,
        sources: {},
        layers: [],
        glyphs: "https://demotiles.maplibre.org/font/{fontstack}/{range}.pbf",
      },
      center: [37.6, 55.75], // Moscow as default
      zoom: 5,
    });

    map.addControl(new maplibregl.NavigationControl(), "top-left");
    map.addControl(new maplibregl.ScaleControl(), "bottom-left");

    map.on("load", () => {
      initTracksLayer(map);
    });

    return () => {
      window.removeEventListener("keydown", handleKeydown);
      stopFpsCounter();
      map.remove();
    };
  });

  // When active map changes, update the raster tile source
  $effect(() => {
    if (!map || !map.isStyleLoaded()) return;

    const am = $activeMap;
    if (!am) return;

    // Remove old map source/layer
    if (currentMapSourceId) {
      if (map.getLayer("map-tiles")) map.removeLayer("map-tiles");
      if (map.getSource(currentMapSourceId)) map.removeSource(currentMapSourceId);
    }

    const sourceId = "active-map";
    currentMapSourceId = sourceId;

    const tileUrl =
      am.kind === "sqlite"
        ? `sqlite://${am.local_path}/${am.base_zoom}/{z}/{x}/{y}`
        : `ozi://${am.local_path}/{z}/{x}/{y}`;

    map.addSource(sourceId, {
      type: "raster",
      tiles: [tileUrl],
      tileSize: 256,
    });

    // Insert below tracks layer
    const tracksLayerId = map.getLayer("tracks-lines") ? "tracks-lines" : undefined;

    map.addLayer(
      {
        id: "map-tiles",
        type: "raster",
        source: sourceId,
        paint: { "raster-opacity": 1 },
      },
      tracksLayerId
    );

    if (am.center_lat !== 0 || am.center_lon !== 0) {
      map.flyTo({
        center: [am.center_lon, am.center_lat],
        zoom: am.base_zoom || 12,
      });
    }
  });

  // When app state changes (tracks added, etc.) refresh tracks GeoJSON
  $effect(() => {
    if (!map || !$appState) return;

    // Run after map is loaded
    if (!map.isStyleLoaded()) {
      map.once("load", async () => {
        const geojson = await getTracksGeojson();
        updateTracksLayer(map, geojson);
      });
      return;
    }

    getTracksGeojson().then((geojson) => updateTracksLayer(map, geojson));
  });
</script>

<div class="map-container" bind:this={mapEl}>
  {#if fpsVisible}
    <div class="fps-badge">{fps} fps</div>
  {/if}
</div>

<style>
  .map-container {
    flex: 1;
    height: 100%;
    min-width: 0;
    position: relative;
  }

  .fps-badge {
    position: absolute;
    top: 8px;
    right: 8px;
    z-index: 10;
    background: rgba(0, 0, 0, 0.55);
    color: #00ff88;
    font-family: monospace;
    font-size: 12px;
    padding: 2px 7px;
    border-radius: 3px;
    pointer-events: none;
    font-variant-numeric: tabular-nums;
  }
</style>
