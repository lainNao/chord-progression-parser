import { convertAstToChordProgressionString } from "./convertAstToChordProgressionString";
import { generateRandomAst } from "./generateRandomAst";

export function generateRandomChordProgressionString(): string {
  const randomAst = generateRandomAst({
    sectionCountRange: { min: 1, max: 10 },
    chordMetaInfoCountRange: { min: 0, max: 1 },
    chordBlockCountRange: { min: 6, max: 10 },
    chordInfoCountRange: { min: 1, max: 2 },
    extensionCountRange: { min: 0, max: 1 },
  });

  const randomChordProgressionString =
    convertAstToChordProgressionString(randomAst);

  return randomChordProgressionString;
}
