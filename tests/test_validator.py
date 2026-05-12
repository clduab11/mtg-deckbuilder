import unittest

from mtgdeckbuilder.ingest.arena import DeckEntry, Decklist
from mtgdeckbuilder.ingest.cards import CardCatalog
from mtgdeckbuilder.rules.validator import validate_deck


class ValidatorTests(unittest.TestCase):
    def test_valid_sixty_card_deck_passes(self):
        deck = Decklist(mainboard=(DeckEntry(60, "Plains"),))

        result = validate_deck(deck)

        self.assertTrue(result.is_valid)
        self.assertEqual(result.issues, ())

    def test_mainboard_minimum_is_enforced(self):
        deck = Decklist(mainboard=(DeckEntry(59, "Plains"),))

        result = validate_deck(deck)

        self.assertFalse(result.is_valid)
        self.assertIn("mainboard_too_small", {issue.code for issue in result.issues})

    def test_sideboard_maximum_is_enforced(self):
        deck = Decklist(
            mainboard=(DeckEntry(60, "Plains"),),
            sideboard=(DeckEntry(16, "Destroy Evil"),),
        )

        result = validate_deck(deck)

        self.assertFalse(result.is_valid)
        self.assertIn("sideboard_too_large", {issue.code for issue in result.issues})

    def test_non_basic_copy_limit_counts_mainboard_and_sideboard(self):
        deck = Decklist(
            mainboard=(DeckEntry(4, "Lightning Bolt"), DeckEntry(56, "Mountain")),
            sideboard=(DeckEntry(1, "Lightning Bolt"),),
        )

        result = validate_deck(deck)

        self.assertFalse(result.is_valid)
        self.assertIn("too_many_copies", {issue.code for issue in result.issues})

    def test_basic_land_copy_limit_exception_uses_fallback_name(self):
        deck = Decklist(mainboard=(DeckEntry(60, "Wastes"),))

        result = validate_deck(deck)

        self.assertTrue(result.is_valid)

    def test_banned_legality_is_enforced_when_available(self):
        catalog = CardCatalog.from_records(
            [
                {
                    "name": "Banned Spell",
                    "type_line": "Sorcery",
                    "legalities": {"standard": "banned"},
                }
            ]
        )
        deck = Decklist(mainboard=(DeckEntry(4, "Banned Spell"), DeckEntry(56, "Plains")))

        result = validate_deck(deck, catalog=catalog)

        self.assertFalse(result.is_valid)
        self.assertIn("banned_card", {issue.code for issue in result.issues})

    def test_valid_deck_has_no_export_compatibility_issue(self):
        deck = Decklist(mainboard=(DeckEntry(60, "Plains"),))

        result = validate_deck(deck)

        self.assertNotIn("arena_export_incompatible", {issue.code for issue in result.issues})


if __name__ == "__main__":
    unittest.main()
