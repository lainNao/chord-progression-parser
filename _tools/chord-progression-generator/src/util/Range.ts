export type Range = {
  min: number;
  max: number;
};

/** Rejects bounds that cannot describe an inclusive range of safe integers. */
export function assertIntegerRange(range: Range): void {
  if (!Number.isSafeInteger(range.min) || !Number.isSafeInteger(range.max)) {
    throw new RangeError("Range bounds must be safe integers");
  }
  if (range.min > range.max) {
    throw new RangeError("Range minimum must not exceed its maximum");
  }
  if (!Number.isSafeInteger(range.max - range.min + 1)) {
    throw new RangeError("Range width must be a safe integer");
  }
}
