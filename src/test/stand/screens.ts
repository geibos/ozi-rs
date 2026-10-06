/**
 * The screens the screenshot matrix photographs, and how to reach each state.
 *
 * Each screen lists the states that apply to it — a settings screen has no
 * overflow, a catalogue has no "empty project" — and for each, the stand URL,
 * the steps that bring the screen up and the element that says it is ready.
 * Selectors, not text: the matrix runs in both languages.
 *
 * The stand's flags (`src/test/stand/tauri-core.ts`): `?state=cold` opens the
 * launcher, `?state=empty` an empty project, `?fail=catalogue` an unreachable
 * catalogue, `?hold=<commands>` leaves those commands unanswered (loading),
 * `?catalogue=N` serves N searches (0 empty, thousands overflow), `?maps=none`
 * a saved project open with no map, `?track=dirty` the fixture track with an
 * outlier and a jump in it.
 */

export const SHOT_STATES = [
  "empty",
  "loading",
  "loaded",
  "error",
  "overflow",
] as const;
export type ShotState = (typeof SHOT_STATES)[number];

export const SHOT_LOCALES = ["ru", "en"] as const;
export type ShotLocale = (typeof SHOT_LOCALES)[number];

export const SHOT_THEMES = ["light", "dark"] as const;
export type ShotTheme = (typeof SHOT_THEMES)[number];

export type ShotStep =
  | { click: string }
  | { press: string }
  | { waitFor: string }
  /**
   * Wait until the whole screen holds still — the map included. A click that
   * moves the map, made while the map is still taking up its opening view,
   * loses to it in some runs and not in others.
   */
  | { settle: true };

export interface ShotSetup {
  /** Path and query on the stand. */
  url: string;
  /** Run in order after the page loads. */
  steps?: ShotStep[];
  /** The element whose presence says the screen is up. */
  ready: string;
  /**
   * At most this many elements match the selector — how the overflow state
   * proves the list is virtualised rather than merely long.
   */
  atMost?: { selector: string; count: number };
}

export interface ScreenEntry {
  id: string;
  /** What the screen is, for a reader of the matrix. */
  title: string;
  states: Partial<Record<ShotState, ShotSetup>>;
}

const TRACKS_TAB = '[role="tab"][data-value="tracks"]';
const WAYPOINTS_TAB = '[role="tab"][data-value="waypoints"]';

export const SCREENS: ScreenEntry[] = [
  {
    id: "launcher",
    title: "The catalogue, as a crew first sees it",
    states: {
      // The catalogue's age appears once the walk has finished and the cache
      // is written; waiting for the list alone photographed the screen on
      // either side of that moment, one run in two.
      loaded: {
        url: "/?state=cold",
        ready: '[data-testid="catalogue-age"]',
      },
      empty: {
        url: "/?state=cold&catalogue=0",
        ready: '[data-testid="project-count"]',
      },
      loading: {
        url: "/?state=cold&hold=load_projects",
        ready: '[data-testid="project-virtual-list"]',
      },
      error: {
        url: "/?state=cold&fail=catalogue",
        ready: '[data-testid="project-virtual-list"]',
      },
      overflow: {
        url: "/?state=cold&catalogue=13000",
        ready: '[data-testid="catalogue-age"]',
        // Thirteen thousand searches, and the DOM holds a screenful of them.
        atMost: {
          selector: '[data-testid="project-virtual-list"] [role="option"]',
          count: 100,
        },
      },
    },
  },
  {
    id: "library-maps",
    title: "The workspace with the Maps tab",
    states: {
      loaded: {
        url: "/project",
        ready: '[data-testid="maps-open-project"]',
      },
      empty: {
        url: "/project?maps=none",
        ready: '[data-testid="maps-empty-state"]',
      },
    },
  },
  {
    id: "library-tracks",
    title: "The Tracks tab",
    states: {
      loaded: {
        url: "/project",
        steps: [{ click: TRACKS_TAB }],
        ready: '[data-testid="tracks-tab-list"]',
      },
      empty: {
        url: "/project?state=empty",
        steps: [{ click: TRACKS_TAB }],
        ready: '[data-testid="library-import-tracks"]',
      },
      error: {
        url: "/project?fail=list_tracks",
        steps: [{ click: TRACKS_TAB }],
        ready: '[data-testid="library-import-tracks"]',
      },
    },
  },
  {
    id: "library-waypoints",
    title: "The Waypoints tab",
    states: {
      loaded: {
        url: "/project",
        steps: [{ click: WAYPOINTS_TAB }],
        ready: '[data-testid="waypoints-tab-list"]',
      },
      empty: {
        url: "/project?state=empty",
        steps: [{ click: WAYPOINTS_TAB }],
        ready: '[data-testid="library-add-waypoint"]',
      },
    },
  },
  {
    id: "track-inspector",
    title: "A track selected, with its inspector",
    states: {
      loaded: {
        url: "/project",
        steps: [
          { click: TRACKS_TAB },
          {
            click:
              '[data-testid="tracks-tab-list"] [data-testid="track-stats"]',
          },
        ],
        ready: '[data-testid="inspector-show-on-map"]',
      },
    },
  },
  {
    id: "track-jumps",
    title: "A track that needs cleaning, its outlier chosen in the list",
    states: {
      loaded: {
        url: "/project?track=dirty",
        steps: [
          { click: TRACKS_TAB },
          {
            click:
              '[data-testid="tracks-tab-list"] [data-testid="track-stats"]',
          },
          { settle: true },
          { click: '[data-testid="track-jump"][data-kind="outlier"]' },
        ],
        ready: '[data-testid="jump-cut-out"]',
      },
    },
  },
  {
    id: "command-palette",
    title: "The command palette over the workspace",
    states: {
      loaded: {
        url: "/project",
        steps: [
          { waitFor: '[data-testid="maps-open-project"]' },
          { press: "Meta+k" },
        ],
        ready: '[data-slot="dialog-content"]',
      },
    },
  },
  {
    id: "settings",
    title: "The settings screen",
    states: {
      loaded: {
        url: "/project",
        steps: [
          { waitFor: '[data-testid="maps-open-project"]' },
          { press: "Meta+Comma" },
        ],
        ready: '[data-testid="settings-stop-distance"]',
      },
    },
  },
];

/** `library-tracks__loaded__ru__light.png` — screen, state, locale, theme. */
export function shotName(
  screen: string,
  state: ShotState,
  locale: ShotLocale,
  theme: ShotTheme,
): string {
  return `${screen}__${state}__${locale}__${theme}.png`;
}
