"""Command-line interface for the foundation package."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
import sys

from mtgdeckbuilder.analysis.deck_rank import rank_decks_report_file
from mtgdeckbuilder.build.deck_builder import build_deck_file
from mtgdeckbuilder.eval.smoke import run_smoke_file
from mtgdeckbuilder.export.arena import export_arena_deck
from mtgdeckbuilder.ingest.arena import ArenaParseError, parse_arena_deck
from mtgdeckbuilder.ingest.cards import CardCatalog
from mtgdeckbuilder.ingest.normalization import normalize_cards_file, normalization_report_file
from mtgdeckbuilder.rules.validator import ValidationConfig, validate_deck
from mtgdeckbuilder.sources.csv_ingest import csv_report_file, normalize_csv_file, profile_csv_report
from mtgdeckbuilder.sources.profiles import profile_json, profiles_json


def main(argv: list[str] | None = None) -> int:
    parser = _build_parser()
    args = parser.parse_args(argv)
    try:
        return args.func(args)
    except ArenaParseError as exc:
        print(f"parse error: {exc}", file=sys.stderr)
        return 2
    except ValueError as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 2


def _build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(prog="mtgdeckbuilder")
    subparsers = parser.add_subparsers(dest="command", required=True)

    validate_parser = subparsers.add_parser("validate", help="validate an Arena decklist")
    validate_parser.add_argument("deck_path")
    validate_parser.add_argument("--cards", dest="cards_path")
    validate_parser.add_argument("--format", dest="format_name", default="standard")
    validate_parser.set_defaults(func=_cmd_validate)

    export_parser = subparsers.add_parser("export", help="write canonical Arena import text")
    export_parser.add_argument("deck_path")
    export_parser.set_defaults(func=_cmd_export)

    smoke_parser = subparsers.add_parser("eval-smoke", help="run parser, validator, features, and export")
    smoke_parser.add_argument("deck_path")
    smoke_parser.add_argument("--cards", dest="cards_path")
    smoke_parser.add_argument("--format", dest="format_name", default="standard")
    smoke_parser.set_defaults(func=_cmd_eval_smoke)

    normalize_parser = subparsers.add_parser("normalize-cards", help="normalize an offline card catalog")
    normalize_parser.add_argument("source", help="source catalog type: scryfall or mtgjson")
    normalize_parser.add_argument("input_json")
    normalize_parser.add_argument("output_json")
    normalize_parser.set_defaults(func=_cmd_normalize_cards)

    report_parser = subparsers.add_parser("normalize-report", help="print normalization metadata diagnostics")
    report_parser.add_argument("normalized_catalog_json")
    report_parser.set_defaults(func=_cmd_normalize_report)

    source_parser = subparsers.add_parser("source-profile", help="list or inspect source profiles")
    source_subparsers = source_parser.add_subparsers(dest="source_profile_command", required=True)
    source_list_parser = source_subparsers.add_parser("list", help="list known source profiles")
    source_list_parser.set_defaults(func=_cmd_source_profile_list)
    source_inspect_parser = source_subparsers.add_parser("inspect", help="inspect one source profile")
    source_inspect_parser.add_argument("profile")
    source_inspect_parser.set_defaults(func=_cmd_source_profile_inspect)

    csv_profile_parser = subparsers.add_parser("csv-profile", help="profile a local CSV export")
    csv_profile_parser.add_argument("input_csv")
    csv_profile_parser.set_defaults(func=_cmd_csv_profile)

    csv_normalize_parser = subparsers.add_parser("csv-normalize", help="normalize a local CSV export")
    csv_normalize_parser.add_argument("profile")
    csv_normalize_parser.add_argument("input_csv")
    csv_normalize_parser.add_argument("output_json")
    csv_normalize_parser.set_defaults(func=_cmd_csv_normalize)

    csv_report_parser = subparsers.add_parser("csv-report", help="print normalized CSV diagnostics")
    csv_report_parser.add_argument("normalized_json")
    csv_report_parser.set_defaults(func=_cmd_csv_report)

    rank_parser = subparsers.add_parser("deck-rank", help="rank decks from normalized match results")
    rank_parser.add_argument("normalized_results_json")
    rank_parser.add_argument("--min-games", type=int, default=30)
    rank_parser.set_defaults(func=_cmd_deck_rank)

    build_parser = subparsers.add_parser("deck-build", help="build an evidence-backed validated deck candidate")
    build_parser.add_argument("--cards", required=True)
    build_parser.add_argument("--results", required=True)
    build_parser.add_argument("--collection")
    build_parser.add_argument("--format", dest="format_name", default="standard")
    build_parser.add_argument("--queue", default="bo1")
    build_parser.add_argument("--min-games", type=int, default=30)
    build_parser.set_defaults(func=_cmd_deck_build)

    return parser


def _cmd_validate(args: argparse.Namespace) -> int:
    deck = parse_arena_deck(Path(args.deck_path).read_text(encoding="utf-8"))
    catalog = CardCatalog.from_json_file(args.cards_path) if args.cards_path else None
    result = validate_deck(
        deck,
        catalog=catalog,
        config=ValidationConfig(format_name=args.format_name),
    )
    if result.is_valid:
        print("valid")
        return 0
    for issue in result.issues:
        print(f"{issue.severity}: {issue.code}: {issue.message}")
    return 1


def _cmd_export(args: argparse.Namespace) -> int:
    deck = parse_arena_deck(Path(args.deck_path).read_text(encoding="utf-8"))
    print(export_arena_deck(deck), end="")
    return 0


def _cmd_eval_smoke(args: argparse.Namespace) -> int:
    result = run_smoke_file(
        args.deck_path,
        cards_path=args.cards_path,
        format_name=args.format_name,
    )
    print(json.dumps(result.to_dict(), indent=2, sort_keys=True))
    return 0 if result.valid else 1


def _cmd_normalize_cards(args: argparse.Namespace) -> int:
    count = normalize_cards_file(args.source, args.input_json, args.output_json)
    print(f"normalized {count} cards to {args.output_json}")
    return 0


def _cmd_normalize_report(args: argparse.Namespace) -> int:
    print(normalization_report_file(args.normalized_catalog_json), end="")
    return 0


def _cmd_source_profile_list(args: argparse.Namespace) -> int:
    print(profiles_json(), end="")
    return 0


def _cmd_source_profile_inspect(args: argparse.Namespace) -> int:
    print(profile_json(args.profile), end="")
    return 0


def _cmd_csv_profile(args: argparse.Namespace) -> int:
    print(profile_csv_report(args.input_csv), end="")
    return 0


def _cmd_csv_normalize(args: argparse.Namespace) -> int:
    count = normalize_csv_file(args.profile, args.input_csv, args.output_json)
    print(f"normalized {count} rows to {args.output_json}")
    return 0


def _cmd_csv_report(args: argparse.Namespace) -> int:
    print(csv_report_file(args.normalized_json), end="")
    return 0


def _cmd_deck_rank(args: argparse.Namespace) -> int:
    print(rank_decks_report_file(args.normalized_results_json, min_games=args.min_games), end="")
    return 0


def _cmd_deck_build(args: argparse.Namespace) -> int:
    print(
        build_deck_file(
            cards_path=args.cards,
            collection_path=args.collection,
            results_path=args.results,
            format_name=args.format_name,
            queue=args.queue,
            min_games=args.min_games,
        ),
        end="",
    )
    return 0
