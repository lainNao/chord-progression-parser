import { expect, test } from "bun:test";
import { readFile } from "node:fs/promises";
import path from "node:path";

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
