import "./main.css";
import { parseChordProgressionString } from "../../../pkg/pkg-bundler/chord_progression_ast_parser";
import {
  ErrorCode,
  getErrorMessage,
} from "../../../pkg/pkg-bundler/error_code_message_map";
// import sampleChords from "./sample-chord-progression.txt?raw";

const getHighlightedTextLines = ({
  text,
  row,
  col,
  length,
}: {
  text: string;
  row: number;
  col: number;
  length: number;
}): string[] => {
  const lines = text.split("\n");

  if (row < 0 || row >= lines.length || col < 0 || col >= lines[row].length) {
    return lines; // 範囲外の場合は元のテキストをそのまま返す
  }

  const start = Math.max(col, 0);
  const end = Math.min(col + length, lines[row].length);

  const beforeHighlight = lines[row].substring(0, start);
  const highlight = lines[row].substring(start, end);
  const afterHighlight = lines[row].substring(end);

  lines[row] = `${beforeHighlight}<mark>${highlight}</mark>${afterHighlight}`;

  return lines;
};

function main() {
  const elms = {
    textarea: document.querySelector<HTMLTextAreaElement>("#textarea")!,
    result: document.querySelector<HTMLTextAreaElement>("#result")!,
    time: document.querySelector<HTMLDivElement>("#time")!,
  };

  const applyValue = (value: string) => {
    try {
      const start = performance.now();
      elms.result.innerHTML = "";
      const ast = parseChordProgressionString(value);
      const end = performance.now();
      elms.time.innerHTML = `${((end - start) * 0.001).toFixed(5)}sec`;
      elms.result.innerHTML = JSON.stringify(ast, null, 2);
      console.info(ast);
    } catch (e: unknown) {
      console.log(e);
      if (typeof e === "string") {
        const parsedError = JSON.parse(e);

        // NOTE: This is a workaround for the case where the error is not a JSON string.
        if (typeof parsedError === "string") {
          elms.result.innerHTML = String(parsedError);
          return;
        }

        // NOTE: This is a controlled error case
        if (
          typeof parsedError == "object" &&
          parsedError !== null &&
          "code" in parsedError
        ) {
          // FIXME: generate and use appropriate guard
          if (parsedError.position) {
            elms.result.innerHTML =
              "" +
              `${parsedError.position.lineNumber}行目: ` +
              getErrorMessage({
                errorCode: parsedError.code as ErrorCode,
                lang: "ja",
              }) +
              `(${parsedError.code})\n` +
              "\n" +
              getHighlightedTextLines({
                text: elms.textarea.value,
                row: parsedError.position.lineNumber - 1,
                col: parsedError.position.columnNumber - 1,
                length: parsedError.position.length,
              })[parsedError.position.lineNumber - 1];
          }
        }
      } else {
        elms.result.innerHTML = "Unknown error";
      }
    }
  };

  const handleChange = (e: Event) => {
    if (!e?.target) return;
    if (!(e.target instanceof HTMLTextAreaElement)) return;
    applyValue(e.target.value);
  };

  elms.textarea.addEventListener("keyup", handleChange);
  elms.textarea.addEventListener("change", handleChange);
}

main();
