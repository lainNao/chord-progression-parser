// to use this file: make generate-error-code-rs
const ErrorGenreSummary = {
  SMIK: "SectionMetaInfo key",
  SMIV: "SectionMetaInfo value",
  CIMK: "ChordInfoMeta key",
  CIMV: "ChordInfoMeta value",
  CHB: "ChordBlock",
  CHO: "Chord",
  DEN: "Denominator",
  EXT: "Extension",
  TKN: "Token",
  BS: "Base",
  BL: "BreakLine",
  OTHER: "Other",
} as const;

export type ShortErrorGenre = keyof typeof ErrorGenreSummary;

export type ErrorCodeFor<Genre extends ShortErrorGenre> = `${Genre}-${number}`;

type UniqueErrorCodeAndMessageMap = {
  [key in ShortErrorGenre]: {
    [k in ErrorCodeFor<key>]: {
      en: string;
      ja: string;
    };
  };
};

export const ERROR_CODE_MESSAGE_MAP = {
  SMIK: {
    "SMIK-1": {
      en: "Invalid section metadata key",
      ja: "セクションメタ情報のキーが不正です",
    },
    "SMIK-2": {
      en: "Section metadata key is missing or is not followed by '='",
      ja: "セクションメタ情報のキーがないか、キーの後に「=」がありません",
    },
  },
  SMIV: {
    "SMIV-1": {
      en: "Section metadata value must not be empty",
      ja: "セクションメタ情報の値は空にできません",
    },
    "SMIV-2": {
      en: "Section metadata must occupy its own line",
      ja: "セクションメタ情報は専用の行に記述する必要があります",
    },
    "SMIV-3": {
      en: "The repeat section metadata value must be a non-negative integer",
      ja: "セクションメタ情報「repeat」の値は0以上の整数である必要があります",
    },
  },
  CIMK: {
    "CIMK-1": {
      en: "Chord metadata key must be followed by '='",
      ja: "コードメタ情報のキーの後に「=」が必要です",
    },
    "CIMK-2": {
      en: "Chord metadata key must not be empty",
      ja: "コードメタ情報のキーは空にできません",
    },
    "CIMK-3": {
      en: "Invalid chord metadata key",
      ja: "コードメタ情報のキーが不正です",
    },
  },
  CIMV: {
    "CIMV-1": {
      en: "Chord metadata value must not contain a line break",
      ja: "コードメタ情報の値に改行を含めることはできません",
    },
    "CIMV-2": {
      en: "Chord metadata value must not be empty",
      ja: "コードメタ情報の値は空にできません",
    },
    "CIMV-3": {
      en: "Chord metadata must end with ']'",
      ja: "コードメタ情報の末尾に閉じ角括弧「]」が必要です",
    },
    "CIMV-4": {
      en: "Invalid chord metadata value",
      ja: "コードメタ情報の値が不正です",
    },
  },
  CHB: {
    "CHB-1": {
      en: "The repeat symbol '%' cannot be used before the first chord in a section",
      ja: "セクション内の最初のコードより前に繰り返し記号「%」を置くことはできません",
    },
    "CHB-2": {
      en: "A bar must not contain a line break",
      ja: "小節内に改行を含めることはできません",
    },
  },
  CHO: {
    "CHO-1": {
      en: "Invalid chord notation",
      ja: "コードが不正です",
    },
    "CHO-2": {
      en: "A chord must not contain a line break",
      ja: "コードに改行を含めることはできません",
    },
    "CHO-3": {
      en: "A chord must not be empty",
      ja: "コードは空にできません",
    },
  },
  DEN: {
    "DEN-1": {
      en: "Invalid slash chord denominator",
      ja: "分母が不正です",
    },
    "DEN-2": {
      en: "A chord can have only one denominator",
      ja: "コードに対して分母は1つまでです",
    },
  },
  EXT: {
    "EXT-1": {
      en: "Invalid chord extension",
      ja: "テンションが不正です",
    },
    "EXT-2": {
      en: "Chord extension must not be empty",
      ja: "テンションは空にできません",
    },
    "EXT-3": {
      en: "Invalid chord extension parentheses",
      ja: "テンションの括弧が不正です",
    },
    "EXT-4": {
      en: "Multiple chord extension groups are not allowed",
      ja: "テンションを複数の括弧に分けることはできません",
    },
  },
  TKN: {
    "TKN-1": {
      en: "Unexpected token",
      ja: "予期しないトークンです",
    },
  },
  BS: {
    "BS-1": {
      en: "Invalid chord root",
      ja: "ルート音が不正です",
    },
  },
  BL: {
    "BL-1": {
      en: "Continuous blank lines are not allowed",
      ja: "連続した空行は許可されていません",
    },
  },
  OTHER: {
    "OTHER-1": {
      en: "Unknown error",
      ja: "不明なエラーです",
    },
  },
} as const satisfies UniqueErrorCodeAndMessageMap;

type ExtractKeys<T> = T extends T ? keyof T : never;

export type ErrorCode = ExtractKeys<
  (typeof ERROR_CODE_MESSAGE_MAP)[keyof typeof ERROR_CODE_MESSAGE_MAP]
>;

export function getErrorMessage({
  errorCode,
  lang,
}: {
  errorCode: ErrorCode;
  lang: "en" | "ja";
}): string | undefined {
  const [genreName, _] = errorCode.split("-") as [ShortErrorGenre, string];
  return (ERROR_CODE_MESSAGE_MAP[genreName] as any)?.[errorCode]?.[lang];
}
