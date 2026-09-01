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
