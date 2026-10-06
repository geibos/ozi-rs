/**
 * The screenshot matrix: every registered screen state, in both languages and
 * both themes, compared against a committed baseline.
 *
 *   node --experimental-strip-types src/test/stand/shots.ts [--compare|--update]
 *        [--screen <id>] [--slice <name>]
 *
 * Run through `just shots`, which runs this inside the Playwright image
 * (`scripts/shots.sh`) so that the pixels are the same on a Mac and in CI:
 * font rendering differs between the two, and a baseline that only matches on
 * one machine is a baseline nobody can update.
 *
 * Determinism: a fixed viewport, the application's own fonts, animations off,
 * a fixed clock, and every request that leaves the stand refused — the OSM
 * basemap included, so the map shows what the application draws and nothing
 * the network happened to deliver.
 */
import { chromium, type Browser, type Page } from "playwright";
import { PNG } from "pngjs";
import pixelmatch from "pixelmatch";
import {
  copyFileSync,
  existsSync,
  mkdirSync,
  readdirSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { join, resolve } from "node:path";
import {
  SCREENS,
  SHOT_LOCALES,
  SHOT_THEMES,
  shotName,
  type ScreenEntry,
  type ShotLocale,
  type ShotSetup,
  type ShotState,
  type ShotTheme,
} from "./screens.ts";

const ROOT = resolve(import.meta.dirname, "../../..");
const BASELINE = join(ROOT, "src/test/stand/baseline");
const STAND = process.env.STAND_URL ?? "http://127.0.0.1:5273";
const VIEWPORT = { width: 1280, height: 800 };
/** The moment every shot believes it is, so dates and ages do not move. */
const FIXED_TIME = new Date("2026-10-02T09:00:00+03:00");
/** Colour distance below which two pixels count as the same (pixelmatch). */
const PIXEL_THRESHOLD = 0.1;
/**
 * Differing pixels a shot may have and still match: the map is drawn by
 * WebGL in a software rasteriser, and antialiasing on a curve is allowed to
 * wobble by a few pixels. A row height, a colour or a missing label is
 * thousands.
 */
const ALLOWED_DIFF_RATIO = 0.0005;

interface Options {
  mode: "compare" | "update" | "render";
  screen: string | null;
  slice: string | null;
}

function parseArgs(argv: string[]): Options {
  const options: Options = { mode: "compare", screen: null, slice: null };
  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === "--compare") options.mode = "compare";
    else if (arg === "--update") options.mode = "update";
    else if (arg === "--render") options.mode = "render";
    else if (arg === "--screen") options.screen = argv[++i] ?? null;
    else if (arg === "--slice") options.slice = argv[++i] ?? null;
    else throw new Error(`unknown argument ${arg}`);
  }
  return options;
}

function outputDir(options: Options): string {
  if (options.slice) {
    const date = new Date().toISOString().slice(0, 10);
    return join(ROOT, "docs/progress", `${date}-${options.slice}`);
  }
  return join(ROOT, "target/shots");
}

/**
 * Wait until the element has been in the same place for a while.
 *
 * Playwright's click waits for an element to be stable, and when it is not —
 * the inspector re-renders as the track's detail arrives — it retries, and a
 * retry scrolls the target into view with a different alignment. Clicking an
 * entry in the inspector that way scrolled the whole inspector column in about
 * one shot in two (2026-10-06). An operator clicks something that is standing
 * still; so does the matrix.
 */
async function stillThere(page: Page, selector: string): Promise<void> {
  const target = page.locator(selector).first();
  await target.waitFor();
  let previous: string | null = null;
  for (let attempt = 0; attempt < 25; attempt += 1) {
    const box = JSON.stringify(await target.boundingBox());
    if (box === previous) return;
    previous = box;
    await page.waitForTimeout(200);
  }
}

