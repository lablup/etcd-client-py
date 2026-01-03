build:
	maturin build

install:
	uv pip install -r requirements.txt
	maturin develop
	uv pip install -e ".[dev]"

test:
	uv run pytest

etcd-clear:
	etcdctl del "" --from-key=true

# Python formatting and linting
fmt-py:
	ruff format tests/ etcd_client.pyi

lint-py:
	ruff check tests/ etcd_client.pyi

fix-py:
	ruff format tests/ etcd_client.pyi
	ruff check --fix tests/ etcd_client.pyi

typecheck:
	mypy tests/ etcd_client.pyi

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

# Convenience target for pre-commit checks
check: lint
	@echo "All checks passed!"

.PHONY: build install etcd-clear fmt-py lint-py fix-py typecheck fmt-rust lint-rust fix-rust fmt lint fix check
