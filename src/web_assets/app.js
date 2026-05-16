const els = {
  deckText: document.querySelector("#deckText"),
  deckFile: document.querySelector("#deckFile"),
  catalogText: document.querySelector("#catalogText"),
  catalogFile: document.querySelector("#catalogFile"),
  catalogFormat: document.querySelector("#catalogFormat"),
  collectionText: document.querySelector("#collectionText"),
  collectionFile: document.querySelector("#collectionFile"),
  resultLogText: document.querySelector("#resultLogText"),
  resultLogFile: document.querySelector("#resultLogFile"),
  resultLogFormat: document.querySelector("#resultLogFormat"),
  replayFile: document.querySelector("#replayFile"),
  replayText: document.querySelector("#replayText"),
  formatName: document.querySelector("#formatName"),
  queue: document.querySelector("#queue"),
  trials: document.querySelector("#trials"),
  seed: document.querySelector("#seed"),
  inputStatus: document.querySelector("#inputStatus"),
  analyze: document.querySelector("#analyze"),
  loadSample: document.querySelector("#loadSample"),
  clearInputs: document.querySelector("#clearInputs"),
  replayReport: document.querySelector("#replayReport"),
  exportJson: document.querySelector("#exportJson"),
  exportMarkdown: document.querySelector("#exportMarkdown"),
  exportCsv: document.querySelector("#exportCsv"),
  statusStrip: document.querySelector("#statusStrip"),
  statusText: document.querySelector("#statusText"),
  errorPanel: document.querySelector("#errorPanel"),
  results: document.querySelector("#results"),
};

let currentReport = null;
let replayText = "";
let replayMode = false;

const sampleDeck = `Deck
24 Forest
4 Test Bear
4 Test Ranger
4 Test Trick
4 Test Growth
4 Test Removal
4 Test Engine
4 Test Hydra
4 Test Scout
4 Test Sentinel

Sideboard
3 Test Shield
3 Test Naturalize
3 Test Grave Hate
3 Test Sweeper Guard
3 Test Control Plan
`;

const sampleCatalog = `Name,Type,Mana Cost,CMC,Colors,Color Identity,Rarity,Legalities
Forest,Basic Land,,0,,G,common,standard:legal;alchemy:legal;historic:legal;explorer:legal;timeless:legal
Test Bear,Creature - Bear,{G},1,G,G,common,standard:legal;alchemy:legal;historic:legal;explorer:legal;timeless:legal
Test Ranger,Creature - Ranger,{1}{G},2,G,G,uncommon,standard:legal;alchemy:legal;historic:legal;explorer:legal;timeless:legal
Test Trick,Instant,{G},1,G,G,common,standard:legal;alchemy:legal;historic:legal;explorer:legal;timeless:legal
Test Growth,Instant,{G},1,G,G,common,standard:legal;alchemy:legal;historic:legal;explorer:legal;timeless:legal
Test Removal,Instant,{1}{G},2,G,G,uncommon,standard:legal;alchemy:legal;historic:legal;explorer:legal;timeless:legal
Test Engine,Enchantment,{2}{G},3,G,G,rare,standard:legal;alchemy:legal;historic:legal;explorer:legal;timeless:legal
Test Hydra,Creature - Hydra,{X}{G},2,G,G,mythic,standard:legal;alchemy:legal;historic:legal;explorer:legal;timeless:legal
Test Scout,Creature - Scout,{G},1,G,G,common,standard:legal;alchemy:legal;historic:legal;explorer:legal;timeless:legal
Test Sentinel,Creature - Elf,{2}{G},3,G,G,uncommon,standard:legal;alchemy:legal;historic:legal;explorer:legal;timeless:legal
Test Shield,Instant,{G},1,G,G,common,standard:legal;alchemy:legal;historic:legal;explorer:legal;timeless:legal
Test Naturalize,Instant,{1}{G},2,G,G,common,standard:legal;alchemy:legal;historic:legal;explorer:legal;timeless:legal
Test Grave Hate,Artifact,{1},1,,G,uncommon,standard:legal;alchemy:legal;historic:legal;explorer:legal;timeless:legal
Test Sweeper Guard,Creature - Treefolk,{2}{G},3,G,G,rare,standard:legal;alchemy:legal;historic:legal;explorer:legal;timeless:legal
Test Control Plan,Enchantment,{2}{G},3,G,G,rare,standard:legal;alchemy:legal;historic:legal;explorer:legal;timeless:legal
`;

