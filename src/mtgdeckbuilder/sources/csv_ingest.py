"""CSV profiling and normalization for local source exports."""

from __future__ import annotations

import csv
from collections import Counter
from datetime import UTC, datetime
import json
from pathlib import Path
from typing import Any

from mtgdeckbuilder.sources.profiles import SourceProfile, get_profile, list_profiles


CSV_SCHEMA_VERSION = "csv-foundation-001.v1"


def profile_csv_file(input_csv: str | Path) -> dict[str, Any]:
    path = Path(input_csv)
    text = path.read_text(encoding="utf-8-sig")
    delimiter = _detect_delimiter(text)
    rows = _read_rows(text, delimiter)
    headers = list(rows[0].keys()) if rows else []
    normalized_headers = [_norm_header(header) for header in headers]
    duplicate_headers = sorted(
        header for header, count in Counter(normalized_headers).items() if header and count > 1
    )
    candidates = [_profile_candidate(profile, normalized_headers) for profile in list_profiles()]
    candidates.sort(key=lambda item: (-item["matched_required_count"], item["missing_required_columns"], item["profile_id"]))
    best = candidates[0] if candidates and not candidates[0]["missing_required_columns"] else None
    return {
        "schema_version": CSV_SCHEMA_VERSION,
        "input_path": str(path),
        "delimiter": delimiter,
        "row_count": len(rows),
        "headers": headers,
        "duplicate_headers": duplicate_headers,
        "candidate_profiles": candidates,
        "detected_profile": best["profile_id"] if best else None,
        "diagnostics": _profile_diagnostics(rows, headers, duplicate_headers, candidates),
    }


def profile_csv_report(input_csv: str | Path) -> str:
    profile = profile_csv_file(input_csv)
    lines = [
        "CSV Profile",
        f"input_path: {profile['input_path']}",
        f"delimiter: {profile['delimiter']}",
        f"row_count: {profile['row_count']}",
        f"detected_profile: {profile['detected_profile'] or 'None'}",
        "headers:",
    ]
    lines.extend(f"  {header}" for header in profile["headers"])
    lines.append("diagnostics:")
    if profile["diagnostics"]:
        lines.extend(f"  {item}" for item in profile["diagnostics"])
    else:
        lines.append("  none")
    return "\n".join(lines) + "\n"


