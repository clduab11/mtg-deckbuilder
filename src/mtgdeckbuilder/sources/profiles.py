"""Declarative source profiles for local MTG Arena-adjacent exports."""

from __future__ import annotations

from dataclasses import asdict, dataclass
import json
from typing import Any


@dataclass(frozen=True)
class SourceProfile:
    profile_id: str
    source_name: str
    source_url: str
    source_type: str
    row_model: str
    required_columns: tuple[str, ...]
    optional_columns: tuple[str, ...] = ()
    field_aliases: dict[str, tuple[str, ...]] | None = None
    supported_extensions: tuple[str, ...] = (".csv",)
    format: str | None = None
    queue: str | None = None
    rank_scope: str | None = None
    event_type: str | None = None
    provenance_notes: str = ""

    def to_dict(self) -> dict[str, Any]:
        payload = asdict(self)
        payload["required_columns"] = list(self.required_columns)
        payload["optional_columns"] = list(self.optional_columns)
        payload["supported_extensions"] = list(self.supported_extensions)
        payload["field_aliases"] = {
            key: list(value) for key, value in (self.field_aliases or {}).items()
        }
        return payload


PROFILES: dict[str, SourceProfile] = {
    "generic_card_csv": SourceProfile(
        profile_id="generic_card_csv",
        source_name="Generic Card CSV",
        source_url="",
        source_type="card_truth",
        row_model="cards",
        required_columns=("name",),
        optional_columns=(
            "mana_cost",
            "mana_value",
            "colors",
            "color_identity",
            "type_line",
            "oracle_text",
            "legalities",
            "rarity",
            "set_code",
            "collector_number",
            "arena_id",
            "digital",
            "games",
            "layout",
        ),
        field_aliases={
            "name": ("name", "card", "card_name"),
            "mana_value": ("mana_value", "mana value", "cmc", "mv"),
            "set_code": ("set_code", "set", "set code"),
            "collector_number": ("collector_number", "collector", "number"),
            "type_line": ("type_line", "type", "card_type"),
        },
        provenance_notes="Card truth should come from Scryfall, MTGJSON, Wizards, or user-supplied current data.",
    ),
    "generic_collection_csv": SourceProfile(
        profile_id="generic_collection_csv",
        source_name="Generic Collection CSV",
        source_url="",
        source_type="collection_export",
        row_model="collection",
        required_columns=("name", "quantity"),
        optional_columns=("set_code", "collector_number", "rarity", "source"),
        field_aliases={
            "name": ("name", "card", "card_name"),
            "quantity": ("quantity", "qty", "owned", "count", "copies"),
            "set_code": ("set_code", "set", "set code"),
            "collector_number": ("collector_number", "collector", "number"),
        },
        provenance_notes="Owned-card exports from trackers or personal spreadsheets.",
    ),
    "generic_deck_csv": SourceProfile(
        profile_id="generic_deck_csv",
        source_name="Generic Deck CSV",
        source_url="",
        source_type="community_deck",
        row_model="decks",
        required_columns=("deck_id", "card_name", "quantity"),
        optional_columns=("deck_name", "archetype", "section", "format", "source_url"),
        field_aliases={
            "deck_id": ("deck_id", "deck id", "id"),
            "deck_name": ("deck_name", "deck", "deck name", "name"),
            "card_name": ("card_name", "card", "card name"),
            "quantity": ("quantity", "qty", "count", "copies"),
            "section": ("section", "board", "zone"),
        },
        provenance_notes="User-provided deck exports from community sites or spreadsheets.",
    ),
    "generic_match_results_csv": SourceProfile(
        profile_id="generic_match_results_csv",
        source_name="Generic Match Results CSV",
        source_url="",
        source_type="tracker_stats",
        row_model="matches",
        required_columns=("deck_id", "wins", "losses", "queue"),
        optional_columns=("deck_name", "archetype", "format", "rank_scope", "event_type", "start_date", "end_date"),
        field_aliases={
            "deck_id": ("deck_id", "deck id", "id"),
            "deck_name": ("deck_name", "deck", "deck name", "name"),
            "wins": ("wins", "win", "w"),
            "losses": ("losses", "loss", "losses_count", "l"),
            "queue": ("queue", "mode", "bo"),
            "format": ("format", "format_name"),
            "rank_scope": ("rank_scope", "rank", "tier"),
        },
        provenance_notes="Performance truth requires enough games and must not be inferred from card text alone.",
    ),
    "untapped_like_csv": SourceProfile(
        profile_id="untapped_like_csv",
        source_name="Untapped.gg-like Tracker Export",
        source_url="https://mtga.untapped.gg/",
        source_type="tracker_stats",
        row_model="matches",
        required_columns=("deck_id", "wins", "losses", "queue"),
        optional_columns=("deck_name", "archetype", "format", "rank_scope", "games", "winrate"),
        field_aliases={
            "deck_id": ("deck_id", "deck", "deck name", "name"),
            "deck_name": ("deck", "deck_name", "name"),
            "wins": ("wins", "win"),
            "losses": ("losses", "loss"),
            "queue": ("queue", "mode"),
            "rank_scope": ("rank_scope", "rank", "tier"),
        },
        queue="bo1",
        provenance_notes="Compatibility target for user-provided tracker/stat exports; no scraping or paywall bypassing.",
    ),
    "aetherhub_like_deck": SourceProfile(
        profile_id="aetherhub_like_deck",
        source_name="AetherHub-like Arena Deck Export",
        source_url="https://aetherhub.com/",
        source_type="arena_metagame",
        row_model="decks",
        required_columns=("deck_id", "card_name", "quantity"),
        optional_columns=("deck_name", "archetype", "format", "section", "performance", "source_url"),
        field_aliases={
            "deck_id": ("deck_id", "deck", "deck name", "name"),
            "deck_name": ("deck", "deck_name", "name"),
            "card_name": ("card", "card_name", "card name"),
            "quantity": ("quantity", "qty", "count", "copies"),
            "section": ("section", "board", "zone"),
        },
        provenance_notes="Compatibility target for user-provided Arena deck exports and metagame deck rows.",
    ),
    "mtggoldfish_like_metagame": SourceProfile(
        profile_id="mtggoldfish_like_metagame",
        source_name="MTGGoldfish-like Metagame Export",
        source_url="https://www.mtggoldfish.com/metagame/",
        source_type="tournament_metagame",
        row_model="metagame",
        required_columns=("archetype", "meta_share"),
        optional_columns=("deck_name", "player", "placing", "event", "format", "source_url"),
        field_aliases={
            "archetype": ("archetype", "deck", "deck_name", "name"),
            "meta_share": ("meta_share", "meta%", "percentage", "share"),
        },
        provenance_notes="Popularity truth is source frequency/metagame share, not a win-rate claim by itself.",
    ),
}


def list_profiles() -> list[SourceProfile]:
    return [PROFILES[key] for key in sorted(PROFILES)]


def get_profile(profile_id: str) -> SourceProfile:
    try:
        return PROFILES[profile_id]
    except KeyError as exc:
        raise ValueError(f"unknown source profile: {profile_id}") from exc


def profiles_json() -> str:
    return json.dumps([profile.to_dict() for profile in list_profiles()], indent=2, sort_keys=True) + "\n"


def profile_json(profile_id: str) -> str:
    return json.dumps(get_profile(profile_id).to_dict(), indent=2, sort_keys=True) + "\n"
