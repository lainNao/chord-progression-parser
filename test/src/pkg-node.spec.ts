import { describe, expect, test } from "bun:test";
import parser from "../../pkg-node/chord_progression_ast_parser";

describe("success", () => {
  test("can parse normal chord progression", () => {
    let successResult;
    try {
      successResult = parser.run(`
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
      console.log(JSON.stringify(JSON.parse(successResult), null, 2));
    } catch (e: unknown) {
      console.log(e);
    }

    expect(successResult).not.toBeUndefined();
  });
});
