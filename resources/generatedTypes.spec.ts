import { expect, test } from "bun:test";
import { readFile } from "node:fs/promises";
import path from "node:path";
import { Extension } from "./generatedTypes";

/** Keeps the shared fixture, generated enum, syntax references, and EBNF aligned. */
test("canonical extension values agree across types, fixture, and docs", async (): Promise<void> => {
  const fixture: { extensions: string[] } = JSON.parse(
    await readFile(
      path.join(import.meta.dir, "../tests/fixtures/canonical_syntax.json"),
      "utf8",
    ),
  );
  expect(fixture.extensions).toEqual(Object.values(Extension));

  for (const language of ["en", "ja"]) {
    const source = await readFile(
      path.join(import.meta.dir, `../_docs/${language}/about-chord-progression-syntax.md`),
      "utf8",
    );
    const block = source.match(
      /<!-- extension-values:start -->\s*```txt\n([\s\S]*?)\n```\s*<!-- extension-values:end -->/,
    );
    expect(block).not.toBeNull();
    expect(block![1].split("\n")).toEqual(fixture.extensions);
  }

  const grammar = await readFile(
    path.join(import.meta.dir, "../_docs/chord-progression.ebnf"),
    "utf8",
  );
  const rule = grammar.match(/^extension\s*=\s*([^;]+);/m);
  expect(rule).not.toBeNull();
  expect(Array.from(rule![1].matchAll(/"([^"]+)"/g), (match): string => match[1]))
    .toEqual(fixture.extensions);
});

/** Keeps generated TypeScript nullability aligned with serde JSON output. */
test("nullable Rust fields are required and nullable in TypeScript", async () => {
  const generatedTypesPath = path.join(import.meta.dir, "generatedTypes.ts");
  const source = await readFile(generatedTypesPath, "utf8");

  expect(source).toContain("\tdenominator: string | null;");
  expect(source).toContain("\taccidental: Accidental | null;");
  expect(source).not.toContain("\tdenominator?: string;");
  expect(source).not.toContain("\taccidental?: Accidental;");
});

/** Keeps F-flat and E-sharp available as separate generated enum values. */
test("enharmonic key spellings remain distinct", async () => {
  const generatedTypesPath = path.join(import.meta.dir, "generatedTypes.ts");
  const source = await readFile(generatedTypesPath, "utf8");

  expect(source).toContain('\tFb_M = "Fb",');
  expect(source).toContain('\tFb_m = "Fbm",');
  expect(source).toContain('\tEs_M = "E#",');
  expect(source).toContain('\tEs_m = "E#m",');
});
