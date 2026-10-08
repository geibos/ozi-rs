<script lang="ts">
  import { onMount } from "svelte";
  import { get } from "svelte/store";
  import { goto } from "$app/navigation";
  import { resolve } from "$app/paths";
  import {
    appState,
    bundleLoaderOpen,
    workspaceWithoutMap,
  } from "../../lib/stores";
  import { worthOpeningWorkspace } from "$lib/workspace-route";
  import WorkspaceShell from "../../components/WorkspaceShell.svelte";
  import LibraryRail from "../../components/LibraryRail.svelte";
  import InspectorRail from "../../components/InspectorRail.svelte";
  import BundleLoader from "../../components/BundleLoader.svelte";
  import * as Sheet from "$lib/components/ui/sheet";

  /**
   * The workspace is worth opening when there is work in it OR a map under it.
   *
   * It used to need a map, and `load_project_from` clears the active map — a
   * `.ozp` from another headquarters may be for ground this machine has never
   * downloaded. So opening a colleague's project navigated here and was thrown
   * straight back to the launcher, with the project loaded and invisible: the
   * operator ended up exactly where they started, and nothing said why. Found
   * by a reviewer on 2026-09-23, after a fix that made the navigation happen
   * and did not notice the guard undoing it.
   *
   * Tracks on the OpenStreetMap backdrop are a workspace. A map with no
   * project is one too — that is a crew looking at the ground before the work
   * arrives. Neither is not. The rule is `worthOpeningWorkspace`, shared with
   * the launcher's start-up redirect, which until 2026-10-02 knew only half of
   * it.
   *
   * Decided once the state has arrived. On start it is empty for a moment, and
   * deciding on that sent every start to the launcher — which forwarded back
   * only when there was a map. The screenshot matrix found it.
   */
  onMount(() => {
    let cancelled = false;
    let stop = () => {};

    (async () => {
      if (get(appState) === null) await appState.refresh();
      if (cancelled) return;
      if (!worthOpeningWorkspace(get(appState), get(workspaceWithoutMap))) {
        goto(resolve("/"));
        return;
      }
      // And back to the loader if both go while the workspace stays mounted —
      // closing the project, or clearing the map.
      stop = appState.subscribe((state) => {
        if (
          state !== null &&
          !worthOpeningWorkspace(state, get(workspaceWithoutMap))
        ) {
          goto(resolve("/"));
        }
      });
    })();

    return () => {
      cancelled = true;
      stop();
    };
  });
</script>

<!--
  Workspace shell. The library rail hosts the three-tab `LibraryRail`
  (Maps / Tracks / Waypoints). The canvas is the visual region into which
  `MapView` (mounted once in `src/routes/+layout.svelte`) shows. The
  inspector rail hosts the context-sensitive `InspectorRail` driven by
  selection state (track / waypoint / map info).

  The bundle-loader Sheet is mounted alongside the shell so the Maps tab
  header's "Maps…" button can toggle it via the `bundleLoaderOpen` store.
  Cold-start at `/` still owns its own full-window loader; this Sheet only
  appears once the user is in the workspace.
-->
<WorkspaceShell>
  {#snippet libraryRail()}
    <LibraryRail />
  {/snippet}
  {#snippet inspectorRail()}
    <InspectorRail />
  {/snippet}
</WorkspaceShell>

<Sheet.Root bind:open={$bundleLoaderOpen}>
  <Sheet.Content
    side="right"
    class="w-[480px] max-w-[480px] p-0 sm:max-w-[480px]"
  >
    <BundleLoader onCloseRequest={() => bundleLoaderOpen.set(false)} />
  </Sheet.Content>
</Sheet.Root>
