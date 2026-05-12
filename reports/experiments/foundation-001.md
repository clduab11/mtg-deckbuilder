# Foundation-001 Assumptions

## Scope

- This milestone is offline only.
- It produces validated decklists, feature summaries, smoke reports, and Arena import text.
- It does not automate MTG Arena gameplay, scrape screens, control mouse or keyboard input, inspect live match state, or provide live match assistance.

## Validation Defaults

- Constructed format defaults to `standard`.
- Mainboard minimum defaults to 60 cards.
- Sideboard maximum defaults to 15 cards.
- Non-basic copy limit defaults to 4 total copies across mainboard and sideboard.
- Basic lands are detected from card metadata when available. Without metadata, the fixed Arena-name fallback is Plains, Island, Swamp, Mountain, Forest, and Wastes.
- Banned legality is enforced only when card metadata includes a matching format legality value of `banned`.

## Data

- Card data is loaded from local JSON only.
- The card schema accepts common Scryfall and MTGJSON field names but does not download external datasets.
- Unknown cards can be parsed and exported. They are still subject to copy limits unless they match the basic land fallback list.

## Next Step

Add an offline Scryfall/MTGJSON normalization command that converts downloaded bulk data into the compact `data/processed` catalog shape used by this milestone.