async function bringUp(page: Page, setup: ShotSetup): Promise<void> {
  await page.goto(STAND + setup.url, { waitUntil: "load" });
  for (const step of setup.steps ?? []) {
    if ("click" in step) {
      await stillThere(page, step.click);
      await page.locator(step.click).first().click();
    } else if ("press" in step) await page.keyboard.press(step.press);
    else if ("settle" in step) await stableShot(page, "body");
    else await page.locator(step.waitFor).first().waitFor();
  }
  await page.locator(setup.ready).first().waitFor({ timeout: 15_000 });
  await page.evaluate(() => document.fonts.ready.then(() => undefined));
  if (setup.atMost) {
    const count = await page.locator(setup.atMost.selector).count();
    if (count > setup.atMost.count) {
      throw new Error(
        `${count} elements match ${setup.atMost.selector}, more than ${setup.atMost.count}`,
      );
    }
  }
}

/**
 * Load the stand once before the first shot. A fresh Vite server bundles its
 * dependencies on the first request and reloads the page when it is done,
 * which ate the first shot's whole timeout.
 */
async function warmUp(browser: Browser): Promise<void> {
  const page = await browser.newPage();
  try {
    await page.goto(STAND + "/project", { waitUntil: "load" });
    await page
      .locator('[data-testid="maps-open-project"]')
      .first()
      .waitFor({ timeout: 120_000 });
  } catch (error) {
    console.warn(`warm-up did not finish: ${String(error).split("\n")[0]}`);
  } finally {
    await page.close();
  }
}

/**
 * Screenshots until two in a row agree: the map flies to its data first.
 *
 * And the screen has to still be there afterwards. Two blank frames in a row
 * agree perfectly — which is what a page reloaded mid-shot looks like, and
 * the first baseline nearly took one.
 */
async function stableShot(page: Page, ready: string): Promise<Buffer> {
  // Three agreeing frames 600 ms apart, not two 400 ms apart: the map's
  // flight to its data can pause between two frames, and the first baseline
  // run after the matrix landed photographed one such pause (2026-10-06).
  const needed = 3;
  let previous: Buffer | null = null;
  let agreeing = 1;
  for (let attempt = 0; attempt < 20; attempt += 1) {
    const shot = await page.screenshot({
      animations: "disabled",
      caret: "hide",
      timeout: 60_000,
    });
    if (previous && diffPixels(previous, shot, null) === 0) {
      agreeing += 1;
      if (
        agreeing >= needed &&
        (await page.locator(ready).first().isVisible())
      ) {
        return shot;
      }
    } else {
      agreeing = 1;
    }
    previous = shot;
    await page.waitForTimeout(600);
  }
  throw new Error("the screen did not hold still");
}

/** Differing pixel count; writes a diff image when given a path. */
function diffPixels(a: Buffer, b: Buffer, diffPath: string | null): number {
  const left = PNG.sync.read(a);
  const right = PNG.sync.read(b);
  if (left.width !== right.width || left.height !== right.height) {
    return Number.POSITIVE_INFINITY;
  }
  const diff = new PNG({ width: left.width, height: left.height });
  const count = pixelmatch(
    left.data,
    right.data,
    diff.data,
    left.width,
    left.height,
    { threshold: PIXEL_THRESHOLD },
  );
  if (diffPath && count > 0) writeFileSync(diffPath, PNG.sync.write(diff));
  return count;
}

async function shoot(
  browser: Browser,
  screen: ScreenEntry,
  state: ShotState,
  setup: ShotSetup,
  locale: ShotLocale,
  theme: ShotTheme,
): Promise<Buffer> {
  const context = await browser.newContext({
    viewport: VIEWPORT,
    deviceScaleFactor: 1,
    locale: locale === "ru" ? "ru-RU" : "en-GB",
    timezoneId: "Europe/Moscow",
    colorScheme: theme,
  });
  try {
    await context.clock.setFixedTime(FIXED_TIME);
    await context.addInitScript(
      ({ locale, theme }) => {
        localStorage.setItem("ozi:locale", locale);
        localStorage.setItem(
          "theme",
          theme === "light" ? "native-light" : "native-dark",
        );
        localStorage.setItem("catppuccinPackEnabled", "0");
      },
      { locale, theme },
    );
    // Nothing leaves the stand: the shot is of the application, not of
    // whatever a tile server answered this minute.
    const standOrigin = new URL(STAND).origin;
    await context.route("**/*", (route) =>
      new URL(route.request().url()).origin === standOrigin
        ? route.continue()
        : route.abort(),
    );
    const page = await context.newPage();
    try {
      await bringUp(page, setup);
      return await stableShot(page, setup.ready);
    } catch (error) {
      throw new Error(
        `${screen.id} / ${state} / ${locale} / ${theme}: ${String(error).split("\n")[0]}`,
        { cause: error },
      );
    }
  } finally {
    await context.close();
  }
}

