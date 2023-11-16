import { test, expect } from "@playwright/test";

test("success simple usage", async ({ page }) => {
  await page.goto("http://localhost:3498/");

  // get result
  const resultText = await page.locator("#result").innerText();

  // check
  expect(JSON.parse(resultText)).toStrictEqual([
    {
      metaInfos: [],
      chordBlocks: [
        [
          {
            metaInfos: [],
            chordExpression: {
              type: "chord",
              value: {
                plain: "C",
                detailed: {
                  base: "C",
                  chordType: "M",
                  extensions: [],
                },
              },
            },
          },
        ],
      ],
    },
  ]);
});
