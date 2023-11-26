import parser from "../../../pkg/pkg-node/chord_progression_parser";
import {
  ErrorCode,
  getErrorMessage,
  // Ignore the following, as it is not important here
  // @ts-ignore TS6059
} from "../../../pkg/pkg-node/error_code_message_map";

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
