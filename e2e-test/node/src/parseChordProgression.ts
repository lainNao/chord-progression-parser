import parser from "../../../pkg/pkg-node/chord_progression_ast_parser";
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
    const astJson = parser.parseChordProgressionString(chordProgressionString);
    return {
      isOk: true,
      value: astJson,
    };
  } catch (e: unknown) {
    console.log(e);
    if (typeof e === "string") {
      const parsedError = JSON.parse(e);
      if (typeof parsedError === "string") {
        return {
          isOk: false,
          error: parsedError,
        };
      }

      if (
        typeof parsedError == "object" &&
        parsedError !== null &&
        "code" in parsedError
      ) {
        const errorMessage = getErrorMessage({
          errorCode: parsedError.code as ErrorCode,
          lang: "ja",
        });
        return {
          isOk: false,
          error: errorMessage ?? "Unknown error",
        };
      }
    }
    return {
      isOk: false,
      error: e,
    };
  }
}
