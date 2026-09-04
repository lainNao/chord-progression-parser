import { expect, test } from "bun:test";
import { mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { prepareWasmPackage } from "./prepare_wasm_package";

type PackageTargetTestCase = {
  expectedName: string;
  expectedType: "module" | undefined;
  target: "web" | "node" | "bundler";
};

const PACKAGE_TARGET_TEST_CASES: PackageTargetTestCase[] = [
  {
    expectedName: "@lainnao/chord-progression-parser-web",
    expectedType: "module",
    target: "web",
  },
  {
    expectedName: "@lainnao/chord-progression-parser-node",
    expectedType: undefined,
    target: "node",
  },
  {
    expectedName: "@lainnao/chord-progression-parser-bundler",
    expectedType: "module",
    target: "bundler",
  },
];

const ADDITIONAL_PACKAGE_FILES = [
  "error_code_message_map.js",
  "error_code_message_map.d.ts",
  "generatedTypes.js",
  "generatedTypes.d.ts",
] as const;

for (const testCase of PACKAGE_TARGET_TEST_CASES) {
  test(`prepares ${testCase.target} package metadata`, async () => {
    const packageDirectory = await mkdtemp(
      path.join(os.tmpdir(), `prepare-wasm-package-${testCase.target}-`),
    );

    try {
      await writeFile(
        path.join(packageDirectory, "package.json"),
        JSON.stringify({
          files: [
            "chord_progression_parser.js",
            "error_code_message_map.js",
          ],
          name: "@lainnao/chord-progression-parser",
          type: "legacy-value",
          version: "1.2.3",
        }),
      );
      await Promise.all(
        ADDITIONAL_PACKAGE_FILES.map((file) =>
          writeFile(path.join(packageDirectory, file), ""),
        ),
      );

      await prepareWasmPackage({ packageDirectory, target: testCase.target });

      const packageJson = JSON.parse(
        await readFile(path.join(packageDirectory, "package.json"), "utf8"),
      );
      expect(packageJson).toMatchObject({
        files: ["chord_progression_parser.js", ...ADDITIONAL_PACKAGE_FILES],
        name: testCase.expectedName,
        version: "1.2.3",
      });
      expect(packageJson.type).toBe(testCase.expectedType);
    } finally {
      await rm(packageDirectory, { force: true, recursive: true });
    }
  });
}

test("rejects generated package metadata with an invalid files field", async () => {
  const packageDirectory = await mkdtemp(
    path.join(os.tmpdir(), "prepare-wasm-package-invalid-files-"),
  );

  try {
    await writeFile(
      path.join(packageDirectory, "package.json"),
      JSON.stringify({ files: "chord_progression_parser.js" }),
    );

    await expect(
      prepareWasmPackage({ packageDirectory, target: "web" }),
    ).rejects.toThrow('string array in "files"');
  } finally {
    await rm(packageDirectory, { force: true, recursive: true });
  }
});

test("rejects packages that are missing required generated files", async () => {
  const packageDirectory = await mkdtemp(
    path.join(os.tmpdir(), "prepare-wasm-package-missing-files-"),
  );

  try {
    await writeFile(
      path.join(packageDirectory, "package.json"),
      JSON.stringify({ files: ["chord_progression_parser.js"] }),
    );

    await expect(
      prepareWasmPackage({ packageDirectory, target: "bundler" }),
    ).rejects.toThrow();
  } finally {
    await rm(packageDirectory, { force: true, recursive: true });
  }
});
