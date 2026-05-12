import json
import tempfile
import unittest
from pathlib import Path

from mtgdeckbuilder.analysis.deck_rank import rank_decks
from mtgdeckbuilder.build.deck_builder import build_deck_file
from mtgdeckbuilder.sources.csv_ingest import normalize_csv_file


FIXTURES = Path(__file__).parent / "fixtures" / "csv"
ROOT = Path(__file__).resolve().parents[1]


class DeckRankBuildTests(unittest.TestCase):
    def test_rank_decks_separates_bo1_and_bo3(self):
        with tempfile.TemporaryDirectory() as tempdir:
            results_path = Path(tempdir) / "results.json"
            normalize_csv_file("untapped_like_csv", FIXTURES / "untapped_like_results.csv", results_path)
            payload = json.loads(results_path.read_text(encoding="utf-8"))

        claims = rank_decks(payload, min_games=30)
        by_key = {(claim.deck_id, claim.queue): claim for claim in claims}

        self.assertEqual(by_key[("white-aggro", "bo1")].label, "60_percent_supported")
        self.assertEqual(by_key[("white-aggro", "bo1")].games, 60)
        self.assertEqual(by_key[("white-aggro", "bo3")].sample_size_status, "inconclusive")

    def test_rank_decks_insufficient_games_is_inconclusive(self):
        payload = {
            "matches": [
                {
                    "deck_id": "small-sample",
                    "deck_name": "Small Sample",
                    "wins": 6,
                    "losses": 4,
                    "queue": "bo1",
                }
            ]
        }

        claim = rank_decks(payload, min_games=30)[0]

        self.assertEqual(claim.win_rate, 0.6)
        self.assertEqual(claim.label, "inconclusive")

    def test_card_only_csv_cannot_rank_decks(self):
        payload = {"cards": [{"name": "Plains"}]}

        with self.assertRaisesRegex(ValueError, "matches"):
            rank_decks(payload)

    def test_deck_build_uses_supported_existing_deck_and_exports_arena_text(self):
        with tempfile.TemporaryDirectory() as tempdir:
            results_path = Path(tempdir) / "results.json"
            deck_path = Path(tempdir) / "deck.json"
            collection_path = Path(tempdir) / "collection.json"
            normalize_csv_file("untapped_like_csv", FIXTURES / "untapped_like_results.csv", results_path)
            normalize_csv_file("aetherhub_like_deck", FIXTURES / "aetherhub_like_deck.csv", deck_path)
            normalize_csv_file("generic_collection_csv", FIXTURES / "generic_collection.csv", collection_path)
            results = json.loads(results_path.read_text(encoding="utf-8"))
            deck_rows = json.loads(deck_path.read_text(encoding="utf-8"))["decks"]
            results["decks"] = deck_rows
            results_path.write_text(json.dumps(results), encoding="utf-8")

            output = json.loads(
                build_deck_file(
                    cards_path=ROOT / "data" / "processed" / "sample_cards.json",
                    collection_path=collection_path,
                    results_path=results_path,
                    queue="bo1",
                    min_games=30,
                )
            )

        self.assertEqual(output["deck_id"], "white-aggro")
        self.assertTrue(output["validation"]["valid"])
        self.assertIn("Deck\n24 Plains", output["arena_export"])
        self.assertEqual(output["evidence"]["label"], "60_percent_supported")


if __name__ == "__main__":
    unittest.main()
