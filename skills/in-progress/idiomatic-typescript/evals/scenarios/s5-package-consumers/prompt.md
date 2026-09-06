Prepare this ESM TypeScript library for its two declared consumers: Node loading the emitted JavaScript directly, and applications whose bundler consumes the package output.

Requirements:
- preserve the exported `greet` function and package export path
- make emitted JavaScript imports load in Node ESM
- emit declarations that a TypeScript consumer can use
- keep the package consumable by a bundler
- use the pinned compiler and add no dependency
- run the typecheck, build, tests, and a smoke import of the built package
