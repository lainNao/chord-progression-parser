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
  "warning_code_message_map.js",
  "warning_code_message_map.d.ts",
  "generatedTypes.js",
  "generatedTypes.d.ts",
] as const;
const README =
  'import * as mod from "https://cdn.jsdelivr.net/npm/@lainnao/chord-progression-parser-web@0.9.3/chord_progression_parser.js";\n';

for (const testCase of PACKAGE_TARGET_TEST_CASES) {
  test(`prepares ${testCase.target} package metadata`, async () => {
    const packageDirectory = await mkdtemp(
      path.join(os.tmpdir(), `prepare-wasm-package-${testCase.target}-`),
    );

    try {
      await writeFile(
        path.join(packageDirectory, "package.json"),
        JSON.stringify({
          files: ["chord_progression_parser.js", "error_code_message_map.js"],
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
      await writeFile(path.join(packageDirectory, "README.md"), README);

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
      expect(
        await readFile(path.join(packageDirectory, "README.md"), "utf8"),
      ).toBe(README.replace("-web@0.9.3/", "-web@1.2.3/"));
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
      JSON.stringify({
        files: ["chord_progression_parser.js"],
        version: "1.2.3",
      }),
    );

    await expect(
      prepareWasmPackage({ packageDirectory, target: "bundler" }),
    ).rejects.toThrow();
  } finally {
    await rm(packageDirectory, { force: true, recursive: true });
  }
});

/** A broken CDN example must fail before modifying generated package metadata. */
test("rejects invalid package READMEs before changing package metadata", async (): Promise<void> => {
  const packageDirectory = await mkdtemp(
    path.join(os.tmpdir(), "prepare-wasm-package-invalid-readme-"),
  );
  const packageJsonPath = path.join(packageDirectory, "package.json");
  const metadata = JSON.stringify({
    files: ["chord_progression_parser.js"],
    version: "1.2.3",
  });
  try {
    await writeFile(packageJsonPath, metadata);
    await writeFile(
      path.join(packageDirectory, "README.md"),
      "# Missing CDN example",
    );
    await Promise.all(
      ADDITIONAL_PACKAGE_FILES.map((file) =>
        writeFile(path.join(packageDirectory, file), ""),
      ),
    );
    await expect(
      prepareWasmPackage({ packageDirectory, target: "web" }),
    ).rejects.toThrow("exactly one fixed web-package CDN import");
    expect(await readFile(packageJsonPath, "utf8")).toBe(metadata);
  } finally {
    await rm(packageDirectory, { force: true, recursive: true });
  }
});
