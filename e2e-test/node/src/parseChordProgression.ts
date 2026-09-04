import parser from "@lainnao/chord-progression-parser-node";
import * as types from "@lainnao/chord-progression-parser-node/generatedTypes";
import {
  ErrorCode,
  getErrorMessage,
} from "@lainnao/chord-progression-parser-node/error_code_message_map";

type ParseChordProgressionResult =
  | {
      isOk: true;
      value: types.Ast;
    }
  | {
      isOk: false;
      errors: unknown[];
    };

export function parseChordProgression(
  chordProgressionString: string
): ParseChordProgressionResult {
  try {
    const result = parser.parseChordProgressionString(chordProgressionString);

    if (!result.success) {
      return {
        isOk: false,
        errors: result.errors.map(
          (error) =>
            getErrorMessage({
              errorCode: error.code as ErrorCode,
              lang: "ja",
            }) ?? "Unknown error"
        ),
      };
    }

    return {
      isOk: true,
      value: result.ast,
    };
  } catch (e: unknown) {
    console.log(e);
    return {
      isOk: false,
      errors: [e],
    };
  }
}
