## 1. The data loss

- [x] 1.1 `partial_path` appends to the whole file name; two files of one bundle no longer collide
- [x] 1.2 The four tests that recomputed the wrong name now assert against the function
- [x] 1.3 Two new tests: the collision, and the original extension surviving

## 2. What quietly lied

- [x] 2.1 A failed read of a layer's marks is `null`, keeps its markers, and is reported
- [x] 2.2 "Показать всё" frames the layer data (`focusPositions`, its own tests)
- [x] 2.3 A finished walk keeps its count to the diagnostics log while a download runs
- [x] 2.4 A ready bundle file matches the map whose file name it is
- [x] 2.5 Applying the active map: run token, reported failure, late `appliedMapPath`

## 3. The operator's history

- [x] 3.1 The redo stack is cleared after the command lands, not before
- [x] 3.2 Coalescing is scoped to a gesture; `apply` never merges
- [x] 3.3 Cancelling a long drawing leaves no scratch track behind the stack depth

## 4. The promises

- [x] 4.1 No remote basemap and no catalogue walk when the machine reports no link
- [x] 4.2 The MapLibre waiver states the advisory's facts; its guard covers `attribution`

## 5. Gates

- [x] 5.1 `just ci` green
- [x] 5.2 `just smoke` green against a freshly built bundle
- [x] 5.3 A contract test over the labels the smoke gate matches on
