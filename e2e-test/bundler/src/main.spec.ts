import { test, expect } from "@playwright/test";
import { testData } from "./main.spec.fixtures";

test("success simple usage", async ({ page }) => {
  await page.goto("http://localhost:3034/");

  // input
  await page.locator("#textarea").pressSequentially(testData.input);

  // get result
  const resultText = await page.locator("#result").innerText();

  // check
  await expect(JSON.parse(resultText)).toStrictEqual(testData.expected);
});
