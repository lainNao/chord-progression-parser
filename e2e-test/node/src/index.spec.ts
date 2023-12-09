import { test, expect } from "bun:test";
import { Accidental } from "@lainnao/chord-progression-parser-node/generatedTypes";
import {
  ERROR_CODE_MESSAGE_MAP,
  getErrorMessage,
} from "@lainnao/chord-progression-parser-node/error_code_message_map";

test("can import", () => {
  expect(Accidental.Flat).toBeDefined();
  expect(ERROR_CODE_MESSAGE_MAP).toBeDefined();
  expect(getErrorMessage).toBeDefined();
});
