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
- [ ] 1.8 Walked in the packaged application: the folder appears with a
      screenshot of the window in it.
