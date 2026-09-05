#!/usr/bin/env python3
"""Record exact input hashes without including internal source research."""
import hashlib
import json
from pathlib import Path
import sys

out, work, scenario = map(Path, sys.argv[1:])

def hashes(root):
    return {
        str(path.relative_to(root)): hashlib.sha256(path.read_bytes()).hexdigest()
        for path in sorted(root.rglob('*')) if path.is_file() and 'target' not in path.parts
    }

inputs = {'start': hashes(work), 'prompt': hashlib.sha256((out / 'prompt.txt').read_bytes()).hexdigest()}
if (out / 'idiomatic-rust').exists():
    inputs['skill'] = hashes(out / 'idiomatic-rust')
if (scenario / 'oracle.rs').exists():
    inputs['oracle'] = hashlib.sha256((scenario / 'oracle.rs').read_bytes()).hexdigest()
(out / 'inputs.json').write_text(json.dumps(inputs, indent=2) + '\n')
