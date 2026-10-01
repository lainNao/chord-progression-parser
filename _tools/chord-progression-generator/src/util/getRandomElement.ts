/** Selects one existing element and rejects empty collections. */
export function getRandomElement<T>(elements: readonly T[]): T {
  if (elements.length === 0) {
    throw new RangeError(
      "Cannot select a random element from an empty collection",
    );
  }
  return elements[Math.floor(Math.random() * elements.length)];
}
