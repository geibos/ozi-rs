# Tasks

- [x] 1.1 `src/lib/drawing-clicks.ts`: one timer per click, a double-click
      cancels all of them, commits chained so two never compute the same
      insertion index.
- [x] 1.2 Five tests against a fake clock, written first: a fast burst keeps
      every click, order is preserved across a slow commit, a double-click
      commits nothing, a lone click waits out the whole window, a refused
      commit does not stop the rest.
- [x] 1.3 `MapView` holds instead of cancelling; the six sites that used to
      clear the single pending timeout now cancel the whole set.
- [x] 1.4 Walked on the stand: six clicks 90 ms apart produce six
      `insert_track_point` calls; a double-click produces none and finishes.
- [x] 1.5 `just ci` green.
