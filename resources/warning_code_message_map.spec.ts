import { describe, expect, it } from "bun:test";
import {
  WARNING_CODE_MESSAGE_MAP,
  type WarningCode,
  getWarningMessage,
} from "./warning_code_message_map";

describe("WARNING_CODE_MESSAGE_MAP", () => {
  /** Keeps runtime handling of unknown warning codes safe for JavaScript callers. */
  it("returns undefined for unsupported runtime warning codes and prototype keys", (): void => {
    expect(
      getWarningMessage({
        warningCode: "UNKNOWN_WARNING" as WarningCode,
        lang: "en",
      }),
    ).toBeUndefined();
    expect(
      getWarningMessage({
        warningCode: "toString" as WarningCode,
        lang: "name" as "en",
      }),
    ).toBeUndefined();
    expect(
      getWarningMessage({
        warningCode: "DUPLICATE_EXTENSION",
        lang: "toString" as "en",
      }),
    ).toBeUndefined();
  });

  it("resolves every declared warning code in both languages", (): void => {
    for (const [warningCode, messages] of Object.entries(
      WARNING_CODE_MESSAGE_MAP,
    )) {
      expect(
        getWarningMessage({
          warningCode: warningCode as WarningCode,
          lang: "en",
        }),
      ).toBe(messages.en);
      expect(
        getWarningMessage({
          warningCode: warningCode as WarningCode,
          lang: "ja",
        }),
      ).toBe(messages.ja);
      expect(messages.en.length).toBeGreaterThan(0);
      expect(messages.ja.length).toBeGreaterThan(0);
    }
  });
});
