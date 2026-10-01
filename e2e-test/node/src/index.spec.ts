import { test, expect } from "bun:test";
import parser from "@lainnao/chord-progression-parser-node";
import type { ParseWarning } from "@lainnao/chord-progression-parser-node";
import {
  Accidental,
  ChordType,
  Extension,
} from "@lainnao/chord-progression-parser-node/generatedTypes";
import {
  ERROR_CODE_MESSAGE_MAP,
  getErrorMessage,
} from "@lainnao/chord-progression-parser-node/error_code_message_map";
import { getWarningMessage } from "@lainnao/chord-progression-parser-node/warning_code_message_map";

test("can import", () => {
  expect(Accidental.Flat).toBeDefined();
  expect(ERROR_CODE_MESSAGE_MAP).toBeDefined();
  expect(getErrorMessage).toBeDefined();
});

/** Keeps warnings non-blocking, required, and separate from the existing error domain. */
test("returns duplicate-extension warnings without changing the AST", (): void => {
  const result = parser.parseChordProgressionString("C(9,9,9)");
  if (!result.success) throw new Error("duplicates must parse");
  const warnings: ParseWarning[] = result.warnings;
  expect(getWarningMessage({ warningCode: warnings[0].code, lang: "ja" })).toBe(
    "同じコード拡張が複数回指定されています",
  );
  expect(getWarningMessage({ warningCode: warnings[0].code, lang: "en" })).toBe(
    "The same chord extension is specified more than once",
  );
  expect(warnings).toEqual(
    [4, 6].map((offset) => ({
      code: "DUPLICATE_EXTENSION",
      additionalInfo: "9",
      position: {
        lineNumber: 1,
        columnNumber: offset + 1,
        length: 1,
        startOffset: offset,
        endOffset: offset + 1,
      },
    })),
  );
  expect(JSON.stringify(result.ast)).toContain('"extensions":["9","9","9"]');
  expect(parser.formatChordProgression(result.ast)).toBe("C(9,9,9)");
  for (const input of ["", "C(9,add9)", "H"]) {
    expect(parser.parseChordProgressionString(input).warnings).toEqual([]);
  }
});

/** Preserves warning ranges and syntax errors in the same JavaScript result. */
test("returns warnings alongside errors using original UTF-16 offsets", (): void => {
  const source = "@section=😀\r\nC(9, 9)-H";
  const result = parser.parseChordProgressionString(source);
  if (result.success) throw new Error("H must remain invalid");
  const offset = source.lastIndexOf("9");
  expect(result.errors.map((error) => error.code)).toEqual(["CHO-1"]);
  expect(result.warnings).toEqual([
    {
      code: "DUPLICATE_EXTENSION",
      additionalInfo: "9",
      position: {
        lineNumber: 2,
        columnNumber: 6,
        length: 1,
        startOffset: offset,
        endOffset: offset + 1,
      },
    },
  ]);
  expect(
    source.slice(
      result.warnings[0].position.startOffset,
      result.warnings[0].position.endOffset,
    ),
  ).toBe("9");
});

/** Keeps the generated enum and the WASM AST aligned for the root-only modifier. */
test("preserves extension 1 through the JavaScript API", (): void => {
  expect(Extension.One.toString()).toBe("1");
  const result = parser.parseChordProgressionString("C(1,5)/G");
  expect(result.success).toBe(true);
  if (!result.success) throw new Error("root-only notation must parse");

  expect(result.ast[0].chordBlocks[0]).toMatchObject({
    type: "bar",
    value: [
      {
        denominator: "G",
        chordExpression: {
          type: "chord",
          value: {
            plain: "C(1,5)",
            detailed: {
              chordType: ChordType.Major,
              extensions: [Extension.One, Extension.Five],
            },
          },
        },
      },
    ],
  });
  const formatted = parser.formatChordProgression(result.ast);
  expect(formatted).toBe("C(1,5)/G");
  expect(parser.parseChordProgressionString(formatted)).toEqual(result);
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
    parser.formatChordProgression([{ metaInfos: [], chordBlocks: [] }]),
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
      "[key=Fb]C - [key=E#]F",
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
    "@section\nH\n@repeat=nope\nI",
  );
  expect(result.success).toBe(false);

  if (!result.success) {
    expect(
      result.errors.map(({ code, position }) => ({
        code,
        lineNumber: position.lineNumber,
        startOffset: position.startOffset,
        endOffset: position.endOffset,
      })),
    ).toEqual([
      { code: "SMIK-2", lineNumber: 1, startOffset: 8, endOffset: 9 },
      { code: "CHO-1", lineNumber: 2, startOffset: 9, endOffset: 10 },
      { code: "SMIV-3", lineNumber: 3, startOffset: 19, endOffset: 23 },
      { code: "CHO-1", lineNumber: 4, startOffset: 24, endOffset: 25 },
    ]);
  }
});
