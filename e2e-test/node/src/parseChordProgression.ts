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
      error: unknown;
    };

export function parseChordProgression(
  chordProgressionString: string
): ParseChordProgressionResult {
  try {
    const result = parser.parseChordProgressionString(chordProgressionString);

    if (!result.success) {
      const errorMessage = getErrorMessage({
        errorCode: result.error.code as ErrorCode,
        lang: "ja",
      });
      return {
        isOk: false,
        error: errorMessage ?? "Unknown error",
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
      error: e,
    };
  }
}
