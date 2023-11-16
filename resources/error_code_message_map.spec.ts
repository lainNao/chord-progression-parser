import { describe, expect, it, test } from "bun:test";
import { ERROR_CODE_MESSAGE_MAP } from "./error_code_message_map";

describe("ERROR_CODE_MESSAGE_MAP", () => {
  it("has unique error messages", () => {
    const englishErrorMessages = Object.values(ERROR_CODE_MESSAGE_MAP)
      .map((errorCodes) =>
        Object.values(errorCodes).map((errorMessage) => errorMessage.en)
      )
      .flat(Infinity);

    const japaneseErrorMessages = Object.values(ERROR_CODE_MESSAGE_MAP)
      .map((errorCodes) =>
        Object.values(errorCodes).map((errorMessage) => errorMessage.ja)
      )
      .flat(Infinity);

    expect(englishErrorMessages).toEqual([...new Set(englishErrorMessages)]);
    expect(japaneseErrorMessages).toEqual([...new Set(japaneseErrorMessages)]);
  });
});
