import { describe, expect, test } from "bun:test";
import { parseChordProgression } from "./parseChordProgression";

describe("success", () => {
  test("can parse normal chord progression", () => {
    const parsedResult = parseChordProgression(` 
      @section=Intro
      |[key=E]E|C#m(7)|Bm(7)|C#(7)|
      |F#m(7)|Am(7)|F#(7)|B|

      @section=Verse
      |E|C#m(7)|Bm(7)|C#(7)|
      |F#m(7)|Am(7)|F#(7)|B|

      @section=Chorus
      |[key=C]C|C(7)|FM(7)|Fm(7)|
      |C|C(7)|FM(7)|Dm(7)|
      |Em(7)|E(7)|

      @section=Interlude
      |C|A,B|?|C| ||
    `);

    expect(parsedResult.isOk).toBe(true);
  });
});
