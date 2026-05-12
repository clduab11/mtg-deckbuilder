"""MTG Arena import-text parser."""

from __future__ import annotations

from dataclasses import dataclass
import re
from typing import Iterable


_COUNT_LINE_RE = re.compile(r"^(?P<count>\d+)\s+(?P<rest>.+?)\s*$")
_TRAILING_ARENA_SET_RE = re.compile(
    r"^(?P<name>.+?)\s+\((?P<set_code>[A-Za-z0-9_]+)\)\s+(?P<collector_number>\S+)$"
)


class ArenaParseError(ValueError):
    """Raised when Arena import text cannot be parsed deterministically."""

    def __init__(self, message: str, line_number: int) -> None:
        super().__init__(f"Line {line_number}: {message}")
        self.message = message
        self.line_number = line_number


@dataclass(frozen=True)
class DeckEntry:
    """One counted card line from an Arena decklist."""

    count: int
    name: str
    set_code: str | None = None
    collector_number: str | None = None
    line_number: int | None = None

    def __post_init__(self) -> None:
        if self.count <= 0:
            raise ValueError("deck entry count must be positive")
        if not self.name.strip():
            raise ValueError("deck entry name must not be empty")
        object.__setattr__(self, "name", " ".join(self.name.split()))
        if self.set_code is not None:
            object.__setattr__(self, "set_code", self.set_code.upper())
        if self.collector_number is not None:
            object.__setattr__(self, "collector_number", self.collector_number.strip())


@dataclass(frozen=True)
class Decklist:
    """Parsed MTG Arena decklist."""

    mainboard: tuple[DeckEntry, ...]
    sideboard: tuple[DeckEntry, ...] = ()

    @property
    def mainboard_count(self) -> int:
        return sum(entry.count for entry in self.mainboard)

    @property
    def sideboard_count(self) -> int:
        return sum(entry.count for entry in self.sideboard)

    def entries(self) -> Iterable[DeckEntry]:
        yield from self.mainboard
        yield from self.sideboard


def parse_arena_deck(text: str) -> Decklist:
    """Parse Arena import text into a structured decklist."""

    section = "mainboard"
    mainboard: list[DeckEntry] = []
    sideboard: list[DeckEntry] = []

    for line_number, raw_line in enumerate(text.splitlines(), start=1):
        line = raw_line.strip()
        if not line:
            continue

        lowered = line.lower()
        if lowered == "deck":
            section = "mainboard"
            continue
        if lowered == "sideboard":
            section = "sideboard"
            continue

        entry = _parse_entry_line(line, line_number)
        if section == "sideboard":
            sideboard.append(entry)
        else:
            mainboard.append(entry)

    return Decklist(mainboard=tuple(mainboard), sideboard=tuple(sideboard))


def _parse_entry_line(line: str, line_number: int) -> DeckEntry:
    match = _COUNT_LINE_RE.match(line)
    if not match:
        raise ArenaParseError("expected '<count> <card name>'", line_number)

    count = int(match.group("count"))
    if count <= 0:
        raise ArenaParseError("card count must be positive", line_number)

    rest = match.group("rest").strip()
    if not rest:
        raise ArenaParseError("missing card name", line_number)

    metadata_match = _TRAILING_ARENA_SET_RE.match(rest)
    if metadata_match:
        name = metadata_match.group("name").strip()
        set_code = metadata_match.group("set_code")
        collector_number = metadata_match.group("collector_number")
    else:
        name = rest
        set_code = None
        collector_number = None

    if not name:
        raise ArenaParseError("missing card name", line_number)

    try:
        return DeckEntry(
            count=count,
            name=name,
            set_code=set_code,
            collector_number=collector_number,
            line_number=line_number,
        )
    except ValueError as exc:
        raise ArenaParseError(str(exc), line_number) from exc
