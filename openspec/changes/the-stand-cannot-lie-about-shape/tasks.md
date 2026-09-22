## 1. Stand

- [x] 1.1 `StandAnswers` types every answer from `commands` in the generated bindings
- [x] 1.2 The tile commands' `ArrayBuffer` is the stated exception
- [x] 1.3 The four loose answers the compiler found are exact

## 2. Verification

- [x] 2.1 Putting the old `{points, removed}` back fails `just check` with "missing the following properties from type 'SimplifiedPreviewDto'", then restored
- [x] 2.2 On the stand: the tracks list and the folder import still behave as before
- [x] 2.3 `just ci` green (334 Rust, 500 frontend)
