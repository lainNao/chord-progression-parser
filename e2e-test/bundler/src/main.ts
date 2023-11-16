import "./main.css";
import { parseChordProgressionString } from "../../../pkg/pkg-bundler/chord_progression_ast_parser";
import {
  ErrorCode,
  getErrorMessage,
} from "../../../pkg/pkg-bundler/error_code_message_map";
// import sampleChords from "./sample-chord-progression.txt?raw";

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
        if (typeof parsedError === "string") {
          elms.result.innerHTML = String(parsedError);
          return;
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
          elms.result.innerHTML = errorMessage ?? "Unknown error";
        }
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
