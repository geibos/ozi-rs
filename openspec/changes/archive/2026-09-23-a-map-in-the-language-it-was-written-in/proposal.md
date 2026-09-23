## Why

A `.map` written on a Russian Windows — which is where they come from — has a
Cyrillic title and, more to the point, a Cyrillic raster file name, in
Windows-1251 as the format requires.

This application read `.map` files by trying UTF-8 and falling back to **lossy**
UTF-8, which turns every cp1251 byte into a replacement character. So the title
came back as question marks and the raster the file names could not be found at
all: the map simply did not open, and the reason was invisible.

Worse, since this morning the application writes `.map` files itself.
Calibrating `Сагра.png` produced a correct cp1251 file that its own reader
could not read — a feature whose output its input rejects, for the file name a
Russian headquarters actually uses.

Neither the writer's tests nor the reader's caught it, because both were right:
they agreed with each other about a file neither of them had written. It took
testing the round trip through the pipeline that serves tiles — calibrate, then
open, then ask for a tile — which is the journey a headquarters makes with a
picture that arrived with nothing.

`decode_plt_bytes` has had the decoding chain since May — BOM, strict UTF-8,
statistical detection, cp1251 — because `.plt` files have exactly the same
problem. There was no reason for a second one, and no reason for `.map` to go
without.

## What Changes

- `.map` files are decoded through the same chain as `.plt` files.
- A calibration this application writes can be opened by this application,
  whatever the picture is called.

## Impact

- Affected specs: `tile-rendering`
- Affected code: `src-tauri/src/infrastructure/import/ozi_map.rs`
