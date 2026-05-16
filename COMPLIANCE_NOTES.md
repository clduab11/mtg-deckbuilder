# Compliance Notes

Date: 2026-05-16

This repository is an unofficial fan/project tool. It is not affiliated with, endorsed by, sponsored by, or approved by Wizards of the Coast, Hasbro, Magic: The Gathering, or MTG Arena.

## Wizards and MTG Arena Boundaries

- Wizards' General Terms restrict unauthorized data mining, reverse engineering, decompilation, unauthorized connections, circumvention, and cheats/bots. This repo must stay at user-provided files, public data, local fixtures, and documented/exported inputs. Source: [Wizards General Terms](https://company.wizards.com/en/legal/terms).
- Wizards' Fan Content Policy allows free fan content but requires unofficial status and restricts use of logos/trademarks and sale/licensing of Wizards IP without permission. Source: [Wizards Fan Content Policy](https://company.wizards.com/en/legal/fancontentpolicy).
- The repo must not bundle copyrighted card images, proprietary MTG Arena files, private user account data, or protected client data.
- Current legality, bans, restrictions, card text, Arena availability, and format state must come from current authoritative data supplied by the user or lawful public sources.
- MTG Arena is available on Steam as a free-to-play official client with in-app purchases and a third-party Wizards account/EULA. This repo must not imply official affiliation, Steam integration, payment integration, account sync, or client control. Sources: [Steam listing](https://store.steampowered.com/app/2141910/Magic_The_Gathering_Arena/) and [Wizards Store/Purchases FAQ](https://mtgarena-support.wizards.com/hc/en-us/articles/115004569623-Store-Purchases-and-Collection-FAQ).
- Wizards documents local support-log locations, including Steam paths. That is evidence that local logs exist, not permission for this repo to tail logs, auto-discover client paths, parse proprietary client state, or operate as a live tracker. Source: [Creating Log Files on PC/Mac/Steam](https://mtgarena-support.wizards.com/hc/en-us/articles/360000726823-Creating-Log-Files-on-PC-Mac-Steam).

## Local Web Workbench Boundary

- Allowed: user-supplied decklists, catalogs, collection CSV files, structured `result-log.v1` records, source hashes, report rendering, report replay, and deterministic local analysis.
- Not allowed in this slice: MTG Arena or Steam client control, overlays, live trackers, account login/sync, scraping, protected APIs, reverse engineering, memory inspection, gameplay automation, log tailing, automatic log-path discovery, or payment UI integration.
- Future local-log import requires explicit product and compliance review. The safe default is a user-uploaded import format only, not live watching or account extraction.

## Competitor Boundaries

- Public competitor pages may inform generic product positioning and broad interoperable data needs.
- Do not decompile, inspect proprietary binaries, bypass protections, access private APIs, or copy proprietary schemas.
- Untapped.gg add-on research is limited to public pages, public docs, lawful local logs, or user-consented exported data.

## LLM Boundaries

- LLMs consume structured artifacts such as `llm_report.v1`.
- LLMs do not determine validation, simulation, win rates, confidence intervals, or seeded outputs.
- LLM-generated advice must label assumptions and low-sample data.

## Monetization Boundary

The open source engine can support paid hosting, dashboards, API access, creator exports, and team workspaces around original software and user-supplied data. Monetization must not sell Wizards IP as gated fan content without appropriate permission.
