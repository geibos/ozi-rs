<script lang="ts">
  import { onMount } from "svelte";
  import maplibregl from "maplibre-gl";
  import "maplibre-gl/dist/maplibre-gl.css";
  import {
    appState,
    activeMap,
    editModeActive,
    selectedPointId,
    selectedTrack,
    addWaypointMode,
    activeWaypointLayerId,
    drawingModeActive,
    drawingTrackLayerId,
    drawingTrackId,
    drawingPointCount,
    drawingFinishRequested,
    drawingSegmentId,
    simplifyState,
  } from "../lib/stores";
  import {
    deleteTrackPoint,
    getTrackDetail,
    getTracksGeojson,
    getOziMetadata,
    insertTrackPoint,
    moveTrackPoint,
    moveWaypoint,
    addWaypoint,
    getWaypoints,
    undo,
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
  let waypointMarkers: maplibregl.Marker[] = [];
  let drawingPreviewPoints: Array<{ lat: number; lon: number }> = [];
  let drawingCommandCount = 0;
  let pendingDrawingClickTimeout: number | null = null;

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
      if ($drawingModeActive) {
        e.preventDefault();
        void cancelDrawingMode();
      }
      if ($addWaypointMode) {
        addWaypointMode.set(false);
      }
    }

    if (e.key === "Enter" && $drawingModeActive) {
      e.preventDefault();
      void finishDrawingMode();
    }
  }

  async function finishDrawingMode() {
    if (!$drawingModeActive) return;
    drawingModeActive.set(false);
    drawingTrackLayerId.set(null);
    drawingTrackId.set(null);
    drawingSegmentId.set(null);
    drawingPointCount.set(0);
    if (pendingDrawingClickTimeout !== null) {
      window.clearTimeout(pendingDrawingClickTimeout);
      pendingDrawingClickTimeout = null;
    }
    await refreshTrackGeometry();
  }

  async function cancelDrawingMode() {
    if (!$drawingModeActive || $drawingTrackId === null) return;
    const undoCount = drawingCommandCount + 1; // +1 for CreateEmptyTrack

    if (pendingDrawingClickTimeout !== null) {
      window.clearTimeout(pendingDrawingClickTimeout);
      pendingDrawingClickTimeout = null;
    }

    try {
      for (let i = 0; i < undoCount; i += 1) {
        await undo();
      }
    } catch (error) {
      console.error("Failed to cancel drawing mode", error);
    } finally {
      drawingModeActive.set(false);
      drawingTrackLayerId.set(null);
      drawingTrackId.set(null);
      drawingSegmentId.set(null);
      drawingPointCount.set(0);
      await refreshTrackGeometry();
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

  function clearWaypointMarkers() {
    for (const m of waypointMarkers) {
      m.remove();
    }
    waypointMarkers = [];
  }

  async function refreshWaypointMarkers() {
    if (!map) return;
    clearWaypointMarkers();
    const layerId = $activeWaypointLayerId;
    if (!$appState || $appState.waypoint_layer_count === 0 || layerId === null) return;
    try {
      const waypoints = await getWaypoints(layerId);
      // Render only visible waypoints — hidden ones stay in the project
      // file and panel but are filtered out of the map source.
      for (const wp of waypoints.filter((w) => w.visible !== false)) {
        const el = document.createElement("div");
        el.className = "waypoint-marker";
        el.style.cursor = "grab";
        const wpId = wp.id;
        const marker = new maplibregl.Marker({ element: el, draggable: true })
          .setLngLat([wp.lon, wp.lat])
          .setPopup(new maplibregl.Popup({ offset: 16 }).setText(wp.name))
          .addTo(map);

        marker.on("dragstart", () => {
          el.style.cursor = "grabbing";
        });

        marker.on("dragend", async () => {
          el.style.cursor = "grab";
          const lngLat = marker.getLngLat();
          try {
            await moveWaypoint(layerId, BigInt(wpId), [lngLat.lat, lngLat.lng]);
          } catch (error) {
            console.error("Failed to move waypoint", error);
            marker.setLngLat([wp.lon, wp.lat]);
          }
        });

        waypointMarkers.push(marker);
      }
    } catch {
      // layer may not exist yet
    }
  }

  async function handleMapClickForWaypoint(e: maplibregl.MapMouseEvent) {
    if ($drawingModeActive) return;
    if (!$addWaypointMode) return;
    const layerId = $activeWaypointLayerId;
    if (layerId === null) return;
    const { lat, lng } = e.lngLat;
    try {
      const currentWaypoints = await getWaypoints(layerId);
      const nextIndex = currentWaypoints.length + 1;
      await addWaypoint(layerId, lat, lng, `Waypoint ${nextIndex}`);
      await refreshWaypointMarkers();
    } catch (error) {
      console.error("Failed to add waypoint:", error);
    } finally {
      addWaypointMode.set(false);
    }
  }

  function clearDrawingPreview() {
    if (!map) return;
    try {
      if (map.getLayer("drawing-preview-line")) {
        map.removeLayer("drawing-preview-line");
      }
      if (map.getLayer("drawing-preview-points")) {
        map.removeLayer("drawing-preview-points");
      }
      if (map.getSource("drawing-preview")) {
        map.removeSource("drawing-preview");
      }
    } catch {
      // source/layer may not exist
    }
  }

  function updateDrawingPreview() {
    if (!map || !map.isStyleLoaded()) return;

    const lineFeature: GeoJSON.Feature<GeoJSON.LineString> = {
      type: "Feature",
      properties: {},
      geometry: {
        type: "LineString",
        coordinates: drawingPreviewPoints.map((pt) => [pt.lon, pt.lat]),
      },
    };

    const pointsFeature: GeoJSON.Feature<GeoJSON.MultiPoint> = {
      type: "Feature",
      properties: {},
      geometry: {
        type: "MultiPoint",
        coordinates: drawingPreviewPoints.map((pt) => [pt.lon, pt.lat]),
      },
    };

    const previewGeojson: GeoJSON.FeatureCollection = {
      type: "FeatureCollection",
      features: [lineFeature, pointsFeature],
    };

    if (!map.getSource("drawing-preview")) {
      map.addSource("drawing-preview", {
        type: "geojson",
        data: previewGeojson,
      });

      map.addLayer({
        id: "drawing-preview-line",
        type: "line",
        source: "drawing-preview",
        filter: ["==", ["geometry-type"], "LineString"],
        layout: {
          "line-join": "round",
          "line-cap": "round",
        },
        paint: {
          "line-color": "#0066ff",
          "line-width": 3,
        },
      });

      map.addLayer({
        id: "drawing-preview-points",
        type: "circle",
        source: "drawing-preview",
        filter: ["==", ["geometry-type"], "Point"],
        paint: {
          "circle-radius": 4,
          "circle-color": "#0066ff",
          "circle-stroke-width": 1,
          "circle-stroke-color": "#ffffff",
        },
      });
      return;
    }

    const source = map.getSource("drawing-preview");
    if (source && source.type === "geojson") {
      source.setData(previewGeojson);
    }
  }

  function handleMapClickForDrawing(e: maplibregl.MapMouseEvent) {
    if (
      !$drawingModeActive
      || $drawingTrackLayerId === null
      || $drawingTrackId === null
      || $drawingSegmentId === null
    ) return;
    const layerId = $drawingTrackLayerId;
    const trackId = $drawingTrackId;
    const segmentId = $drawingSegmentId;

    if (pendingDrawingClickTimeout !== null) {
      window.clearTimeout(pendingDrawingClickTimeout);
      pendingDrawingClickTimeout = null;
    }

    const { lat, lng } = e.lngLat;
    pendingDrawingClickTimeout = window.setTimeout(async () => {
      pendingDrawingClickTimeout = null;
      try {
        await insertTrackPoint(layerId, trackId, segmentId, drawingPreviewPoints.length, [lat, lng]);
        drawingCommandCount += 1;
        drawingPreviewPoints = [...drawingPreviewPoints, { lat, lon: lng }];
        drawingPointCount.set(drawingPreviewPoints.length);
        updateDrawingPreview();
      } catch (error) {
        console.error("Failed to add drawing point", error);
      }
    }, 220);
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

    map.on("load", async () => {
      map.addSource("osm", {
        type: "raster",
        tiles: ["https://tile.openstreetmap.org/{z}/{x}/{y}.png"],
        tileSize: 256,
        maxzoom: 19,
        attribution: "© <a href='https://www.openstreetmap.org/copyright'>OpenStreetMap</a> contributors",
      });
      map.addLayer({ id: "osm-tiles", type: "raster", source: "osm" });

      initTracksLayer(map);

      // Explicitly refresh tracks and waypoints once map is ready,
      // since $effect may have run before map was initialized
      try {
        const geojson = await getTracksGeojson();
        updateTracksLayer(map, geojson);
      } catch {
        // state may not be ready yet
      }
      refreshWaypointMarkers();
    });

    map.on("click", (e) => {
      contextMenu = null;
      handleMapClickForWaypoint(e);
      handleMapClickForDrawing(e);
    });

    map.on("dblclick", (e) => {
      if (!$drawingModeActive) return;
      e.preventDefault();
      if (pendingDrawingClickTimeout !== null) {
        window.clearTimeout(pendingDrawingClickTimeout);
        pendingDrawingClickTimeout = null;
      }
      void finishDrawingMode();
    });

    return () => {
      window.removeEventListener("keydown", handleKeydown);
      stopFpsCounter();
      clearPointMarkers();
      clearWaypointMarkers();
      clearDrawingPreview();
      if (pendingDrawingClickTimeout !== null) {
        window.clearTimeout(pendingDrawingClickTimeout);
        pendingDrawingClickTimeout = null;
      }
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
    void $activeWaypointLayerId;

    // Run after map is loaded
    if (!map.isStyleLoaded()) {
      map.once("load", async () => {
        const geojson = await getTracksGeojson();
        updateTracksLayer(map, geojson);
        refreshWaypointMarkers();
      });
      return;
    }

    getTracksGeojson().then((geojson) => updateTracksLayer(map, geojson));
    refreshWaypointMarkers();
  });

  $effect(() => {
    if (!map) return;
    if ($drawingModeActive) {
      if ($editModeActive) {
        editModeActive.set(false);
      }
      if ($addWaypointMode) {
        addWaypointMode.set(false);
      }
      contextMenu = null;
    }
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

  $effect(() => {
    if (!map) return;
    const canvas = map.getCanvas();
    if ($drawingModeActive) {
      canvas.style.cursor = "crosshair";
    } else if ($addWaypointMode) {
      canvas.style.cursor = "crosshair";
    } else if (!$editModeActive) {
      canvas.style.cursor = "";
    }
  });

  $effect(() => {
    if (!map) return;

    const active = $drawingModeActive;
    void $drawingTrackLayerId;
    void $drawingTrackId;

    if (active) {
      map.dragPan.disable();
      map.doubleClickZoom.disable();
      drawingPreviewPoints = [];
      drawingCommandCount = 0;
      drawingPointCount.set(0);
      updateDrawingPreview();
      return;
    }

    if (!$editModeActive) {
      map.dragPan.enable();
    }
    map.doubleClickZoom.enable();
    drawingPreviewPoints = [];
    drawingCommandCount = 0;
    drawingPointCount.set(0);
    clearDrawingPreview();
  });

  $effect(() => {
    if ($drawingFinishRequested) {
      drawingFinishRequested.set(false);
      void finishDrawingMode();
    }
  });

  $effect(() => {
    if (!map) return;

    const state = $simplifyState;

    function updateSimplifyPreview() {
      if (!map) return;
      if (!state.active || !state.preview) {
        if (map.getLayer("simplify-preview-line")) {
          map.removeLayer("simplify-preview-line");
        }
        if (map.getSource("simplify-preview")) {
          map.removeSource("simplify-preview");
        }
        return;
      }

      const { preview } = state;
      const geojson: GeoJSON.FeatureCollection<GeoJSON.LineString> = {
        type: "FeatureCollection",
        features: preview.segments.map((seg) => ({
          type: "Feature",
          properties: {},
          geometry: {
            type: "LineString",
            coordinates: seg.kept_points.map((pt) => [pt.lon, pt.lat]),
          },
        })),
      };

      if (!map.getSource("simplify-preview")) {
        map.addSource("simplify-preview", {
          type: "geojson",
          data: geojson,
        });
        map.addLayer({
          id: "simplify-preview-line",
          type: "line",
          source: "simplify-preview",
          layout: {
            "line-join": "round",
            "line-cap": "round",
          },
          paint: {
            "line-color": "#ff6600",
            "line-width": 3,
          },
        });
      } else {
        const source = map.getSource("simplify-preview");
        if (source && source.type === "geojson") {
          source.setData(geojson);
        }
      }
    }

    if (!map.isStyleLoaded()) {
      map.once("load", updateSimplifyPreview);
    } else {
      updateSimplifyPreview();
    }
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
  {#if $drawingModeActive}
    <div class="drawing-badge">
      Drawing track · {$drawingPointCount} {$drawingPointCount === 1 ? "point" : "points"}
    </div>
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

  .drawing-badge {
    position: absolute;
    top: 8px;
    left: 8px;
    z-index: 10;
    background: rgba(0, 102, 255, 0.2);
    color: #dbe9ff;
    border: 1px solid #0066ff;
    border-radius: 4px;
    font-size: 12px;
    padding: 4px 8px;
    pointer-events: none;
  }

  :global(.track-point-marker) {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    background: var(--ctp-lavender);
    border: 2px solid var(--ctp-base);
    box-shadow: 0 0 0 1px var(--ctp-blue);
    cursor: grab;
  }

  :global(.track-point-marker.selected) {
    background: var(--ctp-yellow);
    box-shadow: 0 0 0 1px var(--ctp-peach);
  }

  :global(.track-point-marker:active) {
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

  :global(.waypoint-marker) {
    width: 14px;
    height: 14px;
    background: var(--ctp-yellow, #e5c890);
    border: 2px solid var(--ctp-crust, #232634);
    border-radius: 50%;
    cursor: grab;
  }

  :global(.waypoint-marker:active) {
    cursor: grabbing;
  }
</style>
