import { expect, test } from "bun:test";
import { readFileSync } from "node:fs";
import { Extension } from "@lainnao/chord-progression-parser-node/generatedTypes";
import { getExtensionSortOrder } from "./getExtensionSortOrder";

/** Checks every generator sort key against the specification rather than Rust's implementation. */
test("generator extension order matches the complete specification", (): void => {
  const fixture: { extensionOrder: Extension[] } = JSON.parse(
    readFileSync(
      new URL(
        "../../../../tests/fixtures/canonical_syntax.json",
        import.meta.url,
      ),
      "utf8",
    ),
  );
  const actual = Object.values(Extension).sort(
    (left, right): number =>
      getExtensionSortOrder(left) - getExtensionSortOrder(right),
  );
  expect(actual).toEqual(fixture.extensionOrder);
});
