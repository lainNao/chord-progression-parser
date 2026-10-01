import { expect, spyOn, test } from "bun:test";
import { getRandomElement } from "./getRandomElement";
import { getRandomEnum } from "./getRandomEnum";
import { randomBetween } from "./randomBetween";

/** Empty inputs must fail instead of returning undefined under a non-nullable type. */
test("rejects empty element and enum collections", (): void => {
  expect(() => getRandomElement([])).toThrow(RangeError);
  expect(() => getRandomEnum({})).toThrow(RangeError);
});

/** Verifies inclusive integer endpoints and invalid numeric boundaries. */
test("selects inclusive integer endpoints and rejects invalid ranges", (): void => {
  const random = spyOn(Math, "random").mockReturnValue(0);
  try {
    expect(randomBetween({ min: -3, max: 3 })).toBe(-3);
    expect(getRandomElement(["first", "last"])).toBe("first");
    expect(getRandomEnum({ First: "first", Last: "last" })).toBe("first");
    random.mockReturnValue(1 - Number.EPSILON);
    expect(randomBetween({ min: -3, max: 3 })).toBe(3);
    expect(getRandomElement(["first", "last"])).toBe("last");
    expect(getRandomEnum({ First: "first", Last: "last" })).toBe("last");
    expect(randomBetween({ min: 4, max: 4 })).toBe(4);
    for (const range of [
      { min: 1, max: 0 },
      { min: 0.5, max: 1 },
      { min: Number.NaN, max: 1 },
      { min: 0, max: Number.POSITIVE_INFINITY },
      { min: 0, max: Number.MAX_SAFE_INTEGER + 1 },
      { min: -Number.MAX_SAFE_INTEGER, max: Number.MAX_SAFE_INTEGER },
    ]) {
      expect(() => randomBetween(range)).toThrow(RangeError);
    }
  } finally {
    random.mockRestore();
  }
});
