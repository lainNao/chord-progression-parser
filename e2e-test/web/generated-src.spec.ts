import { test, expect } from "@playwright/test";
import type { Ast } from "@lainnao/chord-progression-parser-web/generatedTypes";

/** The direct browser package exports the same reference tag and localized syntax error. */
test("native browser WASM preserves references without allowing chord bodies", async ({
  page,
}): Promise<void> => {
  await page.goto("http://localhost:3498/");
  const result = await page.evaluate(
    async (): Promise<{
      readonly ast: Ast;
      readonly formatted: string;
      readonly codes: readonly string[];
      readonly message: string | undefined;
    }> => {
      const modulePath = "/chord_progression_parser.js";
      const parser = (await import(
        modulePath
      )) as typeof import("@lainnao/chord-progression-parser-web");
      await parser.default();
      const parsed = parser.parseChordProgressionString(
        "@same=サビ😀\n@repeat=2\n@section=Other\nC",
      );
      if (!parsed.success) throw new Error("unresolved references must parse");
      const invalid = parser.parseChordProgressionString("@same=サビ😀\nC");
      if (invalid.success) throw new Error("mixed chord bodies must fail");
      const messagesPath = "/error_code_message_map.js";
      const messages = (await import(
        messagesPath
      )) as typeof import("@lainnao/chord-progression-parser-web/error_code_message_map");
      return {
        ast: parsed.ast,
        formatted: parser.formatChordProgression(parsed.ast),
        codes: invalid.errors.map((error): string => error.code),
        message: messages.getErrorMessage({
          errorCode: invalid.errors[0].code,
          lang: "ja",
        }),
      };
    },
  );
  expect(result.ast).toHaveLength(2);
  expect(result.ast[0]).toEqual({
    metaInfos: [
      { type: "same", value: "サビ😀" },
      { type: "repeat", value: 2 },
    ],
    chordBlocks: [],
  });
  expect(result.formatted).toBe("@same=サビ😀\n@repeat=2\n\n@section=Other\nC");
  expect(result.codes).toEqual(["SMIV-4"]);
  expect(result.message).toBe(
    "「same」を指定したセクションにコード本文は記述できません",
  );
});

/** Direct browser imports retain blank lines without splitting the named section. */
test("native browser WASM keeps one blank line inside the current section", async ({
  page,
}): Promise<void> => {
  await page.goto("http://localhost:3498/");
  const result = await page.evaluate(async () => {
    const modulePath = "/chord_progression_parser.js";
    const parser = (await import(
      modulePath
    )) as typeof import("@lainnao/chord-progression-parser-web");
    await parser.default();
    const parsed = parser.parseChordProgressionString(
      "@section=Verse\n[key=C]C\n\n\n%",
    );
    if (!parsed.success)
      throw new Error("blank lines must not reset the previous chord");
    return {
      ast: parsed.ast,
      formatted: parser.formatChordProgression(parsed.ast),
    };
  });
  expect(result.ast).toHaveLength(1);
  expect(result.ast[0].metaInfos).toEqual([
    { type: "section", value: "Verse" },
  ]);
  expect(result.ast[0].chordBlocks.map((block): string => block.type)).toEqual([
    "bar",
    "br",
    "br",
    "bar",
  ]);
  expect(result.formatted).toBe("@section=Verse\n[key=C]C\n\n%");
});

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

/** Array-like inputs formerly allocated UTF-8 buffers before their encoding failed. */
test("rejects non-string parser inputs without growing WASM memory", async ({
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
        1n,
        Symbol("C"),
        {},
        [],
        ["C"],
        { length: 1 },
      ];

      /** Deliberately bypasses the string type to check rejection and allocator reuse. */
      function rejectInputs(iterations: number): void {
        let rejected = 0;
        for (let iteration = 0; iteration < iterations; iteration += 1) {
          for (const input of invalid) {
            try {
              Reflect.apply(parser.parseChordProgressionString, undefined, [
                input,
              ]);
            } catch (error) {
              if (
                String(error) !==
                "invalid chord progression input: expected a string"
              )
                throw error;
              rejected += 1;
            }
          }
        }
        if (rejected !== iterations * invalid.length) {
          throw new Error("The parser accepted a non-string input");
        }
      }

      rejectInputs(100);
      const before = wasm.memory.buffer.byteLength;
      rejectInputs(2_000);
      if (!parser.parseChordProgressionString("C").success) {
        throw new Error(
          "Parsing must still work after rejecting invalid arguments",
        );
      }
      return { before, after: wasm.memory.buffer.byteLength };
    },
  );
  expect(sizes.after).toBe(sizes.before);
});

