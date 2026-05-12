# Source-Cohesive CSV Deck Intelligence Assumptions

## Scope

- This milestone series introduces local source profiles, CSV profiling, CSV normalization, deck ranking, and conservative deck candidate construction.
- It treats Untapped.gg, AetherHub, MTGGoldfish, MTGDecks, MTGAZone, MTG Arena Pro, 17Lands, Moxfield, Archidekt, GitHub, GitLab, Bitbucket, and user spreadsheets as compatibility targets for user-provided files.
- It does not scrape sites, bypass paywalls, automate downloads, control MTG Arena, inspect live matches, or provide live match assistance.

## Source Classes

- `tracker_stats`: performance rows such as wins, losses, queue, rank scope, and BO1/BO3 segmentation.
- `arena_metagame`: Arena deck rows and metagame deck exports.
- `tournament_metagame`: archetype share, event, placing, and source tournament context.
- `community_deck`: shared decklists from sites or repository-hosted fixtures.
- `collection_export`: owned-card exports from trackers or spreadsheets.

## Evidence Rules

- Card truth comes from Scryfall, MTGJSON, Wizards, official/current sources, or user-supplied local datasets.
- Performance truth comes from local match/deck/result data with sample sizes.
- Popularity truth comes from metagame share or source frequency.
- Recommendation output must remain labeled as derived from local data.
- A “60% BO1” claim requires enough local BO1 games. The initial default threshold is 30 games.

## Implemented Commands

```bash
python3 -m mtgdeckbuilder source-profile list
python3 -m mtgdeckbuilder source-profile inspect untapped_like_csv
python3 -m mtgdeckbuilder csv-profile tests/fixtures/csv/untapped_like_results.csv
python3 -m mtgdeckbuilder csv-normalize untapped_like_csv tests/fixtures/csv/untapped_like_results.csv /tmp/results.json
python3 -m mtgdeckbuilder csv-report /tmp/results.json
python3 -m mtgdeckbuilder deck-rank /tmp/results.json --min-games 30
```

`deck-build` expects normalized performance results that also include deck card rows under `decks`. This keeps generation conservative: it promotes an evidence-backed existing deck rather than inventing a deck from card text.

## Next Step

Split the combined normalized dataset shape into explicit documented files for `cards`, `collection`, `decks`, `matches`, and `metagame`, then add import examples for additional community source formats.
