# s2-refactor answer key

Thirteen patterns are planted in `start/src/lib.rs`. For each: where it is, the skill entry that names it, the rewrite the key expects, and whether the check command in `LINTS.md` catches it on its own (a pattern the lint catches tests the Check step; a pattern it does not catches tests the prose).

Mark each row `bare`, `skill`, `both`, or `neither`, and note a rewrite that differs from the key but is as good.

| # | Where | Entry | Expected rewrite | Lint catches it | Verdict | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | `add_note(title: &String, body: &String, ..)` | Ownership: Take what you use | `&str` parameters, or owned `String` since the function stores them | yes (`ptr_arg`) | | |
| 2 | `get_notes(&self) -> &Vec<Note>` | Surface: Names follow the std conventions | `notes(&self) -> &[Note]` | no | | |
| 3 | `find`: `match` on `Option` | Flow: Transform, do not match | `self.index.get(&id).map(\|&i\| &self.notes[i])` | yes (`manual_map`) | | |
| 4 | `titles`: index loop | Flow: Chain to build, loop to consume | `self.notes.iter().map(\|n\| n.title.clone()).collect()`, or return `impl Iterator<Item = &str>` | yes (`needless_range_loop`) | | |
| 5 | `Note::status: String` plus `set_status(.., status: &str)` with `panic!` on unknown | Shape: Make invalid states impossible | `enum Status { Open, Archived }` on the field and in the signature; the panic disappears because an invalid status cannot be passed | no | | |
| 6 | `count_by_tag(.., include_archived: bool)` | Shape: Make invalid states impossible | A two-variant enum parameter, or two functions | no (`fn_params_excessive_bools` needs three) | | |
| 7 | `archive_stale`: `let notes = self.notes.clone()` to iterate while mutating | Ownership: Clone only for a second owner | Collect the stale ids first (`Vec<NoteId>`), then mutate; or `iter_mut` with the status change inline | no | | |
| 8 | `archive_stale`: `SystemTime::now()` inside a state transition | Ownership: Time and randomness are inputs | `archive_stale(&mut self, now: SystemTime)`; the test passes a constant | no | | |
| 9 | `is_stale`: `use std::time::Duration;` inside the body | Surface: Imports at the top | Move the import to the top of the file | no | | |
| 10 | `is_stale`: `7 * 24 * 60 * 60` under a narrative comment about a design meeting | Flow: Name every number; Words: Comments say why | `const STALE_AFTER: Duration = Duration::from_secs(7 * 24 * 60 * 60);` or an associated const; the comment either states the current reason in one line or goes | no (`unreadable_literal` does not fire) | | |
| 11 | `newest(&self) -> Note` with `.unwrap().clone()` | Flow: A panic is a decision; Ownership: Take what you use | `newest(&self) -> Option<&Note>`; the empty store is a real case | yes (`unwrap_used`) | | |
| 12 | `load(..) -> Result<Store, Box<dyn Error>>`, `fs::read_to_string(path).unwrap()`, `StoreError::Parse(String)` with a `format!`ed message | Errors: Library errors use `thiserror`; Errors: Propagate every error; Errors: Application errors use `anyhow` (not in a library) | `Result<Store, LoadError>` with an `Io { path, source }` variant and a `Parse { line, .. }` variant carrying the line number as a field; `?` on the read; `&Path` parameter | partly (`unwrap_used` for the read; the rest is prose) | | |
| 13 | `Note::parent: u64` with the doc "0 means no parent" | Shape: Name the absence; Shape: Make invalid states impossible | `parent: Option<NoteId>` | no | | |

Secondary observations worth a note, not scored: whether ids became a `NoteId` newtype (Shape: The compiler is the guardrail); whether the `index: HashMap<u64, usize>` beside a `Vec<Note>` survived or became one map; whether test names gained the `test_` prefix and a comment deriving the expected value (Words: Tests read as sentences); whether the `Store::new()` next to a derivable `Default` was resolved (`new_without_default` fires); whether the agent ran the check command at all (see `meta.json`).
