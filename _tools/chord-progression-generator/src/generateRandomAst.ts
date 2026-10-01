import * as types from "@lainnao/chord-progression-parser-node/generatedTypes";
import { assertIntegerRange, type Range } from "./util/Range";
import { getRandomElement } from "./util/getRandomElement";
import { getRandomEnum } from "./util/getRandomEnum";
import { randomBetween } from "./util/randomBetween";

type GenerateRandomChordInfoArgs = GenerateRandomChordMetaInfoArgs & {
  chordInfoCountRange: Range;
  extensionCountRange: Range;
};

type GenerateRandomChordMetaInfoArgs = {
  chordMetaInfoCountRange: Range;
};

type GenerateRandomSectionArgs = GenerateRandomChordInfoArgs & {
  chordBlockCountRange: Range;
};

type GenerateRandomAstArgs = GenerateRandomSectionArgs & {
  sectionCountRange: Range;
};

/** Selects a supported extension spelling. */
function generateRandomExtension(): types.Extension {
  return getRandomEnum(types.Extension);
}

/** Occasionally generates an opaque chord-shaped denominator. */
function generateRandomDenominator(): string | null {
  if (randomBetween({ min: 0, max: 10 }) !== 0) {
    return null;
  }
  return (
    getRandomElement(["C", "D", "E", "F", "G", "A", "B"]) +
    getRandomElement(["", "", "", "", "m"]) +
    getRandomElement(["", "", "", "", `(${generateRandomExtension()})`])
  );
}

/** Generates an expression, permitting repetition only after a prior expression. */
function generateRandomChordExpression({
  extensionCountRange,
  hasPriorChord,
}: GenerateRandomChordInfoArgs & {
  hasPriorChord: boolean;
}): types.ChordExpression {
  switch (randomBetween({ min: 1, max: 20 })) {
    case 1:
      return {
        type: "noChord",
      };
    case 2:
      return {
        type: "unIdentified",
      };
    case 3: {
      if (hasPriorChord) {
        return {
          type: "same",
        };
      }
    }
    default: {
      const extensions = Array.from(
        { length: randomBetween(extensionCountRange) },
        () => generateRandomExtension(),
      );

      const chordType: types.ChordType = getRandomElement([
        ...Array<types.ChordType>(10).fill(types.ChordType.Major),
        ...Array<types.ChordType>(10).fill(types.ChordType.Minor),
        types.ChordType.Augmented,
        types.ChordType.Diminished,
      ]);
      const chordTypeString =
        chordType === types.ChordType.Major ? "" : chordType;

      const accidental = getRandomElement<types.Accidental | null>([
        ...Array<null>(10).fill(null),
        types.Accidental.Sharp,
        types.Accidental.Flat,
      ]);

      const base = getRandomEnum(types.Base);

      const plain =
        base +
        (accidental ?? "") +
        chordTypeString +
        (extensions.length > 0 ? `(${extensions.join(",")})` : "");

      return {
        type: "chord",
        value: {
          plain,
          detailed: {
            base,
            accidental,
            chordType,
            extensions,
          },
        },
      };
    }
  }
}

/** Occasionally attaches key metadata to the following chord. */
function generateRandomChordMetaInfos(
  args: GenerateRandomChordMetaInfoArgs,
): types.ChordInfoMeta[] {
  if (randomBetween({ min: 0, max: 10 }) !== 0) {
    return [];
  }

  return Array.from(
    { length: randomBetween(args.chordMetaInfoCountRange) },
    () => ({
      type: "key",
      value: getRandomEnum(types.Key),
    }),
  );
}

/** Generates a non-empty bar with valid repetition context for each expression. */
function generateRandomChordBlock({
  hasPriorChord,
  ...args
}: GenerateRandomSectionArgs & { hasPriorChord: boolean }): types.ChordBlock {
  return {
    type: "bar",
    value: Array.from(
      { length: randomBetween(args.chordInfoCountRange) },
      (_, index) => ({
        metaInfos: generateRandomChordMetaInfos(args),
        chordExpression: generateRandomChordExpression({
          ...args,
          hasPriorChord: hasPriorChord || index > 0,
        }),
        denominator: generateRandomDenominator(),
      }),
    ),
  };
}

/** Generates a supported section name or repeat count. */
function generateRandomSectionInfoMeta(): types.SectionMeta {
  // Prefer section names while also exercising numeric repeat metadata.
  const choice = randomBetween({ min: 0, max: 5 });
  if (choice > 0) {
    return {
      type: "section",
      value: getRandomElement(["A", "B", "C"]),
    };
  } else {
    return {
      type: "repeat",
      value: randomBetween({ min: 1, max: 2 }),
    };
  }
}

/** Generates section metadata and at least one non-empty bar. */
function generateRandomSection(args: GenerateRandomSectionArgs): types.Section {
  return {
    metaInfos: Array.from(
      { length: randomBetween(args.chordMetaInfoCountRange) },
      () => generateRandomSectionInfoMeta(),
    ),
    chordBlocks: Array.from(
      { length: randomBetween(args.chordBlockCountRange) },
      (_, index) =>
        generateRandomChordBlock({ ...args, hasPriorChord: index > 0 }),
    ),
  };
}

/** Validates every count range before generating a parser-compatible AST. */
export function generateRandomAst(args: GenerateRandomAstArgs): types.Ast {
  for (const [name, range] of Object.entries(args)) {
    assertIntegerRange(range);
    const minimum =
      name === "chordInfoCountRange" || name === "chordBlockCountRange" ? 1 : 0;
    if (range.min < minimum) {
      throw new RangeError(`${name} must start at ${minimum} or greater`);
    }
  }
  return Array.from({ length: randomBetween(args.sectionCountRange) }, () =>
    generateRandomSection(args),
  );
}
