# Steam/Arena Workbench Research Brief

Date: 2026-05-16  
Scope: public-source research for a local-only `mtg-deckbuilder` web workbench. This is product research, not legal advice.

## Official Source Snapshot

- The Steam listing for Magic: The Gathering Arena identifies Wizards of the Coast LLC as developer and publisher, lists the release date as May 23, 2023, marks the game as free to play, includes in-app purchases, and says it requires a Wizards Account System account and third-party EULA. The same listing showed mixed Steam review sentiment at the time of research: recent English reviews were 54% positive across 821 reviews, and all English reviews were 58% positive across 20,500 reviews. Source: [Steam store listing](https://store.steampowered.com/app/2141910/Magic_The_Gathering_Arena/).
- Wizards' May 22, 2023 MTG Arena announcement said Arena would arrive on Steam on May 23, 2023, with users playing through their MTG Arena account on a Windows PC and logging in via QR code plus the Steam mobile app. Source: [Wizards MTG Arena announcement, May 22, 2023](https://magic.wizards.com/en/news/mtg-arena/mtg-arena-announcements-may-22-2023).
- Wizards support documents detailed logs and specific PC/Mac/Steam log locations. This is evidence that local logs exist for support and community tools, not permission to automate, tail, scrape, or control the client. Source: [Creating Log Files on PC/Mac/Steam](https://mtgarena-support.wizards.com/hc/en-us/articles/360000726823-Creating-Log-Files-on-PC-Mac-Steam).
- Wizards' store FAQ says MTG Arena is free to download, gems are optional purchases, purchases are not required to access the full depth of gameplay, and gems can be purchased or earned in limited amounts through events. Source: [Store, Purchases, and Collection FAQ](https://mtgarena-support.wizards.com/hc/en-us/articles/115004569623-Store-Purchases-and-Collection-FAQ).
- Wizards' Fan Content Policy requires unofficial status, free access when using Wizards IP, no Wizards logos/trademarks without permission, and no sale/licensing of Wizards-related fan content without permission. It allows donations, sponsorships, and ad revenue when those do not interfere with community access. Source: [Wizards Fan Content Policy](https://company.wizards.com/en/legal/fancontentpolicy).

## Non-Authoritative Community Signals

These sources are useful for product signal only. They do not define policy, legality, or implementation permission.

- Steam review aggregates are mixed, which suggests real user friction even for an official free-to-play client. The store also foregrounds online PvP, cross-platform multiplayer, and in-app purchases, so users arriving from Steam are likely balancing gameplay interest with account, purchase, and client-friction expectations. Source: [Steam store listing](https://store.steampowered.com/app/2141910/Magic_The_Gathering_Arena/).
- Steam community threads sampled during research include installation/update failures, asset retrieval problems, black-screen/loading issues, connection failures, and users sharing local troubleshooting steps such as verifying files, reinstalling, updating Windows/drivers, and checking logs. Sources: [Steam discussion: WTH Happened to MTGA?](https://steamcommunity.com/app/2141910/discussions/0/597402121489405497/) and [Steam discussion: Connection problems?](https://steamcommunity.com/app/2141910/discussions/0/4040356791033524916/?l=dutch).
- Reddit tracker discussions show demand for private local analytics, match history, deck performance, matchup history, export/import, collection export, draft pool/pick review, clearer installation, and better format detection. They also show some demand for overlays, but overlays are excluded from this repo slice. Sources: [Reddit: open-source MTG Arena tracker project](https://www.reddit.com/r/MagicArena/comments/1rixse8/project_mtg_arena_tracker_opensource_match/) and [Reddit: review of deck trackers](https://www.reddit.com/r/MagicArena/comments/eyqpbv/review_of_various_deck_trackers_for_magic_arena/).
- Reddit posts also surface performance pain in the desktop client and ongoing interest in economy/collection tracking. These are product needs for offline report analysis, not justification for account integration or live client inspection. Sources: [Reddit: bug and optimization issues](https://www.reddit.com/r/MagicArena/comments/1kc2o1y/bug_and_optimization_issues/) and [Reddit: economy tracker](https://www.reddit.com/r/MagicArena/comments/vr1nxa/any_economy_tracker/).

## Steam User Needs

- A workbench that opens locally and works without Steam login, MTG Arena login, payment flow, or hosted service.
- Clear import paths for decklists, catalogs, collection CSV files, user-owned result logs, and previously exported reports.
- Fast feedback on validation issues, missing catalog entries, ownership gaps, format assumptions, queue assumptions, and low sample size.
- Plain-English summaries backed by deterministic report fields, source hashes, and exportable JSON/Markdown/CSV.
- Report replay for users who want to inspect, share, or compare an existing `analysis-report.v1` without rerunning simulation.
- Simple installation and no heavy frontend toolchain in V1, because community tracker discussions show that non-technical users can get stuck on source-build workflows.

## Safe Local-Data Opportunities

- Decklists: paste or upload Arena-style deck text, then validate and export without contacting Arena.
- Catalog files: ingest user-supplied CSV, JSON, JSONL, or YAML catalogs and label bundled examples as dev fixtures only.
- Collection CSV: let users bring their own collection snapshot and explain ownership/wildcard assumptions locally.
- Result logs: accept explicit user-supplied `result-log.v1` CSV, JSON, or JSONL files for match/draft analytics. Do not tail, watch, parse live Arena files, or auto-discover client paths in this slice.
- Report replay: accept existing `analysis-report.v1` JSON for rendering and export.
- Source hashes: compute hashes from uploaded strings in memory so users can compare reports without persisting raw uploads.

## Excluded Opportunities

- MTG Arena client control, Steam client automation, memory inspection, screen scraping, overlays, tracker overlays, protected APIs, reverse engineering, account login, purchase integration, gameplay automation, live log watching, and automatic log-path discovery.
- Bundled Wizards logos, card art, proprietary Arena data, private account data, competitor schemas, or claims of exact MTG Arena parity.

## Monetization Constraints

- Keep the local workbench and any fan-content-facing Wizards IP references free to access unless separate written permission exists.
- Paid commercial layers should be framed around original software and user-owned data: hosted compute, private team dashboards, report storage, API access, collaboration, and export convenience.
- Do not gate Wizards IP, represent the project as official, use Wizards logos/trademarks, or sell Wizards-related fan content as the paid product.
- Use the existing unofficial disclaimer in user-facing docs and UI. Any hosted or paid product should get separate legal review before launch.

## UX Implications

- First screen should be the actual workbench: input pane for deck/catalog/collection/result-log/report replay and a results pane for validation, simulation, metrics, warnings, source hashes, and exports.
- Default copy should say "Bo1/Bo3-oriented" and "local-only"; avoid "Arena parity," "official," "sync," "tracker," or "live" language.
- Result-log panels should distinguish "no result log," "low sample size," and "loaded result log" states, with game count, match count, play/draw split, matchup rows, sideboard impact, and draft card summary only when supported by supplied data.
- Errors should be written for non-technical users: missing catalog columns, invalid JSON, unknown card names, unsupported format, and report version mismatch should each get a direct fix path.
- Exports should be visible and deterministic: JSON for machines, Markdown for sharing, CSV for spreadsheets.
- Privacy posture should be explicit but non-blocking: uploaded content is processed locally in memory and not persisted unless the user exports a report.

