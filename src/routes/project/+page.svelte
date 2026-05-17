<script lang="ts">
  import { onMount } from "svelte";
  import { get } from "svelte/store";
  import { goto } from "$app/navigation";
  import { resolve } from "$app/paths";
  import { activeMap } from "../../lib/stores";
  import Sidebar from "../../components/Sidebar.svelte";
  import TracksPanel from "../../components/TracksPanel.svelte";
  import TrackPointsPanel from "../../components/TrackPointsPanel.svelte";
  import WaypointsPanel from "../../components/WaypointsPanel.svelte";

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

<Sidebar />
<TracksPanel />
<TrackPointsPanel />
<WaypointsPanel />
