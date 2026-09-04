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
  await expect(page.locator("#result")).toHaveAttribute("data-formatted", "C");
});

test("renders every parser diagnostic", async ({ page }) => {
  await page.goto("http://localhost:3034/");

  const input = page.locator("#textarea");
  await input.fill("H,I-C(111,222)");
  await input.dispatchEvent("keyup");

  const resultText = await page.locator("#result").innerText();
  expect(resultText.match(/CHO-1/g)).toHaveLength(2);
  expect(resultText.match(/EXT-1/g)).toHaveLength(2);
});