const sampleCollection = `Name,Quantity
Forest,24
Test Bear,4
Test Ranger,4
Test Trick,4
Test Growth,4
Test Removal,4
Test Engine,4
Test Hydra,4
Test Scout,4
Test Sentinel,4
Test Shield,3
Test Naturalize,3
Test Grave Hate,3
Test Sweeper Guard,3
Test Control Plan,3
`;

const sampleResultLog = `record_type,match_id,game_number,queue,format,play_draw,won,mulligans,sideboarded,opponent_archetype,draft_id,card_name,pack_number,pick_number,seen_at_pick,taken,opening_hand_games,opening_hand_wins,drawn_games,drawn_wins,games,wins,trophies,events,color_pair,archetype,wheeled
game,m1,1,bo1,standard,play,true,0,false,aggro,,,,,,,,,,,,,,,,,
game,m2,1,bo3,standard,draw,loss,1,true,control,,,,,,,,,,,,,,,,,
game,m2,2,bo3,standard,play,win,0,true,control,,,,,,,,,,,,,,,,,
draft_pick,,,,,,,,,,d1,Test Bear,1,2,3,true,3,2,5,4,8,5,1,2,G,stompy,false
`;

function textReady(value) {
  return value.trim().length > 0;
}

function setStatus(message, tone = "neutral") {
  const dot = els.statusStrip.querySelector(".status-dot");
  dot.className = `status-dot ${tone}`;
  els.statusText.textContent = message;
}

function setError(message) {
  if (!message) {
    els.errorPanel.classList.add("hidden");
    els.errorPanel.textContent = "";
    return;
  }
  els.errorPanel.classList.remove("hidden");
  els.errorPanel.textContent = message;
}

function updateReadyState() {
  const ready = textReady(els.deckText.value) && textReady(els.catalogText.value);
  els.analyze.disabled = !ready;
  els.inputStatus.textContent = ready
    ? "Inputs ready. Review formats, seed, and trials before analysis."
    : "Add a decklist and catalog to begin.";
  if (!currentReport && !ready) {
    renderEmpty();
    setStatus("Add a decklist and catalog to begin.", "neutral");
  } else if (!currentReport && ready) {
    renderFileLoaded();
    setStatus("Inputs ready. Review formats, seed, and trials before analysis.", "warning");
  }
}

async function readFileInto(input, target, afterRead) {
  const file = input.files?.[0];
  if (!file) return;
  target.value = await file.text();
  afterRead?.(file);
  updateReadyState();
}

function panel(title, body, badge = "") {
  return `<article class="panel"><h3>${escapeHtml(title)}${badge}</h3>${body}</article>`;
}

function badge(text, tone = "") {
  return `<span class="badge ${tone}">${escapeHtml(text)}</span>`;
}

function metric(label, value) {
  return `<div class="metric"><span>${escapeHtml(label)}</span><strong>${escapeHtml(value)}</strong></div>`;
}

function escapeHtml(value) {
  return String(value ?? "")
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#39;");
}

function percent(value) {
  if (typeof value !== "number" || Number.isNaN(value)) return "n/a";
  return `${Math.round(value * 1000) / 10}%`;
}

function number(value) {
  if (typeof value !== "number" || Number.isNaN(value)) return "n/a";
  return Number.isInteger(value) ? String(value) : String(Math.round(value * 1000) / 1000);
}

function rateCell(rate) {
  if (!rate || typeof rate !== "object") return "n/a";
  return `${percent(rate.rate)} (${rate.successes ?? 0}/${rate.samples ?? 0})`;
}

function renderEmpty() {
  currentReport = null;
  replayMode = false;
  setExports(false);
  els.results.innerHTML = panel(
    "Setup checklist",
    `<ul class="empty-list">
      <li>Add an Arena-style decklist.</li>
      <li>Add a user-supplied catalog in CSV, JSON, JSONL, or YAML.</li>
      <li>Optionally add collection CSV and structured result-log.v1 data.</li>
      <li>Click Analyze to run local deterministic Rust services.</li>
    </ul>`
  );
}

function renderFileLoaded() {
  els.results.innerHTML = panel(
    "Inputs loaded",
    `<div class="metric-grid">
      ${metric("Deck characters", String(els.deckText.value.trim().length))}
      ${metric("Catalog format", els.catalogFormat.value)}
      ${metric("Result log", textReady(els.resultLogText.value) ? "supplied" : "not supplied")}
    </div>
    <p class="notice">Source hashes will be computed from uploaded strings during analysis. Uploaded content is not persisted unless you export a report.</p>`
  );
}