/** Every nested array must reject ordinary wrong types without retaining reference slots. */
test("rejects malformed nested AST arrays without retaining WASM references", async ({
  page,
}): Promise<void> => {
  test.setTimeout(15_000);
  await page.goto("http://localhost:3498/");
  await expect(page.locator("#result")).toHaveAttribute("data-formatted", "C");
  const sizes = await page.evaluate(
    async (): Promise<{ before: number; after: number; formatted: string }> => {
      const modulePath = "/chord_progression_parser.js";
      const parser = (await import(
        modulePath
      )) as typeof import("@lainnao/chord-progression-parser-web");
      const wasm = await parser.default();
      const parsed = parser.parseChordProgressionString("C(9,9)-Dm(7)-G/B");
      if (!parsed.success) throw new Error("The AST fixture must parse");
      const paths = [
        [0, "metaInfos"],
        [0, "chordBlocks"],
        [0, "chordBlocks", 0, "value"],
        [0, "chordBlocks", 0, "value", 0, "metaInfos"],
        [
          0,
          "chordBlocks",
          0,
          "value",
          0,
          "chordExpression",
          "value",
          "detailed",
          "extensions",
        ],
      ];
      const invalidValues = [
        null,
        undefined,
        true,
        0,
        "bad",
        {},
        (): void => {},
      ];
      const invalid = paths.flatMap((path) =>
        invalidValues.map((value) => {
          const ast = structuredClone(parsed.ast);
          let parent: object = ast;
          for (const key of path.slice(0, -1)) {
            parent = Reflect.get(parent, key) as object;
          }
          Reflect.set(parent, path[path.length - 1], value);
          return ast;
        }),
      );

      /** Repeats the same malformed ASTs to distinguish allocator reuse from leakage. */
      function rejectInputs(iterations: number): void {
        let rejected = 0;
        for (let iteration = 0; iteration < iterations; iteration += 1) {
          for (const ast of invalid) {
            try {
              parser.formatChordProgression(ast);
            } catch (error) {
              if (!String(error).startsWith("invalid chord progression AST:"))
                throw error;
              rejected += 1;
            }
          }
        }
        if (rejected !== iterations * invalid.length) {
          throw new Error("The formatter accepted an invalid nested array");
        }
      }

      rejectInputs(30);
      const before = wasm.__wbindgen_externrefs.length;
      rejectInputs(500);
      return {
        before,
        after: wasm.__wbindgen_externrefs.length,
        formatted: parser.formatChordProgression(parsed.ast),
      };
    },
  );
  expect(sizes.after).toBe(sizes.before);
  expect(sizes.formatted).toBe("C(9,9) - Dm(7) - G/B");
});

/** Adjacent enum tags must remain independent of JavaScript property insertion order. */
test("formats ASTs whose enum values precede their tags", async ({
  page,
}): Promise<void> => {
  await page.goto("http://localhost:3498/");
  await expect(page.locator("#result")).toHaveAttribute("data-formatted", "C");
  const result = await page.evaluate(
    async (): Promise<{
      expected: string;
      actual: string;
    }> => {
      const modulePath = "/chord_progression_parser.js";
      const parser = (await import(
        modulePath
      )) as typeof import("@lainnao/chord-progression-parser-web");
      const parsed = parser.parseChordProgressionString(
        "@section=曲😀\n[key=Eb]C(1,9,9)-Dm(7)-G/B\n% - _ - ?",
      );
      if (!parsed.success) throw new Error("The AST fixture must parse");

      /** Changes object property order while preserving every array's element order. */
      function reverseProperties(value: unknown): unknown {
        if (Array.isArray(value)) return value.map(reverseProperties);
        if (value !== null && typeof value === "object") {
          return Object.fromEntries(
            Object.entries(value)
              .reverse()
              .map(([key, child]) => [key, reverseProperties(child)]),
          );
        }
        return value;
      }

      return {
        expected: parser.formatChordProgression(parsed.ast),
        actual: Reflect.apply(parser.formatChordProgression, undefined, [
          reverseProperties(parsed.ast),
        ]) as string,
      };
    },
  );
  expect(result.actual).toBe(result.expected);
});
