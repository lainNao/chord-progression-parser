import type { ParsedResult } from "@lainnao/chord-progression-parser-bundler";
import {
  Base,
  ChordType,
} from "@lainnao/chord-progression-parser-bundler/generatedTypes.js";

type TestData = {
  input: string;
  expected: ParsedResult;
};

export const testData: TestData = {
  input: `C`,
  expected: {
    success: true,
    warnings: [],
    sectionMetaPositions: [],
    ast: [
      {
        metaInfos: [],
        chordBlocks: [
          {
            type: "bar",
            value: [
              {
                metaInfos: [],
                denominator: null,
                chordExpression: {
                  type: "chord",
                  value: {
                    plain: "C",
                    detailed: {
                      base: Base.C,
                      accidental: null,
                      chordType: ChordType.Major,
                      extensions: [],
                    },
                  },
                },
              },
            ],
          },
        ],
      },
    ],
  },
};
