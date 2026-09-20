// OK-standard field name: 8 digits (date), an underscore OR dash separator,
// then a non-empty callsign (Cyrillic allowed). Real field files use both
// separators — e.g. `20260709-ЛИСА15` from the owner's bundle — so `-` is
// accepted alongside the documented `_`. A space is NOT a valid separator.
const OK_STANDARD_TRACK_NAME = /^\d{8}[_-].*\S.*$/u;

export function isOkStandardTrackName(name: string): boolean {
  return OK_STANDARD_TRACK_NAME.test(name);
}
