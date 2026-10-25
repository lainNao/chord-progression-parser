import { Extension } from "@lainnao/chord-progression-parser-node/generatedTypes";

/** Matches the parser's documented structured extension order without changing input spelling. */
export function getExtensionSortOrder(extension: Extension): number {
  switch (extension) {
    case Extension.One:
      return 0;
    case Extension.Two:
      return 1;
    case Extension.Three:
      return 2;
    case Extension.Four:
      return 3;
    case Extension.Five:
      return 4;
    case Extension.Six:
      return 5;
    case Extension.Seven:
      return 6;
    case Extension.MajorSeven:
      return 7;
    case Extension.Nine:
      return 8;
    case Extension.MajorNine:
      return 9;
    case Extension.Eleven:
      return 10;
    case Extension.MajorEleven:
      return 11;
    case Extension.Thirteen:
      return 12;
    case Extension.MajorThirteen:
      return 13;
    case Extension.Sus2:
      return 14;
    case Extension.Sus4:
      return 15;
    case Extension.FlatThree:
      return 16;
    case Extension.FlatFive:
      return 17;
    case Extension.SharpFive:
      return 18;
    case Extension.FlatSix:
      return 19;
    case Extension.FlatNine:
      return 20;
    case Extension.SharpNine:
      return 21;
    case Extension.FlatEleven:
      return 22;
    case Extension.SharpEleven:
      return 23;
    case Extension.FlatThirteen:
      return 24;
    case Extension.SharpThirteen:
      return 25;
    case Extension.Add9:
      return 26;
    case Extension.Add11:
      return 27;
    case Extension.Add13:
      return 28;
    case Extension.Omit5:
      return 29;
    case Extension.HalfDiminish:
      return 30;
    default: {
      const exhaustiveCheck: never = extension;
      throw new Error(`Unexpected extension: ${exhaustiveCheck}`);
    }
  }
}