function setExports(enabled) {
  els.exportJson.disabled = !enabled;
  els.exportMarkdown.disabled = !enabled;
  els.exportCsv.disabled = !enabled;
}

function hasLowSample(report) {
  const text = JSON.stringify(report.result_log ?? {});
  return text.includes("Fewer than 30 samples") || text.includes('"reliability":"low"');
}

function renderReport(report, mode = "analysis") {
  currentReport = report;
  replayMode = mode === "replay";
  setExports(true);
  setError("");

  const validation = report.validation ?? {};
  const opening = report.opening_hand ?? {};
  const early = report.early_turns ?? {};
  const features = report.features ?? {};
  const consistency = report.consistency ?? {};
  const resultLog = report.result_log ?? null;
  const constructed = resultLog?.constructed ?? null;
  const statusTone = validation.status === "PASS" ? "success" : "danger";
  const lowSample = hasLowSample(report);

  setStatus(
    replayMode
      ? "Report replay mode. This view was rendered from an existing analysis-report.v1."
      : "Analysis complete. Outputs are local, seeded, and reproducible.",
    lowSample ? "warning" : statusTone
  );

  const sections = [
    renderValidation(validation),
    renderSimulation(opening, early, consistency),
    renderResultLog(resultLog, constructed),
    renderFeatures(features),
    renderHashes(report.source_hashes ?? {}),
    renderAssumptions(report.assumptions ?? [], lowSample),
    renderRawPreview(report),
  ];

  els.results.innerHTML = sections.join("");
}

function renderValidation(validation) {
  const issues = validation.issues ?? [];
  const issueBody = issues.length
    ? `<ul class="issue-list">${issues
        .map((issue) => `<li><strong>${escapeHtml(issue.severity)}</strong> ${escapeHtml(issue.message)}</li>`)
        .join("")}</ul>`
    : `<p>No validation issues reported.</p>`;
  const wildcards = validation.wildcards_required ?? {};
  const wildcardRows = Object.entries(wildcards)
    .map(([rarity, count]) => `<tr><td>${escapeHtml(rarity)}</td><td>${escapeHtml(count)}</td></tr>`)
    .join("");
  return panel(
    "Validation",
    `<div class="metric-grid">
      ${metric("Status", validation.status ?? "UNKNOWN")}
      ${metric("Main count", String(validation.main_count ?? "n/a"))}
      ${metric("Sideboard count", String(validation.sideboard_count ?? "n/a"))}
    </div>
    ${issueBody}
    ${
      wildcardRows
        ? `<h4>Wildcards required</h4><div class="table-wrap"><table><thead><tr><th>Rarity</th><th>Count</th></tr></thead><tbody>${wildcardRows}</tbody></table></div>`
        : ""
    }`,
    badge(validation.status ?? "UNKNOWN", validation.status === "PASS" ? "success" : "danger")
  );
}

function renderSimulation(opening, early, consistency) {
  const openingMetrics = opening.metrics ?? {};
  const earlyMetrics = early.metrics ?? {};
  return panel(
    "Simulation",
    `<div class="metric-grid">
      ${metric("Consistency score", number(consistency.consistency_score))}
      ${metric("Keepable 7", percent(openingMetrics.keepable_7_rate))}
      ${metric("Turn 1/2 play", percent(openingMetrics.turn_1_or_2_play_rate))}
      ${metric("Threat by turn 3", percent(earlyMetrics.threat_by_turn_3_rate))}
      ${metric("Missed land drop", percent(earlyMetrics.missed_land_drop_before_turn_3_rate))}
      ${metric("Trials / seed", `${opening.trials ?? "n/a"} / ${opening.seed ?? "n/a"}`)}
    </div>`
  );
}

