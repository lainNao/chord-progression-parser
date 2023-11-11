import parser from "../../../pkg-node/chord_progression_ast_parser";

type ParseChordProgressionResult =
  | {
      isOk: true;
      value: parser.Ast;
    }
  | {
      isOk: false;
      error: unknown;
    };

export function parseChordProgression(
  chordProgressionString: string
): ParseChordProgressionResult {
  try {
    const astJson = parser.run(chordProgressionString);
    return {
      isOk: true,
      value: astJson,
    };
  } catch (e: unknown) {
    console.log(222, e);
    return {
      isOk: false,
      error: e,
    };
  }
}
