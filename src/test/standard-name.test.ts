import { describe, expect, it } from "vitest";

import {
  distinctNames,
  isStandardTrackName,
  suggestFromNames,
  suggestTrackName,
  transliterate,
} from "../lib/standard-name";

const MSK = "Europe/Moscow";

describe("a track name by the standard (п. 14–15)", () => {
  it("is the date, an underscore and a callsign, in latin letters", () => {
    expect(isStandardTrackName("20261008_Lisa15")).toBe(true);
    expect(isStandardTrackName("20200421_Lisa1_2")).toBe(true);
    expect(isStandardTrackName("20200421_Lisa2-Mohnatiy")).toBe(true);
  });

  it("is not Cyrillic, spaced, or without its date", () => {
    expect(isStandardTrackName("20260709-ЛИСА15")).toBe(false);
    expect(isStandardTrackName("20261008_Lisa 15")).toBe(false);
    expect(isStandardTrackName("Lisa15")).toBe(false);
    expect(isStandardTrackName("2026-10-08_Lisa15")).toBe(false);
    expect(isStandardTrackName("20261008_")).toBe(false);
  });
});

describe("transliteration as the detachment writes it", () => {
  it("matches the standard's own example", () => {
    expect(transliterate("Мохнатый")).toBe("Mohnatiy");
  });

  it("spells the hard letters one way", () => {
    expect(transliterate("Цыбуля")).toBe("Tsibulya");
    expect(transliterate("щука жук чай шея юла ёж")).toBe(
      "schuka zhuk chay sheya yula yozh",
    );
  });
});

describe("a suggested name", () => {
  it("dates by the group's first point, in the local day", () => {
    // 00:45 Moscow on the 8th is still the 7th in UTC.
    expect(suggestTrackName("Лиса 19 Мина", "2026-10-07T21:45:00Z", MSK)).toBe(
      "20261008_Lisa19_Mina",
    );
  });

  it("puts the group's number on the group", () => {
    expect(suggestTrackName("лиса15 вечер", null, MSK, "20261006")).toBe(
      "20261006_Lisa15_Vecher",
    );
    expect(suggestTrackName("ЛИСА15", null, MSK, "20261006")).toBe(
      "20261006_Lisa15",
    );
    expect(suggestTrackName("Ветер 2", null, MSK, "20261006")).toBe(
      "20261006_Veter2",
    );
    expect(suggestTrackName("БПЛА 1", null, MSK, "20261006")).toBe(
      "20261006_Bpla1",
    );
  });

  it("takes a date already in the name when the points have none", () => {
    expect(suggestTrackName("20261006-Veter1-Maura", null, MSK)).toBe(
      "20261006_Veter1_Maura",
    );
    expect(suggestTrackName("2026-10-06 Лиса 4", null, MSK)).toBe(
      "20261006_Lisa4",
    );
  });

  it("prefers the first point's day to a date in the file name", () => {
    expect(
      suggestTrackName("20261008_Lisa16", "2026-10-07T15:00:00Z", MSK),
    ).toBe("20261007_Lisa16");
  });

  it("leaves a standard name as it is", () => {
    expect(
      suggestTrackName("20261008_Lisa15", "2026-10-08T06:00:00Z", MSK),
    ).toBe("20261008_Lisa15");
  });

  it("has nothing to suggest without a callsign or a date", () => {
    expect(suggestTrackName("File", "2026-10-08T06:00:00Z", MSK)).toBeNull();
    expect(suggestTrackName("track.gpx", null, MSK)).toBeNull();
    expect(suggestTrackName("Лиса 19", null, MSK)).toBeNull();
  });
});

describe("two tracks of one group", () => {
  it("are numbered as п. 18 has it", () => {
    expect(
      distinctNames(["20261008_Lisa19", "20261008_Lisa15", "20261008_Lisa19"]),
    ).toEqual(["20261008_Lisa19_1", "20261008_Lisa15", "20261008_Lisa19_2"]);
  });

  it("are numbered past a name another track already has", () => {
    expect(
      distinctNames(["20261008_Lisa19"], new Set(["20261008_Lisa19"])),
    ).toEqual(["20261008_Lisa19_1"]);
    expect(
      distinctNames(
        ["20261008_Lisa19", "20261008_Lisa19"],
        new Set(["20261008_Lisa19_1"]),
      ),
    ).toEqual(["20261008_Lisa19_2", "20261008_Lisa19_3"]);
  });
});

describe("names as the owner gave them to his eleven tracks", () => {
  it("drops a Ветер's car, keeps a Лиса's callsign", () => {
    expect(suggestTrackName("Ветер 2 Гранта", null, MSK, "20261006")).toBe(
      "20261006_Veter2",
    );
    expect(suggestTrackName("20261006-Veter4-2", null, MSK)).toBe(
      "20261006_Veter4_2",
    );
    expect(suggestTrackName("Лиса3 Klyaksa", null, MSK, "20261006")).toBe(
      "20261006_Lisa3_Klyaksa",
    );
  });

  it("takes the file's name when the track's names no group", () => {
    expect(
      suggestFromNames(
        ["заброс", "20261006 Лиса4"],
        "2026-10-05T21:46:00Z",
        MSK,
      ),
    ).toBe("20261006_Lisa4");
    expect(
      suggestFromNames(["Ветер 1", "File"], "2026-10-05T21:40:00Z", MSK),
    ).toBe("20261006_Veter1");
  });
});
