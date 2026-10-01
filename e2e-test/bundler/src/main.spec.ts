import { test, expect, type Route } from "@playwright/test";
import { testData } from "./main.spec.fixtures";
import type { ParseWarning } from "@lainnao/chord-progression-parser-bundler";
import { getWarningMessage } from "@lainnao/chord-progression-parser-bundler/warning_code_message_map.js";

/** Keeps text entered during WASM loading instead of waiting for another input event. */
test("parses input entered before WASM initialization", async ({
  page,
}): Promise<void> => {
  const pendingRequests: Route[] = [];
  await page.route("**/*.wasm", (route): void => {
    pendingRequests.push(route);
  });
  await page.goto("http://localhost:3034/", { waitUntil: "commit" });
  await page.locator("#textarea").fill("C(9,9)");
  await expect.poll((): number => pendingRequests.length).toBeGreaterThan(0);
  await expect(page.locator("#result")).toBeEmpty();
  for (const request of pendingRequests) await request.continue();
  await expect(page.locator("#result")).toHaveAttribute(
    "data-formatted",
    "C(9,9)",
  );
});

test("success simple usage", async ({ page }) => {
  await page.goto("http://localhost:3034/");

  // input
  await page.locator("#textarea").pressSequentially(testData.input);
  await expect(page.locator("#result")).toHaveAttribute("data-formatted", "C");

  // get result
  const resultText = await page.locator("#result").innerText();

  // check
  await expect(JSON.parse(resultText)).toStrictEqual(testData.expected);
});

/** Resolves localized messages for warnings returned by the bundled parser. */
test("resolves duplicate-extension warning messages", async ({
  page,
}): Promise<void> => {
  await page.goto("http://localhost:3034/");
  await page.locator("#textarea").fill("C(9,9)");
  await expect(page.locator("#result")).toHaveAttribute(
    "data-formatted",
    "C(9,9)",
  );

  const { warnings }: { warnings: ParseWarning[] } = JSON.parse(
    await page.locator("#result").innerText(),
  );
  expect(warnings).toHaveLength(1);
  expect(getWarningMessage({ warningCode: warnings[0].code, lang: "ja" })).toBe(
    "同じコード拡張が複数回指定されています",
  );
  expect(getWarningMessage({ warningCode: warnings[0].code, lang: "en" })).toBe(
    "The same chord extension is specified more than once",
  );
});

test("renders every parser diagnostic", async ({ page }) => {
  await page.goto("http://localhost:3034/");

  const input = page.locator("#textarea");
  await input.fill("H,I-C(111,222)");
  await expect(page.locator("#result mark")).toHaveText([
    "H",
    "I",
    "111",
    "222",
  ]);

  const resultText = await page.locator("#result").innerText();
  expect(resultText.match(/CHO-1/g)).toHaveLength(2);
  expect(resultText.match(/EXT-1/g)).toHaveLength(2);
});

/** Repeated errors on one long line must not duplicate that entire line for each diagnostic. */
test("bounds the surrounding text of diagnostics on a long line", async ({
  page,
}): Promise<void> => {
  await page.goto("http://localhost:3034/");
  await expect(page.locator("#result")).toContainText('"success": true');
  await page.addStyleTag({ content: "#textarea, #result { display: none; }" });
  await page
    .locator("#textarea")
    .evaluate((textarea: HTMLTextAreaElement): void => {
      textarea.value = "H,".repeat(2_000);
      textarea.dispatchEvent(new Event("input", { bubbles: true }));
    });
  const excerpts = await page
    .locator("#result > div > div:last-child")
    .allTextContents();
  expect(excerpts).toHaveLength(2_000);
  expect(excerpts.every((text): boolean => text.length < 400)).toBe(true);
  expect(excerpts[0]).toMatch(/^H,.*…$/);
  expect(excerpts[excerpts.length - 1]).toMatch(/^….*H,$/);
  await expect(page.locator("#result mark")).toHaveCount(2_000);
});

