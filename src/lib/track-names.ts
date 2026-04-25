const OK_STANDARD_TRACK_NAME = /^\d{8}_.*\S.*$/u;

export function isOkStandardTrackName(name: string): boolean {
  return OK_STANDARD_TRACK_NAME.test(name);
}
