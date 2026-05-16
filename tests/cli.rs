use assert_cmd::Command;
use predicates::prelude::*;
use pretty_assertions::assert_eq;

#[test]
fn cli_validate_matches_fixture_golden() {
    let mut cmd = Command::cargo_bin("mtgdeckbuilder").unwrap();
    cmd.args([
        "validate",
        "--deck",
        "examples/sample_deck.txt",
        "--cards",
        "tests/fixtures/cards_scryfall.json",
        "--collection",
        "tests/fixtures/collection.csv",
        "--format",
        "standard",
    ]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\"status\": \"PASS\""));
}

#[test]
fn cli_export_matches_fixture_golden() {
    let output = Command::cargo_bin("mtgdeckbuilder")
        .unwrap()
        .args(["export", "--deck", "examples/sample_deck.txt"])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "Deck\n24 Forest\n4 Test Bear\n4 Test Ranger\n4 Test Trick\n4 Test Growth\n4 Test Removal\n4 Test Engine\n4 Test Hydra\n4 Test Scout\n4 Test Sentinel\n\nSideboard\n3 Test Shield\n3 Test Naturalize\n3 Test Grave Hate\n3 Test Sweeper Guard\n3 Test Control Plan\n"
    );
}

#[test]
fn cli_simulate_opening_reports_seeded_chacha_metrics() {
    let output = Command::cargo_bin("mtgdeckbuilder")
        .unwrap()
        .args([
            "simulate-opening",
            "--deck",
            "examples/sample_deck.txt",
            "--cards",
            "tests/fixtures/cards_scryfall.json",
            "--trials",
            "50",
            "--mode",
            "arena_n2",
            "--seed",
            "42",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("\"keepable_7_rate\": 0.98"));
    assert!(stdout.contains("\"screw_risk_opening_rate\": 0.06"));
    assert!(stdout.contains("\"seed\": 42"));
    assert!(stdout.contains("\"trials\": 50"));
    assert!(stdout.contains("Opening-hand quality is not match win rate."));
}

#[test]
fn cli_v1_smoke_commands_work() {
    Command::cargo_bin("mtgdeckbuilder")
        .unwrap()
        .args(["import-catalog", "--input", "examples/sample_catalog.csv"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"card_count\": 3"));

    Command::cargo_bin("mtgdeckbuilder")
        .unwrap()
        .args([
            "import-result-log",
            "--input",
            "tests/fixtures/result_logs.csv",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "\"schema_version\": \"result-log-load-report.v1\"",
        ))
        .stdout(predicate::str::contains("\"game_count\": 3"));

    Command::cargo_bin("mtgdeckbuilder")
        .unwrap()
        .args([
            "simulate",
            "--deck",
            "examples/sample_deck.txt",
            "--cards",
            "tests/fixtures/cards_scryfall.json",
            "--trials",
            "25",
            "--queue",
            "bo1",
            "--seed",
            "7",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "\"schema_version\": \"simulation-result.v1\"",
        ));

    Command::cargo_bin("mtgdeckbuilder")
        .unwrap()
        .args(["schema", "--name", "catalog"])
        .assert()
        .success()
        .stdout(predicate::str::contains("CatalogDocument"));

    Command::cargo_bin("mtgdeckbuilder")
        .unwrap()
        .args(["schema", "--name", "result-log"])
        .assert()
        .success()
        .stdout(predicate::str::contains("ResultLogDocument"));

    Command::cargo_bin("mtgdeckbuilder")
        .unwrap()
        .args([
            "report",
            "--deck",
            "examples/sample_deck.txt",
            "--cards",
            "tests/fixtures/cards_scryfall.json",
            "--collection",
            "tests/fixtures/collection.csv",
            "--format",
            "standard",
            "--output",
            "markdown",
            "--result-log",
            "tests/fixtures/result_logs.json",
            "--trials",
            "25",
            "--seed",
            "7",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("MTG Deck Analysis Report"))
        .stdout(predicate::str::contains("Result Log"));

    Command::cargo_bin("mtgdeckbuilder")
        .unwrap()
        .args([
            "report",
            "--deck",
            "examples/sample_deck.txt",
            "--cards",
            "tests/fixtures/cards_scryfall.json",
            "--collection",
            "tests/fixtures/collection.csv",
            "--format",
            "standard",
            "--output",
            "json",
            "--result-log",
            "tests/fixtures/result_logs.jsonl",
            "--result-log-format",
            "jsonl",
            "--trials",
            "25",
            "--seed",
            "7",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"result_log\""))
        .stdout(predicate::str::contains("\"constructed\""));

    Command::cargo_bin("mtgdeckbuilder")
        .unwrap()
        .args([
            "llm-artifact",
            "--deck",
            "examples/sample_deck.txt",
            "--cards",
            "tests/fixtures/cards_scryfall.json",
            "--format",
            "standard",
            "--trials",
            "25",
            "--seed",
            "7",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "\"schema_version\": \"llm_report.v1\"",
        ));
}
