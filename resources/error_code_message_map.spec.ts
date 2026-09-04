import { describe, expect, it } from "bun:test";
import {
  ERROR_CODE_MESSAGE_MAP,
  type ErrorCode,
  getErrorMessage,
} from "./error_code_message_map";

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

  it("describes delimiters and structural constraints accurately", () => {
    expect(ERROR_CODE_MESSAGE_MAP.CIMV["CIMV-3"]).toEqual({
      en: "Chord metadata must end with ']'",
      ja: "コードメタ情報の末尾に閉じ角括弧「]」が必要です",
    });
    expect(ERROR_CODE_MESSAGE_MAP.CHB["CHB-2"]).toEqual({
      en: "A bar must not contain a line break",
      ja: "小節内に改行を含めることはできません",
    });
  });

  it("resolves every declared error code in both languages", () => {
    for (const errorCodes of Object.values(ERROR_CODE_MESSAGE_MAP)) {
      for (const [errorCode, messages] of Object.entries(errorCodes)) {
        expect(getErrorMessage({ errorCode: errorCode as ErrorCode, lang: "en" })).toBe(
          messages.en
        );
        expect(getErrorMessage({ errorCode: errorCode as ErrorCode, lang: "ja" })).toBe(
          messages.ja
        );
        expect(messages.en.length).toBeGreaterThan(0);
        expect(messages.ja.length).toBeGreaterThan(0);
      }
    }
  });
});
