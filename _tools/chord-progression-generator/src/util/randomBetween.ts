import { assertIntegerRange, type Range } from "./Range";

/** Selects an integer from an inclusive range after validating its bounds. */
export function randomBetween(range: Range): number {
  assertIntegerRange(range);
  return Math.floor(Math.random() * (range.max - range.min + 1)) + range.min;
}
