<script lang="ts">
  /**
   * Test harness: the Tracks tab needs a `Tooltip.Provider` ancestor, which
   * `WorkspaceShell` supplies in the app.
   */
  import * as Tooltip from "$lib/components/ui/tooltip";
  import TracksTab from "../../components/library/TracksTab.svelte";
</script>

<Tooltip.Provider>
  <TracksTab />
</Tooltip.Provider>
