export function randomBetween(range: { min: number; max: number }): number {
  const min = range.min;
  const max = range.max;
  if (min > max) {
    throw new Error("min should be less than or equal to maxValue");
  }
  return Math.floor(Math.random() * (max - min + 1)) + min;
}
