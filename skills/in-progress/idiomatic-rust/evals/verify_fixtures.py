#!/usr/bin/env python3
"""Prove generic semantic tests reject planted bugs and accept minimal repairs."""

from pathlib import Path
import shutil
import subprocess
import tempfile

HERE = Path(__file__).resolve().parent
REPAIRS = {
    "s6-decoder": [
        ('pub struct Limit(u8);', '#[serde(try_from = "u8")]\npub struct Limit(u8);'),
        ('type Err = std::num::ParseIntError;\n    fn from_str(raw: &str) -> Result<Self, Self::Err> { raw.parse().map(Self) }',
         'type Err = String;\n    fn from_str(raw: &str) -> Result<Self, Self::Err> {\n        let value = raw.parse::<u8>().map_err(|error| error.to_string())?;\n        Self::try_from(value)\n    }'),
        ('pub fn decode(', 'impl TryFrom<u8> for Limit {\n    type Error = String;\n    fn try_from(value: u8) -> Result<Self, Self::Error> { Self::new(value).ok_or_else(|| format!("invalid limit {value}")) }\n}\n\npub fn decode('),
        ('NonZeroI32::new(-value.get())', 'value.get().checked_neg().and_then(NonZeroI32::new)'),
    ],
    "s7-inference": [
        ('while let Some(request) = self.queue.pop_front() {\n            if request.tokens <= self.max_new_tokens { return Some(request); }\n        }\n        None', 'self.queue.pop_front()'),
        ('self.queue.remove(index)?;\n        let children = self.queue.iter().find(|request| request.id == id)\n            .map(|request| request.children.clone()).unwrap_or_default();',
         'let children = self.queue.remove(index)?.children;'),
    ],
    "s8-cli": [
        ('[1 | 2, value] => Ok(Capacity::Bounded((*value).max(1))),',
         '[1, 0] => Ok(Capacity::Unlimited),\n        [1 | 2, value] if *value != 0 => Ok(Capacity::Bounded(*value)),')
    ],
}


def cargo(tree, *args, succeeds=True):
    run = subprocess.run(['cargo', '+1.97.1', *args], cwd=tree, text=True, capture_output=True)
    if (run.returncode == 0) != succeeds:
        raise RuntimeError(run.stdout + run.stderr)
    if not succeeds and 'test result: FAILED' not in run.stdout:
        raise RuntimeError('Expected semantic test failure, got infrastructure or compile failure:\n' + run.stdout + run.stderr)
    return run.stdout


def main():
    for name, repairs in REPAIRS.items():
        scenario = HERE / 'scenarios' / name
        cargo(scenario / 'start', 'generate-lockfile')
        with tempfile.TemporaryDirectory(prefix='rust-fixture-check-') as temp:
            tree = Path(temp) / 'tree'
            shutil.copytree(scenario / 'start', tree, ignore=shutil.ignore_patterns('target'))
            cargo(tree, 'test', '--locked', '--quiet')
            (tree / 'tests').mkdir(exist_ok=True)
            shutil.copyfile(scenario / 'oracle.rs', tree / 'tests/eval_contract.rs')
            cargo(tree, 'test', '--locked', '--quiet', succeeds=False)
            source = tree / 'src/lib.rs'
            text = source.read_text()
            for before, after in repairs:
                if text.count(before) != 1:
                    raise RuntimeError(f'{name}: repair no longer identifies exactly one planted bug')
                text = text.replace(before, after)
            source.write_text(text)
            stdout = cargo(tree, 'test', '--locked', '--quiet')
            print(f'{name}: original compiles; oracle rejects bugs; minimal repairs pass')
            for line in stdout.splitlines():
                if line.startswith('test result:'):
                    print('  ' + line)


if __name__ == '__main__':
    main()
