// @ts-ignore
import * as types from "../../../generatedTypes";
import { ChordType } from "../../../generatedTypes";
import { Range } from "./util/Range";
import { arrayBy } from "./util/arrayBy";
import { randomBetween } from "./util/randomBetween";

type GenerateRandomChordInfoArgs = {
  chordInfoCountRange: Range;
  extensionCountRange: Range;
};

type GenerateRandomSectionArgs = GenerateRandomChordInfoArgs & {
  chordBlockCountRange: Range;
  chordMetaInfoCountRange: Range;
};

type GenerateRandomAstArgs = GenerateRandomSectionArgs & {
  sectionCountRange: Range;
};

function generateRandomChordMetaInfo(): types.ChordInfoMeta {
  return {
    type: "key",
    value: types.Key.C_M,
  };
}

function generateRandomExtension(): types.Extension {
  return types.Extension.Seven;
}

function generateRandomDenominator(): string | undefined {
  return undefined;
}

function generateRandomChordExpression(
  args: GenerateRandomChordInfoArgs
): types.ChordExpression {
  const extensions = arrayBy(randomBetween(args.extensionCountRange)).map(() =>
    generateRandomExtension()
  );

  return {
    type: "chord",
    value: {
      plain: "C" + (extensions.length > 0 ? `(${extensions.join(",")})` : ""),
      detailed: {
        base: types.Base.C,
        accidental: undefined,
        chordType: ChordType.Major,
        extensions: extensions,
      },
    },
  };
}

function generateRandomChordBlock(
  args: GenerateRandomSectionArgs
): types.ChordBlock {
  return arrayBy(randomBetween(args.chordInfoCountRange)).map(() => ({
    metaInfos: arrayBy(randomBetween(args.chordMetaInfoCountRange)).map(() =>
      generateRandomChordMetaInfo()
    ),
    chordExpression: generateRandomChordExpression(args),
    denominator: generateRandomDenominator(),
  }));
}

function generateRandomSectionInfoMeta(): types.SectionMeta {
  return {
    // TODO: randomize
    type: "section",
    value: "A",
  };
}

function generateRandomSection(args: GenerateRandomSectionArgs): types.Section {
  return {
    metaInfos: arrayBy(randomBetween(args.chordMetaInfoCountRange)).map(() =>
      generateRandomSectionInfoMeta()
    ),
    chordBlocks: arrayBy(randomBetween(args.chordBlockCountRange)).map(() =>
      generateRandomChordBlock(args)
    ),
  };
}

function generateRandomAst(args: GenerateRandomAstArgs): types.Ast {
  return arrayBy(randomBetween(args.sectionCountRange)).map(() =>
    generateRandomSection(args)
  );
}

function convertAstToChordProgressionString(ast: types.Ast): string {
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

console.log(
  convertAstToChordProgressionString(
    generateRandomAst({
      sectionCountRange: { min: 1, max: 10 },
      chordMetaInfoCountRange: { min: 0, max: 1 },
      chordBlockCountRange: { min: 6, max: 10 },
      chordInfoCountRange: { min: 1, max: 2 },
      extensionCountRange: { min: 0, max: 1 },
    })
  )
);
