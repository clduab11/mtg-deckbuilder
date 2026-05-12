import unittest

from mtgdeckbuilder.export.arena import export_arena_deck
from mtgdeckbuilder.ingest.arena import DeckEntry, Decklist, parse_arena_deck


class ArenaExporterTests(unittest.TestCase):
    def test_exports_canonical_arena_text_and_coalesces_duplicates(self):
        deck = Decklist(
            mainboard=(
                DeckEntry(2, "Plains"),
                DeckEntry(2, "Plains"),
                DeckEntry(1, "Adeline, Resplendent Cathar", "MID", "1"),
            ),
            sideboard=(DeckEntry(1, "Destroy Evil", "DMU", "17"),),
        )

        self.assertEqual(
            export_arena_deck(deck),
            """Deck
4 Plains
1 Adeline, Resplendent Cathar (MID) 1

Sideboard
1 Destroy Evil (DMU) 17
""",
        )

    def test_export_round_trips_through_parser(self):
        original = Decklist(
            mainboard=(DeckEntry(60, "Plains"),),
            sideboard=(DeckEntry(2, "Destroy Evil"),),
        )

        parsed = parse_arena_deck(export_arena_deck(original))

        self.assertEqual(parsed.mainboard_count, 60)
        self.assertEqual(parsed.sideboard_count, 2)
        self.assertEqual(parsed.mainboard[0].name, "Plains")


if __name__ == "__main__":
    unittest.main()
