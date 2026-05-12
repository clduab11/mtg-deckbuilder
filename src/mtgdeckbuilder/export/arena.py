"""MTG Arena import-text writer."""

from __future__ import annotations

from collections import OrderedDict

from mtgdeckbuilder.ingest.arena import DeckEntry, Decklist, parse_arena_deck


def export_arena_deck(deck: Decklist) -> str:
    """Write canonical Arena import text."""

    lines = ["Deck"]
    lines.extend(_format_entry(entry) for entry in _coalesce(deck.mainboard))

    if deck.sideboard:
        lines.append("")
        lines.append("Sideboard")
        lines.extend(_format_entry(entry) for entry in _coalesce(deck.sideboard))

    return "\n".join(lines) + "\n"


def assert_arena_export_compatible(deck: Decklist) -> None:
    """Raise ValueError if the deck cannot round-trip through Arena text."""

    exported = export_arena_deck(deck)
    parsed = parse_arena_deck(exported)
    if _section_counts(deck.mainboard) != _section_counts(parsed.mainboard):
        raise ValueError("mainboard does not round-trip through Arena export")
    if _section_counts(deck.sideboard) != _section_counts(parsed.sideboard):
        raise ValueError("sideboard does not round-trip through Arena export")


def _format_entry(entry: DeckEntry) -> str:
    if "\n" in entry.name or "\r" in entry.name:
        raise ValueError(f"card name is not Arena export compatible: {entry.name!r}")
    if entry.set_code and entry.collector_number:
        return f"{entry.count} {entry.name} ({entry.set_code}) {entry.collector_number}"
    return f"{entry.count} {entry.name}"


def _coalesce(entries: tuple[DeckEntry, ...]) -> tuple[DeckEntry, ...]:
    grouped: "OrderedDict[tuple[str, str | None, str | None], DeckEntry]" = OrderedDict()
    for entry in entries:
        key = (entry.name.casefold(), entry.set_code, entry.collector_number)
        existing = grouped.get(key)
        if existing is None:
            grouped[key] = entry
        else:
            grouped[key] = DeckEntry(
                count=existing.count + entry.count,
                name=existing.name,
                set_code=existing.set_code,
                collector_number=existing.collector_number,
                line_number=existing.line_number,
            )
    return tuple(grouped.values())


def _section_counts(entries: tuple[DeckEntry, ...]) -> dict[tuple[str, str | None, str | None], int]:
    counts: dict[tuple[str, str | None, str | None], int] = {}
    for entry in entries:
        key = (entry.name.casefold(), entry.set_code, entry.collector_number)
        counts[key] = counts.get(key, 0) + entry.count
    return counts
