# Lints

The mechanical layer of `idiomatic-rust`: the check command the agent runs on the crates a diff touches, the `#[expect]` policy, the rules the command retires from `SKILL.md`, and the same lint set as a workspace block for a repo that wants it in CI. Every lint name is verified against clippy 0.1.97 on Rust 1.97.1. The Calibration section at the end holds the evidence.

## The check command

Run once per crate the diff touches, then `cargo +nightly fmt -p <crate>`. Repeat `-p` for several crates. The `flags` array is shared by the three commands in this file, so the lint set is written once.

```bash
flags=(
  -D warnings
  -W clippy::pedantic -A clippy::similar_names -A clippy::must_use_candidate -A clippy::inline_always
  -D clippy::unwrap_used -D clippy::panic_in_result_fn
  -D clippy::unimplemented -W clippy::todo
  -D clippy::dbg_macro -D clippy::print_stdout -D clippy::print_stderr -D clippy::exit
  -D clippy::undocumented_unsafe_blocks -D clippy::allow_attributes_without_reason
  -D clippy::await_holding_lock -D clippy::large_futures
  -W unreachable_pub -W missing_debug_implementations -W unsafe_op_in_unsafe_fn
)
cargo clippy --no-deps -p <crate> --all-features -- "${flags[@]}"
```

What each part does:

- `--no-deps -p <crate>` confines linting to the named crate. Without `--no-deps`, clippy lints every workspace member in the crate's build graph, and a pedantic pass over a codebase that never ran one fails on the first dependency's pre-existing findings before the touched crate is reached.
- `-D warnings` makes every finding fail the run. The `-W` and `-D` spellings that follow carry into the workspace block below, where the difference between warn and deny matters.
- `-W clippy::pedantic` turns on the group that holds most of the rules in the table below.
- `-A clippy::similar_names`: pairs such as `job_a` and `job_b` are deliberate in code that compares two values of one kind, and the lint has no way to read intent.
- `-A clippy::must_use_candidate`: the lint wants `#[must_use]` on every pure function; the skill puts `#[must_use = "reason"]` only where dropping the value is a bug.
- `-A clippy::inline_always`: the lint's advice is to let the compiler decide; in latency-sensitive code `#[inline(always)]` is a measured decision the lint cannot see.
- The `-D` picks are restriction lints (plus `await_holding_lock` from suspicious and `large_futures` from pedantic) that are allow by default and never turned on as a group: `blanket_clippy_restriction_lints` fires on `-W clippy::restriction`. For a binary whose stdout is its product (a CLI tool, a report generator), add `-A clippy::print_stdout` to that crate's run; a service keeps the `-D`. For a binary crate that uses `pub` to organise its own modules, add `-A unreachable_pub` to that crate's run. `expect_used` is not in the set: `expect` with the invariant as the message is the form the skill asks for at an infallible site, and the lint would fire on that form and demand the same reason again in an `#[expect]` attribute. `unwrap_used` stays, so `unwrap` never passes.
- `unimplemented` is a `-D` and `todo` is a `-W`. Under `-D warnings` both fail the handback check, since finished code carries neither. The workspace block below keeps `todo` at warn, so a work-in-progress build in CI still compiles.
- The `-W` rustc lints: `unreachable_pub` asks for `pub(crate)` on items the crate never exports, `missing_debug_implementations` asks for `Debug` on every public type, and `unsafe_op_in_unsafe_fn` asks for an `unsafe` block around each unsafe operation inside an `unsafe fn` (already warn on edition 2024; the flag covers 2021 crates).

### When the crate has a backlog

A crate that never ran pedantic reports hundreds of pre-existing findings (see Calibration). The fix obligation is the files the diff touches; the rest is the crate's backlog, reported and left alone. This prints only the findings whose primary span sits in a changed file:

```bash
cargo clippy --no-deps -p <crate> --all-features --message-format=json -- "${flags[@]}" | jq -r --argjson files "$(git diff --name-only "$(git merge-base HEAD origin/main)" | jq -R . | jq -s .)" 'select(.reason=="compiler-message") | .message | select(.code != null) | select(any(.spans[]; .is_primary and (.file_name as $f | $files | index($f) != null))) | .rendered'
```

## Test targets

The command above checks the crate's library and binaries, so `#[cfg(test)]` code is not compiled and a test's `unwrap()` never reaches the lint. Test code gets a second run over every target with the two panic lints relaxed, because a test is where `unwrap` and `assert!` inside a `Result`-returning test are the idiom the Words section asks for:

