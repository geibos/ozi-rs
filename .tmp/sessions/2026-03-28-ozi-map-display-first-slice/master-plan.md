# Master Plan: OZI map display first slice

## Architecture
[OZI .map parser] -> [Application OZI map registration] -> [UI supported-raster display]
                                              \
                                               -> [Diagnostics for deferred ozf2/ozfx3]

## Component Order
1. [ ] **OZI MAP metadata import**
   - Parse `.map`
   - Resolve referenced raster path
   - Return explicit metadata/error types
2. [ ] **Application OZI map registration**
   - Add OZI map source import path
   - Keep source provenance separate from UI rendering
3. [ ] **UI supported-raster display**
   - Display OZI maps that reference directly supported image formats
   - Preserve sqlite/walkers behavior for existing mobile maps
4. [ ] **Deferred raster diagnostics**
   - Show clear message when `.map` points to `ozf2/ozfx3`
5. [ ] **Docs and validation**
   - Document supported first-slice behavior and deferrals

## Global Decisions
- First slice excludes native `ozf2/ozfx3` decoding.
- `.map` support is infrastructure/application first, UI second.
- The first working path targets `.map` plus a directly supported raster image format.
