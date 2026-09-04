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

  const resultText = await page.locator("#result").innerText();
  expect(resultText.match(/CHO-1/g)).toHaveLength(2);
  expect(resultText.match(/EXT-1/g)).toHaveLength(2);
});

test("highlights UTF-16 ranges without interpreting source as HTML", async ({
  page,
}) => {
  await page.goto("http://localhost:3034/");

  await page.locator("#textarea").fill('😀\n<img src=x onerror="window.injected=1">');

  await expect(page.locator("#result mark")).toHaveText(["😀", "<img"]);
  await expect(page.locator("#result img")).toHaveCount(0);
  expect(await page.evaluate(() => Reflect.has(window, "injected"))).toBe(false);
});
