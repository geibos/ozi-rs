<script lang="ts">
  /**
   * Tying a picture to the Earth.
   *
   * A headquarters is sometimes handed an image and nothing else: a screenshot
   * of a web map, a photograph of a sheet on a table, a raster out of somebody
   * else's GIS. It is a map of the search area and it is useless, because
   * nothing says where on the Earth it sits — and until now this application
   * had no answer but "convert it in OziExplorer first", which is the program
   * the crew does not have.
   *
   * The form asks for the two opposite corners, because that is what a person
   * looking at a screenshot of a web map can actually read off it: the
   * north-west corner and the south-east one. Two corners fix position and
   * scale and say nothing about rotation, which is honest — a screenshot is
   * north-up, and a photograph of a sheet on a table is not something two
   * corners can straighten.
   *
   * The result is a real OziExplorer `.map` beside the picture, so the same
   * folder opens in OziExplorer on somebody else's laptop too.
   */
  import { toast } from "svelte-sonner";
  import { open } from "@tauri-apps/plugin-dialog";
  import { calibrateRaster, readRasterSize } from "$lib/api";
  import { parseLatLon } from "$lib/coordinates";
  import { t } from "$lib/i18n";

  let { onDone }: { onDone?: () => void } = $props();

  let imagePath = $state<string | null>(null);
  let size = $state<{ width: number; height: number } | null>(null);
  let title = $state("");
  let topLeft = $state("");
  let bottomRight = $state("");
  let busy = $state(false);

  const fileName = $derived(
    imagePath === null ? "" : (imagePath.split(/[\\/]/).pop() ?? imagePath),
  );
  const topLeftAt = $derived(parseLatLon(topLeft));
  const bottomRightAt = $derived(parseLatLon(bottomRight));
  const ready = $derived(
    imagePath !== null &&
      size !== null &&
      topLeftAt !== null &&
      bottomRightAt !== null &&
      !busy,
  );

  async function pickPicture() {
    const picked = await open({
      multiple: false,
      filters: [
        {
          name: "Изображения",
          extensions: [
            "jpg",
            "jpeg",
            "png",
            "tif",
            "tiff",
            "bmp",
            "gif",
            "webp",
          ],
        },
      ],
    });
    if (!picked) return;
    imagePath = picked as string;
    if (title.trim() === "") {
      title = (imagePath.split(/[\\/]/).pop() ?? "").replace(/\.[^.]+$/, "");
    }
    try {
      size = await readRasterSize(imagePath);
    } catch (error) {
      size = null;
      toast.error($t("calibrate.sizeFailed"), { description: String(error) });
    }
  }

  async function calibrate() {
    if (imagePath === null || size === null) return;
    if (topLeftAt === null || bottomRightAt === null) return;

    busy = true;
    try {
      const written = await calibrateRaster(
        imagePath,
        title.trim() || fileName,
        [
          { pixel_x: 0, pixel_y: 0, lat: topLeftAt.lat, lon: topLeftAt.lon },
          {
            pixel_x: size.width,
            pixel_y: size.height,
            lat: bottomRightAt.lat,
            lon: bottomRightAt.lon,
          },
        ],
      );
      toast.success(
        $t("calibrate.done").replace(
          "{file}",
          written.split(/[\\/]/).pop() ?? written,
        ),
        { description: written },
      );
      onDone?.();
    } catch (error) {
      toast.error($t("calibrate.failed"), { description: String(error) });
    } finally {
      busy = false;
    }
  }
</script>

<div class="calibrate" data-testid="calibrate-picture">
  <p class="lead">{$t("calibrate.lead")}</p>

  <button class="pick" onclick={() => void pickPicture()} type="button">
    {imagePath === null ? $t("calibrate.pick") : fileName}
  </button>
  {#if size}
    <p class="size" data-testid="calibrate-size">
      {size.width} × {size.height}
    </p>
  {/if}

  <label class="field">
    <span>{$t("calibrate.title")}</span>
    <input bind:value={title} type="text" />
  </label>

  <label class="field">
    <span>{$t("calibrate.topLeft")}</span>
    <input
      bind:value={topLeft}
      type="text"
      placeholder="60.0500, 30.2000"
      class:bad={topLeft.trim() !== "" && topLeftAt === null}
      data-testid="calibrate-top-left"
    />
  </label>

  <label class="field">
    <span>{$t("calibrate.bottomRight")}</span>
    <input
      bind:value={bottomRight}
      type="text"
      placeholder="59.9500, 30.4000"
      class:bad={bottomRight.trim() !== "" && bottomRightAt === null}
      data-testid="calibrate-bottom-right"
    />
  </label>

  <p class="hint">{$t("calibrate.formats")}</p>

  <button
    class="go"
    disabled={!ready}
    onclick={() => void calibrate()}
    type="button"
    data-testid="calibrate-go"
  >
    {busy ? $t("calibrate.working") : $t("calibrate.go")}
  </button>
</div>

<style>
  .calibrate {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    padding: 0.75rem;
  }
  .lead {
    font-size: 0.8rem;
    color: var(--muted-foreground, #888);
    margin: 0;
  }
  .field {
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
    font-size: 0.75rem;
  }
  .field input {
    padding: 0.35rem 0.5rem;
    border: 1px solid var(--border, #444);
    border-radius: 0.25rem;
    background: var(--background, transparent);
    color: inherit;
    font-size: 0.8rem;
  }
  .field input.bad {
    border-color: var(--destructive, #c0392b);
  }
  .size {
    margin: 0;
    font-size: 0.7rem;
    color: var(--muted-foreground, #888);
  }
  .hint {
    margin: 0;
    font-size: 0.7rem;
    color: var(--muted-foreground, #888);
  }
  .pick,
  .go {
    padding: 0.4rem 0.6rem;
    border: 1px solid var(--border, #444);
    border-radius: 0.25rem;
    background: var(--background, transparent);
    color: inherit;
    font-size: 0.8rem;
    cursor: pointer;
    text-align: left;
  }
  .go {
    text-align: center;
    font-weight: 600;
  }
  .go:disabled {
    opacity: 0.5;
    cursor: default;
  }
</style>
