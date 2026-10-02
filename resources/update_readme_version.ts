import { readFile, writeFile } from "node:fs/promises";
import path from "node:path";

const WEB_PACKAGE_URL =
  "https://cdn.jsdelivr.net/npm/@lainnao/chord-progression-parser-web";
const CDN_IMPORT_URL =
  /https:\/\/cdn\.jsdelivr\.net\/npm\/@lainnao\/chord-progression-parser-web@([^/\s"'`]+)\/chord_progression_parser\.js(?=[\s"'`]|$)/g;

/** Rejects versions that cannot be used as a fixed Cargo package version. */
function assertPackageVersion(version: unknown): asserts version is string {
  if (typeof version !== "string") {
    throw new Error("Package version must be a valid semantic version");
  }
  const match = version.match(
    /^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-([0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*))?(?:\+[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?$/,
  );
  // Numeric prerelease identifiers also prohibit leading zeroes; build identifiers do not.
  if (
    match === null ||
    match[0] !== version ||
    match[4]
      ?.split(".")
      .some((identifier): boolean => /^0\d+$/.test(identifier))
  ) {
    throw new Error("Package version must be a valid semantic version");
  }
}

/** Reads the explicitly declared Cargo package version without interpreting comments as fields. */
function readPackageVersion(source: string): string {
  const manifest: unknown = Bun.TOML.parse(source);
  if (typeof manifest !== "object" || manifest === null) {
    throw new Error("Cargo.toml must contain a package table");
  }
  const packageTable = (manifest as Record<string, unknown>).package;
  if (
    typeof packageTable !== "object" ||
    packageTable === null ||
    Array.isArray(packageTable)
  ) {
    throw new Error("Cargo.toml must contain a package table");
  }
  const version = (packageTable as Record<string, unknown>).version;
  assertPackageVersion(version);
  return version;
}

/** Updates exactly one fixed CDN import while preserving historical version references. */
export function updateReadmeVersion({
  source,
  version,
}: {
  source: string;
  version: string;
}): string {
  assertPackageVersion(version);
  if (typeof source !== "string") {
    throw new Error("README source must be a string");
  }
  const imports = Array.from(source.matchAll(CDN_IMPORT_URL));
  if (imports.length !== 1) {
    throw new Error(
      "README must contain exactly one fixed web-package CDN import",
    );
  }
  assertPackageVersion(imports[0][1]);
  return source.replace(
    imports[0][0],
    `${WEB_PACKAGE_URL}@${version}/chord_progression_parser.js`,
  );
}

/** Updates a README from Cargo.toml, or reports drift without writing either file. */
export async function updateReadmeVersionFile({
  cargoFile,
  readmeFile,
  check,
}: {
  cargoFile: string;
  readmeFile: string;
  check: boolean;
}): Promise<void> {
  const [cargoSource, source] = await Promise.all([
    readFile(cargoFile, "utf8"),
    readFile(readmeFile, "utf8"),
  ]);
  const updated = updateReadmeVersion({
    source,
    version: readPackageVersion(cargoSource),
  });
  if (check) {
    if (updated !== source) {
      throw new Error(
        "README CDN version is stale. Run bun resources/update_readme_version.ts.",
      );
    }
    return;
  }
  if (updated !== source) {
    await writeFile(readmeFile, updated);
  }
}

if (import.meta.main) {
  const args = Bun.argv.slice(2);
  if (args.length > 1 || (args.length === 1 && args[0] !== "--check")) {
    throw new Error("Usage: bun resources/update_readme_version.ts [--check]");
  }
  await updateReadmeVersionFile({
    cargoFile: path.join(import.meta.dir, "../Cargo.toml"),
    readmeFile: path.join(import.meta.dir, "../README.md"),
    check: args[0] === "--check",
  });
}
