#!/usr/bin/env python3
"""Check that every fenced Rust block in a Markdown file is a verbatim excerpt of a source file.

Usage:
  snippets.py MARKDOWN SOURCE_DIR

Every ```rust block in MARKDOWN must appear, byte for byte, inside some .rs file under SOURCE_DIR.
Prints one line per block that does not, and exits 1 when there is any. This keeps the examples in
a reference document identical to the code that compiles and passes its tests.
"""

import re
import sys
from pathlib import Path

FENCE = re.compile(r"^```rust\n(.*?)^```$", re.MULTILINE | re.DOTALL)


def main() -> int:
    """Compare each fenced block against the concatenated sources."""
    if len(sys.argv) != 3:
        print(__doc__.strip(), file=sys.stderr)
        return 2
    markdown, source_dir = Path(sys.argv[1]), Path(sys.argv[2])
    sources = {
        path: path.read_text(encoding="utf-8")
        for path in sorted(source_dir.rglob("*.rs"))
    }
    blocks = FENCE.findall(markdown.read_text(encoding="utf-8"))
    missing = [
        block
        for block in blocks
        if not any(block.rstrip("\n") in text for text in sources.values())
    ]
    for block in missing:
        first_line = block.strip().splitlines()[0]
        print(
            f"{markdown}: no file under {source_dir} contains the block starting: {first_line}"
        )
    print(
        f"{markdown}: {len(blocks) - len(missing)} of {len(blocks)} Rust blocks found verbatim"
    )
    return 1 if missing else 0


if __name__ == "__main__":
    sys.exit(main())
