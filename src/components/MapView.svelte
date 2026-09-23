<script lang="ts">
  // @ts-nocheck
  // MapLibre integration kept under @ts-nocheck because the wrapper-only
  // migration in `migrate-panels-to-shadcn` (section 10) explicitly forbids
  // touching MapLibre internals, source/layer setup, drag handlers, click
  // handlers, or the tile-protocol code — tightening types there is a
  // follow-up change.
  import { onMount } from "svelte";
  import { get } from "svelte/store";
  import maplibregl from "maplibre-gl";
  import "maplibre-gl/dist/maplibre-gl.css";
  import {
    appState,
    activeMap,
    activeMapRef,
    editModeActive,
    selectedPointId,
    selectedTrack,
    addWaypointMode,
    activeWaypointLayerId,
    drawingModeActive,
    measuringActive,
    measuredPoints,
    graticuleVisible,
    replayPosition,
    setMeasuring,
    ringActive,
    ringCentre,
    ringRadiusKm,
    setRing,
    projectionActive,
    projectionOrigin,
    projectionBearing,
    projectionDistanceM,
    setProjection,
    drawingTrackLayerId,
    drawingTrackId,
    drawingPointCount,
    drawingFinishRequested,
    drawingSegmentId,
    mapFocusRequest,
    mapViewportBounds,
    simplifyState,
    tracksFingerprint,
    tracksGeometryVersion,
    visibleWaypointLayers,
    waypointsFingerprint,
    measureMode,
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
    cancelDrawing,
  } from "../lib/api";
  import type {
    OziMetadataDto,
    PointDetail,
    SegmentDetail,
    TrackDetail,
  } from "../lib/types";
  import { locale, t as i18n } from "../lib/i18n";
  import { toast } from "svelte-sonner";
  import { registerSqliteProtocol } from "../lib/maplibre/sqlite-protocol";
  import { registerOziProtocol } from "../lib/maplibre/ozi-protocol";
  import { createLatestRun } from "$lib/latest-run";
  import { mayReachNetworkNow } from "$lib/network-reach";
  import { reportEditFailure } from "$lib/edit-failure";
  import {
    boundsOf,
    centreOf,
    focusPositions,
    isDegenerate,
    toLngLatBounds,
  } from "$lib/map-bounds";
  import { waypointColorCss, waypointGlyph } from "$lib/waypoint-symbols";
  import {
    declutter,
    segmentsOf,
    type LabelCandidate,
  } from "$lib/track-labels";
  import {
    destinationPoint,
    distanceKm,
    formatMeasuredDistance,
    pathLengthKm,
    ringAround,
    polygonAreaSqKm,
    formatMeasuredArea,
  } from "$lib/geo";
  import { isEditableTarget } from "$lib/editable-target";
  import {
    initMeasureLayer,
    updateMeasureLayer,
    raiseMeasureLayer,
  } from "$lib/maplibre/measure-layer";
  import { pendingClicks } from "$lib/drawing-clicks";
  import { measuredBetween, snapToWaypoint } from "$lib/measure-snap";
  import {
    attachGraticule,
    type GraticuleHandle,
  } from "$lib/maplibre/graticule-layer";
  import {
    TRACKS_LAYER_SELECTED,
    highlightTrack,
    initTracksLayer,
    updateTracksLayer,
  } from "../lib/maplibre/tracks-layer";

  let mapEl: HTMLDivElement;
  let map: maplibregl.Map;
  let currentMapSourceId: string | null = null;
  let appliedMapPath: string | null = null;
  let pointMarkers: maplibregl.Marker[] = [];
  let markerElements = new Map<number, HTMLDivElement>();
  // Keyed by `${layerId}:${waypointId}` so that toggling a single layer's
  // visibility removes only that layer's markers without disturbing the rest.
  let waypointMarkers = new Map<string, maplibregl.Marker>();
  let drawingPreviewPoints: Array<{ lat: number; lon: number }> = [];
  let drawingCommandCount = 0;
  /**
   * Clicks waiting out the double-click window before they become points.
   *
   * One click used to be held at a time, and a second one inside the window
   * cancelled it — so a route plotted at any normal pace lost most of its
   * points silently. See `$lib/drawing-clicks`.
   */
  /** "ШТАБ → ЗАБРОС", when the tape caught a mark at both ends. */
  const measuredMarks = $derived(measuredBetween($measuredPoints));

  /**
   * Where the crew was at the moment the replay slider is on.
   *
   * A marker rather than a layer: it is one point, it moves on every frame of
   * a playback, and a source rewritten ten times a second is a source that
   * makes the map stutter.
   */
  let replayMarker: maplibregl.Marker | null = null;

  $effect(() => {
    const at = $replayPosition;
    if (!map) return;
    if (at === null) {
      replayMarker?.remove();
      replayMarker = null;
      return;
    }
    if (replayMarker === null) {
      const element = document.createElement("div");
      element.setAttribute("data-testid", "replay-marker");
      element.style.cssText = [
        "width: 16px",
        "height: 16px",
        "border-radius: 50%",
        "border: 2px solid #fff",
        "box-shadow: 0 0 0 1px rgba(0,0,0,0.5)",
        "pointer-events: none",
      ].join(";");
      replayMarker = new maplibregl.Marker({ element }).setLngLat([
        at.lon,
        at.lat,
      ]);
      replayMarker.addTo(map);
    } else {
      replayMarker.setLngLat([at.lon, at.lat]);
    }
    // Amber in a silence: the position there is interpolated across a gap the
    // navigator did not record, and it should not look like a fix.
    const element = replayMarker.getElement();
    element.style.background = at.inGap ? "#f59e0b" : "#2563eb";
  });

  let graticuleHandle: GraticuleHandle | null = null;

  function syncGraticule() {
    if (!map) return;
    const wanted = $graticuleVisible;
    if (wanted && graticuleHandle === null) {
      graticuleHandle = attachGraticule(map);
    } else if (!wanted && graticuleHandle !== null) {
      graticuleHandle.detach();
      graticuleHandle = null;
    }
  }

  const drawingClicks = pendingClicks<{ lat: number; lon: number }>({
    commit: async ({ lat, lon }) => {
      const layerId = $drawingTrackLayerId;
      const trackId = $drawingTrackId;
      const segmentId = $drawingSegmentId;
      if (layerId === null || trackId === null || segmentId === null) return;
      try {
        await insertTrackPoint(
          layerId,
          trackId,
          segmentId,
          drawingPreviewPoints.length,
          [lat, lon],
        );
        drawingCommandCount += 1;
        drawingPreviewPoints = [...drawingPreviewPoints, { lat, lon }];
        drawingPointCount.set(drawingPreviewPoints.length);
        updateDrawingPreview();
      } catch (error) {
        reportEditFailure("map.addDrawingPointFailed", error);
      }
    },
  });

  type PointMenuTarget = {
    layerId: bigint;
    trackId: bigint;
    segment: SegmentDetail;
    point: PointDetail;
    pointIndex: number;
  };

  let contextMenu = $state<{
    x: number;
    y: number;
    target: PointMenuTarget;
  } | null>(null);

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
        fps = Math.round((frames * 1000) / (now - last));
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
      if ($measuringActive) {
        e.preventDefault();
        setMeasuring(false);
      }
      if ($ringActive) {
        e.preventDefault();
        setRing(false);
      }
      if ($projectionActive) {
        e.preventDefault();
        setProjection(false);
      }
    }

    // Misclicks happen, and starting the measurement again because of one is
    // worse than the misclick.
    if (
      (e.key === "Backspace" || e.key === "Delete") &&
      $measuringActive &&
      !isEditableTarget(e.target)
    ) {
      e.preventDefault();
      measuredPoints.update((points) => points.slice(0, -1));
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
    drawingClicks.cancel();
    await refreshTrackGeometry();
  }

  async function cancelDrawingMode() {
    if (!$drawingModeActive || $drawingTrackId === null) return;
    const layerId = $drawingTrackLayerId;
    const trackId = $drawingTrackId;
    if (layerId === null) return;
    // +1 for the command that created the track itself.
    const commandCount = drawingCommandCount + 1;

    drawingClicks.cancel();

    try {
      // One discard rather than a loop of undos: undoing left the abandoned
      // track in the redo stack, where a later redo brought it back, and kept
      // the project marked as changed although nothing had changed.
      // The track is named as well as counted: the undo stack is bounded, so
      // a drawing longer than it has already lost its own creation command
      // and the count alone would leave an empty track behind.
      await cancelDrawing(layerId, trackId, commandCount);
    } catch (error) {
      reportEditFailure("map.cancelDrawingFailed", error);
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
    // Panning stays enabled in edit mode (CJ-4): point markers are
    // draggable maplibregl.Markers whose elements capture their own
    // pointer events, so marker drag never reaches the map's dragPan
    // handler — disabling it only made the map feel broken.
    map.getCanvas().style.cursor = active ? "crosshair" : "";
  }

  async function refreshTrackGeometry() {
    if (!map || !mapLoaded) return;
    const geojson = await getTracksGeojson();
    updateTracksLayer(map, geojson);
    lastTracksGeojson = geojson;
    refreshTrackLabels();
  }

  /**
   * The names on the map.
   *
   * A day of recordings drew in twelve colours with not one name on it. The
   * symbol layer that would have carried them needs SDF glyphs this
   * application does not bundle, and pointing the style at a remote glyphs URL
   * does not merely lose the labels — it stops the *lines* rendering offline.
   * So the names are DOM markers, like the waypoints, and the collision
   * avoidance a symbol layer would have done is in `track-labels.ts`.
   *
   * Kept in step with the camera as well as the data: a label is placed by
   * where it lands on screen, so panning and zooming change which names fit.
   */
  let lastTracksGeojson: GeoJSON.FeatureCollection | null = null;
  const trackLabelMarkers = new Map<string, maplibregl.Marker>();
  /** Below this the whole district is on screen and names are a smear. */
  const LABEL_MIN_ZOOM = 10;

  function labelCandidates(): LabelCandidate[] {
    const features = (lastTracksGeojson?.features ?? []) as Array<{
      geometry?: unknown;
      properties?: Record<string, unknown> | null;
    }>;
    const selected = get(selectedTrack);
    const out: LabelCandidate[] = [];
    for (const feature of features) {
      const properties = feature.properties ?? {};
      if (properties.visible === false) continue;
      const name = typeof properties.name === "string" ? properties.name : "";
      if (name.trim().length === 0) continue;
      const layerId = String(properties.layer_id ?? "");
      const trackId = String(properties.track_id ?? "");
      out.push({
        key: `${layerId}:${trackId}`,
        name,
        color:
          typeof properties.color === "string"
            ? properties.color
            : "rgba(255,255,255,1)",
        segments: segmentsOf(feature.geometry),
        selected:
          selected !== null &&
          String(selected.layerId) === layerId &&
          String(selected.trackId) === trackId,
      });
    }
    return out;
  }

  function clearTrackLabels() {
    for (const marker of trackLabelMarkers.values()) marker.remove();
    trackLabelMarkers.clear();
  }

  function refreshTrackLabels() {
    // No `mapLoaded` here on purpose. These are DOM markers, not a style
    // layer: they need a camera, not a loaded style. Gating on `mapLoaded`
    // meant the first geometry — which arrives before the style's `load`
    // fires — drew its lines and no names, and nothing asked again until the
    // operator happened to pan.
    if (!map) return;
    if (map.getZoom() < LABEL_MIN_ZOOM) {
      clearTrackLabels();
      return;
    }

    const placed = declutter(labelCandidates(), (at) =>
      map.project([at.lon, at.lat]),
    );
    const wanted = new Set(placed.map((label) => label.key));
    for (const [key, marker] of trackLabelMarkers) {
      if (wanted.has(key)) continue;
      marker.remove();
      trackLabelMarkers.delete(key);
    }

    for (const label of placed) {
      const existing = trackLabelMarkers.get(label.key);
      if (existing) {
        existing.setLngLat([label.at.lon, label.at.lat]);
        const element = existing.getElement();
        // `textContent`, never `innerHTML`: a track's name is whatever the
        // crew typed, and `maplibre-waiver.test.ts` fails on the alternative.
        if (element.textContent !== label.name)
          element.textContent = label.name;
        element.style.color = label.color;
        continue;
      }
      const element = document.createElement("div");
      element.className = "track-label";
      element.textContent = label.name;
      element.style.color = label.color;
      // The label must never eat a click meant for the line underneath it.
      element.style.pointerEvents = "none";
      const marker = new maplibregl.Marker({ element, anchor: "center" })
        .setLngLat([label.at.lon, label.at.lat])
        .addTo(map);
      trackLabelMarkers.set(label.key, marker);
    }
  }

  // Raise track line + label layers above every other layer (notably the
  // active raster `map-tiles`) so track geometry is never hidden under a
  // map image. Safe to call repeatedly; missing layers are skipped.
  function raiseTrackLayers() {
    if (!map) return;
    // The selected track's casing first, so it ends up under the coloured
    // line it belongs to rather than over the other tracks.
    for (const id of [TRACKS_LAYER_SELECTED, "tracks-lines", "tracks-labels"]) {
      if (map.getLayer(id)) map.moveLayer(id);
    }
  }

  // Which route is ЛИСА15, among twelve colours and no names on the map: the
  // selected row's track gets a casing. Cheap, and it needs no glyphs.
  $effect(() => {
    // The selection is read first, on purpose. An effect is subscribed to what
    // it actually reads, so a guard that exits before the read leaves it
    // subscribed to nothing and it never runs again — which is what this
    // effect did until it was turned around. `effect-reads-before-guarding`
    // now fails on the shape.
    const selected = $selectedTrack;
    if (!map) return;
    highlightTrack(map, selected);
    // A selected track keeps its name when space is short, so the label set
    // changes with the selection as well as with the camera.
    refreshTrackLabels();
  });

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
    pointIndex: number,
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

    const marker = new maplibregl.Marker({
      element: pointElement,
      draggable: true,
    })
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
          [lngLat.lat, lngLat.lng],
        );
        await reloadEditableTrackPoints(layerId, trackId);
      } catch (error) {
        reportEditFailure("map.movePointFailed", error);
        // The marker is where the operator dropped it and the data is not, so
        // the map would otherwise go on showing a point that is not there.
        await reloadEditableTrackPoints(layerId, trackId).catch(() => {});
      }
    });

    pointMarkers.push(marker);
  }

  function renderEditableTrackPoints(
    layerId: bigint,
    trackId: bigint,
    detail: TrackDetail,
  ) {
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
    // Re-check after the await: switching the mode off or selecting another
    // track during the request used to be ignored, and the late answer drew
    // editable markers for a track nobody was editing any more, with handlers
    // still carrying the old ids. External review, 2026-09-22.
    const selected = get(selectedTrack);
    if (
      !get(editModeActive) ||
      selected?.layerId !== layerId ||
      selected?.trackId !== trackId
    ) {
      return;
    }
    renderEditableTrackPoints(layerId, trackId, detail);
    await refreshTrackGeometry();
  }

  async function handleDeletePoint() {
    if (!contextMenu) return;
    const { layerId, trackId, segment, point } = contextMenu.target;
    contextMenu = null;
    try {
      await deleteTrackPoint(
        layerId,
        trackId,
        BigInt(segment.id),
        BigInt(point.id),
      );
      await reloadEditableTrackPoints(layerId, trackId);
    } catch (error) {
      reportEditFailure("map.deletePointFailed", error);
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
        [point.lat, point.lon],
      );
      await reloadEditableTrackPoints(layerId, trackId);
    } catch (error) {
      reportEditFailure("map.insertPointFailed", error);
    }
  }

  /**
   * Legacy clear-all-and-recreate path. Retained as an explicit debugging
   * affordance per `consolidate-state-event-flow` design — the normal hot
   * path goes through the incremental reconciler in `refreshWaypointMarkers`.
   * Also called from teardown and from the empty-state branch of the
   * reconciler to drop every marker in one pass.
   */
  function clearWaypointMarkers() {
    for (const m of waypointMarkers.values()) {
      m.remove();
    }
    waypointMarkers.clear();
    appliedWaypoints.clear();
  }

  function waypointMarkerKey(layerId: bigint, waypointId: number): string {
    return `${layerId.toString()}:${waypointId}`;
  }

  /**
   * Snapshot of the marker metadata last applied to MapLibre. Used by the
   * incremental reconciler (`refreshWaypointMarkers`) so that subsequent
   * state updates only do the DOM work that actually changed. Keyed by the
   * same `${layerId}:${waypointId}` string as `waypointMarkers`.
   */
  interface AppliedWaypoint {
    lat: number;
    lon: number;
    name: string;
    isActive: boolean;
    /** Kept so the reconciler redraws the glyph when the symbol changes. */
    symbol: string | null;
    /** RGBA, or null for the default the stylesheet gives every marker. */
    color: [number, number, number, number] | null;
  }
  let appliedWaypoints = new Map<string, AppliedWaypoint>();

  /**
   * Renders waypoint markers from every visible waypoint layer in the current
   * project. Edit interactions (drag-to-move) are routed only for markers
   * belonging to the active waypoint layer; markers from inactive layers are
   * rendered as read-only context (no drag handle) so switching the active
   * layer is non-destructive — markers from other layers stay on screen.
   *
   * Reconciliation contract (consolidate-state-event-flow): the function
   * diffs the incoming `(layer_id, waypoint_id) → snapshot` map against the
   * current `waypointMarkers` map and performs the minimum DOM work —
   * create on additions, remove on deletions, setLngLat / popup text update
   * on coordinate or name changes. The active-vs-inactive flag forces a
   * marker rebuild because drag-handler attachment is set at construction
   * time. The legacy clear-all-and-recreate path is kept as
   * `clearWaypointMarkers()` for debugging.
   */
  // The tape follows the points. Raised each time because the track and
  // waypoint layers are re-added underneath it as the project changes.
  $effect(() => {
    const points = $measuredPoints;
    const mode = $measureMode;
    const centre = $ringCentre;
    const radius = $ringRadiusKm;
    if (!map || !mapLoaded) return;
    const ring = centre === null ? [] : ringAround(centre, radius);
    const projected = projectionPreview;
    updateMeasureLayer(
      map,
      projected === null ? points : [projected.from, projected.to],
      ring,
      // Closed only while measuring an area: the outline has to agree with
      // the number beside it.
      mode === "area" && projected === null,
    );
    raiseMeasureLayer(map);
  });

  /**
   * Where the projected waypoint would land, shown before it is committed.
   *
   * A bearing dictated over a radio is easy to mishear; seeing the point on
   * the map before placing it is how that gets caught.
   */
  const projectionPreview = $derived.by(() => {
    const from = $projectionOrigin;
    if (from === null || $projectionDistanceM <= 0) return null;
    return {
      from,
      to: destinationPoint(
        from,
        $projectionBearing,
        $projectionDistanceM / 1000,
      ),
    };
  });

  async function placeProjectedWaypoint() {
    const preview = projectionPreview;
    const layerId = $activeWaypointLayerId;
    if (preview === null || layerId === null) return;
    try {
      const existing = await getWaypoints(layerId);
      await addWaypoint(
        layerId,
        preview.to.lat,
        preview.to.lon,
        $i18n("map.newWaypointName").replace(
          "{n}",
          String(existing.length + 1),
        ),
      );
      await refreshWaypointMarkers();
      setProjection(false);
    } catch (error) {
      reportEditFailure("map.projectionFailed", error);
    }
  }

  /** See `createLatestRun`: an overtaken refresh must not draw its markers. */
  const waypointMarkerRuns = createLatestRun();
  const trackGeometryRuns = createLatestRun();
  /**
   * The active-map apply awaits the OZI metadata before it touches the map,
   * so switching maps during that read left two applies removing and adding
   * the same source. External review, 2026-09-22.
   */
  const activeMapRuns = createLatestRun();

  async function refreshWaypointMarkers() {
    if (!map) return;
    const run = waypointMarkerRuns.begin();
    // Read appState non-reactively. This function is invoked both from a
    // slice $effect (where reactivity is already scoped) and from explicit
    // mutation handlers (where we want a snapshot, not a subscription).
    const snapshot = get(appState);
    if (!snapshot || snapshot.waypoint_layer_count === 0) {
      clearWaypointMarkers();
      return;
    }

    const activeId = get(activeWaypointLayerId);
    const layers = get(visibleWaypointLayers);

    // Build the desired marker set first; only then mutate MapLibre. This
    // avoids a window where the map shows zero markers between clear and
    // recreate (the failure mode of the previous implementation).
    type Incoming = AppliedWaypoint & { wpId: number };
    const incoming = new Map<string, { layerId: bigint; data: Incoming }>();

    // Every layer at once: one await per layer made the marker set cost a
    // round trip per layer, on a path that runs on every state change.
    const perLayer = await Promise.all(
      layers.map(async (layer) => {
        const layerId = BigInt(layer.id);
        try {
          return { layerId, waypoints: await getWaypoints(layerId) };
        } catch (error) {
          // An empty array here would be indistinguishable from "this layer
          // has no marks", and the reconciler below would then remove the
          // markers that are on the map — a read failure silently erasing the
          // ШТАБ. `null` means "unknown", and an unknown layer keeps what it
          // has. External review, 2026-09-22.
          reportEditFailure("map.waypointsLoadFailed", error);
          return { layerId, waypoints: null };
        }
      }),
    );

    // An older refresh finishing here would draw a marker set the operator has
    // already moved past — a waypoint just added, gone again.
    if (!waypointMarkerRuns.isCurrent(run)) return;

    // Layers whose marks could not be read: their markers stay as they are.
    const unreadable = new Set(
      perLayer
        .filter((entry) => entry.waypoints === null)
        .map((entry) => String(entry.layerId)),
    );

    for (const { layerId, waypoints } of perLayer) {
      if (waypoints === null) continue;
      const isActive = activeId !== null && layerId === activeId;
      for (const wp of waypoints.filter((w) => w.visible !== false)) {
        const key = waypointMarkerKey(layerId, wp.id);
        incoming.set(key, {
          layerId,
          data: {
            wpId: wp.id,
            lat: wp.lat,
            lon: wp.lon,
            name: wp.name,
            isActive,
            symbol: wp.symbol ?? null,
            color: wp.color ?? null,
          },
        });
      }
    }

    // 1. Remove markers that are gone or whose draggable state flipped
    //    (draggable is wired at Marker construction; we can't toggle it
    //    in place without re-creating the marker).
    for (const [key, marker] of waypointMarkers) {
      // `layerId:waypointId` — a marker of an unreadable layer is left alone.
      if (unreadable.has(key.split(":")[0])) continue;
      const next = incoming.get(key);
      const applied = appliedWaypoints.get(key);
      if (!next || (applied && applied.isActive !== next.data.isActive)) {
        marker.remove();
        waypointMarkers.delete(key);
        appliedWaypoints.delete(key);
      }
    }

    // 2. Add new markers and update existing ones in place.
    for (const [key, { layerId, data }] of incoming) {
      const existing = waypointMarkers.get(key);
      if (!existing) {
        const isActive = data.isActive;
        const el = document.createElement("div");
        el.className = "waypoint-marker";
        // The symbol is what tells the task point from what was found from
        // where the danger is. It was stored, listed and exported, and the one
        // place it did not appear was the map.
        el.textContent = waypointGlyph(data.symbol);
        // The symbol says what a mark is; the colour says whose it is. An
        // uncoloured waypoint keeps the stylesheet's default, so changing that
        // default later moves every uncoloured marker with it.
        if (data.color) {
          el.style.background = waypointColorCss(data.color);
        }
        if (!isActive) {
          el.classList.add("inactive-layer");
        }
        el.style.cursor = isActive ? "grab" : "default";

        const marker = new maplibregl.Marker({
          element: el,
          draggable: isActive,
        })
          .setLngLat([data.lon, data.lat])
          .setPopup(new maplibregl.Popup({ offset: 16 }).setText(data.name))
          .addTo(map);

        if (isActive) {
          const wpId = data.wpId;
          marker.on("dragstart", () => {
            el.style.cursor = "grabbing";
          });

          marker.on("dragend", async () => {
            el.style.cursor = "grab";
            const lngLat = marker.getLngLat();
            try {
              await moveWaypoint(layerId, BigInt(wpId), [
                lngLat.lat,
                lngLat.lng,
              ]);
            } catch (error) {
              reportEditFailure("map.moveWaypointFailed", error);
              // The state-changed → reconcile pass that follows will pull
              // the authoritative coords; we just reset to the last
              // applied snapshot to avoid a flash at a wrong location.
              const last = appliedWaypoints.get(key);
              if (last) marker.setLngLat([last.lon, last.lat]);
            }
          });
        }

        waypointMarkers.set(key, marker);
        appliedWaypoints.set(key, { ...data });
        continue;
      }

      const applied = appliedWaypoints.get(key);
      if (!applied || applied.lat !== data.lat || applied.lon !== data.lon) {
        existing.setLngLat([data.lon, data.lat]);
      }
      if (!applied || applied.name !== data.name) {
        const popup = existing.getPopup();
        if (popup) popup.setText(data.name);
      }
      appliedWaypoints.set(key, {
        lat: data.lat,
        lon: data.lon,
        name: data.name,
        isActive: data.isActive,
      });
    }
  }

  async function handleMapClickForWaypoint(e: maplibregl.MapMouseEvent) {
    if ($drawingModeActive) return;
    if (!$addWaypointMode) return;
    // Invariant (layers spec): $activeWaypointLayerId is non-null whenever a
    // project is open. Add-waypoint mode is only reachable from /project.
    const layerId = $activeWaypointLayerId!;
    const { lat, lng } = e.lngLat;
    try {
      const currentWaypoints = await getWaypoints(layerId);
      const nextIndex = currentWaypoints.length + 1;
      await addWaypoint(
        layerId,
        lat,
        lng,
        $i18n("map.newWaypointName").replace("{n}", String(nextIndex)),
      );
      await refreshWaypointMarkers();
    } catch (error) {
      reportEditFailure("map.addWaypointFailed", error);
    }
    // The mode stays armed. It used to switch itself off after every mark,
    // including after a failed one, so a headquarters cutting the next outing
    // into tasks — ten marks: the base camp, the finds, the hazards — pressed
    // the toolbar button ten times between them. OziExplorer keeps its
    // waypoint tool selected until you pick another, and CJ-5 says "поставить
    // вейпоинты", plural.
    //
    // Leaving is Esc, the mode chip, or starting to draw: all three already
    // clear it. Panning does not, because a drag is not a click.
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
    if (!map || !mapLoaded) return;

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

  /**
   * CJ-4: publish the current viewport as lat/lon bounds. Registered on
   * `moveend` (not per-frame `move`) plus once on load, so the store is
   * cheap to keep fresh; `null` until the map is ready (crop-to-view
   * stays disabled until then).
   */
  function updateViewportBounds() {
    if (!map) return;
    const bounds = map.getBounds();
    mapViewportBounds.set({
      minLat: bounds.getSouth(),
      minLon: bounds.getWest(),
      maxLat: bounds.getNorth(),
      maxLon: bounds.getEast(),
    });
  }

  /**
   * CJ-4: select a track by clicking its rendered line. Skipped while a
   * modal map mode owns clicks (drawing adds points, edit mode drags
   * points, add-waypoint places a waypoint). Takes the topmost feature of
   * the `tracks-lines` layer; empty-map clicks do NOT clear the selection.
   */
  function handleMapClickForTrackSelect(e: maplibregl.MapMouseEvent) {
    if ($drawingModeActive || $editModeActive || $addWaypointMode) return;
    if (!map.getLayer("tracks-lines")) return;
    const features = map.queryRenderedFeatures(e.point, {
      layers: ["tracks-lines"],
    });
    const props = features[0]?.properties;
    if (props?.layer_id == null || props?.track_id == null) return;
    selectedTrack.set({
      layerId: BigInt(props.layer_id),
      trackId: BigInt(props.track_id),
    });
  }

  function handleMapClickForDrawing(e: maplibregl.MapMouseEvent) {
    if (
      !$drawingModeActive ||
      $drawingTrackLayerId === null ||
      $drawingTrackId === null ||
      $drawingSegmentId === null
    )
      return;
    // No cancel here. Cancelling the click before it was what dropped points:
    // a route plotted at three clicks a second kept about one in five, and the
    // rest vanished without a word. Only a double-click cancels, and it
    // cancels every click still waiting, because both of its own halves are
    // inside the window.
    const { lat, lng } = e.lngLat;
    drawingClicks.hold({ lat, lon: lng });
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
        // No `glyphs` on purpose: the app is offline-first and no SDF glyph
        // PBFs are bundled yet. A remote glyphs URL here does not just fail to
        // show labels — it POISONS tiling of any source shared with a symbol
        // layer. The `tracks` GeoJSON source feeds both `tracks-lines` (line)
        // and `tracks-labels` (symbol); a source tile only finishes parsing
        // once every layer's dependencies resolve, and the symbol layer's
        // glyph fetch hangs offline, so the tile never completes and the LINE
        // never renders either (owner's "треки не отображаются"). Leaving
        // glyphs undefined makes `map.getGlyphs()` falsy, so initTracksLayer
        // skips the symbol layer entirely and the line tiles cleanly. Bundling
        // SDF glyphs + setting this URL is the follow-up that re-enables labels.
      },
      center: [37.6, 55.75], // Moscow as default
      zoom: 5,
    });

    map.addControl(new maplibregl.NavigationControl(), "top-left");
    map.addControl(new maplibregl.ScaleControl(), "bottom-left");

    map.on("load", async () => {
      // One-shot: MapLibre "load" fires once per map lifetime. All refresh
      // paths defer to this moment via `mapLoaded` instead of gating on
      // isStyleLoaded()/once("load"), which silently dropped refreshes
      // forever when it ran after startup (tracks-never-render bug).
      mapLoaded = true;
      // CJ-2 promises a field launch makes no network requests, and this
      // source made one per visible tile — for a basemap that is covered by
      // the local raster the moment a map is opened, and that offline only
      // ever renders as grey anyway. A link found later in the session does
      // not bring it back: re-inserting a layer underneath the active raster
      // is how the "JPG накладывается поверх" bug happened, and the backdrop
      // is not worth that risk. The next launch with a link has it.
      // External review, 2026-09-22.
      if (mayReachNetworkNow()) {
        map.addSource("osm", {
          type: "raster",
          tiles: ["https://tile.openstreetmap.org/{z}/{x}/{y}.png"],
          tileSize: 256,
          maxzoom: 19,
          attribution:
            "© <a href='https://www.openstreetmap.org/copyright'>OpenStreetMap</a> contributors",
        });
        map.addLayer({ id: "osm-tiles", type: "raster", source: "osm" });
      }

      initTracksLayer(map);
      initMeasureLayer(map);

      // The grid the sectors are read off. Attached when it is asked for and
      // taken off when it is not: a layer that can only be added is a layer
      // that leaks, and this one redraws on every camera move.
      syncGraticule();

      // Pointer affordance over track lines (CJ-4 click-to-select).
      // Registered after initTracksLayer so the delegated events bind to
      // an existing layer. The mode guards keep the crosshair cursors of
      // drawing / edit / add-waypoint modes untouched.
      map.on("mouseenter", "tracks-lines", () => {
        if ($drawingModeActive || $editModeActive || $addWaypointMode) return;
        map.getCanvas().style.cursor = "pointer";
      });
      map.on("mouseleave", "tracks-lines", () => {
        if ($drawingModeActive || $editModeActive || $addWaypointMode) return;
        map.getCanvas().style.cursor = "";
      });

      updateViewportBounds();

      // Explicitly refresh tracks and waypoints once map is ready,
      // since $effect may have run before map was initialized
      try {
        const geojson = await getTracksGeojson();
        updateTracksLayer(map, geojson);
        raiseTrackLayers();
      } catch {
        // state may not be ready yet
      }
      refreshWaypointMarkers();
    });

    map.on("moveend", updateViewportBounds);
    // Which names fit depends on where they land on screen, so the set is
    // recomputed when the camera settles rather than only when the data
    // changes.
    //
    // `idle` as well as `moveend`, and it is not belt and braces. The map
    // opens at zoom 5 over Moscow and only then flies to the data. The first
    // geometry arrives at that opening zoom, where every name is suppressed
    // as a smear; the flight that follows is programmatic and had already
    // finished by the time this handler was attached, so `moveend` never came
    // and the map sat there with twelve coloured lines and no names — the
    // exact thing this change exists to fix.
    map.on("moveend", refreshTrackLabels);
    map.on("idle", refreshTrackLabels);

    map.on("click", (e) => {
      contextMenu = null;
      if ($measuringActive) {
        // Measuring takes the click whole: a click meant for the tape should
        // not also select a track or drop a waypoint.
        //
        // And it catches on a mark. "How far from the headquarters to the
        // drop-off" is the commonest measurement there is, and clicking as
        // near each mark as the hand manages is eighty metres out at a
        // two-kilometre view — for a number that goes out over the radio.
        const snapped = snapToWaypoint(
          { lat: e.lngLat.lat, lon: e.lngLat.lng },
          [...appliedWaypoints.values()].map((w) => ({
            lat: w.lat,
            lon: w.lon,
            name: w.name,
          })),
          (at) => {
            const point = map!.project([at.lon, at.lat]);
            return { x: point.x, y: point.y };
          },
        );
        measuredPoints.update((points) => [...points, snapped]);
        return;
      }
      if ($projectionActive) {
        projectionOrigin.set({ lat: e.lngLat.lat, lon: e.lngLat.lng });
        return;
      }
      if ($ringActive) {
        const here = { lat: e.lngLat.lat, lon: e.lngLat.lng };
        const centre = $ringCentre;
        if (centre === null) {
          ringCentre.set(here);
          ringRadiusKm.set(0);
        } else {
          // A second click sets the radius; a third starts a new ring, because
          // a crew drawing rings draws several and should not have to reach
          // for the palette between them.
          ringRadiusKm.set(distanceKm(centre, here));
        }
        return;
      }
      handleMapClickForWaypoint(e);
      handleMapClickForDrawing(e);
      handleMapClickForTrackSelect(e);
    });

    map.on("dblclick", (e) => {
      if (!$drawingModeActive) return;
      e.preventDefault();
      drawingClicks.cancel();
      void finishDrawingMode();
    });

    return () => {
      window.removeEventListener("keydown", handleKeydown);
      stopFpsCounter();
      clearPointMarkers();
      clearWaypointMarkers();
      clearTrackLabels();
      clearDrawingPreview();
      mapViewportBounds.set(null);
      drawingClicks.cancel();
      map.remove();
    };
  });

  // When the active map's local_path changes, update the raster tile source.
  // Driven by the `activeMapRef` slice selector (a string) so the effect body
  // skips work on state-changed bursts that don't touch the active map
  // (downloads, track mutations). `activeMap` is read non-reactively here so
  // the effect does NOT re-run on every `derived(appState, ...)` recompute —
  // only on `local_path` change. The `appliedMapPath` guard is kept as a
  // belt-and-braces second filter.
  $effect(() => {
    const ref = $activeMapRef;
    if (!ref || !map) return;
    if (ref === appliedMapPath) return;
    const am = get(activeMap);
    if (!am) return;

    async function applyActiveMap() {
      const run = activeMapRuns.begin();

      let meta: OziMetadataDto | null = null;
      if (am.kind === "ozi") {
        // Read before touching the map: a metadata read that fails or is
        // overtaken must leave the raster the crew is looking at alone.
        try {
          meta = await getOziMetadata(am.local_path);
        } catch (error) {
          reportEditFailure("map.activeMapFailed", error);
          return;
        }
        if (!activeMapRuns.isCurrent(run)) return;
      }

      // Only now is the apply going to happen, so only now has this path
      // been applied. Setting it before the await meant a read that failed
      // blocked every retry of the same map for the rest of the session.
      appliedMapPath = am.local_path;

      // Remove old map source/layer
      if (currentMapSourceId) {
        if (map.getLayer("map-tiles")) map.removeLayer("map-tiles");
        if (map.getSource(currentMapSourceId))
          map.removeSource(currentMapSourceId);
      }

      const sourceId = "active-map";
      currentMapSourceId = sourceId;

      let fitBoundsTarget: [number, number, number, number] | null = null;

      if (meta) {
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
      const tracksLayerId = map.getLayer("tracks-lines")
        ? "tracks-lines"
        : undefined;

      map.addLayer(
        {
          id: "map-tiles",
          type: "raster",
          source: sourceId,
          paint: { "raster-opacity": 1 },
        },
        tracksLayerId,
      );

      // Invariant: track lines/labels must always render ABOVE the active
      // raster. The beforeId above usually achieves this, but it silently
      // fails if tracks-lines was recreated after a style change or added in
      // a different order — the raster JPG then covers the tracks (owner's
      // "JPG накладывается поверх" report). Re-assert by raising them to the
      // top; a no-op when already on top.
      raiseTrackLayers();

      if (fitBoundsTarget) {
        map.fitBounds(fitBoundsTarget, { padding: 0, animate: true });
      } else if (am.center_lat !== 0 || am.center_lon !== 0) {
        map.flyTo({
          center: [am.center_lon, am.center_lat],
          zoom: am.base_zoom || 12,
        });
      }
    }

    if (!mapLoaded) {
      map.once("load", () => void applyActiveMap());
    } else {
      void applyActiveMap();
    }
  });

  /**
   * Last applied tracks fingerprint. Used by the slice effect below to skip
   * the IPC + GeoJSON apply when the fingerprint hasn't changed since the
   * last run. The empty-string sentinel matches the "no tracks loaded" state
   * before any project is open.
   */
  let mapLoaded = false;
  let appliedTracksFingerprint: string | null = null;
  /**
   * Last applied waypoints fingerprint (over `waypoint_layers`). The
   * incremental reconciler is cheap, but a coarse pre-check still avoids
   * the per-layer `getWaypoints` IPC round-trip when nothing layer-side
   * changed (e.g., a download-progress emit that triggers a `state-changed`
   * via per-file readiness).
   */
  let appliedWaypointsFingerprint: string | null = null;

  // Slice effect: re-fetch tracks GeoJSON only when the tracks-relevant
  // fingerprint changes. Replaces the old "run on every $appState change"
  // path — during a bundle download, the fingerprint is stable, so
  // `get_tracks_geojson` is no longer called on every per-file `state-changed`
  // emit (`consolidate-state-event-flow` change, Decision 2).
  $effect(() => {
    // Composite slice key: the fingerprint misses geometry-only edits
    // (sorting reorders points without changing point_count), so track
    // cleanup actions bump $tracksGeometryVersion to force a re-fetch.
    const fp = `${$tracksFingerprint}|${$tracksGeometryVersion}`;
    if (!map) return;
    if (fp === appliedTracksFingerprint) return;

    if (!mapLoaded) {
      map.once("load", async () => {
        // A newer fingerprint/version will schedule its own run.
        if (fp !== `${$tracksFingerprint}|${$tracksGeometryVersion}`) return;
        const geojson = await getTracksGeojson();
        updateTracksLayer(map, geojson);
        raiseTrackLayers();
        // This is the branch a cold start takes, and it is the third place
        // that draws the geometry. The names were added to the other two
        // first and did not appear at all until this one had them too —
        // worth remembering before adding a fourth.
        lastTracksGeojson = geojson;
        refreshTrackLabels();
        appliedTracksFingerprint = fp;
      });
      return;
    }

    appliedTracksFingerprint = fp;
    const run = trackGeometryRuns.begin();
    getTracksGeojson().then((geojson) => {
      // Two changes in quick succession leave two fetches in flight, and the
      // older one can answer last. Drawing it would put the map behind the
      // rail it is supposed to match.
      if (!trackGeometryRuns.isCurrent(run)) return;
      updateTracksLayer(map, geojson);
      raiseTrackLayers();
      // The names are DOM markers, so they follow the geometry rather than the
      // source. This is the path the map actually takes on a data change;
      // `refreshTrackGeometry` is the explicit one the drawing tools call.
      lastTracksGeojson = geojson;
      refreshTrackLabels();
    });
  });

  // Slice effect: reconcile waypoint markers only when the waypoint-layer
  // slice changes, or when the active waypoint layer toggles (because
  // drag-handler attachment depends on `isActive`). Per-waypoint mutations
  // arrive via explicit `refreshWaypointMarkers()` calls from the action
  // handlers (e.g. `handleMapClickForWaypoint`), not via AppStateDto, so
  // the fingerprint can stay coarse without losing correctness for the
  // common edit paths.
  $effect(() => {
    const fp = $waypointsFingerprint;
    const activeId = $activeWaypointLayerId;
    if (!map) return;

    // We treat the (fingerprint, activeId) pair as the slice key. Concatenate
    // into the local sentinel so a flip in either re-runs reconciliation.
    const compositeKey = `${fp}|${activeId === null ? "" : activeId.toString()}`;
    if (compositeKey === appliedWaypointsFingerprint) return;
    appliedWaypointsFingerprint = compositeKey;

    if (!mapLoaded) {
      map.once("load", () => {
        void refreshWaypointMarkers();
      });
      return;
    }

    void refreshWaypointMarkers();
  });

  $effect(() => {
    const drawing = $drawingModeActive;
    const editing = $editModeActive;
    if (!map) return;
    if (drawing) {
      if (editing) {
        editModeActive.set(false);
      }
      if ($addWaypointMode) {
        addWaypointMode.set(false);
      }
      contextMenu = null;
    }
    applyEditModeMapInteraction($editModeActive);
    syncGraticule();

    if (!$editModeActive || !$selectedTrack) {
      clearPointMarkers();
      contextMenu = null;
      return;
    }

    const active = $appState;
    void active;

    reloadEditableTrackPoints(
      $selectedTrack.layerId,
      $selectedTrack.trackId,
    ).catch((error) =>
      console.error("Failed to load editable track points", error),
    );
  });

  $effect(() => {
    updateSelectedPointMarkerState();
  });

  $effect(() => {
    const drawing = $drawingModeActive;
    const adding = $addWaypointMode;
    const editing = $editModeActive;
    if (!map) return;
    const canvas = map.getCanvas();
    if (drawing) {
      canvas.style.cursor = "crosshair";
    } else if (adding) {
      canvas.style.cursor = "crosshair";
    } else if (!editing) {
      canvas.style.cursor = "";
    }
  });

  $effect(() => {
    const active = $drawingModeActive;
    void $drawingTrackLayerId;
    void $drawingTrackId;
    if (!map) return;

    if (active) {
      map.dragPan.disable();
      map.doubleClickZoom.disable();
      drawingPreviewPoints = [];
      drawingCommandCount = 0;
      drawingPointCount.set(0);
      updateDrawingPreview();
      return;
    }

    map.dragPan.enable();
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
    const state = $simplifyState;
    if (!map) return;

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

    if (!mapLoaded) {
      map.once("load", updateSimplifyPreview);
    } else {
      updateSimplifyPreview();
    }
  });

  // ── "Show on map" (mapFocusRequest consumer) ─────────────────────────
  //
  // Library track rows and the Track Inspector write a one-shot request
  // into `mapFocusRequest` (nonce bumps on every click so repeat clicks on
  // the same track re-fire). MapView owns the map instance, so it resolves
  // the request here: fetch the track detail, compute the bbox over ALL
  // points in ALL segments, fitBounds, and reset the store to null.
  async function focusTrack(layerId: bigint, trackId: bigint) {
    try {
      const detail = await getTrackDetail(layerId, trackId);
      let minLat = Infinity;
      let minLon = Infinity;
      let maxLat = -Infinity;
      let maxLon = -Infinity;
      let hasPoints = false;
      for (const segment of detail.segments) {
        for (const point of segment.points) {
          hasPoints = true;
          if (point.lat < minLat) minLat = point.lat;
          if (point.lat > maxLat) maxLat = point.lat;
          if (point.lon < minLon) minLon = point.lon;
          if (point.lon > maxLon) maxLon = point.lon;
        }
      }
      if (!hasPoints) {
        toast.message(get(i18n)("track.noPoints"));
        return;
      }
      map.fitBounds(
        [
          [minLon, minLat],
          [maxLon, maxLat],
        ],
        { padding: 60, maxZoom: 16 },
      );
    } catch (error) {
      console.error("Failed to focus track on map", error);
      toast.error(get(i18n)("track.showOnMapFailed"), {
        description: String(error),
      });
    }
  }

  /**
   * Fit the camera to everything the project holds.
   *
   * Asked for after an import — otherwise the new tracks render wherever they
   * are, off the active raster, and look like the import failed — and after a
   * project is opened, where the same emptiness looks like a project that
   * failed to load.
   *
   * Waypoints count. A project whose content is a headquarters and a drop-off
   * point has nothing in its track geometry, and fitting to tracks alone left
   * it unframed. Their positions come from the markers already on the map, so
   * this costs one round trip, not one per waypoint layer.
   */
  async function focusAllData() {
    try {
      // The marks come from the layers, not from the markers that happen to
      // be on the map: those are placed by an asynchronous reconciler, so a
      // click that lands before it finishes used to frame the tracks and
      // leave the ШТАБ off-camera. External review, 2026-09-22.
      const layers = get(visibleWaypointLayers);
      const [geojson, perLayer] = await Promise.all([
        getTracksGeojson(),
        Promise.all(
          layers.map(async (layer) => {
            const layerId = BigInt(layer.id);
            try {
              return {
                layerId: String(layerId),
                waypoints: await getWaypoints(layerId),
              };
            } catch (error) {
              reportEditFailure("map.waypointsLoadFailed", error);
              return { layerId: String(layerId), waypoints: null };
            }
          }),
        ),
      ]);
      const points = focusPositions(
        geojson.features as { geometry?: { coordinates?: unknown } | null }[],
        perLayer,
        (layerId) => {
          const drawn: { lon: number; lat: number }[] = [];
          for (const [key, marker] of waypointMarkers) {
            if (key.split(":")[0] !== layerId) continue;
            const { lng, lat } = marker.getLngLat();
            drawn.push({ lon: lng, lat });
          }
          return drawn;
        },
      );
      const bounds = boundsOf(points);
      if (!bounds) return;
      if (isDegenerate(bounds)) {
        // One point, or a few metres across: `fitBounds` answers that with its
        // maximum zoom, which puts the crew inside a building.
        map.easeTo({
          center: centreOf(bounds),
          zoom: Math.max(map.getZoom(), 14),
        });
        return;
      }
      map.fitBounds(toLngLatBounds(bounds), { padding: 60, maxZoom: 16 });
    } catch (error) {
      reportEditFailure("map.fitAllTracksFailed", error);
    }
  }

  $effect(() => {
    const request = $mapFocusRequest;
    if (!request || !map) return;
    // Consume the request before the async work so a second click during
    // the fetch registers as a fresh store write (new nonce → new run).
    mapFocusRequest.set(null);
    if (request.kind === "track") {
      void focusTrack(request.layerId, request.trackId);
    } else if (request.kind === "all-data") {
      void focusAllData();
    } else if (request.kind === "waypoint") {
      map.easeTo({
        center: [request.lon, request.lat],
        zoom: Math.max(map.getZoom(), 14),
      });
    }
  });
</script>

<div class="relative h-full min-w-0 flex-1" bind:this={mapEl}>
  {#if contextMenu}
    <div
      class="bg-popover text-popover-foreground border-border absolute z-40 flex min-w-40 flex-col gap-0.5 rounded-md border p-1 shadow-lg"
      style={`left:${contextMenu.x}px; top:${contextMenu.y}px;`}
    >
      <button
        class="hover:bg-muted rounded-sm px-2 py-1.5 text-left text-xs"
        onclick={handleDeletePoint}>{$i18n("map.deletePoint")}</button
      >
      <button
        class="hover:bg-muted rounded-sm px-2 py-1.5 text-left text-xs"
        onclick={handleInsertPointAfter}>{$i18n("map.insertPointAfter")}</button
      >
    </div>
  {/if}
  {#if $measuringActive}
    <!-- Over the canvas rather than in the status bar: a crew reads the number
         where they are clicking, not at the other end of the window. -->
    <div
      class="pointer-events-none absolute top-2 left-1/2 z-10 -translate-x-1/2 rounded-sm bg-black/70 px-2 py-1 text-center text-xs text-white tabular-nums"
      data-testid="measure-readout"
    >
      {#if $measureMode === "area"}
        <!-- Area first, perimeter beside it: a sector is described by both —
             "прочесать 2.4 км², обойти по кромке 6 км". -->
        <span class="font-mono text-sm"
          >{formatMeasuredArea(polygonAreaSqKm($measuredPoints), $locale) ||
            "—"}</span
        >
        {#if $measuredPoints.length >= 3}
          <span class="font-mono text-sm opacity-90"
            >· {formatMeasuredDistance(
              pathLengthKm([...$measuredPoints, $measuredPoints[0]]),
              $locale,
            )}</span
          >
        {/if}
        <span class="ml-2 opacity-70">{$i18n("map.areaHint")}</span>
      {:else}
        <!-- A length on its own. The area of an open path is not a small
             number, it is a meaningless one, and a number in a place where
             people read numbers gets read. Owner, 2026-09-23: "часто надо
             померить длину". -->
        <span class="font-mono text-sm"
          >{formatMeasuredDistance(
            pathLengthKm($measuredPoints),
            $locale,
          )}</span
        >
        <!-- When both ends caught a mark, say which two. It is the answer to
             "от штаба до заброса сколько" in the words the question was asked
             in, and it is also what makes the catching visible: nothing else
             on screen would show that the click moved to the marker. -->
        {#if measuredMarks}
          <span
            class="ml-2 font-mono text-sm opacity-90"
            data-testid="measure-between">{measuredMarks}</span
          >
        {/if}
        <span class="ml-2 opacity-70">{$i18n("map.measureHint")}</span>
      {/if}
    </div>
  {/if}

  {#if $ringActive}
    <div
      class="pointer-events-none absolute top-2 left-1/2 z-10 -translate-x-1/2 rounded-sm bg-black/70 px-2 py-1 text-center text-xs text-white tabular-nums"
      data-testid="ring-readout"
    >
      <span class="font-mono text-sm"
        >{formatMeasuredDistance($ringRadiusKm, $locale)}</span
      >
      {#if $ringRadiusKm > 0}
        <!-- A ring is drawn to say "everything within five hundred metres of
             the last known position", and the next question is always how much
             ground that is. Owner, 2026-09-23. -->
        <span class="font-mono text-sm opacity-90"
          >· {formatMeasuredArea(
            Math.PI * $ringRadiusKm * $ringRadiusKm,
            $locale,
          )}</span
        >
      {/if}
      <span class="ml-2 opacity-70"
        >{$ringCentre === null
          ? $i18n("map.ringCentreHint")
          : $i18n("map.ringRadiusHint")}</span
      >
    </div>
  {/if}

  {#if $projectionActive}
    <div
      class="bg-card border-border absolute top-2 left-1/2 z-10 flex -translate-x-1/2 items-end gap-2 rounded-md border px-2 py-1.5 text-xs shadow-lg"
      data-testid="projection-panel"
    >
      {#if $projectionOrigin === null}
        <span class="text-muted-foreground py-1"
          >{$i18n("map.projectionOriginHint")}</span
        >
      {:else}
        <label class="flex flex-col gap-0.5">
          <span class="text-muted-foreground text-[10px]"
            >{$i18n("map.projectionBearing")}</span
          >
          <input
            class="border-border h-7 w-16 rounded-sm border bg-transparent px-1 text-right tabular-nums"
            type="number"
            min="0"
            max="360"
            step="1"
            data-testid="projection-bearing"
            bind:value={$projectionBearing}
          />
        </label>
        <label class="flex flex-col gap-0.5">
          <span class="text-muted-foreground text-[10px]"
            >{$i18n("map.projectionDistance")}</span
          >
          <input
            class="border-border h-7 w-20 rounded-sm border bg-transparent px-1 text-right tabular-nums"
            type="number"
            min="0"
            step="10"
            data-testid="projection-distance"
            bind:value={$projectionDistanceM}
          />
        </label>
        <button
          class="bg-primary text-primary-foreground h-7 rounded-sm px-2 disabled:opacity-50"
          disabled={projectionPreview === null}
          onclick={() => void placeProjectedWaypoint()}
          data-testid="projection-place"
        >
          {$i18n("map.projectionPlace")}
        </button>
      {/if}
    </div>
  {/if}

  {#if fpsVisible}
    <div
      class="pointer-events-none absolute top-2 right-2 z-10 rounded-sm bg-black/55 px-1.5 py-0.5 font-mono text-xs text-emerald-400 tabular-nums"
    >
      {fps} fps
    </div>
  {/if}
  {#if $drawingModeActive}
    <div
      class="bg-primary/20 border-primary text-primary pointer-events-none absolute top-2 left-2 z-10 rounded-md border px-2 py-1 text-xs"
    >
      <!-- Abbreviated, as the tracks list is: a Russian count needs three
           plural forms and "тчк" needs none. -->
      {get(i18n)("map.drawingTrack").replace(
        "{count}",
        String($drawingPointCount),
      )}
    </div>
  {/if}
</div>

<style>
  /* The names on the map. A halo rather than a box: a track runs under its
     own label, and a filled chip would hide the very line the name is for.
     `text-shadow` in four directions is the cheapest halo that works over
     both a dark forest and a pale field. */
  :global(.track-label) {
    font-size: 11px;
    font-weight: 600;
    line-height: 1;
    white-space: nowrap;
    letter-spacing: 0.01em;
    /* A white halo, not a dark one. The names are drawn in the track's own
       colour — saturated red, blue, teal — over a topographic map that is
       mostly pale green and grey. A dark halo under a saturated hue turns it
       muddy at 11px; white separates the letters from the map the way a
       printed map does, and still reads on the dark theme because the text
       itself stays bright. */
    text-shadow:
      0 0 3px rgba(255, 255, 255, 0.95),
      1px 0 2px rgba(255, 255, 255, 0.9),
      -1px 0 2px rgba(255, 255, 255, 0.9),
      0 1px 2px rgba(255, 255, 255, 0.9),
      0 -1px 2px rgba(255, 255, 255, 0.9);
    user-select: none;
  }

  /* Load-bearing rules for MapLibre marker DOM elements created by
     new maplibregl.Marker({ element }). Tailwind utilities cannot
     reach these because the elements are created via document.createElement
     in MapLibre internals, not in this component's template. */
  :global(.track-point-marker) {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    /* Per-marker colours stay independent of the active theme. Catppuccin
     * variables are kept as the primary value (so the Catppuccin pack still
     * tints them) with hex fallbacks for the native default theme where
     * the `--ctp-*` variables are absent. */
    background: var(--ctp-lavender, #b4befe);
    border: 2px solid var(--ctp-base, hsl(var(--background)));
    box-shadow: 0 0 0 1px var(--ctp-blue, hsl(var(--primary)));
    cursor: grab;
  }

  :global(.track-point-marker.selected) {
    background: var(--ctp-yellow, #f9e2af);
    box-shadow: 0 0 0 1px var(--ctp-peach, #fab387);
  }

  :global(.track-point-marker:active) {
    cursor: grabbing;
  }

  :global(.waypoint-marker) {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 22px;
    height: 22px;
    /* The disc stays: a glyph alone over aerial imagery is unreadable, and a
       marker has to be findable before it can be identified. */
    background: var(--ctp-yellow, #e5c890);
    border: 2px solid var(--ctp-crust, #232634);
    border-radius: 50%;
    font-size: 12px;
    line-height: 1;
    cursor: grab;
    user-select: none;
  }

  :global(.waypoint-marker:active) {
    cursor: grabbing;
  }

  :global(.waypoint-marker.inactive-layer) {
    opacity: 0.6;
    cursor: default;
  }
</style>
