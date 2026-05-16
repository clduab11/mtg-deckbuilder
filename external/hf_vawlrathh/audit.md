# Legacy Hugging Face Space Audit: MCP-1st-Birthday/vawlrathh

Access date: 2026-05-15
Status: Remote tree inspected; files were not imported into core services.
Treatment: External reference only. Not authoritative for card data, legality, collection ownership, validation, simulations, or matchup scoring.

## 1. File inventory inspected

Remote tree showed these top-level directories:

- `.github/`
- `docs/`
- `examples/`
- `scripts/`
- `src/`
- `tests/`

Remote tree showed these notable top-level files:

- `.env.example`
- `.gitattributes`
- `.gitignore`
- `.pre-commit-config.yaml`
- `.python-version`
- `.secrets.baseline`
- `.yamllint.yaml`
- `CHANGELOG.md`
- `CLAUDE.md`
- `DEPLOYMENT.md`
- `Dockerfile`
- `HUGGINGFACE_SPACE_SETUP.md`
- `LICENSE`
- `Makefile`
- `README.md`
- `SECRETS_CHECKLIST.md`
- `SESSION_SUMMARY.md`
- `app.py`
- `check_pr_status.py`
- `check_space_status.py`
- `create_hf_pr.py`
- `deploy_gradio_fix.py`
- `direct_upload.py`
- `docker-compose.yml`
- `hf_payload_manifest.json`
- `mcp_config.json`
- `pyproject.toml`
- `requirements-dev.txt`
- `requirements.txt`
- `test_hf_auth.py`

## 2. Dependency inventory

Observed dependencies include:

- UI/runtime: `gradio`, `spaces`, `huggingface-hub`
- MCP: `mcp`, Node-backed MCP servers through `npx`
- Data/storage: `pandas`, `numpy`, `sqlalchemy`, `aiosqlite`, `aiofiles`
- HTTP/API: `httpx`
- LLM/ML: `openai`, `anthropic`, `sentence-transformers`, `torch`
- Testing: `pytest`, `pytest-asyncio`
- Config: `python-dotenv`, environment API keys

## 3. Entrypoint map

| Entrypoint | Observed role | Incorporation action |
|---|---|---|
| `app.py` | Large pure-Gradio HF Space wrapper; initializes SQL, deck analyzer, collection analyzer, meta intelligence, chat, inference, and card market services at module load. | QUARANTINE as a reference. Do not import directly. Extract only UI concepts into `src/mtgdeckbuilder/ui/gradio_app.py` after tests. |
| `mcp_config.json` | Configures `server-memory`, `server-sequential-thinking`, and `cld-omnisearch` via `npx`, with multiple API key environment variables. | QUARANTINE. Translate only safe deck-intelligence tool names into local deterministic tools. |
| `README.md` | Project overview, feature claims, API examples, MCP tool names, Gradio/FastAPI architecture notes. | ADAPT selected workflow ideas; ignore personality, win-rate, and unsupported claims. |
| `requirements.txt` / `pyproject.toml` | Heavy runtime dependency sets including LLM, ML, database, MCP, and HF Space packages. | QUARANTINE. Keep foundation dependency-light until validators and simulators mature. |
| `docs/` | Likely architecture/deployment/security notes. | ADAPT after targeted file-level review. |
| `tests/` | Potential reusable examples. | ADAPT only after running and separating deterministic tests from external-service tests. |

## 4. Reusable components

| Component idea | Category | Reason | Proposed destination | Required test |
|---|---|---|---|---|
| CSV upload UI with row counting and progress | ADAPT | Useful for large Arena collection uploads; must call deterministic parser. | `src/mtgdeckbuilder/ui/gradio_app.py` | Upload fixture CSV; verify owned counts and failed-row reporting. |
| Deck text paste/import panel | ADAPT | Useful thin UI pattern; parser must remain in ingest layer. | `src/mtgdeckbuilder/ui/gradio_app.py` | Paste Arena text; verify round-trip export. |
| JSON result display pattern | KEEP | Useful report display concept. | `src/mtgdeckbuilder/ui/gradio_app.py` | Render validation/simulation JSON without mutating service output. |
| MCP tool naming concept | ADAPT | Useful safe interface pattern; must expose only deck-intelligence operations. | `src/mtgdeckbuilder/mcp/tools.py` | Tool contract tests for validate/simulate/export only. |
| Event logging idea | ADAPT | Aligns with experiment logging requirement. | `src/mtgdeckbuilder/observability/experiment_logger.py` | Verify experiment ID, hashes, assumptions, and JSONL append. |

## 5. Risk list

