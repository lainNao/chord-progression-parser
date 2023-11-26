import { test, expect } from "@playwright/test";

test("success simple usage", async ({ page }) => {
  await page.goto("http://localhost:3498/");

  // get result
  const resultText = await page.locator("#result").innerText();

  // check
  expect(JSON.parse(resultText)).toStrictEqual({
    success: true,
    ast: [
      {
        metaInfos: [],
        chordBlocks: [
          [
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
        ],
      },
    ],
  });
});
