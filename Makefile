# Huuk — repo tasks. Run `make` (or `make help`) to list them.
# Entry-point scripts live in ops/scripts/; this is just a thin front door.
SHELL := bash
.DEFAULT_GOAL := help

DEV := ops/scripts/dev.sh
DEPLOY := ops/scripts/deploy.sh

.PHONY: help dev dev-mock dev-server test build install deploy

help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN{FS=":.*?## "}{printf "  \033[1m%-11s\033[0m %s\n", $$1, $$2}'

dev: ## Run backend + frontend (Ctrl+C stops both)
	$(DEV)

dev-mock: ## Frontend only, in-memory mock data (no Rust)
	$(DEV) --mock

dev-server: ## Backend only (Rust, :8787)
	$(DEV) --server

test: ## Rust tests + frontend tests
	cargo test --manifest-path server/Cargo.toml
	cd app && bun run test

build: ## Release backend + production frontend build
	cargo build --release --manifest-path server/Cargo.toml
	cd app && bun run build

install: ## Docker-free production install (systemd)
	sudo ops/scripts/install.sh

deploy: ## Docker Compose (e.g. make deploy ARGS="init" / ARGS="up --tls")
	$(DEPLOY) $(ARGS)
