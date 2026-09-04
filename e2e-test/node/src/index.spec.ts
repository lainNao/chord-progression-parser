import { test, expect } from "bun:test";
import parser from "@lainnao/chord-progression-parser-node";
import { Accidental } from "@lainnao/chord-progression-parser-node/generatedTypes";
import {
  ERROR_CODE_MESSAGE_MAP,
  getErrorMessage,
} from "@lainnao/chord-progression-parser-node/error_code_message_map";

test("can import", () => {
  expect(Accidental.Flat).toBeDefined();
  expect(ERROR_CODE_MESSAGE_MAP).toBeDefined();
  expect(getErrorMessage).toBeDefined();
});

test("can format a parsed AST", () => {
  const result = parser.parseChordProgressionString("C-D");
  expect(result.success).toBe(true);

  if (result.success) {
    expect(parser.formatChordProgression(result.ast)).toBe("C - D");
  }
});

test("rejects an AST that cannot round trip", () => {
  expect(() =>
    parser.formatChordProgression([{ metaInfos: [], chordBlocks: [] }])
  ).toThrow("AST cannot be represented");
});

test("preserves F-flat and E-sharp as distinct JavaScript values", () => {
  const result = parser.parseChordProgressionString("[key=Fb]C-[key=E#]F");
  expect(result.success).toBe(true);

  if (result.success) {
    const serializedAst = JSON.stringify(result.ast);

    expect(serializedAst).toContain('"value":"Fb"');
    expect(serializedAst).toContain('"value":"E#"');
    expect(parser.formatChordProgression(result.ast)).toBe(
      "[key=Fb]C - [key=E#]F"
    );
  }
});

test("reports multiple diagnostics with exact editor ranges", () => {
  const result = parser.parseChordProgressionString("H-C(111)-I");
  expect(result.success).toBe(false);

  if (!result.success) {
    expect(result.errors).toHaveLength(3);
    expect(result.errors.map(({ position }) => position)).toEqual([
      {
        lineNumber: 1,
        columnNumber: 1,
        length: 1,
        startOffset: 0,
        endOffset: 1,
      },
      {
        lineNumber: 1,
        columnNumber: 5,
        length: 3,
        startOffset: 4,
        endOffset: 7,
      },
      {
        lineNumber: 1,
        columnNumber: 10,
        length: 1,
        startOffset: 9,
        endOffset: 10,
      },
    ]);
  }
});

test("keeps reporting after malformed metadata on earlier lines", () => {
  const result = parser.parseChordProgressionString(
    "@section\nH\n@repeat=nope\nI"
  );
  expect(result.success).toBe(false);

  if (!result.success) {
    expect(
      result.errors.map(({ code, position }) => ({
        code,
        lineNumber: position.lineNumber,
        startOffset: position.startOffset,
        endOffset: position.endOffset,
      }))
    ).toEqual([
      { code: "SMIK-2", lineNumber: 1, startOffset: 8, endOffset: 9 },
      { code: "CHO-1", lineNumber: 2, startOffset: 9, endOffset: 10 },
      { code: "SMIV-3", lineNumber: 3, startOffset: 19, endOffset: 23 },
      { code: "CHO-1", lineNumber: 4, startOffset: 24, endOffset: 25 },
    ]);
  }
});
