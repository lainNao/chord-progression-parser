import { write } from "bun";
import path from "path";
import {
  ERROR_CODE_MESSAGE_MAP,
  ErrorCode,
  ShortErrorGenre,
} from "./error_code_message_map";

function convertErrorCodeToPascalCase(str: string): string {
  return str
    .split("-")
    .map((word) => word[0].toUpperCase() + word.slice(1).toLowerCase())
    .join("");
}

type ErrorCodeSummary = {
  errorCode: string;
  description: {
    en: string;
    ja: string;
  };
};

function makeErrorCodeRsContent(
  errorCodeSummaries: ErrorCodeSummary[]
): string {
  return `// NOTE: Do not edit this file manually, it is generated by error_code_message_map.util.ts
use strum_macros::{Display, EnumString};

#[derive(Debug, Display, PartialEq, EnumString)]
pub enum ErrorCode {
${errorCodeSummaries
  .map(
    ({ errorCode, description }) =>
      `    #[strum(serialize = "${errorCode}")]\n` +
      `    /**\n` +
      `     * en: ${description.en}\n` +
      `     * ja: ${description.ja}\n` +
      `     */\n` +
      `    ${convertErrorCodeToPascalCase(errorCode)},\n`
  )
  .join("\n")}}

#[derive(Debug, PartialEq)]
pub struct ErrorInfo {
    pub code: ErrorCode,
    pub additional_info: Option<String>,
}

// implement to_string() for ErrorInfo
impl std::fmt::Display for ErrorInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let additional_info = match &self.additional_info {
            Some(info) => format!(": {}", info),
            None => "".to_string(),
        };
        write!(f, "{}{}", self.code, additional_info)
    }
}
`;
}

async function run() {
  const shortErrorGenres = Object.keys(
    ERROR_CODE_MESSAGE_MAP
  ) as ShortErrorGenre[];

  const errorSummaries = shortErrorGenres.reduce(
    (errorCodeSummaries, shortErrorGenre) => {
      const summaries = Object.keys(
        ERROR_CODE_MESSAGE_MAP[shortErrorGenre]
      ).map((errorCode: unknown) => {
        const description = (ERROR_CODE_MESSAGE_MAP[shortErrorGenre] as any)[
          errorCode as ErrorCode
        ];
        return {
          errorCode: errorCode as string,
          description,
        };
      });

      return errorCodeSummaries.concat(summaries);
    },
    [] as ErrorCodeSummary[]
  );

  const fileContent = makeErrorCodeRsContent(errorSummaries);

  // write to file "src/error_code.rs"
  const dir = import.meta.dir;
  await write(path.join(dir, "../src/error_code.rs"), fileContent);
}

run();