function renderResultLog(resultLog, constructed) {
  if (!resultLog) {
    return panel(
      "Result-log summary",
      `<p>No result log supplied. Match, matchup, play/draw, and draft-card summaries are hidden.</p>`,
      badge("not supplied")
    );
  }
  const source = resultLog.source ?? {};
  const matchupRows = tableRows(constructed?.matchup_matrix ?? {});
  const playDrawRows = tableRows(constructed?.play_draw_performance ?? {});
  const draftRows = Object.values(resultLog.draft_cards ?? {})
    .map(
      (card) => `<tr>
        <td>${escapeHtml(card.card_name)}</td>
        <td>${rateCell(card.card_win_rate)}</td>
        <td>${rateCell(card.game_in_hand_win_rate)}</td>
        <td>${escapeHtml(card.sample_size_reliability ?? "n/a")}</td>
      </tr>`
    )
    .join("");
  return panel(
    "Result-log summary",
    `<div class="metric-grid">
      ${metric("Games", String(source.game_count ?? constructed?.games ?? 0))}
      ${metric("Matches", String(constructed?.matches ?? 0))}
      ${metric("Game win rate", rateCell(constructed?.game_win_rate))}
      ${metric("Match win rate", rateCell(constructed?.match_win_rate))}
      ${metric("Sideboard impact", rateCell(constructed?.sideboard_impact))}
      ${metric("Draft cards", String(Object.keys(resultLog.draft_cards ?? {}).length))}
    </div>
    <p class="notice">Low sample size: treat these rates as directional.</p>
    ${metricTable("Play/draw split", playDrawRows)}
    ${metricTable("Matchup rows", matchupRows)}
    ${
      draftRows
        ? `<h4>Draft card summary</h4><div class="table-wrap"><table><thead><tr><th>Card</th><th>Card win rate</th><th>In-hand win rate</th><th>Reliability</th></tr></thead><tbody>${draftRows}</tbody></table></div>`
        : ""
    }`,
    badge("loaded", "success")
  );
}

function tableRows(map) {
  return Object.entries(map)
    .map(
      ([name, rate]) => `<tr>
        <td>${escapeHtml(name)}</td>
        <td>${rateCell(rate)}</td>
        <td>${escapeHtml(rate?.reliability ?? "n/a")}</td>
      </tr>`
    )
    .join("");
}

function metricTable(title, rows) {
  if (!rows) return "";
  return `<h4>${escapeHtml(title)}</h4><div class="table-wrap"><table><thead><tr><th>Name</th><th>Rate</th><th>Reliability</th></tr></thead><tbody>${rows}</tbody></table></div>`;
}

function renderFeatures(features) {
  return panel(
    "Deck features",
    `<div class="metric-grid">
      ${metric("Deck size", String(features.deck_size ?? "n/a"))}
      ${metric("Land count", String(features.land_count ?? "n/a"))}
      ${metric("Land ratio", percent(features.land_ratio))}
      ${metric("Avg nonland mana value", number(features.average_nonland_mana_value))}
      ${metric("Threat density", percent(features.threat_density))}
      ${metric("Interaction density", percent(features.interaction_density))}
    </div>`
  );
}

function renderHashes(hashes) {
  const rows = Object.entries(hashes)
    .map(([name, hash]) => `<div class="hash-row"><span>${escapeHtml(name)}</span><code title="${escapeHtml(hash)}">${escapeHtml(hash)}</code></div>`)
    .join("");
  return panel("Source hashes", rows ? `<div class="hash-list">${rows}</div>` : "<p>No source hashes reported.</p>");
}

function renderAssumptions(assumptions, lowSample) {
  const warning = lowSample ? `<p class="notice">Low sample size: treat these rates as directional.</p>` : "";
  const items = assumptions
    .map((assumption) => `<li>${escapeHtml(assumption)}</li>`)
    .join("");
  return panel("Assumptions", `${warning}<ul class="assumption-list">${items}</ul>`);
}

function renderRawPreview(report) {
  return panel("Report preview", `<pre class="json-preview">${escapeHtml(JSON.stringify(report, null, 2))}</pre>`);
}

function requestPayload() {
  return {
    deck_text: els.deckText.value,
    catalog_text: els.catalogText.value,
    catalog_format: els.catalogFormat.value,
    collection_text: els.collectionText.value.trim() ? els.collectionText.value : null,
    result_log_text: els.resultLogText.value.trim() ? els.resultLogText.value : null,
    result_log_format: els.resultLogFormat.value,
    format_name: els.formatName.value,
    queue: els.queue.value,
    trials: Number.parseInt(els.trials.value, 10) || 250,
    seed: Number.parseInt(els.seed.value, 10) || 1,
  };
}

