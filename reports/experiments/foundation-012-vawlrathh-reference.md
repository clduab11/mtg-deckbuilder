# Foundation-012: Vawlrathh Reference Bridge

## Purpose

Foundation-012 uses `clduab11/vawlrathh` as a prototype reference for future AI advisor, MCP, Steam, and app layers without importing its heavy runtime stack into the offline core.

## Reusable Ideas

- MCP tool shapes: parse deck CSV/text, analyze deck, optimize deck, record match, list decks, and summarize stored deck statistics.
- Steam/Arena-style CSV shape: `Quantity,Name,Set,Type,Mana Cost,CMC,Colors,Rarity`.
- Advisor/report UX: a single command that parses, validates, featureizes, and explains local deck state.
- Future optional app layer: FastAPI, Gradio, WebSocket, or MCP surfaces can sit above the offline core later.

## Intentionally Excluded

- FastAPI, Gradio, SQLAlchemy, pandas, OpenAI, Anthropic, Torch, sentence-transformers, and other `vawlrathh` runtime dependencies.
- Live Scryfall or metagame calls.
- Predicted win-rate claims without local evidence.
- Vawlrathh personality branding or tone in core output.
- Gameplay automation, Arena client control, screen scraping, or live match assistance.

## Implemented

- Added `steam_arena_deck_csv` source profile for user-provided Steam/Arena-style deck CSV exports.
- Added filename-derived deck defaults for that profile: `deck_id`, `deck_name`, and `section`.
- Added `advisor-report`, a deterministic offline CLI report inspired by the `vawlrathh` advisor concept.
- Advisor reports use only local parser, catalog, feature, validation, and exporter modules.

## Validation

```bash
make lint
make test
make eval-smoke
python3 -m mtgdeckbuilder advisor-report data/raw/sample_arena_deck.txt --catalog data/processed/sample_cards.json
```
