/**
 * The project file's extension, in one place.
 *
 * The format is `.ozp` — AGENTS.md, `docs/project-map.md` and the
 * `project-persistence` capability all say so, and that capability requires a
 * save "to a user-chosen `.ozp` file". Both dialogs filtered on `json`
 * instead, which meant two things: what a crew saved was not the format the
 * documentation names, and an `.ozp` they had from anywhere else was invisible
 * in the open dialog, because a filter hides what it does not match.
 *
 * Open still accepts `json`. Everything this app has saved until now carries
 * that extension, and a filter that hid those would lose a crew their work far
 * more surely than the wrong extension ever did.
 */
export const PROJECT_SAVE_EXTENSION = "ozp";

/** Newest first: the dialog offers the first as its default. */
export const PROJECT_OPEN_EXTENSIONS = ["ozp", "json"];
