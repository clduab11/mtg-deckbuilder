# Foundation-004 Assumptions

## Scope

- This milestone adds a read-only `normalize-report` CLI command.
- It reads embedded normalization metadata from an existing normalized catalog JSON file.
- It does not mutate files, make network calls, automate gameplay, scrape screens, control the Arena client, or add advanced ML.

## Report Behavior

- The command requires a JSON object with a top-level `cards` list and top-level `metadata` object.
- Missing metadata fails clearly because legacy compact catalogs such as `data/processed/sample_cards.json` predate Foundation-003 metadata.
- Report output is deterministic:
  - fixed section order
  - skipped reasons sorted by key
  - missing high-value fields sorted by key

## Validation Commands

```bash
make lint
make test
make eval-smoke
python3 -m mtgdeckbuilder normalize-report data/processed/sample_cards.json
python3 -m mtgdeckbuilder normalize-cards scryfall tests/fixtures/scryfall_cards.json /tmp/normalized_cards.json
python3 -m mtgdeckbuilder normalize-report /tmp/normalized_cards.json
```

The `sample_cards.json` report command is expected to fail with a metadata-missing error.

## Next Step

Add fixture-backed docs for producing a normalized local catalog from real Scryfall or MTGJSON bulk exports without requiring network access in tests.
