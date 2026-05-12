import unittest

from mtgdeckbuilder.ingest.arena import ArenaParseError, parse_arena_deck


class ArenaParserTests(unittest.TestCase):
    def test_parses_mainboard_sideboard_and_set_metadata(self):
        deck = parse_arena_deck(
            """Deck
4 Kumano Faces Kakkazan (NEO) 152
2 Plains

Sideboard
1 Destroy Evil (DMU) 17
"""
        )

        self.assertEqual(deck.mainboard_count, 6)
        self.assertEqual(deck.sideboard_count, 1)
        self.assertEqual(deck.mainboard[0].name, "Kumano Faces Kakkazan")
        self.assertEqual(deck.mainboard[0].set_code, "NEO")
        self.assertEqual(deck.mainboard[0].collector_number, "152")
        self.assertEqual(deck.sideboard[0].name, "Destroy Evil")

    def test_defaults_lines_before_header_to_mainboard(self):
        deck = parse_arena_deck("4 Plains\n4 Island\n")

        self.assertEqual(deck.mainboard_count, 8)
        self.assertEqual(deck.sideboard_count, 0)

    def test_malformed_line_reports_line_number(self):
        with self.assertRaises(ArenaParseError) as raised:
            parse_arena_deck("Deck\nnot a deck line\n")

        self.assertEqual(raised.exception.line_number, 2)
        self.assertIn("Line 2", str(raised.exception))

    def test_zero_count_is_rejected(self):
        with self.assertRaises(ArenaParseError) as raised:
            parse_arena_deck("Deck\n0 Plains\n")

        self.assertEqual(raised.exception.line_number, 2)
        self.assertIn("positive", str(raised.exception))


if __name__ == "__main__":
    unittest.main()
