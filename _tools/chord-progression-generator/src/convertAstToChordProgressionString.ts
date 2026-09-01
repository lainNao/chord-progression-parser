import * as types from "@lainnao/chord-progression-parser-node/generatedTypes";

/** Reports an unhandled generated AST variant to the type checker and runtime. */
function assertNever(value: never): never {
  throw new Error(`Unhandled AST variant: ${JSON.stringify(value)}`);
}

/** Converts one chord expression back to its source representation. */
function convertChordExpression(expression: types.ChordExpression): string {
  switch (expression.type) {
    case "noChord":
      return "_";
    case "same":
      return "%";
    case "unIdentified":
      return "?";
    case "chord":
      return expression.value.plain;
    default:
      return assertNever(expression);
  }
}

/** Converts one bar to source text. */
function convertBar(bar: types.Bar): string {
  return bar
    .map((chordInfo) => {
      const metaInfos = chordInfo.metaInfos.length
        ? `[${chordInfo.metaInfos
            .map((metaInfo) => `${metaInfo.type}=${metaInfo.value}`)
            .join(",")}]`
        : "";
      const denominator = chordInfo.denominator
        ? `/${chordInfo.denominator}`
        : "";

      return `${metaInfos}${convertChordExpression(
        chordInfo.chordExpression
      )}${denominator}`;
    })
    .join(" , ");
}

/** Converts chord blocks while preserving line breaks between bars. */
function convertChordBlocks(chordBlocks: types.ChordBlock[]): string {
  let source = "";
  let needsBarSeparator = false;

  for (const chordBlock of chordBlocks) {
    switch (chordBlock.type) {
      case "bar":
        source += `${needsBarSeparator ? " - " : ""}${convertBar(
          chordBlock.value
        )}`;
        needsBarSeparator = true;
        break;
      case "br":
        source += "\n";
        needsBarSeparator = false;
        break;
      default:
        assertNever(chordBlock);
    }
  }

  return source;
}

/** Converts a generated AST into parser input. */
export function convertAstToChordProgressionString(ast: types.Ast): string {
  return ast
    .map((section) => {
      const sectionMetaInfos =
        section.metaInfos.length > 0
          ? section.metaInfos
              .map((metaInfo) => `@${metaInfo.type}=${metaInfo.value}`)
              .join("\n") + "\n"
          : "";

      const chords = convertChordBlocks(section.chordBlocks);

      return `${sectionMetaInfos}${chords}`;
    })
    .join("\n\n");
}
