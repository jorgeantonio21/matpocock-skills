#!/usr/bin/env python3
"""Check that every fenced Rust block in Markdown files is a verbatim excerpt of a source file.

Usage:
  snippets.py SOURCE_DIR MARKDOWN...

Every ```rust block in each MARKDOWN file must appear inside some .rs file under SOURCE_DIR,
byte for byte apart from one uniform indentation: a block written at column 0 in the document
may sit inside an impl, a function, or a nested module in the source. A fence indented inside a
Markdown list item is dedented by the fence's own indentation first. Prints one line per block
that is not found, and exits 1 when there is any. This keeps the examples in a reference
document identical to the code that compiles, passes its tests, and passes the lint check.
"""

import re
import sys
from pathlib import Path

FENCE = re.compile(
    r"^([ \t]*)```rust[ \t]*\n(.*?)^\1```[ \t]*$", re.MULTILINE | re.DOTALL
)
INDENTS = ("", "    ", "        ", "            ")


def blocks(markdown: Path) -> list[str]:
    """The Rust blocks in `markdown`, each dedented by its fence's indentation."""
    found = []
    for indent, body in FENCE.findall(markdown.read_text(encoding="utf-8")):
        lines = body.splitlines()
        found.append("\n".join(line.removeprefix(indent) for line in lines))
    return found


def indented(block: str, indent: str) -> str:
    """`block` with `indent` in front of every non-blank line."""
    return "\n".join(
        indent + line if line.strip() else line for line in block.splitlines()
    )


def found_in(block: str, sources: dict[Path, str]) -> bool:
    """Whether some source contains `block` under one uniform indentation."""
    return any(
        indented(block, indent) in text
        for text in sources.values()
        for indent in INDENTS
    )


def main() -> int:
    """Compare each fenced block against the concatenated sources."""
    if len(sys.argv) < 3:
        print(__doc__.strip(), file=sys.stderr)
        return 2
    source_dir = Path(sys.argv[1])
    sources = {
        path: path.read_text(encoding="utf-8")
        for path in sorted(source_dir.rglob("*.rs"))
    }
    status = 0
    for markdown in map(Path, sys.argv[2:]):
        all_blocks = blocks(markdown)
        missing = [block for block in all_blocks if not found_in(block, sources)]
        for block in missing:
            first_line = block.strip().splitlines()[0]
            print(
                f"{markdown}: no file under {source_dir} contains the block starting: {first_line}"
            )
        print(
            f"{markdown}: {len(all_blocks) - len(missing)} of {len(all_blocks)} Rust blocks found verbatim"
        )
        if missing:
            status = 1
    return status


if __name__ == "__main__":
    sys.exit(main())
