import json
import tempfile
import unittest
from contextlib import redirect_stdout
from io import StringIO
from pathlib import Path

from mtgdeckbuilder.cli import main
from mtgdeckbuilder.sources.csv_ingest import normalize_csv_file, profile_csv_file
from mtgdeckbuilder.sources.profiles import get_profile, list_profiles


FIXTURES = Path(__file__).parent / "fixtures" / "csv"


class SourceProfilesCsvTests(unittest.TestCase):
    def test_source_profiles_include_popular_source_targets(self):
        profile_ids = [profile.profile_id for profile in list_profiles()]

        self.assertIn("untapped_like_csv", profile_ids)
        self.assertIn("aetherhub_like_deck", profile_ids)
        self.assertIn("mtggoldfish_like_metagame", profile_ids)
        self.assertEqual(get_profile("untapped_like_csv").source_type, "tracker_stats")

    def test_csv_profile_detects_untapped_like_results(self):
        profile = profile_csv_file(FIXTURES / "untapped_like_results.csv")

        self.assertEqual(profile["row_count"], 3)
        self.assertEqual(profile["delimiter"], ",")
        self.assertEqual(profile["detected_profile"], "untapped_like_csv")
        self.assertEqual(profile["diagnostics"], [])

    def test_csv_profile_reports_malformed_missing_columns(self):
        profile = profile_csv_file(FIXTURES / "malformed.csv")

        self.assertIsNone(profile["detected_profile"])
        self.assertIn("no_complete_profile_match", profile["diagnostics"])

    def test_csv_normalize_untapped_like_results(self):
        with tempfile.TemporaryDirectory() as tempdir:
            output = Path(tempdir) / "results.json"
            count = normalize_csv_file("untapped_like_csv", FIXTURES / "untapped_like_results.csv", output)
            payload = json.loads(output.read_text(encoding="utf-8"))

        self.assertEqual(count, 3)
        self.assertEqual(payload["metadata"]["source_profile"], "untapped_like_csv")
        self.assertEqual(payload["metadata"]["source_type"], "tracker_stats")
        self.assertEqual(payload["metadata"]["row_model"], "matches")
        self.assertEqual(payload["matches"][0]["deck_id"], "white-aggro")
        self.assertEqual(payload["matches"][0]["wins"], 36)
        self.assertEqual(payload["matches"][0]["losses"], 24)
        self.assertEqual(payload["matches"][0]["queue"], "bo1")

    def test_csv_normalize_aetherhub_like_deck(self):
        with tempfile.TemporaryDirectory() as tempdir:
            output = Path(tempdir) / "deck.json"
            count = normalize_csv_file("aetherhub_like_deck", FIXTURES / "aetherhub_like_deck.csv", output)
            payload = json.loads(output.read_text(encoding="utf-8"))

        self.assertEqual(count, 10)
        self.assertEqual(payload["metadata"]["source_type"], "arena_metagame")
        self.assertEqual(payload["metadata"]["row_model"], "decks")
        self.assertEqual(payload["decks"][0]["deck_id"], "white-aggro")
        self.assertEqual(payload["decks"][0]["card_name"], "Plains")
        self.assertEqual(payload["decks"][0]["quantity"], 24)

    def test_csv_normalize_mtggoldfish_like_metagame(self):
        with tempfile.TemporaryDirectory() as tempdir:
            output = Path(tempdir) / "meta.json"
            count = normalize_csv_file("mtggoldfish_like_metagame", FIXTURES / "mtggoldfish_like_metagame.csv", output)
            payload = json.loads(output.read_text(encoding="utf-8"))

        self.assertEqual(count, 2)
        self.assertEqual(payload["metadata"]["source_type"], "tournament_metagame")
        self.assertEqual(payload["metagame"][0]["archetype"], "Mono White Aggro")
        self.assertEqual(payload["metagame"][0]["meta_share"], 0.125)

    def test_cli_source_profile_and_csv_commands(self):
        with tempfile.TemporaryDirectory() as tempdir:
            output = Path(tempdir) / "collection.json"
            stdout = StringIO()
            with redirect_stdout(stdout):
                self.assertEqual(main(["source-profile", "inspect", "generic_collection_csv"]), 0)
                self.assertEqual(main(["csv-profile", str(FIXTURES / "generic_collection.csv")]), 0)
                self.assertEqual(
                    main([
                        "csv-normalize",
                        "generic_collection_csv",
                        str(FIXTURES / "generic_collection.csv"),
                        str(output),
                    ]),
                    0,
                )
                self.assertEqual(main(["csv-report", str(output)]), 0)


if __name__ == "__main__":
    unittest.main()
