import { expect, test } from "bun:test";
import { readFile } from "node:fs/promises";
import path from "node:path";
import { generateDiagnosticSources } from "./generate_diagnostic_codes";

/** Fails when localized diagnostic definitions have not been regenerated for Rust. */
test("Rust diagnostic sources match the localized message maps", async (): Promise<void> => {
  const sources = generateDiagnosticSources();
  expect(
    await readFile(path.join(import.meta.dir, "../src/error_code.rs"), "utf8"),
  ).toBe(sources.errorCode);
  expect(
    await readFile(
      path.join(import.meta.dir, "../src/warning_code.rs"),
      "utf8",
    ),
  ).toBe(sources.warningCode);
});
