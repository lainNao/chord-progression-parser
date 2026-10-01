import { getRandomElement } from "./getRandomElement";

/** Selects a value from one of the generated string enums without losing its type. */
export function getRandomEnum<T extends string>(
  enumObject: Readonly<Record<string, T>>,
): T {
  return getRandomElement(Object.values(enumObject));
}
