import unittest

from mtgdeckbuilder.eval.smoke import run_smoke_evaluation
from mtgdeckbuilder.ingest.cards import CardCatalog


class SmokeEvaluatorTests(unittest.TestCase):
    def test_valid_sample_returns_features_and_export(self):
        catalog = CardCatalog.from_records(
            [
                {
                    "name": "Plains",
                    "type_line": "Basic Land - Plains",
                    "color_identity": ["W"],
                    "mana_value": 0,
                    "legalities": {"standard": "legal"},
                },
                {
                    "name": "Known Spell",
                    "type_line": "Sorcery",
                    "colors": ["W"],
                    "color_identity": ["W"],
                    "mana_value": 2,
                    "legalities": {"standard": "legal"},
                },
            ]
        )
        deck_text = "Deck\n56 Plains\n4 Known Spell\n"

        result = run_smoke_evaluation(deck_text, catalog=catalog)

        self.assertTrue(result.valid)
        self.assertEqual(result.features["mainboard_count"], 60)
        self.assertEqual(result.features["colors"], ["W"])
        self.assertIn("Deck\n", result.arena_export)

    def test_invalid_deck_returns_validation_issues(self):
        result = run_smoke_evaluation("Deck\n59 Plains\n")

        self.assertFalse(result.valid)
        self.assertIn("mainboard_too_small", {issue.code for issue in result.issues})

    def test_parse_error_returns_invalid_smoke_result(self):
        result = run_smoke_evaluation("Deck\nthis is not a deck line\n")

        self.assertFalse(result.valid)
        self.assertIn("parse_error", {issue.code for issue in result.issues})


if __name__ == "__main__":
    unittest.main()
