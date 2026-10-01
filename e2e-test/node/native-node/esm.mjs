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

/** Rejected extension text must not become a chord or a duplicate warning in WASM. */
test("native WASM keeps recovery outside rejected chord lists", () => {
  const source = "H(9,C(9,9))-%-D(11,11)";
  const result = parseChordProgressionString(source);
  assert.equal(result.success, false);
  assert.deepEqual(
    result.errors.map((error) => error.code),
    ["CHO-1", "CHB-1"],
  );
  assert.equal(result.warnings.length, 1);
  assert.equal(result.warnings[0].additionalInfo, "11");
  assert.equal(
    result.warnings[0].position.startOffset,
    source.lastIndexOf("11"),
  );
});

/** Metadata and nested extension recovery retain only warnings from outer values. */
test("native WASM respects bracket and parenthesis boundaries during recovery", () => {
  const metadata = parseChordProgressionString("[key=C,D(9,9)]E-%");
  assert.equal(metadata.success, false);
  assert.deepEqual(
    metadata.errors.map((error) => error.code),
    ["CIMV-3", "CHB-1"],
  );
  assert.deepEqual(metadata.warnings, []);

  const nested = parseChordProgressionString("C(9,(11,9),9)-D(11,11)");
  assert.equal(nested.success, false);
  assert.deepEqual(
    nested.errors.map((error) => error.code),
    ["EXT-2"],
  );
  assert.deepEqual(
    nested.warnings.map((warning) => warning.position.startOffset),
    [11, 19],
  );
});
