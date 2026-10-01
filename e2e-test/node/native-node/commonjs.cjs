const assert = require("node:assert/strict");
const { test } = require("node:test");
const parser = require("@lainnao/chord-progression-parser-node");
const {
  getWarningMessage,
} = require("@lainnao/chord-progression-parser-node/warning_code_message_map");

/** Keeps the CommonJS entry point usable without an ESM loader or bundler. */
test("native CommonJS loads WASM and warning messages", () => {
  const result = parser.parseChordProgressionString("C(1,1)");
  assert.equal(result.success, true);
  assert.equal(result.warnings.length, 1);
  assert.equal(parser.formatChordProgression(result.ast), "C(1,1)");
  assert.equal(
    getWarningMessage({ warningCode: result.warnings[0].code, lang: "en" }),
    "The same chord extension is specified more than once",
  );
});
