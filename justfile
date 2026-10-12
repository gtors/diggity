# Diggity — Rust + Python (PyO3) library
#
# Common tasks: `just --list`

python := "uv run"
out_dir := "dist"

# List available recipes
default:
    @just --list

# Install dev + testing dependencies
deps:
    uv sync --group testing

# Build and install the extension in editable mode
build:
    uv pip install -e .

# Build the extension in debug mode
build-debug:
    uv run maturin develop

# Build release wheels into dist/
build-release:
    uv run maturin build --release --out {{out_dir}} --find-interpreter

# Build the source distribution into dist/
build-sdist:
    uv run maturin sdist --out {{out_dir}}

# Install the project in editable mode
install:
    uv pip install -e .

# Run the test suite
test *args:
    uv run rtk pytest {{args}}

# Run the test suite with coverage
test-cov *args:
    uv run rtk pytest --cov=diggity --cov-report=term-missing {{args}}

# Run Rust unit tests
test-rust:
    cargo test

# Run Rust lints (clippy) and formatting check
lint-rust:
    cargo fmt --check
    cargo clippy --all-targets -- -D warnings

# Apply Rust formatting
fmt-rust:
    cargo fmt

# Format Python code with ruff
fmt-python:
    uvx ruff format src tests
    uvx ruff check --fix src tests

# Run all linters
lint: lint-rust fmt-python

# Run cargo check
check-rust:
    cargo check --all-targets

# Build, lint and test everything
ci: lint build test

# Build release wheels and publish to PyPI (requires MATURIN_PYPI_TOKEN)
publish: build-release
    uv run maturin upload --non-interactive --skip-existing {{out_dir}}/*

# Remove build artifacts
clean:
    cargo clean
    rm -rf target {{out_dir}} .pytest_cache .ruff_cache .mypy_cache
    uv pip uninstall diggity || true

# Show available recipes
help:
    @just --list