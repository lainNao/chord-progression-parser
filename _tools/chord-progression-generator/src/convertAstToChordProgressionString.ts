// @ts-ignore
import * as types from "../../../generatedTypes";

export function convertAstToChordProgressionString(ast: types.Ast): string {
  return ast
    .map((section) => {
      const sectionMetaInfos =
        section.metaInfos.length > 0
          ? section.metaInfos
              .map((metaInfo) => `@${metaInfo.type}=${metaInfo.value}`)
              .join("\n") + "\n"
          : "";

      const chords = section.chordBlocks
        .map((chordInfos) =>
          chordInfos
            .map((chordInfo) => {
              const metaInfos = !!chordInfo.metaInfos.length
                ? `[${chordInfo.metaInfos
                    .map((metaInfo) => `${metaInfo.type}=${metaInfo.value}`)
                    .join(",")}]`
                : "";

              const chord =
                chordInfo.chordExpression.type === "noChord"
                  ? "_"
                  : chordInfo.chordExpression.type === "same"
                  ? "%"
                  : chordInfo.chordExpression.type === "unidentified"
                  ? "?"
                  : chordInfo.chordExpression.type === "chord"
                  ? chordInfo.chordExpression.value.plain
                  : "ERROR!!!!!!!";
              const denominator = !!chordInfo.denominator
                ? `/${chordInfo.denominator}`
                : "";

              return `${metaInfos}${chord}${denominator}`;
            })
            .join(" , ")
        )
        .join(" - ");

      return `${sectionMetaInfos}${chords}`;
    })
    .join("\n\n");
}
