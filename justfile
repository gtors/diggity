# Diggity — Rust + Python (PyO3) library
#
# Common tasks: `just --list`
#
# Notes:
#   * The project is not managed by uv (`[tool.uv] package = false`), so the
#     extension has to be installed separately: `uv pip install -e .`.
#   * `uv sync` uninstalls anything it does not manage, hence `--inexact` in
#     the `deps` recipe — otherwise it removes the built extension.
#   * The editable install is a *copy* of the compiled extension, so it has to
#     be rebuilt whenever the Rust sources change. That is why `test` depends
#     on `build`.

out_dir := "dist"

# List available recipes
default:
    @just --list

# Install dev + testing dependencies
deps:
    uv sync --inexact --group testing

# Build the extension and install it into the active environment
build: deps
    uv pip install -e .

# Build release wheels into dist/
build-release:
    uv run maturin build --release --out {{out_dir}} --find-interpreter

# Build the source distribution into dist/
build-sdist:
    uv run maturin sdist --out {{out_dir}}

# Run the test suite
test *args: build
    uv run --no-sync pytest {{args}}

# Run the test suite with coverage
test-cov *args: build
    uv run --no-sync pytest --cov=diggity --cov-report=term-missing {{args}}

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
    uv run ruff format src tests
    uv run ruff check --fix src tests

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
