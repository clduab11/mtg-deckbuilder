import json
import tempfile
import unittest
from contextlib import redirect_stderr, redirect_stdout
from io import StringIO
from pathlib import Path

from mtgdeckbuilder.cli import main
from mtgdeckbuilder.ingest.cards import Card, CardCatalog
from mtgdeckbuilder.ingest.normalization import normalize_cards_file, normalize_payload


FIXTURES = Path(__file__).parent / "fixtures"


class CardNormalizationTests(unittest.TestCase):
    def test_card_preserves_extended_fields(self):
        card = Card.from_mapping(
            {
                "name": "Lightning Bolt",
                "type_line": "Instant",
                "colors": ["R"],
                "color_identity": ["R"],
                "mana_value": 1,
                "mana_cost": "{R}",
                "oracle_text": "Lightning Bolt deals 3 damage to any target.",
                "rarity": "common",
                "digital": False,
                "games": ["paper", "arena", "mtgo"],
                "layout": "normal",
                "set_code": "clu",
                "collector_number": "141",
                "scryfall_id": "00000000-0000-0000-0000-000000000001",
                "arena_id": 67356,
                "legalities": {"legacy": "legal"},
            }
        )

        self.assertEqual(card.mana_cost, "{R}")
        self.assertEqual(card.oracle_text, "Lightning Bolt deals 3 damage to any target.")
        self.assertEqual(card.rarity, "common")
        self.assertFalse(card.digital)
        self.assertEqual(card.games, ("paper", "arena", "mtgo"))
        self.assertEqual(card.layout, "normal")
        self.assertEqual(card.set_code, "CLU")
        self.assertEqual(card.scryfall_id, "00000000-0000-0000-0000-000000000001")
        self.assertEqual(card.arena_id, 67356)

    def test_normalize_scryfall_fixture(self):
        with tempfile.TemporaryDirectory() as tempdir:
            output_path = Path(tempdir) / "normalized.json"

            count = normalize_cards_file("scryfall", FIXTURES / "scryfall_cards.json", output_path)

            self.assertEqual(count, 2)
            payload = json.loads(output_path.read_text(encoding="utf-8"))
        self.assertEqual([card["name"] for card in payload["cards"]], ["Lightning Bolt", "Island"])
        bolt = payload["cards"][0]
        self.assertEqual(bolt["mana_cost"], "{R}")
        self.assertEqual(bolt["mana_value"], 1.0)
        self.assertEqual(bolt["oracle_text"], "Lightning Bolt deals 3 damage to any target.")
        self.assertEqual(bolt["rarity"], "common")
        self.assertFalse(bolt["digital"])
        self.assertEqual(bolt["games"], ["paper", "arena", "mtgo"])
        self.assertEqual(bolt["layout"], "normal")
        self.assertEqual(bolt["set_code"], "CLU")
        self.assertEqual(bolt["collector_number"], "141")
        self.assertEqual(bolt["scryfall_id"], "00000000-0000-0000-0000-000000000001")
        self.assertEqual(bolt["arena_id"], 67356)
        metadata = payload["metadata"]
        self.assertEqual(metadata["source"], "scryfall")
        self.assertEqual(metadata["input_path"], str(FIXTURES / "scryfall_cards.json"))
        self.assertEqual(metadata["output_path"], str(output_path))
        self.assertEqual(metadata["input_count"], 2)
        self.assertEqual(metadata["normalized_count"], 2)
        self.assertEqual(metadata["skipped_count"], 0)
        self.assertEqual(metadata["skipped_reasons"], {})
        self.assertEqual(metadata["schema_version"], "foundation-003.v1")
        self.assertRegex(metadata["generated_at"], r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z$")

    def test_normalize_scryfall_object_shape(self):
        catalog = normalize_payload(
            "scryfall",
            {"cards": [{"name": "Opt", "type_line": "Instant"}]},
            generated_at="2026-05-12T00:00:00Z",
        )

        self.assertEqual(catalog["cards"][0]["name"], "Opt")
        self.assertEqual(catalog["cards"][0]["mana_value"], 0.0)
        self.assertEqual(catalog["metadata"]["generated_at"], "2026-05-12T00:00:00Z")

    def test_normalize_mtgjson_fixture(self):
        with tempfile.TemporaryDirectory() as tempdir:
            output_path = Path(tempdir) / "normalized.json"

            count = normalize_cards_file("mtgjson", FIXTURES / "mtgjson_cards.json", output_path)

            self.assertEqual(count, 2)
            payload = json.loads(output_path.read_text(encoding="utf-8"))
        self.assertEqual([card["name"] for card in payload["cards"]], ["Arcane Signet", "Duress"])
        signet = payload["cards"][0]
        self.assertEqual(signet["type_line"], "Artifact")
        self.assertEqual(signet["mana_cost"], "{2}")
        self.assertEqual(signet["mana_value"], 2.0)
        self.assertEqual(signet["color_identity"], [])
        self.assertEqual(signet["scryfall_id"], "00000000-0000-0000-0000-000000000101")
        self.assertEqual(signet["arena_id"], 76543)
        self.assertEqual(signet["set_code"], "CMM")
        self.assertEqual(signet["collector_number"], "653")
        self.assertEqual(
            signet["oracle_text"],
            "{T}: Add one mana of any color in your commander's color identity.",
        )
        self.assertEqual(payload["cards"][1]["set_code"], "TST")
        metadata = payload["metadata"]
        self.assertEqual(metadata["source"], "mtgjson")
        self.assertEqual(metadata["input_count"], 2)
        self.assertEqual(metadata["normalized_count"], 2)
        self.assertEqual(metadata["skipped_count"], 0)
        self.assertEqual(metadata["missing_high_value_fields_by_field_name"]["arena_id"], 1)
        self.assertGreater(metadata["missing_high_value_fields_count"], 0)

    def test_normalize_mtgjson_plain_list_shape(self):
        catalog = normalize_payload(
            "mtgjson",
            [
                {
                    "colorIdentity": ["U"],
                    "identifiers": {"scryfallId": "00000000-0000-0000-0000-000000000201"},
                    "manaCost": "{U}",
                    "manaValue": 1,
                    "name": "Opt",
                    "number": "59",
                    "setCode": "m21",
                    "type": "Instant",
                }
            ],
        )

        self.assertEqual(catalog["cards"][0]["name"], "Opt")
        self.assertEqual(catalog["cards"][0]["color_identity"], ["U"])
        self.assertEqual(catalog["cards"][0]["set_code"], "M21")
        self.assertEqual(catalog["cards"][0]["collector_number"], "59")

    def test_normalize_mtgjson_top_level_cards_shape(self):
        catalog = normalize_payload(
            "mtgjson",
            {
                "cards": [
                    {
                        "manaCost": "{G}",
                        "manaValue": 1,
                        "name": "Snakeskin Veil",
                        "number": "208",
                        "setCode": "sta",
                        "type": "Instant",
                    }
                ]
            },
        )

        self.assertEqual(catalog["cards"][0]["name"], "Snakeskin Veil")
        self.assertEqual(catalog["cards"][0]["set_code"], "STA")
        self.assertEqual(catalog["cards"][0]["collector_number"], "208")

    def test_skips_malformed_records_and_reports_reasons(self):
        catalog = normalize_payload(
            "scryfall",
            {
                "cards": [
                    {"name": "First Valid", "type_line": "Instant"},
                    {"type_line": "Creature"},
                    ["not", "an", "object"],
                    {"name": "Bad Arena Id", "arena_id": "not-an-int"},
                    {"name": "Second Valid", "type_line": "Sorcery"},
                ]
            },
            generated_at="2026-05-12T00:00:00Z",
        )

        self.assertEqual([card["name"] for card in catalog["cards"]], ["First Valid", "Second Valid"])
        metadata = catalog["metadata"]
        self.assertEqual(metadata["input_count"], 5)
        self.assertEqual(metadata["normalized_count"], 2)
        self.assertEqual(metadata["skipped_count"], 3)
        self.assertEqual(metadata["skipped_reasons"]["scryfall_missing_required_name"], 1)
        self.assertEqual(metadata["skipped_reasons"]["record_not_object"], 1)
        self.assertEqual(metadata["skipped_reasons"]["invalid_arena_id"], 1)

    def test_missing_field_diagnostics_count_by_field(self):
        catalog = normalize_payload(
            "mtgjson",
            [
                {
                    "name": "Sparse Card",
                    "manaValue": 0,
                    "type": "Artifact",
                }
            ],
            generated_at="2026-05-12T00:00:00Z",
        )

        missing = catalog["metadata"]["missing_high_value_fields_by_field_name"]
        self.assertEqual(missing["mana_cost"], 1)
        self.assertEqual(missing["colors"], 1)
        self.assertEqual(missing["color_identity"], 1)
        self.assertEqual(missing["legalities"], 1)
        self.assertEqual(missing["arena_id"], 1)
        self.assertEqual(catalog["metadata"]["missing_high_value_fields_count"], sum(missing.values()))

    def test_source_record_order_is_preserved(self):
        catalog = normalize_payload(
            "scryfall",
            [
                {"name": "Zeta Spell"},
                {"name": "Alpha Spell"},
                {"name": "Middle Spell"},
            ],
        )

        self.assertEqual(
            [card["name"] for card in catalog["cards"]],
            ["Zeta Spell", "Alpha Spell", "Middle Spell"],
        )

    def test_normalized_catalog_loads_for_foundation_001_compatibility(self):
        with tempfile.TemporaryDirectory() as tempdir:
            output_path = Path(tempdir) / "normalized.json"
            normalize_cards_file("scryfall", FIXTURES / "scryfall_cards.json", output_path)

            catalog = CardCatalog.from_json_file(output_path)

        self.assertIn("lightning bolt", catalog)
        card = catalog.get("Lightning Bolt")
        self.assertIsNotNone(card)
        assert card is not None
        self.assertEqual(card.mana_cost, "{R}")

    def test_bad_source_and_bad_shape_raise_value_error(self):
        with self.assertRaisesRegex(ValueError, "source"):
            normalize_payload("unknown", [])
        with self.assertRaisesRegex(ValueError, "mtgjson"):
            normalize_payload("mtgjson", {"data": {"cards": {}}})

    def test_cli_normalize_cards_writes_file_and_prints_count(self):
        with tempfile.TemporaryDirectory() as tempdir:
            output_path = Path(tempdir) / "normalized.json"
            stdout = StringIO()

            with redirect_stdout(stdout):
                result = main(
                    [
                        "normalize-cards",
                        "scryfall",
                        str(FIXTURES / "scryfall_cards.json"),
                        str(output_path),
                    ]
                )

            self.assertEqual(result, 0)
            self.assertTrue(output_path.exists())
            self.assertIn("normalized 2 cards", stdout.getvalue())
            payload = json.loads(output_path.read_text(encoding="utf-8"))
            self.assertEqual(payload["metadata"]["normalized_count"], 2)

    def test_cli_bad_source_returns_2(self):
        with tempfile.TemporaryDirectory() as tempdir:
            output_path = Path(tempdir) / "normalized.json"
            stderr = StringIO()

            with redirect_stderr(stderr):
                result = main(
                    [
                        "normalize-cards",
                        "unknown",
                        str(FIXTURES / "scryfall_cards.json"),
                        str(output_path),
                    ]
                )

            self.assertEqual(result, 2)
            self.assertFalse(output_path.exists())
            self.assertIn("source must be one of", stderr.getvalue())


if __name__ == "__main__":
    unittest.main()