async function analyze() {
  setError("");
  currentReport = null;
  setExports(false);
  setStatus("Running deterministic local analysis...", "busy");
  els.results.innerHTML = panel(
    "Analyzing",
    `<div class="metric-grid">
      ${metric("Validation", "pending")}
      ${metric("Simulation", "pending")}
      ${metric("Report", "pending")}
    </div>`
  );
  els.analyze.disabled = true;
  try {
    const response = await fetch("/api/analyze", {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify(requestPayload()),
    });
    const data = await response.json();
    if (!response.ok) {
      throw new Error(data.error || `Analysis failed with HTTP ${response.status}`);
    }
    renderReport(data, "analysis");
  } catch (error) {
    setStatus("Analysis could not run. Fix the issues below and try again.", "danger");
    setError(error.message);
    els.results.innerHTML = panel(
      "Invalid input",
      `<p>Analysis could not run. Fix the issues above and try again.</p>`
    );
  } finally {
    els.analyze.disabled = !(textReady(els.deckText.value) && textReady(els.catalogText.value));
  }
}

function loadSample() {
  els.deckText.value = sampleDeck;
  els.catalogText.value = sampleCatalog;
  els.catalogFormat.value = "csv";
  els.collectionText.value = sampleCollection;
  els.resultLogText.value = sampleResultLog;
  els.resultLogFormat.value = "csv";
  els.trials.value = "50";
  els.seed.value = "42";
  replayText = "";
  currentReport = null;
  replayMode = false;
  updateReadyState();
}

function clearInputs() {
  for (const key of ["deckText", "catalogText", "collectionText", "resultLogText"]) {
    els[key].value = "";
  }
  els.replayText.value = "";
  for (const key of ["deckFile", "catalogFile", "collectionFile", "resultLogFile", "replayFile"]) {
    els[key].value = "";
  }
  replayText = "";
  currentReport = null;
  replayMode = false;
  setError("");
  renderEmpty();
  updateReadyState();
}

function replayReport() {
  try {
    const report = JSON.parse(replayText || els.replayText.value || "{}");
    if (report.schema_version !== "analysis-report.v1") {
      throw new Error('Report replay requires schema_version "analysis-report.v1".');
    }
    renderReport(report, "replay");
  } catch (error) {
    setStatus("Analysis could not run. Fix the issues below and try again.", "danger");
    setError(error.message);
  }
}

async function exportReport(output) {
  if (!currentReport) return;
  const response = await fetch("/api/report/render", {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ report: currentReport, output }),
  });
  const data = await response.text();
  if (!response.ok) {
    setError(data);
    return;
  }
  const extension = output === "markdown" ? "md" : output;
  const mime = output === "json" ? "application/json" : output === "csv" ? "text/csv" : "text/markdown";
  const blob = new Blob([data], { type: mime });
  const url = URL.createObjectURL(blob);
  const link = document.createElement("a");
  link.href = url;
  link.download = `analysis-report.${extension}`;
  link.click();
  URL.revokeObjectURL(url);
}

els.deckText.addEventListener("input", updateReadyState);
els.catalogText.addEventListener("input", updateReadyState);
els.resultLogText.addEventListener("input", updateReadyState);
els.collectionText.addEventListener("input", updateReadyState);
els.deckFile.addEventListener("change", () => readFileInto(els.deckFile, els.deckText));
els.catalogFile.addEventListener("change", () =>
  readFileInto(els.catalogFile, els.catalogText, (file) => {
    const ext = file.name.split(".").pop()?.toLowerCase();
    if (ext === "yml") els.catalogFormat.value = "yaml";
    if (["csv", "json", "jsonl", "yaml"].includes(ext)) els.catalogFormat.value = ext;
  })
);
els.collectionFile.addEventListener("change", () => readFileInto(els.collectionFile, els.collectionText));
els.resultLogFile.addEventListener("change", () =>
  readFileInto(els.resultLogFile, els.resultLogText, (file) => {
    const ext = file.name.split(".").pop()?.toLowerCase();
    if (["csv", "json", "jsonl"].includes(ext)) els.resultLogFormat.value = ext;
  })
);
els.replayFile.addEventListener("change", async () => {
  const file = els.replayFile.files?.[0];
  replayText = file ? await file.text() : "";
  els.replayText.value = replayText;
});
els.analyze.addEventListener("click", analyze);
els.loadSample.addEventListener("click", loadSample);
els.clearInputs.addEventListener("click", clearInputs);
els.replayReport.addEventListener("click", replayReport);
els.exportJson.addEventListener("click", () => exportReport("json"));
els.exportMarkdown.addEventListener("click", () => exportReport("markdown"));
els.exportCsv.addEventListener("click", () => exportReport("csv"));

renderEmpty();
updateReadyState();
