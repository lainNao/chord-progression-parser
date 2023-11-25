type TestData = {
  input: string;
  expected: object;
};

export const testData: TestData = {
  input: `C`,
  expected: [
    {
      metaInfos: [],
      chordBlocks: [
        [
          {
            metaInfos: [],
            chordExpression: {
              type: "chord",
              value: {
                plain: "C",
                detailed: {
                  base: "C",
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
};
