import json
import os
import unittest
from pathlib import Path
from unittest.mock import patch

from mtgdeckbuilder.advisor.report import advisor_report, advisor_report_file
from mtgdeckbuilder.ingest.cards import CardCatalog


ROOT = Path(__file__).resolve().parents[1]


class AdvisorReportTests(unittest.TestCase):
    def test_valid_sample_deck_produces_offline_advisor_report(self):
        report = json.loads(
            advisor_report_file(
                ROOT / "data" / "raw" / "sample_arena_deck.txt",
                ROOT / "data" / "processed" / "sample_cards.json",
            )
        )

        self.assertEqual(report["schema_version"], "advisor-foundation-001.v1")
        self.assertEqual(report["mode"], "offline")
        self.assertEqual(report["reference"], "vawlrathh")
        self.assertTrue(report["valid"])
        self.assertEqual(report["validation"]["issues"], [])
        self.assertEqual(report["features"]["mainboard_count"], 60)
        self.assertTrue(report["findings"])
        self.assertIn("Deck\n24 Plains", report["arena_export"])

    def test_catalog_backed_invalid_deck_withholds_arena_export(self):
        catalog = CardCatalog.from_records(
            [
                {
                    "name": "Plains",
                    "type_line": "Basic Land - Plains",
                    "color_identity": ["W"],
                    "mana_value": 0,
                    "legalities": {"standard": "legal"},
                }
            ]
        )

        report = advisor_report("Deck\n56 Plains\n4 Unknown Spell\n", catalog=catalog)

        self.assertFalse(report["valid"])
        self.assertEqual(report["arena_export"], "")
        self.assertIn(
            "unknown_card",
            {issue["code"] for issue in report["validation"]["issues"]},
        )
        self.assertIn("validation_failed", {finding["code"] for finding in report["findings"]})

    def test_advisor_report_does_not_require_ai_or_network_keys(self):
        catalog = CardCatalog.from_records(
            [
                {
                    "name": "Plains",
                    "type_line": "Basic Land - Plains",
                    "color_identity": ["W"],
                    "mana_value": 0,
                    "legalities": {"standard": "legal"},
                }
            ]
        )

        with patch.dict(os.environ, {}, clear=True):
            report = advisor_report("Deck\n60 Plains\n", catalog=catalog)

        self.assertTrue(report["valid"])
        self.assertEqual(report["mode"], "offline")
        self.assertIn("validation_passed", {finding["code"] for finding in report["findings"]})


if __name__ == "__main__":
    unittest.main()