```bash
cargo clippy --no-deps -p <crate> --all-targets --all-features -- "${flags[@]}" -A clippy::unwrap_used -A clippy::panic_in_result_fn
```

The two trailing `-A` flags override the `-D` for the same lints inside `flags`, because rustc applies lint flags in order and the last one wins.

The trade-off against stopping after the first run: the first run alone leaves tests, benches, and examples unlinted, and the second run costs one more build of the test targets. Run the second once the first is clean, since a failing library target cancels the targets that depend on it; both pass or the check fails.

A `clippy.toml` with `allow-unwrap-in-tests` would fold the two runs into one, at the price of a file in the target repo; the two-run form keeps the repo untouched.

## `#[expect]` policy

A finding is fixed, or the one item it fires on carries `#[expect(lint, reason = "...")]`: one lint per attribute, the attribute on the statement, function, or type the finding names, and a reason that states the invariant the lint cannot see. `expect` warns when the lint stops firing, so a suppression that outlives its cause removes itself; `allow_attributes_without_reason` in the command makes the reason mechanical.

The check command holds the only `-D warnings`, and source files set no lint level. A toolchain bump then arrives as findings to fix on the next run rather than as a broken build.

## What the command retires

Each rule below is enforced by the command. A row marked *reinforced* also keeps one sentence in `SKILL.md`, because an LLM writes the pattern even with the lint on. Every other row appears only here. A rule with no active lint stays in the skill: a clone to satisfy the borrow checker, `Arc::clone(&x)` spelled out, a `_` arm on a local enum, and five or fewer parameters (`too_many_arguments` fires only at eight). Level: `default` is a style, complexity, perf, or suspicious lint that is warn out of the box; `pedantic` arrives with `-W clippy::pedantic`; `pick` is a `-D` in the command; `rustc` is a `-W` in the command.

