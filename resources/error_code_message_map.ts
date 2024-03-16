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
      en: "SectionMetaInfoKey is invalid",
      ja: "セクションメタ情報のキーが不正です",
    },
    "SMIK-2": {
      en: "SectionMetaInfoKey should not contains line break",
      ja: "セクションメタ情報のキーに改行を含めることはできません",
    },
  },
  SMIV: {
    "SMIV-1": {
      en: "SectionMetaInfoValue should not be empty",
      ja: "セクションメタ情報の値は空にできません",
    },
    "SMIV-2": {
      en: "SectionMetaInfoValue needs line break after",
      ja: "セクションメタ情報の値の後に改行が必要です",
    },
    "SMIV-3": {
      en: "SectionMetaInfoValue of repeat needs to be number",
      ja: "セクションメタ情報の値のrepeatの値は数値である必要があります",
    },
  },
  CIMK: {
    "CIMK-1": {
      en: "ChordInfoMetaKey should not contains line break",
      ja: "コードメタ情報のキーに改行を含めることはできません",
    },
    "CIMK-2": {
      en: "MetaInfoKey should not be empty",
      ja: "コードメタ情報のキーは空にできません",
    },
    "CIMK-3": {
      en: "MetaInfoKey is invalid",
      ja: "コードメタ情報のキーが不正です",
    },
  },
  CIMV: {
    "CIMV-1": {
      en: "MetaInfoValue should not contains line break",
      ja: "コードメタ情報の値に改行を含めることはできません",
    },
    "CIMV-2": {
      en: "MetaInfoValue should not be empty",
      ja: "コードメタ情報の値は空にできません",
    },
    "CIMV-3": {
      en: "MetaInfoValue needs close parenthesis after",
      ja: "コードメタ情報の値の後に閉じ括弧が必要です",
    },
    "CIMV-4": {
      en: "MetaInfoValue is invalid",
      ja: "コードメタ情報の値が不正です",
    },
  },
  CHB: {
    "CHB-1": {
      en: "% should not be placed first of Bar",
      ja: "コードブロックの先頭に%を置くことはできません",
    },
    "CHB-2": {
      en: "Bar should not contains line break",
      ja: "コードブロックに改行を含めることはできません",
    },
  },
  CHO: {
    "CHO-1": {
      en: "Invalid chord",
      ja: "コードが不正です",
    },
    "CHO-2": {
      en: "Chord should not contains line break",
      ja: "コードに改行を含めることはできません",
    },
    "CHO-3": {
      en: "Chord should not be empty",
      ja: "コードは空にできません",
    },
  },
  DEN: {
    "DEN-1": {
      en: "Invalid denominator",
      ja: "分母が不正です",
    },
    "DEN-2": {
      en: "Denominator is limited to one per chord",
      ja: "コードに対して分母は1つまでです",
    },
  },
  EXT: {
    "EXT-1": {
      en: "Invalid extension",
      ja: "テンションが不正です",
    },
    "EXT-2": {
      en: "Extension must not be empty",
      ja: "テンションは空にできません",
    },
    "EXT-3": {
      en: "Extension must be surrounded by parenthesis",
      ja: "テンションは括弧で囲む必要があります",
    },
    "EXT-4": {
      en: "No multiple extension parenthesis",
      ja: "テンションの括弧は1つまでです",
    },
  },
  TKN: {
    "TKN-1": {
      en: "Invalid token type",
      ja: "不正なトークンタイプです",
    },
  },
  BS: {
    "BS-1": {
      en: "Invalid base",
      ja: "不正なベース音です",
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
