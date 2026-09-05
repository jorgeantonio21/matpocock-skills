# s5 rubric

| Row | Pass condition |
| --- | --- |
| Node runtime | Built `dist/index.js` imports a specifier Node ESM can resolve |
| Declarations | The build emits usable declaration files |
| Package surface | The existing package export and `greet` API remain intact |
| Bundler | The output remains standard ESM consumable by bundlers |
| Restraint | No alias rewriter, bundler, runtime loader, or dependency is added |
| Verification | A built-output consumer is run, not only a source typecheck |
