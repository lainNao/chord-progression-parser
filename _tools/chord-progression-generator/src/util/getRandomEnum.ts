export function getRandomEnum<T>(anEnum: T): T[keyof T] {
  // @ts-ignore
  const enumValues = Object.keys(anEnum).map((key) => anEnum[key]);
  const randomIndex = Math.floor(Math.random() * enumValues.length);
  return enumValues[randomIndex];
}
