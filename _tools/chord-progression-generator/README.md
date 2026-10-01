# chord-progression-generator

Ready-made chord progression generator for internal testing use, like stress test, performance test, etc.

`bun run test` checks TypeScript types and runs both randomized and seeded round-trip tests.
Count ranges must contain safe integers. Bars and their chord lists require at least one
entry; section, metadata, and extension counts may start at zero. Invalid ranges and
empty random-choice collections throw `RangeError`.