- `app.py` is a 55 KB app/module that mixes UI, service initialization, persistence, analysis, chat, and external integrations.
- Several features rely on external API keys and online services.
- The README claims AI optimization and win-rate improvement; this conflicts with the deterministic-first and no-guaranteed-win-rate contract unless converted into clearly labeled estimates.
- MCP config launches external Node packages through `npx`; this is not acceptable without pinning, sandboxing, and explicit security review.
- Sequential-thinking / hidden-reasoning style features must not expose chain-of-thought or become a project requirement.
- Dependencies are heavy for a foundation milestone and include ML/runtime packages that should be deferred.
- Any prompt-only logic for legality, metagame, or card evaluation must be replaced by deterministic services.
- Any live chat or real-time strategy function must not become live opponent decision automation.

## 6. Keep / adapt / quarantine / drop table

| Source file or area | Category | Reason | Destination | Required test |
|---|---|---|---|---|
| `app.py` Gradio layout concepts | ADAPT | Useful UI shell, upload, progress, and display patterns. | `src/mtgdeckbuilder/ui/gradio_app.py` | UI calls service functions only; no validation in UI. |
| `app.py` service initialization at import | DROP | Creates hidden state and mixes layers. | None | Importing UI must not initialize databases or external services. |
| `app.py` collection chunking concept | ADAPT | Good UX for large CSV files. | `ingest/collection_parser.py` plus UI adapter | Chunked parser preserves ownership aggregation. |
| `README.md` MCP tool list | ADAPT | Some tool names are useful. | `mcp/tools.py` | Only safe tools exposed. |
| `README.md` personality layer | DROP | Not appropriate for client-facing deterministic reports. | None | Reports remain neutral and evidence-based. |
| `README.md` win-rate-improvement claims | DROP | Unsupported exact improvement claims are forbidden. | None | Output grader checks no win-rate guarantee. |
| `mcp_config.json` memory server | QUARANTINE | External state must not become authoritative. | None until security review | Verify no card/legality truth from memory. |
| `mcp_config.json` sequential-thinking server | DROP | Hidden reasoning exposure is not needed. | None | No chain-of-thought tooling requirement. |
| `mcp_config.json` omnisearch server | QUARANTINE | Potentially useful for research, but API-key and supply-chain risk. | Future metagame research adapter | Source snapshot and citation tests. |
| `requirements.txt` / `pyproject.toml` dependency list | QUARANTINE | Too heavy for foundation; contains LLM/ML/runtime dependencies. | Future optional extras only | Dependency audit and lockfile review. |
| `docs/` | ADAPT | Potential architecture/security guidance. | `docs/legacy_notes/` after review | Link-check and relevance review. |
| `tests/` | ADAPT | Potential test patterns. | `tests/legacy_adapted/` after review | Must pass offline and avoid external service calls. |
| Deployment scripts | QUARANTINE | Likely HF/GitHub-specific actions and credentials handling. | None | Static security review before use. |

## 7. Proposed destination paths

- UI ideas: `src/mtgdeckbuilder/ui/gradio_app.py`
- Safe MCP tool wrappers: `src/mtgdeckbuilder/mcp/tools.py`
- CSV parser improvements: `src/mtgdeckbuilder/ingest/collection_parser.py`
- Experiment/event log ideas: `src/mtgdeckbuilder/observability/experiment_logger.py`
- Security notes: `docs/security.md` after future docs inspection
- Legacy excerpts, if downloaded later: `external/hf_vawlrathh/raw/`

## 8. Required tests before integration

- `test_collection_parser_large_csv_chunked`
- `test_gradio_adapter_calls_services_only`
- `test_mcp_validate_deck_rejects_unknown_card`
- `test_mcp_simulate_opening_hands_no_winrate_claim`
- `test_experiment_log_contains_source_hashes`
- `test_no_ui_import_external_services`
- `test_no_tool_controls_arena_client`

## 9. Security notes

- Do not copy `.env.example` values into runtime secrets.
- Do not execute `npx` MCP servers from the legacy config without pinning and review.
- Do not import legacy deployment scripts into the project path.
- Treat all legacy service calls as untrusted until audited.
- Keep API-key-dependent research separate from deterministic validation.
- Do not let memory, chat, or prompt state override card database, collection index, or validator output.

## 10. Final incorporation recommendation

Do not import legacy code directly. Use it only as UI/MCP inspiration after file-level review. The foundation should continue with deterministic validators, local structured card data loaders, collection ownership checks, simulators, CLI, tests, and experiment logs. The first safe incorporation target is a thin Gradio UI that calls the existing parser, validator, simulator, exporter, and logger services.
