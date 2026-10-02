import { describe, expect, it, vi } from "vitest";
import { readdirSync, readFileSync } from "fs";
import { join } from "path";
import {
  glyphAssetPath,
  loadGlyphRange,
  SHIPPED_GLYPH_RANGES,
} from "../lib/maplibre/glyphs-protocol";

/**
 * The map's text comes from glyphs shipped with the application.
 *
 * A glyph request that fails does not lose one label: MapLibre holds up the
 * whole tile the label belongs to, and the tile carries the track lines. That
 * is why a remote glyphs URL took the lines down offline in July. So the
 * protocol must answer every range it is asked for — the shipped ones with
 * their bytes, every other one with an empty set — and never throw.
 */
describe("glyph ranges", () => {
  it("serves a shipped range from the application's own files", () => {
    expect(glyphAssetPath("glyphs://Noto Sans Bold/1024-1279.pbf")).toBe(
      "/glyphs/noto-sans-bold/1024-1279.pbf",
    );
  });

  it("reads a fontstack the map URL-encoded", () => {
    expect(glyphAssetPath("glyphs://Noto%20Sans%20Bold/0-255.pbf")).toBe(
      "/glyphs/noto-sans-bold/0-255.pbf",
    );
  });

  it("draws a layer that asks for another font in the one that ships", () => {
    expect(glyphAssetPath("glyphs://Open Sans Regular/0-255.pbf")).toBe(
      "/glyphs/noto-sans-bold/0-255.pbf",
    );
  });

  it("has no file for a range that is not shipped", () => {
    expect(glyphAssetPath("glyphs://Noto Sans Bold/127744-127999.pbf")).toBe(
      null,
    );
  });

  it("answers a range that is not shipped with an empty set, without asking", async () => {
    const fetchImpl = vi.fn();
    const data = await loadGlyphRange(
      "glyphs://Noto Sans Bold/127744-127999.pbf",
      fetchImpl,
    );
    expect(data.byteLength).toBe(0);
    expect(fetchImpl).not.toHaveBeenCalled();
  });

  it("answers a shipped range with its bytes", async () => {
    const bytes = new Uint8Array([10, 2, 3, 4]).buffer;
    const fetchImpl = vi.fn(async () => new Response(bytes, { status: 200 }));
    const data = await loadGlyphRange(
      "glyphs://Noto Sans Bold/1024-1279.pbf",
      fetchImpl,
    );
    expect(fetchImpl).toHaveBeenCalledWith(
      "/glyphs/noto-sans-bold/1024-1279.pbf",
    );
    expect(new Uint8Array(data)).toEqual(new Uint8Array([10, 2, 3, 4]));
  });

  it("answers an empty set when a shipped range cannot be read", async () => {
    const warn = vi.spyOn(console, "warn").mockImplementation(() => {});
    const missing = vi.fn(async () => new Response(null, { status: 404 }));
    const broken = vi.fn(async () => {
      throw new TypeError("network down");
    });
    for (const fetchImpl of [missing, broken]) {
      const data = await loadGlyphRange(
        "glyphs://Noto Sans Bold/0-255.pbf",
        fetchImpl,
      );
      expect(data.byteLength).toBe(0);
    }
    warn.mockRestore();
  });
});

/**
 * What a callsign is written in has to be in the shipped files.
 *
 * Decoded from the files themselves rather than trusted from a list: a range
 * that was copied wrong, or a font without Cyrillic, would pass every test
 * above and draw `20240601_` and nothing after it.
 */
describe("the shipped glyphs", () => {
  const DIR = join("static", "glyphs", "noto-sans-bold");

  /**
   * The fields of one protobuf message, as `[field, value]` — a varint for
   * wire type 0, the bytes for wire type 2. Enough of the format to read
   * `glyphs { fontstack = 1 { glyph = 3 { id = 1 } } }`.
   */
  function fields(bytes: Uint8Array): Array<[number, number | Uint8Array]> {
    const out: Array<[number, number | Uint8Array]> = [];
    let pos = 0;
    const varint = (): number => {
      let value = 0;
      let shift = 0;
      for (;;) {
        const byte = bytes[pos++];
        value += (byte & 0x7f) * 2 ** shift;
        if (byte < 0x80) return value;
        shift += 7;
      }
    };
    while (pos < bytes.length) {
      const key = varint();
      const wire = key & 7;
      const field = Math.floor(key / 8);
      if (wire === 0) out.push([field, varint()]);
      else if (wire === 2) {
        const length = varint();
        out.push([field, bytes.subarray(pos, pos + length)]);
        pos += length;
      } else if (wire === 5) pos += 4;
      else if (wire === 1) pos += 8;
      else throw new Error(`wire type ${wire} in a glyph file`);
    }
    return out;
  }

  function glyphIds(file: string): Set<number> {
    const ids = new Set<number>();
    for (const [stackField, stack] of fields(readFileSync(join(DIR, file)))) {
      if (stackField !== 1 || typeof stack === "number") continue;
      for (const [glyphField, glyph] of fields(stack)) {
        if (glyphField !== 3 || typeof glyph === "number") continue;
        for (const [field, value] of fields(glyph)) {
          if (field === 1 && typeof value === "number") ids.add(value);
        }
      }
    }
    return ids;
  }

  it("ships exactly the ranges the protocol says it ships", () => {
    const files = readdirSync(DIR)
      .filter((f) => f.endsWith(".pbf"))
      .map((f) => f.replace(/\.pbf$/, ""))
      .sort();
    expect(files).toEqual([...SHIPPED_GLYPH_RANGES].sort());
  });

  it("can write every letter of a Russian or Latin callsign, and its date", () => {
    const available = new Set<number>();
    for (const range of SHIPPED_GLYPH_RANGES) {
      for (const id of glyphIds(`${range}.pbf`)) available.add(id);
    }
    const wanted =
      "АБВГДЕЁЖЗИЙКЛМНОПРСТУФХЦЧШЩЪЫЬЭЮЯабвгдеёжзийклмнопрстуфхцчшщъыьэюя" +
      "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz" +
      "0123456789_-.,:;()/«»—–№ ";
    const missing = [...wanted].filter(
      (ch) => !available.has(ch.codePointAt(0)!),
    );
    expect(missing).toEqual([]);
  });

  it("carries its licence", () => {
    const licence = readFileSync(join("static", "glyphs", "OFL.txt"), "utf-8");
    expect(licence).toContain("SIL Open Font License");
  });
});
