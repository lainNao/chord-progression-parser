import "./main.css";
import { parseChordProgressionString } from "@lainnao/chord-progression-parser-bundler/chord_progression_parser";
import {
  ErrorCode,
  getErrorMessage,
} from "@lainnao/chord-progression-parser-bundler/error_code_message_map";

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

function createErrorMessage({
  currentValue,
  errorCode,
  lineNumber,
  columnNumber,
  length,
}: {
  currentValue: string;
  errorCode: ErrorCode;
  lineNumber: number;
  columnNumber: number;
  length: number;
}) {
  return (
    "" +
    `${lineNumber}行目: ` +
    getErrorMessage({
      errorCode,
      lang: "ja",
    }) +
    `(${errorCode})\n` +
    "\n" +
    getHighlightedTextLines({
      text: currentValue,
      row: lineNumber - 1,
      col: columnNumber - 1,
      length: length,
    })[lineNumber - 1]
  );
}

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
      const result = parseChordProgressionString(value);
      const end = performance.now();
      console.info(result);

      // time
      elms.time.innerHTML = `${((end - start) * 0.001).toFixed(5)}sec`;

      // result
      elms.result.innerHTML = result.success
        ? JSON.stringify(result, null, 2)
        : createErrorMessage({
            currentValue: value,
            errorCode: result.error.code as ErrorCode,
            lineNumber: result.error.position.lineNumber,
            columnNumber: result.error.position.columnNumber,
            length: result.error.position.length,
          });
    } catch (e: unknown) {
      console.log(e);
      elms.result.innerHTML = JSON.stringify(e, null, 2);
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
