<script lang="ts">
  /**
   * The selected track's jumps and outliers (`$lib/track-jumps`), and the
   * edits an operator makes at each: split a jump; delete an outlier's apex,
   * or delete it and split the track there.
   *
   * A list, not a highlight: in OziExplorer these are found by reading the
   * point list's Dist and KPH columns, row by row. Here the rows that matter
   * are already picked out, and choosing one takes the map to it.
   */
  import { Button } from "$lib/components/ui/button";
  import { cutOutTrackPoint, deleteTrackPoint, splitSegment } from "$lib/api";
  import { reportEditFailure } from "$lib/edit-failure";
  import { locale, t } from "$lib/i18n";
  import {
    focusTrackPoint,
    selectedPointId,
    tracksGeometryVersion,
  } from "$lib/stores";
  import { findSuspects, formatLeg, type Suspect } from "$lib/track-jumps";
  import type { TrackDetail } from "$lib/types";

  interface Props {
    detail: TrackDetail;
    layerId: bigint;
    trackId: bigint;
  }

  let { detail, layerId, trackId }: Props = $props();

  const suspects = $derived(findSuspects(detail.segments));

  /** Position of each point in the whole track, 1-based, as the table counts. */
  const ordinal = $derived(
    new Map(
      detail.segments
        .flatMap((s) => s.points.map((p) => p.id))
        .map((id, i) => [id, i + 1] as const),
    ),
  );

  function choose(suspect: Suspect) {
    selectedPointId.set(BigInt(suspect.pointId));
    focusTrackPoint(suspect.lat, suspect.lon);
  }

  async function edit(run: () => Promise<void>, clearSelection: boolean) {
    try {
      await run();
      if (clearSelection) selectedPointId.set(null);
      tracksGeometryVersion.update((v) => v + 1);
    } catch (error) {
      reportEditFailure("jumps.failed", error);
    }
  }

  function split(s: Suspect) {
    void edit(
      () =>
        splitSegment(
          layerId,
          trackId,
          BigInt(s.segmentId),
          BigInt(s.previousPointId),
        ),
      false,
    );
  }

  function deleteApex(s: Suspect) {
    void edit(
      () =>
        deleteTrackPoint(
          layerId,
          trackId,
          BigInt(s.segmentId),
          BigInt(s.pointId),
        ),
      true,
    );
  }

  function cutOut(s: Suspect) {
    void edit(
      () =>
        cutOutTrackPoint(
          layerId,
          trackId,
          BigInt(s.segmentId),
          BigInt(s.pointId),
        ),
      true,
    );
  }
</script>

<div class="border-border border-b" data-testid="track-jumps">
  <div
    class="text-muted-foreground flex items-center justify-between px-3 py-1 text-[10px] font-semibold tracking-wider uppercase"
    title={$t("jumps.hint")}
  >
    <span>{$t("jumps.title")}</span>
    <span class="tabular-nums" data-testid="track-jumps-count"
      >{suspects.length}</span
    >
  </div>
  {#if suspects.length === 0}
    <div class="text-muted-foreground px-3 pb-2 text-[11px]">
      {$t("jumps.none")}
    </div>
  {:else}
    <ul class="max-h-40 overflow-y-auto pb-1" role="listbox">
      {#each suspects as s (s.pointId)}
        {@const chosen = $selectedPointId === BigInt(s.pointId)}
        <li
          role="option"
          aria-selected={chosen}
          class="hover:bg-muted/60 cursor-pointer px-3 py-1 text-[11px] {chosen
            ? 'bg-muted'
            : ''}"
          data-testid="track-jump"
          data-kind={s.kind}
          onclick={() => choose(s)}
          onkeydown={(e) => {
            if (e.key === "Enter" || e.key === " ") {
              e.preventDefault();
              choose(s);
            }
          }}
          tabindex="0"
        >
          <div class="flex items-baseline justify-between gap-2">
            <span
              class="font-medium {s.kind === 'outlier'
                ? 'text-destructive'
                : 'text-foreground'}"
              >{s.kind === "outlier"
                ? $t("jumps.outlier")
                : $t("jumps.jump")}</span
            >
            <span class="text-muted-foreground tabular-nums"
              >{$t("jumps.point").replace(
                "{i}",
                String(ordinal.get(s.pointId) ?? ""),
              )}</span
            >
          </div>
          <div class="text-muted-foreground font-mono text-[10px]">
            {formatLeg(s.before, $locale)}{#if s.kind === "outlier" && s.after}
              {" → "}{formatLeg(s.after, $locale)}{/if}
          </div>
          {#if chosen}
            <div class="flex flex-wrap gap-1 pt-1">
              {#if s.kind === "jump"}
                <Button
                  variant="outline"
                  size="xs"
                  data-testid="jump-split"
                  onclick={(e: MouseEvent) => {
                    e.stopPropagation();
                    split(s);
                  }}>{$t("jumps.split")}</Button
                >
              {:else}
                <Button
                  variant="outline"
                  size="xs"
                  data-testid="jump-delete-apex"
                  onclick={(e: MouseEvent) => {
                    e.stopPropagation();
                    deleteApex(s);
                  }}>{$t("jumps.deleteApex")}</Button
                >
                <Button
                  variant="outline"
                  size="xs"
                  data-testid="jump-cut-out"
                  onclick={(e: MouseEvent) => {
                    e.stopPropagation();
                    cutOut(s);
                  }}>{$t("jumps.cutOut")}</Button
                >
              {/if}
            </div>
          {/if}
        </li>
      {/each}
    </ul>
  {/if}
</div>
