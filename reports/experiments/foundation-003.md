# Foundation-003 Assumptions

## Scope

- This milestone adds deterministic metadata reporting and normalization diagnostics to the offline card normalizer.
- Metadata is embedded in the normalized catalog JSON under a top-level `metadata` object.
- The compact catalog remains compatible with existing consumers because the top-level `cards` list is preserved.
- No network calls, gameplay automation, Arena client control, screen scraping, or unrelated rewrites are included.

## Metadata Fields

The normalizer writes:

- `schema_version`: currently `foundation-003.v1`.
- `generated_at`: UTC timestamp formatted as `YYYY-MM-DDTHH:MM:SSZ`.
- `source`: `scryfall` or `mtgjson`.
- `input_path` and `output_path`: paths passed to the CLI/file helper.
- `input_count`: number of source records found in the accepted payload shape.
- `normalized_count`: number of records written to `cards`.
- `skipped_count`: number of malformed records skipped.
- `skipped_reasons`: deterministic counts by reason.
- `missing_high_value_fields_count`: total missing high-value field observations across normalized cards.
- `missing_high_value_fields_by_field_name`: counts grouped by field name.

## Diagnostics

- Missing high-value fields are counted when a normalized field is absent or has an empty value such as `""`, `[]`, or `{}`.
- Malformed records inside an otherwise valid payload are skipped. Examples include non-object records, missing required `name`, and invalid `arena_id`.
- Invalid source payload shapes still raise `ValueError`.
- Source record order is preserved for normalized cards.

## Validation Commands

```bash
make lint
make test
make eval-smoke
```

## Next Step

Add an optional `normalize-report` command that reads a normalized catalog and prints a concise human-readable diagnostics summary from the embedded metadata.
