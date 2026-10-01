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
