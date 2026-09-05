Bump this tool's configuration format to version 2.

- Version 2 writes `"workers"` as the word `"auto"` or a count from 1 to 64. Version 1 wrote `0` for auto; version 2 rejects `0`.
- `cfgtool validate <file>` accepts both versions. It prints `ok: version <n>, workers <auto or count>` on stdout and exits 0. A file that does not validate prints one line on stderr that names the file and the reason, and exits 2.
- Add `cfgtool migrate <in> <out>`, which writes the version 2 form of `<in>` to `<out>`. A version 1 `workers: 0` migrates to `"auto"`; a count stays a count; `listen` is unchanged. A file that does not validate is not migrated: nothing is written, and the exit code is 2. A version 2 input is written as it is.
- The files in `fixtures/` were written by shipped builds and must keep loading.

`cargo test` must pass when you finish. Write the code you would put in a pull request.
