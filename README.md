<p align="center">
  <img src="docs/logo.png" alt="llm-lint — Ferris inspecting code on a conveyor belt" width="420" />
</p>

# llm-lint

Static checks for “LLM slop” and related drift: oversized files/functions, duplicate bodies, noisy helpers, risky env fallbacks, and secret-shaped strings. Implemented in Rust; configuration lives in **`llm-lint.toml`** or **`llm-lint.json`** at the root of the tree you scan.

---

## Python environments

The CLI is published to PyPI as **`llm-lint`** (pre-built wheels for common platforms via [Maturin](https://www.maturin.rs/)).

### pip / virtualenv

```bash
python -m venv .venv
source .venv/bin/activate   # Windows: .venv\Scripts\activate
pip install llm-lint
llm-lint scan .
```

### uv (recommended)

```bash
uv tool install llm-lint
llm-lint scan .
```

Or run without a global install:

```bash
uv tool run --from llm-lint llm-lint scan .
```

### pipx

```bash
pipx install llm-lint
llm-lint scan .
```

---

## JavaScript / Node environments

The npm package **`llm-lint-cli`** is a thin wrapper: it runs the same binary via **`uv tool run --from llm-lint`**, so the real executable comes from PyPI.

### Prerequisites

- **[uv](https://docs.astral.sh/uv/)** must be available. On macOS/Linux the package’s install flow can bootstrap uv; on **Windows** install uv yourself if needed.

### Install in a project

```bash
npm install -D llm-lint-cli
npx llm-lint scan .
```

`package.json` scripts:

```json
{
  "scripts": {
    "lint:llm": "llm-lint scan ."
  }
}
```

### Postinstall config

On a normal (non-global) **`npm install`**, if **`llm-lint.toml`** is missing in the project root, a starter file is copied from the package template. Existing files are left unchanged.

---

## Other install options

| Channel | Command |
|--------|---------|
| **Cargo** (from source / [crates.io](https://crates.io/crates/llm-lint)) | `cargo install llm-lint` |

---

## Usage

```bash
llm-lint scan .
llm-lint scan . -f json --fail-threshold 20
llm-lint scan . -v --max-file-lines 300
```

See **`llm-lint.toml`** in this repo for configuration keys (`include` rules, thresholds, `exclude-dirs`, etc.).

---

## Release & publishing

See **[RELEASING.md](RELEASING.md)** for PyPI trusted publishing, npm token, and **`scripts/release.py`**.

---

## Repo layout

- **`docs/logo.png`** — project logo (README header)
- **`Cargo.toml`** / **`src/`** — CLI and library
- **`pyproject.toml`** — PyPI metadata + maturin (`bindings = "bin"`)
- **`npm/`** — **`llm-lint-cli`** wrapper + **`templates/llm-lint.toml`**
- **`.github/workflows/`** — CI + publish on GitHub Release
