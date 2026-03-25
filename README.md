# llm-lint

Static checks for “LLM slop” and related drift: oversized files/functions, duplicate bodies, noisy helpers, risky env fallbacks, and secret-shaped strings. Rust implementation; config via **`llm-lint.toml`** (or **`llm-lint.json`**) in the scan root.

## Install

| Channel | Command |
|--------|---------|
| **Cargo** | `cargo install --path .` or install from crates.io when published |
| **pip** | `pip install llm-lint` (wheels via [maturin](https://www.maturin.rs/)) |
| **npm** | `npm install -D llm-lint-cli` then `npx llm-lint scan .` (uses [uv](https://docs.astral.sh/uv/) + PyPI `llm-lint`) |

## Usage

```bash
llm-lint scan .
llm-lint scan . -f json --fail-threshold 20
```

## Release & publishing

See **[RELEASING.md](RELEASING.md)** for PyPI trusted publishing, npm token, and **`scripts/release.py`**.

## Repo layout

- **`Cargo.toml`** / **`src/`** — CLI and library
- **`pyproject.toml`** — PyPI metadata + maturin (`bindings = "bin"`)
- **`npm/`** — `llm-lint-cli` wrapper + starter **`templates/llm-lint.toml`**
- **`.github/workflows/`** — CI + publish on GitHub Release
