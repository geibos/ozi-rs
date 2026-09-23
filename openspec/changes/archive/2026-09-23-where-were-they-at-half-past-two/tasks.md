# Tasks

- [x] 1.1 `src/lib/track-replay.ts`: the range, the position at a moment with
      interpolation, the silence threshold, and the moment as a radio log
      writes it.
- [x] 1.2 Fifteen tests written first, including the three things a recording
      does that a naive reading does not survive: points out of order (which is
      why "sort by time" exists as an edit), points with no time or a broken
      one, and a moment outside the recording at either end.
- [x] 1.3 The control in the Track Inspector, with play; the marker on the map,
      amber in a silence.
- [x] 1.4 Walked on the stand: the slider opens at 12:00:00, the middle reads
      14:30:30 and moves the marker, play advances nine minutes of recording in
      nine hundred milliseconds and the button becomes «Пауза».
- [x] 1.5 `just ci` green: 391 Rust, 660 frontend.
