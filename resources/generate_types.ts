import { mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";

type RequiredReplacement = {
  source: string;
  before: string;
  after: string;
};

/** Applies one expected generated-code replacement and fails if typeshare output changed. */
function replaceRequired({
  source,
  before,
  after,
}: RequiredReplacement): string {
  if (!source.includes(before)) {
    throw new Error(`Expected generated declaration was not found: ${before}`);
  }

  return source.replace(before, after);
}

/** Generates declarations in isolation, then checks or updates the committed file. */
export async function generateTypes({
  sourceDirectory,
  outputFile,
  check,
}: {
  sourceDirectory: string;
  outputFile: string;
  check: boolean;
}): Promise<void> {
  const directory = await mkdtemp(
    path.join(os.tmpdir(), "chord-parser-types-"),
  );
  try {
    const temporaryOutput = path.join(directory, "generatedTypes.ts");
    const child = Bun.spawn(
      [
        "typeshare",
        sourceDirectory,
        "--lang=typescript",
        "--output-file",
        temporaryOutput,
      ],
      { stdout: "pipe", stderr: "pipe" },
    );
    const [exitCode, stdout, stderr] = await Promise.all([
      child.exited,
      new Response(child.stdout).text(),
      new Response(child.stderr).text(),
    ]);
    if (exitCode !== 0) {
      throw new Error(`typeshare failed (${exitCode}):\n${stdout}${stderr}`);
    }

    let source = await readFile(temporaryOutput, "utf8");
    source = replaceRequired({
      source,
      before: "\tdenominator?: string;",
      after: "\tdenominator: string | null;",
    });
    source = replaceRequired({
      source,
      before: "\taccidental?: Accidental;",
      after: "\taccidental: Accidental | null;",
    });

    if (check) {
      if ((await readFile(outputFile, "utf8")) !== source) {
        throw new Error(
          "Generated TypeScript declarations are stale. Run make generate-ts-types.",
        );
      }
      return;
    }
    await writeFile(outputFile, source);
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
}

if (import.meta.main) {
  const args = Bun.argv.slice(2);
  if (args.length > 1 || (args.length === 1 && args[0] !== "--check")) {
    throw new Error("Usage: bun resources/generate_types.ts [--check]");
  }
  await generateTypes({
    sourceDirectory: path.join(import.meta.dir, "../src"),
    outputFile: path.join(import.meta.dir, "generatedTypes.ts"),
    check: args[0] === "--check",
  });
}
