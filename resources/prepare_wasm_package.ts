import { access, readFile, writeFile } from "node:fs/promises";
import path from "node:path";

type PackageTarget = "web" | "node" | "bundler";

type PackageJson = Record<string, unknown> & {
  files: string[];
};

type PrepareWasmPackageArgs = {
  packageDirectory: string;
  target: PackageTarget;
};

const ADDITIONAL_PACKAGE_FILES = [
  "error_code_message_map.js",
  "error_code_message_map.d.ts",
  "generatedTypes.js",
  "generatedTypes.d.ts",
] as const;

/** Parses and validates a package target supplied on the command line. */
function parsePackageTarget(value: string | undefined): PackageTarget {
  switch (value) {
    case "web":
    case "node":
    case "bundler":
      return value;
    default:
      throw new Error(`Expected package target to be web, node, or bundler: ${value}`);
  }
}

/** Returns the npm package name for a generated wasm target. */
function getPackageName(target: PackageTarget): string {
  switch (target) {
    case "web":
      return "@lainnao/chord-progression-parser-web";
    case "node":
      return "@lainnao/chord-progression-parser-node";
    case "bundler":
      return "@lainnao/chord-progression-parser-bundler";
    default: {
      const exhaustiveCheck: never = target;
      throw new Error(`Unsupported package target: ${exhaustiveCheck}`);
    }
  }
}

/** Parses package.json and verifies the fields required for post-processing. */
function parsePackageJson(source: string): PackageJson {
  const value: unknown = JSON.parse(source);

  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw new Error("Generated package.json must contain an object");
  }

  const packageJson = value as Record<string, unknown>;
  if (
    !Array.isArray(packageJson.files) ||
    !packageJson.files.every((file) => typeof file === "string")
  ) {
    throw new Error('Generated package.json must contain a string array in "files"');
  }

  return packageJson as PackageJson;
}

/** Ensures every manually generated file exists before adding it to package.json. */
async function verifyAdditionalFiles(packageDirectory: string): Promise<void> {
  await Promise.all(
    ADDITIONAL_PACKAGE_FILES.map((file) => access(path.join(packageDirectory, file))),
  );
}

/** Applies deterministic metadata to a wasm-pack generated package.json. */
export async function prepareWasmPackage({
  packageDirectory,
  target,
}: PrepareWasmPackageArgs): Promise<void> {
  const packageJsonPath = path.join(packageDirectory, "package.json");
  const packageJson = parsePackageJson(await readFile(packageJsonPath, "utf8"));

  await verifyAdditionalFiles(packageDirectory);

  packageJson.name = getPackageName(target);
  packageJson.files = [...new Set([...packageJson.files, ...ADDITIONAL_PACKAGE_FILES])];

  switch (target) {
    case "web":
    case "bundler":
      packageJson.type = "module";
      break;
    case "node":
      delete packageJson.type;
      break;
    default: {
      const exhaustiveCheck: never = target;
      throw new Error(`Unsupported package target: ${exhaustiveCheck}`);
    }
  }

  await writeFile(packageJsonPath, `${JSON.stringify(packageJson, null, 2)}\n`);
}

if (import.meta.main) {
  const target = parsePackageTarget(Bun.argv[2]);
  await prepareWasmPackage({
    packageDirectory: path.join(import.meta.dir, "..", "pkg", `pkg-${target}`),
    target,
  });
}
