/** Localized messages for non-blocking notation warnings. */
export const WARNING_CODE_MESSAGE_MAP = {
  DUPLICATE_EXTENSION: {
    en: "The same chord extension is specified more than once",
    ja: "同じコード拡張が複数回指定されています",
  },
} as const;

export type WarningCode = keyof typeof WARNING_CODE_MESSAGE_MAP;

/** Returns a localized message, or undefined for an unsupported runtime warning code. */
export function getWarningMessage({
  warningCode,
  lang,
}: {
  warningCode: WarningCode;
  lang: "en" | "ja";
}): string | undefined {
  if (!Object.hasOwn(WARNING_CODE_MESSAGE_MAP, warningCode)) {
    return undefined;
  }
  const messages = WARNING_CODE_MESSAGE_MAP[warningCode];
  return Object.hasOwn(messages, lang) ? messages[lang] : undefined;
}
