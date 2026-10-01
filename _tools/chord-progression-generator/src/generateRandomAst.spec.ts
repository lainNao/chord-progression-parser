import { expect, spyOn, test } from "bun:test";
import {
  formatChordProgression,
  parseChordProgressionString,
} from "@lainnao/chord-progression-parser-node";
import { generateRandomAst } from "./generateRandomAst";

const countRanges = {
  sectionCountRange: { min: 2, max: 2 },
  chordMetaInfoCountRange: { min: 0, max: 0 },
  chordBlockCountRange: { min: 2, max: 2 },
  chordInfoCountRange: { min: 2, max: 2 },
  extensionCountRange: { min: 3, max: 3 },
};

/** Rejects invalid counts before any random branch can silently discard them. */
test("rejects count ranges that cannot produce a valid AST", (): void => {
  for (const range of [
    { min: -1, max: 1 },
    { min: 2, max: 1 },
    { min: 0.5, max: 1 },
    { min: 0, max: Number.POSITIVE_INFINITY },
    { min: Number.NaN, max: 1 },
  ]) {
    expect(() =>
      generateRandomAst({ ...countRanges, extensionCountRange: range }),
    ).toThrow(RangeError);
  }
  for (const field of [
    "chordBlockCountRange",
    "chordInfoCountRange",
  ] as const) {
    expect(() =>
      generateRandomAst({ ...countRanges, [field]: { min: 0, max: 0 } }),
    ).toThrow(RangeError);
  }
  expect(
    generateRandomAst({
      ...countRanges,
      sectionCountRange: { min: 0, max: 0 },
    }),
  ).toEqual([]);
});

/** Allows repetition within the first bar, while resetting it at each section boundary. */
test("generates valid repetition context and duplicate extension warnings", (): void => {
  const random = spyOn(Math, "random").mockReturnValue(0.125);
  try {
    const ast = generateRandomAst(countRanges);
    const result = parseChordProgressionString(formatChordProgression(ast));
    expect(result.success).toBe(true);
    switch (result.success) {
      case true:
        expect(result.ast).toEqual(ast);
        expect(result.warnings).toHaveLength(4);
        break;
      case false:
        throw new Error(JSON.stringify(result.errors));
      default: {
        const exhaustiveCheck: never = result;
        throw new Error(`Unexpected result: ${exhaustiveCheck}`);
      }
    }
    for (const section of ast) {
      expect(section.chordBlocks).toHaveLength(2);
      for (const [index, block] of section.chordBlocks.entries()) {
        switch (block.type) {
          case "bar":
            expect(block.value[0].chordExpression.type).toBe(
              index === 0 ? "chord" : "same",
            );
            expect(block.value[1].chordExpression.type).toBe("same");
            break;
          case "br":
            throw new Error("Expected generated bars");
          default: {
            const exhaustiveCheck: never = block;
            throw new Error(`Unexpected block: ${exhaustiveCheck}`);
          }
        }
      }
    }
  } finally {
    random.mockRestore();
  }
});

/** Covers larger extension lists and metadata through a repeatable generated corpus. */
test("seeded ASTs round trip through the public formatter and parser", (): void => {
  let state = 0x4d595df4;
  const random = spyOn(Math, "random").mockImplementation((): number => {
    state ^= state << 13;
    state ^= state >>> 17;
    state ^= state << 5;
    return (state >>> 0) / 0x100000000;
  });
  try {
    for (let index = 0; index < 100; index += 1) {
      const ast = generateRandomAst({
        sectionCountRange: { min: 1, max: 3 },
        chordMetaInfoCountRange: { min: 0, max: 3 },
        chordBlockCountRange: { min: 1, max: 5 },
        chordInfoCountRange: { min: 1, max: 4 },
        extensionCountRange: { min: 0, max: 6 },
      });
      const result = parseChordProgressionString(formatChordProgression(ast));
      switch (result.success) {
        case true:
          expect(result.ast).toEqual(ast);
          break;
        case false:
          throw new Error(JSON.stringify(result.errors));
        default: {
          const exhaustiveCheck: never = result;
          throw new Error(`Unexpected result: ${exhaustiveCheck}`);
        }
      }
    }
  } finally {
    random.mockRestore();
  }
});
