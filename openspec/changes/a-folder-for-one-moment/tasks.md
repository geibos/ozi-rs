# Tasks

- [x] 1.1 `commands/report.rs`: the folder, `state.json`, `diagnostics.txt`,
      `about.txt`, and the window captured through `screencapture` against the
      window's rectangle in points — a Retina screen is a factor of two, and
      getting it wrong photographs a quarter of the window.
- [x] 1.2 Diagnostics carry a time. The line a session opens with carries none,
      deliberately: it is the initial status rather than an event, and stamping
      it made the generated fixtures differ on every run so
      `fixtures_are_up_to_date` could never pass again.
- [x] 1.3 Shift+Cmd/Ctrl+D in the layout, and two entries in the command
      palette — the palette one waits for the palette to close, or it
      photographs itself.
- [x] 1.4 The description asked afterwards, as a toast action, into the folder
      already written; Cmd+Enter saves it.
- [x] 1.5 Stand answers, the reference page, and the classification that a
      report does not change the project — all three guards caught the new
      commands before I did.
- [x] 1.6 The command-reference extractor reads the last segment of
      `commands::<module>::<name>`; it named `tiles` explicitly and so reported
      the new `report` module as a missing command.
- [x] 1.7 `just ci` green: 404 Rust, 668 frontend.
- [x] 1.8 Walked in the packaged application, and it found a real defect.
      `smoke_report_capture_moment` presses Shift+Cmd+D in the bundled app and
      reads the folder back off disk. The first walk produced a 588 KB
      `screenshot.png` of the owner's **desktop wallpaper**: without the Screen
      Recording grant `screencapture` does not fail, it exits zero and writes a
      picture with every window missing, so the old check — does the file exist
      — was satisfied by it. An operator would have handed over a folder
      believing their screen was in it.
- [x] 1.9 Fixed at the cause: `CGPreflightScreenCaptureAccess` is asked before
      capturing, and `CGRequestScreenCaptureAccess` shows the system's request
      when the answer is no. A refusal now writes no `screenshot.png` at all
      and the toast says to allow it and restart. A guard test fails if the
      question is ever dropped or moved after the capture.
- [x] 1.10 Re-walked: the folder holds the window itself — title bar, the three
      library tabs, the toolbar saying there are unsaved changes, the Lazarevo
      raster over OSM, the scale bar — framed to the window with nothing cut
      off. Looked at, not inferred from the file size.