def normalize_csv_file(profile_id: str, input_csv: str | Path, output_json: str | Path) -> int:
    profile = get_profile(profile_id)
    input_path = Path(input_csv)
    output_path = Path(output_json)
    text = input_path.read_text(encoding="utf-8-sig")
    delimiter = _detect_delimiter(text)
    rows = _read_rows(text, delimiter)
    normalized = normalize_csv_rows(profile, rows, str(input_path), str(output_path), delimiter)
    output_path.write_text(json.dumps(normalized, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    return int(normalized["metadata"]["normalized_count"])


def normalize_csv_rows(
    profile: SourceProfile,
    rows: list[dict[str, str]],
    input_path: str | None = None,
    output_path: str | None = None,
    delimiter: str = ",",
) -> dict[str, Any]:
    skipped: Counter[str] = Counter()
    normalized_rows: list[dict[str, Any]] = []
    for row in rows:
        try:
            normalized_rows.append(_normalize_row(profile, row))
        except ValueError as exc:
            skipped[str(exc)] += 1
    return {
        "metadata": {
            "schema_version": CSV_SCHEMA_VERSION,
            "generated_at": _utc_timestamp(),
            "source_profile": profile.profile_id,
            "source_name": profile.source_name,
            "source_url": profile.source_url,
            "source_type": profile.source_type,
            "row_model": profile.row_model,
            "input_path": input_path,
            "output_path": output_path,
            "delimiter": delimiter,
            "input_count": len(rows),
            "normalized_count": len(normalized_rows),
            "skipped_count": sum(skipped.values()),
            "skipped_reasons": dict(sorted(skipped.items())),
            "format": profile.format,
            "queue": profile.queue,
            "rank_scope": profile.rank_scope,
            "event_type": profile.event_type,
            "provenance_notes": profile.provenance_notes,
        },
        profile.row_model: normalized_rows,
    }


def csv_report_file(normalized_json: str | Path) -> str:
    payload = json.loads(Path(normalized_json).read_text(encoding="utf-8"))
    if not isinstance(payload, dict) or not isinstance(payload.get("metadata"), dict):
        raise ValueError("normalized CSV dataset must include metadata")
    metadata = payload["metadata"]
    row_model = metadata.get("row_model")
    rows = payload.get(str(row_model), [])
    if not isinstance(rows, list):
        raise ValueError("normalized CSV dataset row model is malformed")
    skipped = metadata.get("skipped_reasons", {})
    if not isinstance(skipped, dict):
        skipped = {}
    lines = [
        "CSV Normalization Report",
        f"schema_version: {metadata.get('schema_version')}",
        f"source_profile: {metadata.get('source_profile')}",
        f"source_type: {metadata.get('source_type')}",
        f"row_model: {row_model}",
        f"input_count: {metadata.get('input_count')}",
        f"normalized_count: {metadata.get('normalized_count')}",
        f"skipped_count: {metadata.get('skipped_count')}",
        "skipped_reasons:",
    ]
    lines.extend(_mapping_lines(skipped))
    lines.append(f"rows_available: {len(rows)}")
    return "\n".join(lines) + "\n"


def _detect_delimiter(text: str) -> str:
    sample = text[:4096]
    try:
        return csv.Sniffer().sniff(sample, delimiters=",\t;|").delimiter
    except csv.Error:
        return ","


def _read_rows(text: str, delimiter: str) -> list[dict[str, str]]:
    reader = csv.DictReader(text.splitlines(), delimiter=delimiter)
    if not reader.fieldnames:
        return []
    return [dict(row) for row in reader]


def _profile_candidate(profile: SourceProfile, normalized_headers: list[str]) -> dict[str, Any]:
    aliases = _aliases(profile)
    matched_required = [
        field for field in profile.required_columns if _find_header(field, normalized_headers, aliases) is not None
    ]
    missing_required = [
        field for field in profile.required_columns if _find_header(field, normalized_headers, aliases) is None
    ]
    matched_optional = [
        field for field in profile.optional_columns if _find_header(field, normalized_headers, aliases) is not None
    ]
    return {
        "profile_id": profile.profile_id,
        "source_type": profile.source_type,
        "row_model": profile.row_model,
        "matched_required_count": len(matched_required),
        "missing_required_columns": missing_required,
        "matched_optional_columns": matched_optional,
    }


def _profile_diagnostics(
    rows: list[dict[str, str]],
    headers: list[str],
    duplicate_headers: list[str],
    candidates: list[dict[str, Any]],
) -> list[str]:
    diagnostics: list[str] = []
    if not headers:
        diagnostics.append("missing_header")
    if not rows:
        diagnostics.append("no_data_rows")
    for header in duplicate_headers:
        diagnostics.append(f"duplicate_header:{header}")
    if not candidates or candidates[0]["missing_required_columns"]:
        diagnostics.append("no_complete_profile_match")
    return diagnostics


def _normalize_row(profile: SourceProfile, row: dict[str, str]) -> dict[str, Any]:
    normalized_headers = {_norm_header(header): header for header in row}
    aliases = _aliases(profile)
    output: dict[str, Any] = {}
    for field in tuple(profile.required_columns) + tuple(profile.optional_columns):
        source_header = _find_header(field, list(normalized_headers), aliases)
        value = row.get(normalized_headers[source_header], "") if source_header else ""
        if field in profile.required_columns and value.strip() == "":
            raise ValueError(f"missing_required_{field}")
        if value.strip() != "":
            output[field] = _coerce_field(field, value)
    _apply_profile_defaults(profile, output)
    return output


def _apply_profile_defaults(profile: SourceProfile, output: dict[str, Any]) -> None:
    if profile.format and "format" not in output:
        output["format"] = profile.format
    if profile.queue and "queue" not in output:
        output["queue"] = profile.queue
    if profile.rank_scope and "rank_scope" not in output:
        output["rank_scope"] = profile.rank_scope
    if profile.event_type and "event_type" not in output:
        output["event_type"] = profile.event_type


def _aliases(profile: SourceProfile) -> dict[str, tuple[str, ...]]:
    aliases = dict(profile.field_aliases or {})
    for field in tuple(profile.required_columns) + tuple(profile.optional_columns):
        aliases.setdefault(field, (field,))
    return aliases


def _find_header(field: str, normalized_headers: list[str], aliases: dict[str, tuple[str, ...]]) -> str | None:
    for alias in aliases.get(field, (field,)):
        normalized_alias = _norm_header(alias)
        if normalized_alias in normalized_headers:
            return normalized_alias
    return None


def _norm_header(header: str) -> str:
    return " ".join(str(header).strip().casefold().replace("_", " ").replace("-", " ").split())


def _coerce_field(field: str, value: str) -> Any:
    text = value.strip()
    if field in {"quantity", "wins", "losses", "games", "placing"}:
        return int(float(text))
    if field in {"winrate", "meta_share"}:
        return _parse_ratio(text)
    if field in {"colors", "color_identity"}:
        return [part.strip() for part in text.replace("/", ",").split(",") if part.strip()]
    return text


def _parse_ratio(text: str) -> float:
    stripped = text.strip()
    if stripped.endswith("%"):
        return float(stripped[:-1]) / 100.0
    value = float(stripped)
    return value / 100.0 if value > 1 else value


def _utc_timestamp() -> str:
    return datetime.now(UTC).replace(microsecond=0).isoformat().replace("+00:00", "Z")


def _mapping_lines(values: dict[str, Any]) -> list[str]:
    if not values:
        return ["  none"]
    return [f"  {key}: {values[key]}" for key in sorted(values)]