| Section | Rule | Lint | Level |
| --- | --- | --- | --- |
| Shape | Implement `From`, never `Into` | `from_over_into` | default |
| Shape | `Default` beside `new()`; derive what a hand-written impl would duplicate | `new_without_default`, `derivable_impls` | default |
| Shape | `Debug` on every public type | `missing_debug_implementations` | rustc |
| Shape | Three or more `bool` fields or parameters become an enum | `struct_excessive_bools`, `fn_params_excessive_bools` | pedantic |
| Shape | Box the outsized enum variant; alias a nested generic | `large_enum_variant`, `type_complexity` | default |
| Shape | `T::default()` when the type is known | `default_trait_access` | pedantic |
| Shape | `as_`, `to_`, `into_` receivers by convention | `wrong_self_convention` | default |
| Surface | An acronym as one word: `Uuid`, not `UUID` | `upper_case_acronyms` | default |
| Shape | One hundred or fewer lines | `too_many_lines` | pedantic |
| Shape | `Result` or `Option` only where a function can fail or be absent | `unnecessary_wraps` | pedantic |
| Ownership | `&str`, `&[T]`, `&T` parameters; owned only when consumed (reinforced) | `ptr_arg`, `borrowed_box`, `needless_pass_by_value` | default, pedantic |
| Ownership | Small `Copy` by value, large values by reference | `trivially_copy_pass_by_ref`, `large_types_passed_by_value` | pedantic |
| Ownership | `Option<&T>` over `&Option<T>` | `ref_option` | pedantic |
| Ownership | No `.clone()` on `Copy`; no `.to_owned()` of a value only borrowed | `clone_on_copy`, `unnecessary_to_owned`, `implicit_clone` | default, pedantic |
| Ownership | `clone_from` to reuse an allocation | `assigning_clones` | pedantic |
| Ownership | Borrow only where auto-deref does not | `needless_borrow`, `needless_borrows_for_generic_args` | default |
| Ownership | Elided lifetimes; no `'static` on `const` | `needless_lifetimes`, `redundant_static_lifetimes` | default |
| Ownership | `Option::take` and `mem::take` over `mem::replace` | `mem_replace_option_with_none`, `mem_replace_with_default` | default |
| Ownership | `.copied()` for `Copy` items | `cloned_instead_of_copied` | pedantic |
| Flow | End with the expression: no `return x;`, no `let x = e; x`, no late init | `needless_return`, `let_and_return`, `needless_late_init` | default |
| Flow | `?` over a `match` that returns `Err` or `None` | `question_mark`, `needless_question_mark` | default |
| Flow | `let ... else` over a `match` with a diverging arm | `manual_let_else` | pedantic |
| Flow | `map`, `ok_or`, `unwrap_or`, `map_or`, `is_some_and` over `match` | `manual_map`, `manual_ok_or`, `manual_unwrap_or`, `map_unwrap_or`, `manual_is_variant_and` | default, pedantic |
| Flow | `if let` over a one-arm `match`; `matches!` over a boolean `match`; `is_some()` over `if let Some(_)` | `single_match`, `single_match_else`, `match_like_matches_macro`, `redundant_pattern_matching` | default, pedantic |
| Flow | Positive branch first; no `else` after a diverging branch | `if_not_else`, `redundant_else`, `needless_continue` | pedantic |
| Flow | Identical arms merged with `\|`; nested or-patterns; the last variant named | `match_same_arms`, `unnested_or_patterns`, `match_wildcard_for_single_variants` | pedantic |
| Flow | `cond`, not `if cond { true } else { false }`; `usize::from(cond)` | `needless_bool`, `bool_comparison`, `bool_to_int_with_if` | default, pedantic |
| Flow | The std method: `contains`, `is_empty`, `strip_prefix`, `split_once`, `clamp`, `div_ceil`, `abs_diff` | `manual_range_contains`, `len_zero`, `manual_strip`, `manual_split_once`, `manual_clamp`, `manual_div_ceil`, `manual_abs_diff` | default |
| Flow | Iterate items: `for x in &v`, never `for i in 0..v.len()` or `.iter()` in a `for` | `needless_range_loop`, `explicit_iter_loop`, `explicit_into_iter_loop` | default, pedantic |
| Flow | A `for` loop over `for_each` with a block body | `needless_for_each` | pedantic |
| Flow | Pass the function, not a closure around it | `redundant_closure`, `redundant_closure_for_method_calls` | default, pedantic |
| Flow | `filter_map`, `find_map`, `flatten`, `.copied()` in chains | `manual_filter_map`, `manual_find_map`, `manual_flatten`, `map_flatten`, `filter_map_identity`, `map_clone`, `iter_cloned_collect`, `iter_overeager_cloned`, `flat_map_option` | default, pedantic |
| Flow | `unwrap` only in tests; elsewhere `?`, or `expect` with the invariant as the message at a provably infallible site (reinforced) | `unwrap_used` | pick |
| Flow | `?` and `Err`, never `panic!` or `assert!`, inside a `Result` function (`debug_assert!` and `unreachable!` pass) | `panic_in_result_fn` | pick |
| Flow | `assert!` over `if !cond { panic!() }` | `manual_assert` | pedantic |
| Flow | `dbg!`, `println!`, `eprintln!`, `process::exit` stay out of committed code | `dbg_macro`, `print_stdout`, `print_stderr`, `exit` | pick |
| Flow | `unimplemented!` and `todo!` stay out of committed code | `unimplemented`, `todo` | pick |
| Flow | `1_000_000`, not `1000000` | `unreadable_literal` | pedantic |
| Flow | `From` and `TryFrom` over `as` for numeric conversions | `cast_possible_truncation`, `cast_sign_loss`, `cast_possible_wrap`, `cast_precision_loss`, `cast_lossless` | pedantic |
| Flow | Semicolon after a unit statement; nested items before the first statement | `semicolon_if_nothing_returned`, `items_after_statements` | pedantic |
| Surface | Explicit imports, no globs | `wildcard_imports`, `enum_glob_use` | pedantic |
| Surface | `pub(crate)` on items the crate never exports | `unreachable_pub` | rustc |
| Surface | Inline format args; `String::new()`; `write!` over `push_str(&format!(..))` | `uninlined_format_args`, `manual_string_new`, `format_push_string`, `to_string_in_format_args`, `useless_format` | pedantic, default |
| Surface | An associated function when `self` is unused; `async` only with an `await` | `unused_self`, `unused_async` | pedantic |
| Surface | `#[must_use]` on builder methods, never on `()` | `return_self_not_must_use`, `must_use_unit`, `double_must_use` | pedantic, default |
| Surface | A `reason` on every `allow` and `expect` | `allow_attributes_without_reason` | pick |
| Surface | `// SAFETY:` above every `unsafe` block; an `unsafe` block around each unsafe op in an `unsafe fn` | `undocumented_unsafe_blocks`, `unsafe_op_in_unsafe_fn` | pick, rustc |
| Surface | Restriction lints picked one by one, never as a group | `blanket_clippy_restriction_lints` | default |
| Words | `# Errors`, `# Panics`, `# Safety` on the public functions that need them | `missing_errors_doc`, `missing_panics_doc`, `missing_safety_doc` | pedantic, default |
| Words | Identifiers in backticks inside doc comments | `doc_markdown` | pedantic |
| Runtime | No `std::sync` or `parking_lot` guard held across `.await` (reinforced, in `RUNTIME.md`) | `await_holding_lock` | pick |
| Runtime | A future too large to pass by value is boxed | `large_futures` | pick |
| Runtime | A lock guard is never bound to `_` | `let_underscore_lock` | default (deny) |

