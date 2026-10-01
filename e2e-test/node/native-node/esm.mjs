import assert from "node:assert/strict";
import { test } from "node:test";
import {
  formatChordProgression,
  parseChordProgressionString,
} from "@lainnao/chord-progression-parser-node";
import { getWarningMessage } from "@lainnao/chord-progression-parser-node/warning_code_message_map.js";
import { getErrorMessage } from "@lainnao/chord-progression-parser-node/error_code_message_map.js";
import { Extension } from "@lainnao/chord-progression-parser-node/generatedTypes.js";

/** Exercises the documented imports and WASM API under native Node.js ESM. */
test("native ESM parses, formats, and resolves localized diagnostics", () => {
  const result = parseChordProgressionString("C(9,9)");
  assert.equal(result.success, true);
  assert.equal(result.warnings.length, 1);
  assert.equal(formatChordProgression(result.ast), "C(9,9)");
  assert.equal(Extension.Nine, "9");
  assert.equal(
    getWarningMessage({ warningCode: result.warnings[0].code, lang: "ja" }),
    "同じコード拡張が複数回指定されています",
  );

  const failure = parseChordProgressionString("H");
  assert.equal(failure.success, false);
  assert.equal(
    getErrorMessage({ errorCode: failure.errors[0].code, lang: "en" }),
    "Invalid chord notation",
  );
});
