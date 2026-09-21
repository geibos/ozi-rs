<script lang="ts">
  /**
   * Test harness: the Waypoints tab needs a `Tooltip.Provider` ancestor,
   * which `WorkspaceShell` supplies in the app.
   */
  import * as Tooltip from "$lib/components/ui/tooltip";
  import WaypointsTab from "../../components/library/WaypointsTab.svelte";
</script>

<Tooltip.Provider>
  <WaypointsTab />
</Tooltip.Provider>
