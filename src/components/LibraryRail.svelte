<script lang="ts">
  /**
   * LibraryRail — fills the `library-rail` slot of `WorkspaceShell`. Three
   * persistent tabs: Maps, Tracks, Waypoints. Backed by the shadcn `Tabs`
   * primitive; active-tab is held in a session-scoped writable
   * (`libraryActiveTab` in `$lib/stores`).
   *
   * Project lifecycle (Open / Save / Undo / Redo) and mode toggles (Draw /
   * Add Waypoint) intentionally do NOT live here — see Decision 7/8 of
   * `redesign-library-sidebar/design.md`. They belong on the top context-bar
   * delivered by `redesign-inspector-pane`.
   */
  import * as Tabs from "$lib/components/ui/tabs";
  import { libraryActiveTab } from "$lib/stores";
  import MapsTab from "./library/MapsTab.svelte";
  import TracksTab from "./library/TracksTab.svelte";
  import WaypointsTab from "./library/WaypointsTab.svelte";
</script>

<div class="flex h-full min-h-0 flex-col">
  <Tabs.Root
    bind:value={$libraryActiveTab}
    class="flex h-full min-h-0 flex-col"
  >
    <Tabs.List class="bg-card mx-2 mt-2">
      <Tabs.Trigger value="maps">Maps</Tabs.Trigger>
      <Tabs.Trigger value="tracks">Tracks</Tabs.Trigger>
      <Tabs.Trigger value="waypoints">Waypoints</Tabs.Trigger>
    </Tabs.List>
    <Tabs.Content value="maps" class="flex-1 min-h-0 m-0">
      <MapsTab />
    </Tabs.Content>
    <Tabs.Content value="tracks" class="flex-1 min-h-0 m-0">
      <TracksTab />
    </Tabs.Content>
    <Tabs.Content value="waypoints" class="flex-1 min-h-0 m-0">
      <WaypointsTab />
    </Tabs.Content>
  </Tabs.Root>
</div>