## Optional workspace block

The same set as manifest lints, for a repository that wants the check in CI. This block is an option the repository owner takes. It is never a step the skill takes. With the block in the root `Cargo.toml`, each member crate opts in with `[lints] workspace = true`, and the check command shrinks to `cargo clippy --no-deps -p <crate> --all-features -- -D warnings`. The block is formatted with `taplo fmt` (aligned entries, reordered keys).

```toml
[workspace.lints.rust]
missing_debug_implementations = "warn"
unreachable_pub               = "warn"
unsafe_op_in_unsafe_fn        = "warn"

[workspace.lints.clippy]
allow_attributes_without_reason = "deny"
await_holding_lock              = "deny"
dbg_macro                       = "deny"
exit                            = "deny"
inline_always                   = "allow"
large_futures                   = "deny"
must_use_candidate              = "allow"
panic_in_result_fn              = "deny"
pedantic                        = { level = "warn", priority = -1 }
print_stderr                    = "deny"
print_stdout                    = "deny"
similar_names                   = "allow"
todo                            = "warn"
undocumented_unsafe_blocks      = "deny"
unimplemented                   = "deny"
unwrap_used                     = "deny"
```

The test relaxation has no manifest form; a repo with the block adds `allow-unwrap-in-tests = true` to a `clippy.toml`, and `panic_in_result_fn` still needs the second run or an `#[expect]` per `Result`-returning test.

## Tools

- `cargo-nextest`: runs one process per test, so a hanging or leaking test is isolated and named.
- `cargo-machete`: finds unused dependencies on stable. Run it in CI.
- `cargo-expand`: shows what a derive emits. Expand every new derive once and read the output for an allocation or dynamic dispatch.

## Calibration

The commands were run on 2026-09-04 against Rust 1.97.1 (clippy 0.1.97). Every lint above resolves on that toolchain. To repeat the name check after a toolchain bump, run this with the `flags` array defined; it prints one line per flag the toolchain does not know, and nothing when all resolve:

```bash
rustup run 1.97.1 clippy-driver -W help | rg -o 'clippy::[a-z-]+' | sort -u > /tmp/clippy-lints
for f in "${flags[@]}"; do case $f in clippy::*) rg -qx "${f//_/-}" /tmp/clippy-lints || echo "unknown: $f";; esac; done
```

Three flags changed as a result of the first run. `module_name_repetitions` is not in the `-A` list, because it is a `restriction` lint since 1.93 and relaxing it under pedantic is a no-op. `mem_forget` is not in the `-D` picks, because in a workspace with zero-copy wire types every one of its findings came from a serialization derive expansion, none from hand-written code, and the lint does not skip external macros. `-A clippy::inline_always` was added, because a low-latency workspace had 110 deliberate uses in one crate.

On a scratch crate, the first command passes an iterator-chain function, fails with `clippy::unwrap-used` on a `parse().unwrap()` in library code, and ignores test code. The second command passes a test module that holds an `unwrap()` and an `assert!` inside a `Result`-returning test. `panic_in_result_fn` fires on `assert!` and `panic!`, not on `debug_assert!` or `unreachable!`. A `clippy.toml` with `allow-unwrap-in-tests` clears the test `unwrap` but not `panic_in_result_fn`, so a config file cannot replace the second run.

On a 40-crate workspace that never ran pedantic, the shipped set reported 254 findings in a small primitives crate (top: `doc_markdown` 51, `cast_lossless` 46, `missing_errors_doc` 35) and 407 in the largest domain crate (top: `missing_errors_doc` 65, `doc_markdown` 65, `expect_used` 53, `unwrap_used` 46; `expect_used` was in the set at the time). Each crate took under ten seconds once the dependencies were built. Without `--no-deps`, the run failed on two findings in a proc-macro crate in the build graph before it reached the named crate. The two binaries of the largest crate carried 82 `print_stdout` findings, which is the case the `-A` above is for. Those counts are why the check is diff-scoped. The filter under "When the crate has a backlog" printed 17 findings for one changed file.
