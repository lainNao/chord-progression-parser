import { expect, test } from "bun:test";
import { mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import {
  updateReadmeVersion,
  updateReadmeVersionFile,
} from "./update_readme_version";

const README = `# Parser
Packages at v0.9.3 and earlier do not expose warnings.
import * as mod from "https://cdn.jsdelivr.net/npm/@lainnao/chord-progression-parser-web@0.9.3/chord_progression_parser.js";
Other package: https://cdn.jsdelivr.net/npm/@lainnao/chord-progression-parser-node@0.9.3/chord_progression_parser.js
`;

/** Only the documented web import follows the release version, including prereleases. */
test("updates the fixed CDN import without changing historical references", (): void => {
  const version = "1.2.3-rc.1+build.01";
  const updated = updateReadmeVersion({ source: README, version });
  expect(updated).toBe(README.replace("-web@0.9.3/", `-web@${version}/`));
  expect(updateReadmeVersion({ source: updated, version })).toBe(updated);
});

/** Missing or repeated target imports must fail instead of silently updating the wrong examples. */
test("rejects missing, duplicate, and non-versioned CDN imports", (): void => {
  for (const source of [
    "# No CDN example",
    README + README,
    README + README.replace("-web@0.9.3/", "-web@0.9.4/"),
    README.replace("-web@0.9.3/", "-web@latest/"),
  ]) {
    expect(() => updateReadmeVersion({ source, version: "0.9.4" })).toThrow();
  }
});

/** Cargo versions must remain strict semantic versions and cannot introduce URL syntax. */
test("rejects malformed package versions", (): void => {
  for (const version of [
    "",
    "v0.9.4",
    "0.9",
    "01.2.3",
    "1.02.3",
    "1.2.03",
    "1.2.3-01",
    "1.2.3-rc..1",
    "1.2.3+",
    "1.2.3/other",
    "1.2.3\n",
  ]) {
    expect(() => updateReadmeVersion({ source: README, version })).toThrow(
      "valid semantic version",
    );
  }
});

/** Drift checks leave files intact; the explicit update reads the actual package table. */
test("checks drift without writing and updates from Cargo.toml", async (): Promise<void> => {
  const directory = await mkdtemp(
    path.join(os.tmpdir(), "chord-readme-version-"),
  );
  const cargoFile = path.join(directory, "Cargo.toml");
  const readmeFile = path.join(directory, "README.md");
  const cargoSource = `# version = "9.9.9"
[package]
name = "chord-progression-parser"
version = "0.9.4"

[dependencies.example]
version = "8.8.8"
`;
  try {
    await writeFile(cargoFile, cargoSource);
    await writeFile(readmeFile, README);
    await expect(
      updateReadmeVersionFile({ cargoFile, readmeFile, check: true }),
    ).rejects.toThrow("README CDN version is stale");
    expect(await readFile(readmeFile, "utf8")).toBe(README);
    expect(await readFile(cargoFile, "utf8")).toBe(cargoSource);

    await updateReadmeVersionFile({ cargoFile, readmeFile, check: false });
    const updated = updateReadmeVersion({ source: README, version: "0.9.4" });
    expect(await readFile(readmeFile, "utf8")).toBe(updated);
    await updateReadmeVersionFile({ cargoFile, readmeFile, check: true });
    await updateReadmeVersionFile({ cargoFile, readmeFile, check: false });
    expect(await readFile(readmeFile, "utf8")).toBe(updated);
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});

/** Invalid Cargo metadata must fail before modifying the README. */
test("preserves the README when Cargo has no valid package version", async (): Promise<void> => {
  const directory = await mkdtemp(
    path.join(os.tmpdir(), "chord-readme-manifest-"),
  );
  const cargoFile = path.join(directory, "Cargo.toml");
  const readmeFile = path.join(directory, "README.md");
  try {
    await writeFile(readmeFile, README);
    for (const source of [
      "[dependencies.example]\nversion = '0.9.4'\n",
      "[package]\nname = 'parser'\n",
      "[package]\nversion = 3\n",
      "[package]\nversion = '0.9.4/other'\n",
      "[package\nversion = '0.9.4'\n",
    ]) {
      await writeFile(cargoFile, source);
      await expect(
        updateReadmeVersionFile({ cargoFile, readmeFile, check: false }),
      ).rejects.toThrow();
      expect(await readFile(readmeFile, "utf8")).toBe(README);
    }
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});
