import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import path from "node:path";
import { performance } from "node:perf_hooks";
import { pathToFileURL } from "node:url";

const args = process.argv.slice(2);
if (args.length > 1) {
  throw new Error(
    "Usage: node _tools/benchmark-wasm.mjs [web-package-directory]",
  );
}
const packageDirectory =
  args.length === 0
    ? new URL("../pkg/pkg-web/", import.meta.url)
    : pathToFileURL(`${path.resolve(args[0])}${path.sep}`);
const moduleUrl = new URL("chord_progression_parser.js", packageDirectory);
const parser = await import(moduleUrl.href);
const wasmBytes = await readFile(
  new URL("chord_progression_parser_bg.wasm", packageDirectory),
);
parser.initSync({ module: wasmBytes });

const shortSource = "C";
const longSource = "C(9,9)-Am(7)-Dm(7)-G/B\n".repeat(100);
const unicodeSource =
  "@section=日本語😀\n@repeat=2\n" +
  "[key=C]C/(😀あ)-[key=Dm]Dm(7)\n".repeat(100);
const errorSource = "H\n".repeat(100);

/** Validates each fixture once, outside the timed parse and format operations. */
function parsedAst(source) {
  const parsed = parser.parseChordProgressionString(source);
  assert.equal(parsed.success, true, "benchmark fixture must parse");
  const formatted = parser.formatChordProgression(parsed.ast);
  assert.deepEqual(
    parser.parseChordProgressionString(formatted).ast,
    parsed.ast,
  );
  return parsed.ast;
}

const shortAst = parsedAst(shortSource);
const longAst = parsedAst(longSource);
const unicodeAst = parsedAst(unicodeSource);
const errors = parser.parseChordProgressionString(errorSource);
assert.equal(errors.success, false);
assert.equal(errors.errors.length, 100);
const invalidAst = [{ metaInfos: null, chordBlocks: [] }];
assert.throws(
  () => parser.formatChordProgression(invalidAst),
  (error) => String(error).startsWith("invalid chord progression AST:"),
);

/** Includes exception handling in the invalid-input workload without timing assertions. */
function rejectInvalidAst() {
  try {
    parser.formatChordProgression(invalidAst);
  } catch {
    return null;
  }
  throw new Error("The benchmark fixture must be rejected");
}

const workloads = [
  {
    name: "parse_short",
    operation: () => parser.parseChordProgressionString(shortSource),
  },
  {
    name: "parse_long_with_warnings",
    operation: () => parser.parseChordProgressionString(longSource),
  },
  {
    name: "parse_unicode",
    operation: () => parser.parseChordProgressionString(unicodeSource),
  },
  {
    name: "parse_errors",
    operation: () => parser.parseChordProgressionString(errorSource),
  },
  {
    name: "format_short",
    operation: () => parser.formatChordProgression(shortAst),
  },
  {
    name: "format_long",
    operation: () => parser.formatChordProgression(longAst),
  },
  {
    name: "format_unicode",
    operation: () => parser.formatChordProgression(unicodeAst),
  },
  { name: "reject_nested_null", operation: rejectInvalidAst },
];
let lastResult;

/** Times calls to the generated JS API, retaining only the most recent result. */
function measure({ operation, iterations }) {
  const start = performance.now();
  for (let index = 0; index < iterations; index += 1) {
    lastResult = operation();
  }
  return performance.now() - start;
}

/** Calibrates a bounded iteration count toward a 100 ms sample. */
function calibrate(operation) {
  let iterations = 1;
  while (true) {
    const elapsed = measure({ operation, iterations });
    if (elapsed >= 100 || iterations === 1_000_000) return iterations;
    iterations = Math.min(
      1_000_000,
      iterations *
        Math.min(10, Math.max(2, Math.ceil(100 / Math.max(elapsed, 0.001)))),
    );
  }
}

console.log(`# Node ${process.version}; package ${moduleUrl.href}`);
console.log("workload,iterations,median_us,min_us,max_us");
for (const { name, operation } of workloads) {
  measure({ operation, iterations: 100 });
  const iterations = calibrate(operation);
  const samples = Array.from(
    { length: 7 },
    () => (measure({ operation, iterations }) * 1_000) / iterations,
  ).sort((left, right) => left - right);
  console.log(
    [
      name,
      iterations,
      ...[samples[3], samples[0], samples[6]].map((value) => value.toFixed(6)),
    ].join(","),
  );
}
assert.notEqual(lastResult, undefined);
