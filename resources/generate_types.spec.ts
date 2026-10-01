import { expect, test } from "bun:test";
import { mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { generateTypes } from "./generate_types";

const RUST_SOURCE = `
#[typeshare]
pub enum Accidental { Sharp, Flat }
#[typeshare]
pub struct Chord {
    pub denominator: Option<String>,
    pub accidental: Option<Accidental>,
    pub extensions: Vec<String>,
}
`;

/** Checks real typeshare output and verifies that stale fields are never silently repaired. */
test("checks every generated field without modifying the committed declarations", async (): Promise<void> => {
  const directory = await mkdtemp(
    path.join(os.tmpdir(), "chord-parser-types-test-"),
  );
  const outputFile = path.join(directory, "types.ts");
  try {
    await writeFile(path.join(directory, "model.rs"), RUST_SOURCE);
    await generateTypes({
      sourceDirectory: directory,
      outputFile,
      check: false,
    });
    const generated = await readFile(outputFile, "utf8");
    expect(generated).toContain("denominator: string | null;");
    expect(generated).toContain("accidental: Accidental | null;");
    expect(generated).toContain("extensions: string[];");
    await generateTypes({
      sourceDirectory: directory,
      outputFile,
      check: true,
    });
    expect(await readFile(outputFile, "utf8")).toBe(generated);

    const stale = generated.replace(
      "extensions: string[];",
      "extensions: number[];",
    );
    await writeFile(outputFile, stale);
    await expect(
      generateTypes({ sourceDirectory: directory, outputFile, check: true }),
    ).rejects.toThrow("make generate-ts-types");
    expect(await readFile(outputFile, "utf8")).toBe(stale);

    await generateTypes({
      sourceDirectory: directory,
      outputFile,
      check: false,
    });
    expect(await readFile(outputFile, "utf8")).toBe(generated);
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});

for (const [label, source, error] of [
  ["malformed Rust", "#[typeshare] pub struct Broken {", "typeshare failed"],
  [
    "missing expected nullable fields",
    "#[typeshare] pub struct Changed { pub value: String }",
    "Expected generated declaration was not found",
  ],
]) {
  /** Generator failures must leave an existing declaration file intact. */
  test(`preserves declarations on ${label}`, async (): Promise<void> => {
    const directory = await mkdtemp(
      path.join(os.tmpdir(), "chord-parser-types-failure-"),
    );
    const outputFile = path.join(directory, "types.ts");
    try {
      await writeFile(path.join(directory, "model.rs"), source);
      await writeFile(outputFile, "existing declarations");
      await expect(
        generateTypes({ sourceDirectory: directory, outputFile, check: false }),
      ).rejects.toThrow(error);
      expect(await readFile(outputFile, "utf8")).toBe("existing declarations");
    } finally {
      await rm(directory, { recursive: true, force: true });
    }
  });
}
