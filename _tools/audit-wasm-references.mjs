import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";

// Run after `make build-wasm-web`. Each workload gets its own WASM instance.
const args = process.argv.slice(2);
if (args.length > 1 || (args.length === 1 && args[0] !== "--js-exceptions")) {
  throw new Error(
    "Usage: node _tools/audit-wasm-references.mjs [--js-exceptions]",
  );
}
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
const referenceFixture = fixtureParser.parseChordProgressionString(
  "@section=Reprise\n@same=曲😀\n@repeat=2",
);
assert.equal(referenceFixture.success, true);
const blankLineFixture = fixtureParser.parseChordProgressionString(
  "@section=Verse\n[key=C]C\n\n\n%",
);
assert.equal(blankLineFixture.success, true);
const scenarios = [
  { name: "valid AST", input: [], rejects: false },
  { name: "valid chord AST", input: fixture.ast, rejects: false },
  { name: "valid reference AST", input: referenceFixture.ast, rejects: false },
  { name: "valid blank-line AST", input: blankLineFixture.ast, rejects: false },
  { name: "invalid object", input: {}, rejects: true },
  { name: "null AST", input: null, rejects: true },
];
const arrayFields = [
  { name: "section metadata", path: [0, "metaInfos"] },
  { name: "section blocks", path: [0, "chordBlocks"] },
  { name: "bar contents", path: [0, "chordBlocks", 0, "value"] },
  {
    name: "chord metadata",
    path: [0, "chordBlocks", 0, "value", 0, "metaInfos"],
  },
  {
    name: "extensions",
    path: [
      0,
      "chordBlocks",
      0,
      "value",
      0,
      "chordExpression",
      "value",
      "detailed",
      "extensions",
    ],
  },
];
const invalidValues = [
  { name: "null", value: null },
  { name: "undefined", value: undefined },
  { name: "boolean", value: true },
  { name: "number", value: 0 },
  { name: "string", value: "bad" },
  { name: "object", value: {} },
  { name: "function", value: () => {} },
];
// Reference values are required strings; reject malformed JSON-like payloads
// through the same public WASM boundary used for ordinary ASTs.
const invalidReferenceNames = [
  ...invalidValues.filter((entry) => typeof entry.value !== "string"),
  { name: "array", value: [] },
  { name: "empty string", value: "" },
];
for (const invalid of invalidReferenceNames) {
  scenarios.push({
    name: `${invalid.name} reference name`,
    input: [
      { metaInfos: [{ type: "same", value: invalid.value }], chordBlocks: [] },
    ],
    rejects: true,
  });
}
scenarios.push({
  name: "reference AST with forbidden chord body",
  input: [
    {
      metaInfos: [{ type: "same", value: "Verse" }],
      chordBlocks: fixture.ast[0].chordBlocks,
    },
  ],
  rejects: true,
});
for (const field of arrayFields) {
  for (const invalid of invalidValues) {
    const input = structuredClone(fixture.ast);
    let parent = input;
    for (const key of field.path.slice(0, -1))
      parent = Reflect.get(parent, key);
    Reflect.set(parent, field.path.at(-1), invalid.value);
    scenarios.push({
      name: `${invalid.name} ${field.name}`,
      input,
      rejects: true,
    });
  }
}

// Opt-in diagnostics for unexpected JS exceptions: these need a separate decision
// about WASM exception handling and currently fail in the published build strategy.
if (args[0] === "--js-exceptions") {
  const { proxy, revoke } = Proxy.revocable([], {});
  revoke();
  const sectionGetter = structuredClone(fixture.ast);
  Object.defineProperty(sectionGetter[0], "metaInfos", {
    get() {
      throw new Error("section getter");
    },
  });
  const tagGetter = structuredClone(fixture.ast);
  tagGetter[0].chordBlocks[0].value[0].denominator = Object.defineProperty(
    {},
    Symbol.toStringTag,
    {
      get() {
        throw new Error("tag getter");
      },
    },
  );
  scenarios.splice(
    0,
    scenarios.length,
    {
      name: "revoked root Proxy",
      input: proxy,
      rejects: true,
      jsException: true,
      errorPattern: /revoked/,
      iterations: 2_000,
    },
    {
      name: "throwing section getter",
      input: sectionGetter,
      rejects: true,
      jsException: true,
      errorPattern: /^Error: section getter$/,
      iterations: 2_000,
    },
    {
      name: "throwing toStringTag getter",
      input: tagGetter,
      rejects: true,
      jsException: true,
      errorPattern: /^Error: tag getter$/,
      iterations: 300,
    },
  );
}

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
        if (scenario.jsException)
          assert.match(String(error), scenario.errorPattern);
        else assert.match(String(error), /^invalid chord progression AST:/);
        rejected += 1;
      }
    }
    assert.equal(rejected, scenario.rejects ? count : 0);
  }

  // Warm the allocator first; a fixed workload should then reuse its slots.
  runBatch(scenario.jsException ? 100 : 1_000);
  const before = table.length;
  // Bound unexpected-JS-exception probes so escaping exceptions do not turn into
  // a memory trap before we can check their reference retention.
  runBatch(scenario.jsException ? scenario.iterations : 20_000);
  const after = table.length;
  assert.equal(parser.formatChordProgression([]), "");
  console.log(JSON.stringify({ scenario: scenario.name, before, after }));
  if (after !== before) process.exitCode = 1;
}
