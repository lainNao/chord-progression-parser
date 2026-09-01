import { formatChordProgression } from "@lainnao/chord-progression-parser-node";
import { generateRandomAst } from "./generateRandomAst";

/** Generates parser-compatible source through the public formatter. */
export function generateRandomChordProgressionString(): string {
  const randomAst = generateRandomAst({
    sectionCountRange: { min: 1, max: 10 },
    chordMetaInfoCountRange: { min: 0, max: 1 },
    chordBlockCountRange: { min: 6, max: 10 },
    chordInfoCountRange: { min: 1, max: 2 },
    extensionCountRange: { min: 0, max: 1 },
  });

  const randomChordProgressionString = formatChordProgression(randomAst);

  return randomChordProgressionString;
}
