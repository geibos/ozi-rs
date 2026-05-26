<script lang="ts">
  import { onMount } from "svelte";
  import { get } from "svelte/store";
  import { goto } from "$app/navigation";
  import { resolve } from "$app/paths";
  import { activeMap, bundleLoaderOpen } from "../../lib/stores";
  import WorkspaceShell from "../../components/WorkspaceShell.svelte";
  import LibraryRail from "../../components/LibraryRail.svelte";
  import InspectorRail from "../../components/InspectorRail.svelte";
  import BundleLoader from "../../components/BundleLoader.svelte";
  import * as Sheet from "$lib/components/ui/sheet";

  onMount(() => {
    if (!get(activeMap)) {
      goto(resolve("/"));
      return;
    }

    // Redirect back to the loader if the active map is cleared while the
    // workspace stays mounted (e.g. user closes the current project).
    const unsubscribe = activeMap.subscribe((m) => {
      if (!m) goto(resolve("/"));
    });
    return unsubscribe;
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
    class="w-[480px] max-w-[480px] sm:max-w-[480px] p-0"
  >
    <BundleLoader onCloseRequest={() => bundleLoaderOpen.set(false)} />
  </Sheet.Content>
</Sheet.Root>
