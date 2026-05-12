import json
import tempfile
import unittest
from contextlib import redirect_stderr, redirect_stdout
from io import StringIO
from pathlib import Path

from mtgdeckbuilder.cli import main
from mtgdeckbuilder.ingest.normalization import format_normalization_report, normalize_payload


class NormalizeReportTests(unittest.TestCase):
    def test_valid_metadata_report(self):
        payload = normalize_payload(
            "scryfall",
            [{"name": "Report Card", "type_line": "Instant"}],
            input_path="input.json",
            output_path="output.json",
            generated_at="2026-05-12T00:00:00Z",
        )

        report = format_normalization_report(payload["metadata"])

        self.assertIn("Normalization Report\n", report)
        self.assertIn("schema_version: foundation-003.v1\n", report)
        self.assertIn("source: scryfall\n", report)
        self.assertIn("input_path: input.json\n", report)
        self.assertIn("output_path: output.json\n", report)
        self.assertIn("generated_at: 2026-05-12T00:00:00Z\n", report)
        self.assertIn("input_count: 1\n", report)
        self.assertIn("normalized_count: 1\n", report)
        self.assertIn("skipped_count: 0\n", report)
        self.assertIn("skipped_reasons:\n  none\n", report)

    def test_missing_metadata_failure(self):
        with tempfile.TemporaryDirectory() as tempdir:
            path = Path(tempdir) / "catalog.json"
            path.write_text(json.dumps({"cards": []}), encoding="utf-8")
            stderr = StringIO()

            with redirect_stderr(stderr):
                result = main(["normalize-report", str(path)])

        self.assertEqual(result, 2)
        self.assertIn("missing embedded metadata", stderr.getvalue())

    def test_malformed_catalog_failure(self):
        with tempfile.TemporaryDirectory() as tempdir:
            path = Path(tempdir) / "catalog.json"
            path.write_text(json.dumps({"metadata": {}}), encoding="utf-8")
            stderr = StringIO()

            with redirect_stderr(stderr):
                result = main(["normalize-report", str(path)])

        self.assertEqual(result, 2)
        self.assertIn("top-level 'cards' list", stderr.getvalue())

    def test_deterministic_output_ordering(self):
        metadata = {
            "schema_version": "foundation-003.v1",
            "source": "scryfall",
            "input_path": "input.json",
            "output_path": "output.json",
            "generated_at": "2026-05-12T00:00:00Z",
            "input_count": 4,
            "normalized_count": 2,
            "skipped_count": 2,
            "skipped_reasons": {
                "record_not_object": 1,
                "invalid_arena_id": 1,
            },
            "missing_high_value_fields_count": 3,
            "missing_high_value_fields_by_field_name": {
                "mana_cost": 1,
                "arena_id": 2,
            },
        }

        report = format_normalization_report(metadata)

        self.assertLess(report.index("  invalid_arena_id: 1"), report.index("  record_not_object: 1"))
        self.assertLess(report.index("  arena_id: 2"), report.index("  mana_cost: 1"))

    def test_cli_report_prints_metadata(self):
        payload = normalize_payload(
            "mtgjson",
            [{"name": "CLI Report Card", "type": "Artifact"}],
            input_path="input.json",
            output_path="output.json",
            generated_at="2026-05-12T00:00:00Z",
        )
        with tempfile.TemporaryDirectory() as tempdir:
            path = Path(tempdir) / "normalized.json"
            path.write_text(json.dumps(payload), encoding="utf-8")
            stdout = StringIO()

            with redirect_stdout(stdout):
                result = main(["normalize-report", str(path)])

        self.assertEqual(result, 0)
        self.assertIn("source: mtgjson", stdout.getvalue())


if __name__ == "__main__":
    unittest.main()
