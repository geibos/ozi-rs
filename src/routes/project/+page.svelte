<script lang="ts">
  import { onMount } from "svelte";
  import { get } from "svelte/store";
  import { goto } from "$app/navigation";
  import { resolve } from "$app/paths";
  import { activeMap } from "../../lib/stores";
  import WorkspaceShell from "../../components/WorkspaceShell.svelte";
  import InspectorRail from "../../components/InspectorRail.svelte";

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
  The 2-column `Sidebar.svelte` + floating `TracksPanel` / `TrackPointsPanel`
  / `WaypointsPanel` mount has been replaced by the 3-pane WorkspaceShell.
  Those components stay in the repository tree for the follow-up
  `library-sidebar` and `inspector-pane` changes to reuse or delete; this
  page no longer mounts any of them. The `library-rail` and `inspector-rail`
  slots are intentionally empty here — they receive content from the next
  two changes. `MapView` continues to be mounted from `src/routes/+layout.svelte`
  and becomes visible inside the canvas region while the workspace route is
  active.
-->
<WorkspaceShell>
  {#snippet inspectorRail()}
    <InspectorRail />
  {/snippet}
</WorkspaceShell>
