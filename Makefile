.PHONY: test lint fmt clippy eval-smoke export-sample smoke web

test:
	cargo test --all-features

lint:
	cargo fmt --check
	cargo clippy --all-targets --all-features -- -D warnings

fmt:
	cargo fmt

clippy:
	cargo clippy --all-targets --all-features -- -D warnings

eval-smoke:
	cargo run --bin mtgdeckbuilder -- eval-smoke \
		--deck examples/sample_deck.txt \
		--cards tests/fixtures/cards_scryfall.json \
		--collection tests/fixtures/collection.csv \
		--format standard \
		--trials 200

export-sample:
	cargo run --bin mtgdeckbuilder -- export \
		--deck examples/sample_deck.txt

smoke:
	cargo run --bin mtgdeckbuilder -- validate --deck examples/sample_deck.txt --cards tests/fixtures/cards_scryfall.json --collection tests/fixtures/collection.csv --format standard
	cargo run --bin mtgdeckbuilder -- import-catalog --input examples/sample_catalog.csv
	cargo run --bin mtgdeckbuilder -- simulate --deck examples/sample_deck.txt --cards tests/fixtures/cards_scryfall.json --queue bo1 --trials 25 --seed 7
	cargo run --bin mtgdeckbuilder -- export --deck examples/sample_deck.txt
	cargo run --bin mtgdeckbuilder -- report --deck examples/sample_deck.txt --cards tests/fixtures/cards_scryfall.json --collection tests/fixtures/collection.csv --format standard --output markdown --trials 25 --seed 7
	cargo run --bin mtgdeckbuilder -- schema --name catalog
	cargo run --bin mtgdeckbuilder -- llm-artifact --deck examples/sample_deck.txt --cards tests/fixtures/cards_scryfall.json --format standard --trials 25 --seed 7

web:
	cargo run --bin mtgdeckbuilder-web
