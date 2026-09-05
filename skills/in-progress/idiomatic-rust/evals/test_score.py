#!/usr/bin/env python3
"""Exercise the shipped shell scorer with deterministic Cargo process fixtures."""

import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest

HERE = Path(__file__).resolve().parent


class ScorerTests(unittest.TestCase):
    def score(self, *, status=0, stdout='', stderr=''):
        with tempfile.TemporaryDirectory(prefix='rust-score-test-') as temp:
            root = Path(temp)
            evals = root / 'evals'
            evals.mkdir()
            shutil.copyfile(HERE / 'score.sh', evals / 'score.sh')
            shutil.copyfile(HERE.parent / 'LINTS.md', root / 'LINTS.md')
            tree = evals / 'results/s4-async/bare/tree'
            (tree / 'src').mkdir(parents=True)
            (tree / 'src/main.rs').write_text('fn main() {}\n')
            stub = root / 'cargo'
            stub.write_text('''#!/usr/bin/env python3
import json, os, sys
from pathlib import Path
with Path(os.environ['CARGO_CALLS']).open('a') as log:
    log.write(json.dumps(sys.argv[1:]) + '\\n')
if sys.argv[1] == 'test':
    print('test result: ok. 1 passed')
    sys.exit(0)
print(os.environ['CARGO_STDOUT'])
print(os.environ['CARGO_STDERR'], file=sys.stderr)
sys.exit(int(os.environ['CARGO_STATUS']))
''')
            stub.chmod(0o755)
            env = dict(os.environ, PATH=str(root) + os.pathsep + os.environ['PATH'],
                       CARGO_CALLS=str(root / 'calls'), CARGO_STDOUT=stdout,
                       CARGO_STDERR=stderr, CARGO_STATUS=str(status))
            result = subprocess.run(['bash', str(evals / 'score.sh'), 's4-async'],
                                    env=env, capture_output=True, text=True, check=True)
            calls = [json.loads(line) for line in (root / 'calls').read_text().splitlines()]
            log = (tree.parent / 'score.log').read_text()
            return result.stdout, calls, log

    def test_startup_failure_is_incomplete(self):
        output, _, log = self.score(status=1, stderr='compiler could not start')
        self.assertIn('INCOMPLETE', output)
        self.assertNotIn('0 ()', output)
        self.assertIn('compiler could not start', log)

    def test_empty_success_is_incomplete(self):
        output, _, _ = self.score()
        self.assertIn('INCOMPLETE', output)

    def test_invalid_json_is_incomplete(self):
        output, _, _ = self.score(status=1, stdout='{"reason":')
        self.assertIn('INCOMPLETE', output)

    def test_non_object_json_is_incomplete(self):
        output, _, _ = self.score(stdout='[]')
        self.assertIn('INCOMPLETE', output)

    def test_lint_without_build_completion_is_incomplete(self):
        event = {'reason': 'compiler-message', 'message': {
            'level': 'error', 'code': {'code': 'clippy::unwrap_used'},
            'message': 'unwrap used'}}
        output, _, _ = self.score(status=101, stdout=json.dumps(event))
        self.assertIn('INCOMPLETE', output)

    def test_rustc_errors_remain_build_failures(self):
        for code in ({'code': 'E0308'}, None):
            event = {'reason': 'compiler-message', 'message': {
                'level': 'error', 'code': code, 'message': 'invalid program'}}
            output, _, _ = self.score(status=101, stdout=json.dumps(event))
            self.assertIn('BUILD FAILED (rustc errors: 1)', output)

    def test_completed_lint_failure_is_counted(self):
        events = [
            {'reason': 'compiler-message', 'message': {
                'level': 'error', 'code': {'code': 'clippy::unwrap_used'},
                'message': 'unwrap used'}},
            {'reason': 'build-finished', 'success': False},
        ]
        output, _, log = self.score(status=101, stdout='\n'.join(map(json.dumps, events)))
        self.assertIn('1 (clippy::unwrap_used 1)', output)
        self.assertNotIn('INCOMPLETE', output)
        self.assertIn('unwrap used', log)

    def test_clean_build_and_stdout_exception_in_both_passes(self):
        output, calls, _ = self.score(stdout='{"reason":"build-finished","success":true}')
        self.assertIn('0 ()', output)
        clippy = [call for call in calls if call[0] == 'clippy']
        self.assertEqual(len(clippy), 2)
        for call in clippy:
            index = len(call) - 1 - call[::-1].index('clippy::print_stdout')
            self.assertEqual(call[index - 1], '-A')

    def test_warning_cannot_mask_failed_build(self):
        events = [
            {'reason': 'compiler-message', 'message': {
                'level': 'warning', 'code': {'code': 'unused_imports'},
                'message': 'unused import'}},
            {'reason': 'build-finished', 'success': False},
        ]
        output, _, _ = self.score(status=1, stdout='\n'.join(map(json.dumps, events)))
        self.assertIn('INCOMPLETE', output)


if __name__ == '__main__':
    unittest.main()