/** Truncation must keep complete UTF-16 pairs on both sides of the highlighted range. */
test("keeps Unicode context intact when shortening long diagnostic lines", async ({
  page,
}): Promise<void> => {
  await page.goto("http://localhost:3034/");
  for (const source of [`C/${"😀".repeat(81)},H`, `H,C/${"😀".repeat(81)}`]) {
    await page.locator("#textarea").fill(source);
    await expect(page.locator("#result mark")).toHaveText("H");
    const excerpt = await page
      .locator("#result > div > div:last-child")
      .textContent();
    expect(excerpt).toContain("…");
    expect(excerpt).toContain("😀");
    // Array.from iterates complete code points; a remaining surrogate is an unpaired half.
    expect(
      Array.from(excerpt!).some((point): boolean => {
        const code = point.codePointAt(0)!;
        return code >= 0xd800 && code <= 0xdfff;
      }),
    ).toBe(false);
  }
});

/** Short lines, EOF carets, and the complete invalid token remain visible after context trimming. */
test("preserves diagnostic ranges and line boundaries in excerpts", async ({
  page,
}): Promise<void> => {
  await page.goto("http://localhost:3034/");
  await page.locator("#textarea").fill("C\nH\nD");
  await expect(page.locator("#result > div > div:last-child")).toHaveText("H");

  await page.locator("#textarea").fill("C(");
  await expect(page.locator("#result mark")).toHaveText(["(", "▏"]);
  await expect(page.locator("#result > div > div:last-child")).toHaveText([
    "C(",
    "C(▏",
  ]);

  const invalidToken = "H".repeat(1_000);
  await page.locator("#textarea").fill(invalidToken);
  await expect(page.locator("#result mark")).toHaveText(invalidToken);
  await expect(page.locator("#result > div > div:last-child")).toHaveText(
    invalidToken,
  );
});

/** A failed render must show its error and discard the preceding formatted result. */
test("reports unexpected exceptions and recovers on the next input", async ({
  page,
}): Promise<void> => {
  await page.goto("http://localhost:3034/");
  await page.locator("#textarea").fill("C");
  await expect(page.locator("#result")).toHaveAttribute("data-formatted", "C");
  await page
    .locator("#textarea")
    .evaluate((textarea: HTMLTextAreaElement): void => {
      const originalNow = performance.now.bind(performance);
      // Fail once at the start of processing, then restore the browser clock immediately.
      performance.now = (): number => {
        performance.now = originalNow;
        throw new Error("Simulated processing failure");
      };
      textarea.dispatchEvent(new Event("input", { bubbles: true }));
    });
  await expect(page.locator("#result")).toHaveText(
    "Error: Simulated processing failure",
  );
  await expect(page.locator("#result")).toHaveAttribute("data-formatted", "");

  await page.locator("#textarea").fill("Dm(7)");
  await expect(page.locator("#result")).toHaveAttribute(
    "data-formatted",
    "Dm(7)",
  );
  await expect(page.locator("#result")).toContainText('"success": true');
});

/** Large result arrays must not become a single call with too many arguments. */
test("renders large diagnostic lists without exceeding argument limits", async ({
  page,
}): Promise<void> => {
  test.skip(
    test.info().project.name !== "chromium",
    "Run the large DOM stress case once",
  );
  test.setTimeout(60_000);
  await page.goto("http://localhost:3034/");
  await expect(page.locator("#result")).toContainText('"success": true');

  // Isolate diagnostic construction from textarea editing and large-page layout costs.
  await page.addStyleTag({ content: "#textarea, #result { display: none; }" });
  await page
    .locator("#textarea")
    .evaluate((textarea: HTMLTextAreaElement): void => {
      textarea.value = "H\n".repeat(150_000);
      textarea.dispatchEvent(new Event("input", { bubbles: true }));
    });
  await expect(page.locator("#result > div")).toHaveCount(150_000);
  await expect(page.locator("#result")).toHaveAttribute("data-formatted", "");
});

test("highlights UTF-16 ranges without interpreting source as HTML", async ({
  page,
}) => {
  await page.goto("http://localhost:3034/");

  await page
    .locator("#textarea")
    .fill('😀\n<img src=x onerror="window.injected=1">');

  await expect(page.locator("#result mark")).toHaveText(["😀", "<img"]);
  await expect(page.locator("#result img")).toHaveCount(0);
  expect(await page.evaluate(() => Reflect.has(window, "injected"))).toBe(
    false,
  );
});
