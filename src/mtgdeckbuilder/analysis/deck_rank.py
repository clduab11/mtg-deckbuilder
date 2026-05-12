"""Deck performance aggregation from normalized local CSV datasets."""

from __future__ import annotations

from collections import defaultdict
from dataclasses import dataclass
import json
import math
from pathlib import Path
from typing import Any


DEFAULT_MIN_GAMES = 30


@dataclass(frozen=True)
class PerformanceClaim:
    deck_id: str
    deck_name: str
    archetype: str
    queue: str
    games: int
    wins: int
    losses: int
    win_rate: float
    confidence_interval: tuple[float, float]
    sample_size_status: str
    label: str

    def to_dict(self) -> dict[str, Any]:
        return {
            "deck_id": self.deck_id,
            "deck_name": self.deck_name,
            "archetype": self.archetype,
            "queue": self.queue,
            "games": self.games,
            "wins": self.wins,
            "losses": self.losses,
            "win_rate": round(self.win_rate, 4),
            "confidence_interval": [round(self.confidence_interval[0], 4), round(self.confidence_interval[1], 4)],
            "sample_size_status": self.sample_size_status,
            "label": self.label,
        }


def rank_decks_file(normalized_results_json: str | Path, min_games: int = DEFAULT_MIN_GAMES) -> str:
    payload = json.loads(Path(normalized_results_json).read_text(encoding="utf-8"))
    claims = rank_decks(payload, min_games=min_games)
    return json.dumps({"claims": [claim.to_dict() for claim in claims]}, indent=2, sort_keys=True) + "\n"


def rank_decks_report_file(normalized_results_json: str | Path, min_games: int = DEFAULT_MIN_GAMES) -> str:
    payload = json.loads(Path(normalized_results_json).read_text(encoding="utf-8"))
    claims = rank_decks(payload, min_games=min_games)
    lines = ["Deck Rank Report", f"min_games: {min_games}", "claims:"]
    if not claims:
        lines.append("  none")
    for claim in claims:
        lines.append(
            "  "
            f"{claim.deck_id} | {claim.deck_name or claim.archetype or 'Unknown'} | "
            f"{claim.queue} | {claim.wins}-{claim.losses} | "
            f"{claim.win_rate:.1%} | {claim.sample_size_status} | {claim.label}"
        )
    return "\n".join(lines) + "\n"


def rank_decks(payload: dict[str, Any], min_games: int = DEFAULT_MIN_GAMES) -> list[PerformanceClaim]:
    if "matches" not in payload or not isinstance(payload["matches"], list):
        raise ValueError("deck-rank requires a normalized matches dataset")
    grouped: dict[tuple[str, str], dict[str, Any]] = defaultdict(
        lambda: {"wins": 0, "losses": 0, "deck_name": "", "archetype": ""}
    )
    for row in payload["matches"]:
        if not isinstance(row, dict):
            continue
        deck_id = str(row.get("deck_id") or row.get("deck_name") or "")
        queue = str(row.get("queue") or "").casefold()
        if not deck_id or not queue:
            continue
        key = (deck_id, queue)
        grouped[key]["wins"] += int(row.get("wins", 0))
        grouped[key]["losses"] += int(row.get("losses", 0))
        grouped[key]["deck_name"] = str(row.get("deck_name", grouped[key]["deck_name"]) or "")
        grouped[key]["archetype"] = str(row.get("archetype", grouped[key]["archetype"]) or "")

    claims = [_claim_for_group(deck_id, queue, values, min_games) for (deck_id, queue), values in grouped.items()]
    claims.sort(key=lambda claim: (-claim.win_rate, -claim.games, claim.deck_id, claim.queue))
    return claims


def _claim_for_group(deck_id: str, queue: str, values: dict[str, Any], min_games: int) -> PerformanceClaim:
    wins = int(values["wins"])
    losses = int(values["losses"])
    games = wins + losses
    win_rate = wins / games if games else 0.0
    status = "supported" if games >= min_games else "inconclusive"
    label = _claim_label(win_rate, status)
    return PerformanceClaim(
        deck_id=deck_id,
        deck_name=str(values["deck_name"]),
        archetype=str(values["archetype"]),
        queue=queue,
        games=games,
        wins=wins,
        losses=losses,
        win_rate=win_rate,
        confidence_interval=_wilson_interval(wins, games),
        sample_size_status=status,
        label=label,
    )


def _claim_label(win_rate: float, status: str) -> str:
    if status != "supported":
        return "inconclusive"
    if win_rate >= 0.60:
        return "60_percent_supported"
    if win_rate >= 0.50:
        return "positive_supported"
    return "below_even_supported"


def _wilson_interval(wins: int, games: int) -> tuple[float, float]:
    if games == 0:
        return (0.0, 0.0)
    z = 1.96
    p = wins / games
    denominator = 1 + z * z / games
    center = (p + z * z / (2 * games)) / denominator
    margin = (z * math.sqrt((p * (1 - p) + z * z / (4 * games)) / games)) / denominator
    return (max(0.0, center - margin), min(1.0, center + margin))
