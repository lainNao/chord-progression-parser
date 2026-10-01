import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";

// Run after `make build-wasm-web`. Each workload gets its own WASM instance.
const moduleUrl = new URL(
  "../pkg/pkg-web/chord_progression_parser.js",
  import.meta.url,
);
const wasmBytes = await readFile(
  new URL("chord_progression_parser_bg.wasm", moduleUrl),
);
const fixtureParser = await import(moduleUrl.href);
fixtureParser.initSync({ module: wasmBytes });
const fixture = fixtureParser.parseChordProgressionString("C(9,9)-Dm(7)-G/B");
assert.equal(fixture.success, true);
const nullChordMetadata = structuredClone(fixture.ast);
nullChordMetadata[0].chordBlocks[0].value[0].metaInfos = null;
const nullExtensions = structuredClone(fixture.ast);
nullExtensions[0].chordBlocks[0].value[0].chordExpression.value.detailed.extensions =
  null;
const scenarios = [
  { name: "valid AST", input: [], rejects: false },
  { name: "valid chord AST", input: fixture.ast, rejects: false },
  { name: "invalid object", input: {}, rejects: true },
  { name: "null AST", input: null, rejects: true },
  {
    name: "null section metadata",
    input: [{ metaInfos: null, chordBlocks: [] }],
    rejects: true,
  },
  {
    name: "null section blocks",
    input: [{ metaInfos: [], chordBlocks: null }],
    rejects: true,
  },
  {
    name: "null bar contents",
    input: [{ metaInfos: [], chordBlocks: [{ type: "bar", value: null }] }],
    rejects: true,
  },
  { name: "null chord metadata", input: nullChordMetadata, rejects: true },
  { name: "null extensions", input: nullExtensions, rejects: true },
];

for (const [index, scenario] of scenarios.entries()) {
  moduleUrl.searchParams.set("scenario", String(index));
  const parser = await import(moduleUrl.href);
  const wasm = parser.initSync({ module: wasmBytes });
  const table = wasm.__wbindgen_externrefs;
  assert.ok(table instanceof WebAssembly.Table, "externref table is missing");

  /** Exercises the public formatter while checking its success/error contract. */
  function runBatch(count) {
    let rejected = 0;
    for (let iteration = 0; iteration < count; iteration += 1) {
      try {
        parser.formatChordProgression(scenario.input);
      } catch (error) {
        assert.match(String(error), /^invalid chord progression AST:/);
        rejected += 1;
      }
    }
    assert.equal(rejected, scenario.rejects ? count : 0);
  }

  // Warm the allocator first; a fixed workload should then reuse its slots.
  runBatch(1_000);
  const before = table.length;
  runBatch(20_000);
  const after = table.length;
  console.log(JSON.stringify({ scenario: scenario.name, before, after }));
  if (after !== before) process.exitCode = 1;
}
