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
