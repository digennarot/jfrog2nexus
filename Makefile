.PHONY: test test-integration build clippy check

build:
	cargo build --release

check:
	cargo check

test:
	cargo test

test-integration: ## Run Rust integration tests against the Docker simulation
	cargo test --test integration_test

clippy:
	cargo clippy -- -D warnings
	cargo fmt --all -- --check

.PHONY: test-env-up
test-env-up: ## Start the Docker mock test environment
	docker compose -f tests/docker-compose.yml up -d

.PHONY: test-env-down
test-env-down: ## Stop the Docker mock test environment
	docker compose -f tests/docker-compose.yml down

.PHONY: real-test-env-up
real-test-env-up: ## Start the Docker real test environment (Artifactory OSS + Nexus)
	docker compose -f tests/docker-compose.real.yml up -d

.PHONY: real-test-env-down
real-test-env-down: ## Stop the Docker real test environment
	docker compose -f tests/docker-compose.real.yml down

.PHONY: real-test-bootstrap
real-test-bootstrap: ## Setup repositories and artifacts in the real test environment
	@./tests/scripts/bootstrap_real.sh

.PHONY: test-integration-real
test-integration-real: ## Run Rust integration tests against real services
	J2N_ALLOW_HTTP=true J2N_REAL_SERVICES=true NEXUS_URL=http://localhost:8083 cargo test --test integration_test -- --nocapture
