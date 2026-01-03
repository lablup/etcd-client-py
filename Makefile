# Build targets
build:
	uv run maturin build

install:
	uv sync --all-extras
	uv run maturin develop

# Test targets
test:
	uv run pytest

# Utility targets
etcd-clear:
	etcdctl del "" --from-key=true

# Python formatting and linting
fmt-py:
	uv run ruff format tests/ etcd_client.pyi

lint-py:
	uv run ruff check tests/ etcd_client.pyi

fix-py:
	uv run ruff format tests/ etcd_client.pyi
	uv run ruff check --fix tests/ etcd_client.pyi

typecheck:
	uv run mypy tests/ etcd_client.pyi

# Rust formatting and linting
fmt-rust:
	cargo fmt

lint-rust:
	cargo clippy

fix-rust:
	cargo fmt
	cargo clippy --fix --allow-dirty --allow-staged

# Combined targets
fmt: fmt-py fmt-rust

lint: lint-py typecheck lint-rust

fix: fix-py fix-rust

check: lint
	@echo "All checks passed!"

.PHONY: build install test etcd-clear fmt-py lint-py fix-py typecheck fmt-rust lint-rust fix-rust fmt lint fix check
