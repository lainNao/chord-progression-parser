type TestData = {
  input: string;
  expected: object;
};

export const testData: TestData = {
  input: `C`,
  expected: {
    success: true,
    ast: [
      {
        metaInfos: [],
        chordBlocks: [
          [
            {
              metaInfos: [],
              denominator: null,
              chordExpression: {
                type: "chord",
                value: {
                  plain: "C",
                  detailed: {
                    base: "C",
                    accidental: null,
                    chordType: "M",
                    extensions: [],
                  },
                },
              },
            },
          ],
        ],
      },
    ],
  },
};
