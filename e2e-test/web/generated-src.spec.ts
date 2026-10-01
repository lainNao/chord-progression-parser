import { test, expect } from "@playwright/test";

test("success simple usage", async ({ page }) => {
  await page.goto("http://localhost:3498/");
  await expect(page.locator("#result")).toHaveAttribute("data-formatted", "C");

  // get result
  const resultText = await page.locator("#result").innerText();

  // check
  expect(JSON.parse(resultText)).toStrictEqual({
    success: true,
    warnings: [],
    ast: [
      {
        metaInfos: [],
        chordBlocks: [
          {
            type: "bar",
            value: [
              {
                metaInfos: [],
                denominator: null,
                chordExpression: {
                  type: "chord",
                  value: {
                    plain: "C",
                    detailed: {
                      base: "C",
                      accidental: null,
                      chordType: "M",
                      extensions: [],
                    },
                  },
                },
              },
            ],
          },
        ],
      },
    ],
  });
});

/** Imports localized warnings as native browser modules from the generated package. */
test("imports warning messages without a bundler", async ({
  page,
}): Promise<void> => {
  await page.goto("http://localhost:3498/");

  const messages = await page.evaluate(
    async (): Promise<{ ja: string; en: string }> => {
      const modulePath = "/warning_code_message_map.js";
      const { getWarningMessage } = await import(modulePath);
      return {
        ja: getWarningMessage({
          warningCode: "DUPLICATE_EXTENSION",
          lang: "ja",
        }),
        en: getWarningMessage({
          warningCode: "DUPLICATE_EXTENSION",
          lang: "en",
        }),
      };
    },
  );

  expect(messages).toEqual({
    ja: "同じコード拡張が複数回指定されています",
    en: "The same chord extension is specified more than once",
  });
});

/** Invalid top-level values must be rejected without retaining a WASM reference slot per call. */
test("rejects non-array ASTs without growing the WASM reference table", async ({
  page,
}): Promise<void> => {
  await page.goto("http://localhost:3498/");
  await expect(page.locator("#result")).toHaveAttribute("data-formatted", "C");
  const sizes = await page.evaluate(
    async (): Promise<{ before: number; after: number }> => {
      const modulePath = "/chord_progression_parser.js";
      const parser = (await import(
        modulePath
      )) as typeof import("@lainnao/chord-progression-parser-web");
      const wasm = await parser.default();
      const invalid = [
        null,
        undefined,
        0,
        true,
        "C",
        Symbol("C"),
        1n,
        {},
        new Set(),
      ];

      /** Checks rejection while deliberately calling outside the TypeScript AST contract. */
      function rejectInputs(iterations: number): void {
        let rejected = 0;
        for (let iteration = 0; iteration < iterations; iteration += 1) {
          for (const input of invalid) {
            try {
              Reflect.apply(parser.formatChordProgression, undefined, [input]);
            } catch (error) {
              if (!String(error).startsWith("invalid chord progression AST:"))
                throw error;
              rejected += 1;
            }
          }
        }
        if (rejected !== iterations * invalid.length) {
          throw new Error("The formatter accepted a non-array AST");
        }
      }

      rejectInputs(100);
      const before = wasm.__wbindgen_externrefs.length;
      rejectInputs(2_000);
      return { before, after: wasm.__wbindgen_externrefs.length };
    },
  );
  expect(sizes.after).toBe(sizes.before);
});
