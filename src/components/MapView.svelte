<script lang="ts">
  import { onMount } from "svelte";
  import maplibregl from "maplibre-gl";
  import "maplibre-gl/dist/maplibre-gl.css";
  import { appState, activeMap, editModeActive, selectedPointId, selectedTrack } from "../lib/stores";
  import {
    deleteTrackPoint,
    getTrackDetail,
    getTracksGeojson,
    getOziMetadata,
    insertTrackPoint,
    moveTrackPoint,
  } from "../lib/api";
  import type { PointDetail, SegmentDetail, TrackDetail } from "../lib/types";
  import { registerSqliteProtocol } from "../lib/maplibre/sqlite-protocol";
  import { registerOziProtocol } from "../lib/maplibre/ozi-protocol";
  import { initTracksLayer, updateTracksLayer } from "../lib/maplibre/tracks-layer";

  let mapEl: HTMLDivElement;
  let map: maplibregl.Map;
  let currentMapSourceId: string | null = null;
  let appliedMapPath: string | null = null;
  let pointMarkers: maplibregl.Marker[] = [];
  let markerElements = new Map<number, HTMLDivElement>();

  type PointMenuTarget = {
    layerId: bigint;
    trackId: bigint;
    segment: SegmentDetail;
    point: PointDetail;
    pointIndex: number;
  };

  let contextMenu = $state<{ x: number; y: number; target: PointMenuTarget } | null>(null);

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

    if (e.key === "Escape") {
      contextMenu = null;
    }
  }

  function clearPointMarkers() {
    for (const marker of pointMarkers) {
      marker.remove();
    }
    pointMarkers = [];
    markerElements.clear();
  }

  function applyEditModeMapInteraction(active: boolean) {
    if (!map) return;
    if (active) {
      map.dragPan.disable();
      map.getCanvas().style.cursor = "crosshair";
      return;
    }

    map.dragPan.enable();
    map.getCanvas().style.cursor = "";
  }

  async function refreshTrackGeometry() {
    if (!map || !map.isStyleLoaded()) return;
    const geojson = await getTracksGeojson();
    updateTracksLayer(map, geojson);
  }

  function openContextMenu(event: MouseEvent, target: PointMenuTarget) {
    event.preventDefault();
    event.stopPropagation();
    const rect = mapEl.getBoundingClientRect();
    contextMenu = {
      x: event.clientX - rect.left,
      y: event.clientY - rect.top,
      target,
    };
  }

  function updateSelectedPointMarkerState() {
    const selectedId = $selectedPointId;
    for (const [id, element] of markerElements.entries()) {
      element.classList.toggle("selected", selectedId === BigInt(id));
    }
  }

  function createPointMarker(
    layerId: bigint,
    trackId: bigint,
    segment: SegmentDetail,
    point: PointDetail,
    pointIndex: number
  ) {
    const pointElement = document.createElement("div");
    pointElement.className = "track-point-marker";
    markerElements.set(point.id, pointElement);

    pointElement.addEventListener("contextmenu", (event) => {
      openContextMenu(event, { layerId, trackId, segment, point, pointIndex });
    });

    pointElement.addEventListener("click", () => {
      selectedPointId.set(BigInt(point.id));
      contextMenu = null;
    });

    const marker = new maplibregl.Marker({ element: pointElement, draggable: true })
      .setLngLat([point.lon, point.lat])
      .addTo(map);

    marker.on("dragstart", () => {
      contextMenu = null;
    });

    marker.on("dragend", async () => {
      const lngLat = marker.getLngLat();
      try {
        await moveTrackPoint(
          layerId,
          trackId,
          BigInt(segment.id),
          BigInt(point.id),
          [lngLat.lat, lngLat.lng]
        );
        await reloadEditableTrackPoints(layerId, trackId);
      } catch (error) {
        console.error("Failed to move track point", error);
      }
    });

    pointMarkers.push(marker);
  }

  function renderEditableTrackPoints(layerId: bigint, trackId: bigint, detail: TrackDetail) {
    clearPointMarkers();
    for (const segment of detail.segments) {
      segment.points.forEach((point, index) => {
        createPointMarker(layerId, trackId, segment, point, index);
      });
    }
    updateSelectedPointMarkerState();
  }

  async function reloadEditableTrackPoints(layerId: bigint, trackId: bigint) {
    if (!map || !$editModeActive) return;
    const detail = await getTrackDetail(layerId, trackId);
    renderEditableTrackPoints(layerId, trackId, detail);
    await refreshTrackGeometry();
  }

  async function handleDeletePoint() {
    if (!contextMenu) return;
    const { layerId, trackId, segment, point } = contextMenu.target;
    contextMenu = null;
    try {
      await deleteTrackPoint(layerId, trackId, BigInt(segment.id), BigInt(point.id));
      await reloadEditableTrackPoints(layerId, trackId);
    } catch (error) {
      console.error("Failed to delete track point", error);
    }
  }

  async function handleInsertPointAfter() {
    if (!contextMenu) return;
    const { layerId, trackId, segment, pointIndex, point } = contextMenu.target;
    contextMenu = null;

    try {
      await insertTrackPoint(
        layerId,
        trackId,
        BigInt(segment.id),
        pointIndex + 1,
        [point.lat, point.lon]
      );
      await reloadEditableTrackPoints(layerId, trackId);
    } catch (error) {
      console.error("Failed to insert track point", error);
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
      map.addSource("osm", {
        type: "raster",
        tiles: ["https://tile.openstreetmap.org/{z}/{x}/{y}.png"],
        tileSize: 256,
        maxzoom: 19,
        attribution: "© <a href='https://www.openstreetmap.org/copyright'>OpenStreetMap</a> contributors",
      });
      map.addLayer({ id: "osm-tiles", type: "raster", source: "osm" });

      initTracksLayer(map);
    });

    map.on("click", () => {
      contextMenu = null;
    });

    return () => {
      window.removeEventListener("keydown", handleKeydown);
      stopFpsCounter();
      clearPointMarkers();
      map.remove();
    };
  });

  // When active map changes, update the raster tile source
  $effect(() => {
    const am = $activeMap;
    if (!am || !map) return;
    if (am.local_path === appliedMapPath) return;

    async function applyActiveMap() {
      appliedMapPath = am.local_path;

      // Remove old map source/layer
      if (currentMapSourceId) {
        if (map.getLayer("map-tiles")) map.removeLayer("map-tiles");
        if (map.getSource(currentMapSourceId)) map.removeSource(currentMapSourceId);
      }

      const sourceId = "active-map";
      currentMapSourceId = sourceId;

      let fitBoundsTarget: [number, number, number, number] | null = null;

      if (am.kind === "ozi") {
        const meta = await getOziMetadata(am.local_path);
        const sourceSpec: maplibregl.RasterSourceSpecification = {
          type: "raster",
          tiles: [`ozi://${am.local_path}/{z}/{x}/{y}`],
          tileSize: 256,
          // maxzoom enables overzoom (pixelated) when zooming past the map's native resolution.
          // minzoom prevents requesting tiles when zoomed out too far.
          maxzoom: meta.native_zoom,
          minzoom: meta.min_zoom,
        };
        if (meta.bounds) {
          sourceSpec.bounds = meta.bounds;
          fitBoundsTarget = meta.bounds;
        }
        map.addSource(sourceId, sourceSpec);
      } else {
        map.addSource(sourceId, {
          type: "raster",
          tiles: [`sqlite://${am.local_path}/${am.base_zoom}/{z}/{x}/{y}`],
          tileSize: 256,
        });
      }

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

      if (fitBoundsTarget) {
        map.fitBounds(fitBoundsTarget, { padding: 0, animate: true });
      } else if (am.center_lat !== 0 || am.center_lon !== 0) {
        map.flyTo({
          center: [am.center_lon, am.center_lat],
          zoom: am.base_zoom || 12,
        });
      }
    }

    if (!map.isStyleLoaded()) {
      map.once("load", applyActiveMap);
    } else {
      applyActiveMap();
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

  $effect(() => {
    if (!map) return;
    applyEditModeMapInteraction($editModeActive);

    if (!$editModeActive || !$selectedTrack) {
      clearPointMarkers();
      contextMenu = null;
      return;
    }

    const active = $appState;
    void active;

    reloadEditableTrackPoints($selectedTrack.layerId, $selectedTrack.trackId)
      .catch((error) => console.error("Failed to load editable track points", error));
  });

  $effect(() => {
    updateSelectedPointMarkerState();
  });
</script>

<div class="map-container" bind:this={mapEl}>
  {#if contextMenu}
    <div class="point-context-menu" style={`left:${contextMenu.x}px; top:${contextMenu.y}px;`}>
      <button onclick={handleDeletePoint}>Delete Point</button>
      <button onclick={handleInsertPointAfter}>Insert Point After</button>
    </div>
  {/if}
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

  .track-point-marker {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    background: var(--ctp-lavender);
    border: 2px solid var(--ctp-base);
    box-shadow: 0 0 0 1px var(--ctp-blue);
    cursor: grab;
  }

  .track-point-marker.selected {
    background: var(--ctp-yellow);
    box-shadow: 0 0 0 1px var(--ctp-peach);
  }

  .track-point-marker:active {
    cursor: grabbing;
  }

  .point-context-menu {
    position: absolute;
    z-index: 40;
    min-width: 160px;
    padding: 4px;
    border-radius: 6px;
    border: 1px solid var(--ctp-surface1);
    background: var(--ctp-mantle);
    box-shadow: 0 8px 20px rgba(0, 0, 0, 0.35);
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .point-context-menu button {
    border: none;
    background: transparent;
    color: var(--ctp-text);
    text-align: left;
    border-radius: 4px;
    padding: 6px 8px;
    font-size: 12px;
    cursor: pointer;
  }

  .point-context-menu button:hover {
    background: var(--ctp-surface0);
  }
</style>
