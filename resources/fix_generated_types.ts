import { readFile, writeFile } from "node:fs/promises";
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

const generatedTypesPath = path.join(import.meta.dir, "generatedTypes.ts");
let source = await readFile(generatedTypesPath, "utf8");

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

await writeFile(generatedTypesPath, source);
