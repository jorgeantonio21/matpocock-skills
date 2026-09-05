"""A result event alone must not turn an interrupted evaluation into success."""

import importlib.util
import json
from pathlib import Path
import sys
import tempfile
import unittest

SPEC = importlib.util.spec_from_file_location("eval_analyze", Path(__file__).with_name("analyze.py"))
analyze = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = analyze
SPEC.loader.exec_module(analyze)


class CompletionTests(unittest.TestCase):
    def test_successful_result(self):
        self.assertTrue(analyze.Transcript(result={"type": "result", "subtype": "success", "is_error": False}).finished())

    def test_budget_limit_is_incomplete(self):
        self.assertFalse(analyze.Transcript(result={"type": "result", "subtype": "error_max_budget_usd", "is_error": False}).finished())

    def test_api_error_is_incomplete(self):
        self.assertFalse(analyze.Transcript(result={"type": "result", "subtype": "success", "is_error": True}).finished())

    def test_truncated_transcript_preserves_last_message_without_completion(self):
        with tempfile.TemporaryDirectory() as temp:
            path = Path(temp) / "transcript.jsonl"
            path.write_text(json.dumps({"type": "assistant", "message": {"content": [{"type": "text", "text": "still working"}]}}) + '\n{"type":')
            transcript = analyze.parse_transcript(path)
        self.assertFalse(transcript.finished())
        self.assertEqual(transcript.last_text, "still working")


if __name__ == "__main__":
    unittest.main()