async function main(): Promise<number> {
  const options = parseArgs(process.argv.slice(2));
  const screens = SCREENS.filter(
    (s) => !options.screen || s.id === options.screen,
  );
  if (screens.length === 0) {
    console.error(`no screen named ${options.screen}`);
    return 2;
  }

  const out = outputDir(options);
  if (!options.slice) rmSync(out, { recursive: true, force: true });
  mkdirSync(join(out, "diff"), { recursive: true });

  const browser = await chromium.launch({
    // MapLibre 6 needs WebGL2; headless Chromium in the image gets it from
    // SwiftShader, which has to be asked for.
    args: ["--use-angle=swiftshader", "--enable-unsafe-swiftshader"],
  });
  const produced: string[] = [];
  const failures: string[] = [];
  try {
    await warmUp(browser);
    for (const screen of screens) {
      for (const [state, setup] of Object.entries(screen.states) as Array<
        [ShotState, ShotSetup]
      >) {
        for (const locale of SHOT_LOCALES) {
          for (const theme of SHOT_THEMES) {
            const name = shotName(screen.id, state, locale, theme);
            try {
              const png = await shoot(
                browser,
                screen,
                state,
                setup,
                locale,
                theme,
              );
              writeFileSync(join(out, name), png);
              produced.push(name);
              console.log(`shot ${name}`);
            } catch (error) {
              failures.push(
                String(error instanceof Error ? error.message : error),
              );
            }
          }
        }
      }
    }
  } finally {
    await browser.close();
  }

  if (options.mode === "update") {
    mkdirSync(BASELINE, { recursive: true });
    if (!options.screen) {
      for (const stale of readdirSync(BASELINE)) {
        if (stale.endsWith(".png") && !produced.includes(stale)) {
          rmSync(join(BASELINE, stale));
        }
      }
    }
    for (const name of produced)
      copyFileSync(join(out, name), join(BASELINE, name));
    console.log(`baseline updated: ${produced.length} shots`);
  } else if (options.mode === "compare") {
    for (const name of produced) {
      const baseline = join(BASELINE, name);
      if (!existsSync(baseline)) {
        failures.push(
          `${name}: no baseline — a new screen state needs \`just shots --update\``,
        );
        continue;
      }
      const count = diffPixels(
        readFileSync(baseline),
        readFileSync(join(out, name)),
        join(out, "diff", name),
      );
      const allowed = VIEWPORT.width * VIEWPORT.height * ALLOWED_DIFF_RATIO;
      if (count > allowed) {
        failures.push(
          `${name}: ${count === Number.POSITIVE_INFINITY ? "size changed" : `${count} pixels differ`} — diff in ${join("target/shots/diff", name)}`,
        );
      }
    }
    if (!options.screen && existsSync(BASELINE)) {
      for (const name of readdirSync(BASELINE)) {
        if (name.endsWith(".png") && !produced.includes(name)) {
          failures.push(
            `${name}: in the baseline, no longer rendered — remove it with \`just shots --update\``,
          );
        }
      }
    }
  }

  for (const failure of failures) console.error(`FAIL ${failure}`);
  console.log(
    `${produced.length} shots, ${failures.length} failures, written to ${out}`,
  );
  return failures.length === 0 ? 0 : 1;
}

main().then(
  (code) => process.exit(code),
  (error) => {
    console.error(error);
    process.exit(2);
  },
);
