PYTHON ?= python3
export PYTHONPATH := src

.PHONY: test lint eval-smoke export-sample

test:
	$(PYTHON) -m unittest discover -s tests

lint:
	$(PYTHON) -m compileall -q mtgdeckbuilder src tests

eval-smoke:
	$(PYTHON) -m mtgdeckbuilder eval-smoke data/raw/sample_arena_deck.txt --cards data/processed/sample_cards.json

export-sample:
	$(PYTHON) -m mtgdeckbuilder export data/raw/sample_arena_deck.txt
