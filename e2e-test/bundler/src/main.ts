import "./main.css";
import {
  formatChordProgression,
  parseChordProgressionString,
} from "@lainnao/chord-progression-parser-bundler/chord_progression_parser";
import {
  type ErrorCode,
  getErrorMessage,
} from "@lainnao/chord-progression-parser-bundler/error_code_message_map";

type ErrorElementArgs = {
  currentValue: string;
  endOffset: number;
  errorCode: ErrorCode;
  lineNumber: number;
  startOffset: number;
};

/** Returns the start and exclusive end offsets of the line containing a diagnostic. */
function getLineRange({
  source,
  startOffset,
}: {
  source: string;
  startOffset: number;
}): { endOffset: number; startOffset: number } {
  const lineStart =
    startOffset === 0 ? 0 : source.lastIndexOf("\n", startOffset - 1) + 1;
  const newlineOffset = source.indexOf("\n", startOffset);
  const rawLineEnd = newlineOffset === -1 ? source.length : newlineOffset;
  const lineEnd = source[rawLineEnd - 1] === "\r" ? rawLineEnd - 1 : rawLineEnd;

  return { endOffset: lineEnd, startOffset: lineStart };
}

/** Creates one diagnostic using text nodes so malformed source cannot become HTML. */
function createErrorElement({
  currentValue,
  endOffset,
  errorCode,
  lineNumber,
  startOffset,
}: ErrorElementArgs): HTMLDivElement {
  const container = document.createElement("div");
  const title = document.createElement("div");
  title.textContent = `${lineNumber}行目: ${getErrorMessage({
    errorCode,
    lang: "ja",
  })}(${errorCode})`;

  const source = document.createElement("div");
  const line = getLineRange({ source: currentValue, startOffset });
  const safeStart = Math.min(Math.max(startOffset, line.startOffset), line.endOffset);
  const safeEnd = Math.min(Math.max(endOffset, safeStart), line.endOffset);
  const mark = document.createElement("mark");
  mark.textContent = currentValue.slice(safeStart, safeEnd) || "▏";
  source.append(
    document.createTextNode(currentValue.slice(line.startOffset, safeStart)),
    mark,
    document.createTextNode(currentValue.slice(safeEnd, line.endOffset)),
  );

  container.append(title, source);
  return container;
}

/** Runs the interactive bundler example. */
function main(): void {
  const elms = {
    textarea: document.querySelector<HTMLTextAreaElement>("#textarea")!,
    result: document.querySelector<HTMLDivElement>("#result")!,
    time: document.querySelector<HTMLSpanElement>("#time")!,
  };

  /** Parses and renders the current source value. */
  const applyValue = (value: string): void => {
    try {
      const start = performance.now();
      const result = parseChordProgressionString(value);
      const end = performance.now();
      console.info(result);

      elms.time.textContent = `${((end - start) * 0.001).toFixed(5)}sec`;
      if (result.success) {
        elms.result.textContent = JSON.stringify(result, null, 2);
        elms.result.dataset.formatted = formatChordProgression(result.ast);
        return;
      }

      elms.result.replaceChildren(
        ...result.errors.map((error) =>
          createErrorElement({
            currentValue: value,
            endOffset: error.position.endOffset,
            errorCode: error.code as ErrorCode,
            lineNumber: error.position.lineNumber,
            startOffset: error.position.startOffset,
          }),
        ),
      );
      elms.result.dataset.formatted = "";
    } catch (error: unknown) {
      console.error(error);
      elms.result.textContent = JSON.stringify(error, null, 2);
    }
  };

  /** Revalidates after every direct edit, paste, or drop operation. */
  const handleInput = (event: Event): void => {
    if (!(event.target instanceof HTMLTextAreaElement)) return;
    applyValue(event.target.value);
  };

  elms.textarea.addEventListener("input", handleInput);
}

main();
