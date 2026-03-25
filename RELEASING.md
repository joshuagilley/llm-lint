# Releasing llm-lint (PyPI, npm, GitHub Actions)

One-time setup for automated publish on **GitHub Release**. Version source of truth: **`Cargo.toml`** (`version = "…"`). The release workflow syncs **`npm/package.json`** at publish time; **`scripts/release.py`** bumps both before you tag.

## 1. PyPI — trusted publishing (OIDC)

No long-lived PyPI password in GitHub secrets.

1. **PyPI account**  
   Register at [pypi.org](https://pypi.org/account/register/) (and [test.pypi.org](https://test.pypi.org/) if you want a dry run first).

2. **Create the project**  
   - First upload can be manual (`maturin publish` locally with API token) *or* the first successful Actions run will create **`llm-lint`** if the name is free.  
   - If the name is taken, change `name` in **`pyproject.toml`** and the `llm-lint` / `uv tool run` strings in **`npm/bin/llm-lint.js`** to match.

3. **Enable trusted publishing**  
   - PyPI → your account → **Publishing** → **Add a new pending publisher** (or project → **Manage** → **Publishing**).  
   - **PyPI** (not TestPyPI unless you use that workflow).  
   - **Repository:** `OWNER/llm-lint` (your GitHub repo).  
   - **Workflow name:** `publish.yml`  
   - **Environment:** `release`  

4. **GitHub environment `release`**  
   - Repo → **Settings** → **Environments** → **New environment** → name **`release`**.  
   - Add **Deployment branches** (e.g. only `main`) if you want.  
   - No PyPI secret needed when OIDC is configured on PyPI.

5. **Workflow permissions**  
   Repo → **Settings** → **Actions** → **General** → **Workflow permissions**: allow **read** (and write only if needed for other jobs). The publish job uses **`permissions: id-token: write`** for PyPI.

After the first release, confirm the project appears at `https://pypi.org/project/llm-lint/`.

## 2. npm — automation token

1. **npm account**  
   Sign in at [npmjs.com](https://www.npmjs.com/).

2. **Granular access token** (recommended)  
   - npm → **Access Tokens** → **Generate New Token** → **Granular Access Token**.  
   - Permissions: **Read and write**, packages limited to **`llm-lint-cli`** (create the package on first publish if needed).  

   Or a classic **Automation** token for CI.

3. **GitHub secret**  
   - Repo → **Settings** → **Secrets and variables** → **Actions**.  
   - New repository secret: **`NPM_TOKEN`** = the token value.

4. **Link environment**  
   The **`publish-npm`** job uses `environment: release`, so you can attach **`NPM_TOKEN`** to the **`release`** environment (recommended) instead of plain repo secrets.

5. **First publish**  
   The package name on npm is **`llm-lint-cli`** (see `npm/package.json`). If 2FA is required, use a token type npm allows for publish.

## 3. Release from your machine

```bash
# optional: avoid accidental fork release
export LLM_LINT_RELEASE_EXPECT_REPO="joshuagilley/llm-lint"

chmod +x scripts/release.py
./scripts/release.py 0.2.0 --dry-run   # inspect
./scripts/release.py 0.2.0
```

Requires **`gh` CLI** logged in (`gh auth login`). Script bumps **`Cargo.toml`** and **`npm/package.json`**, commits, pushes **`main`**, creates tag **`v0.2.0`**, pushes tag, runs **`gh release create`** (triggers **Publish to PyPI and npm**).

**Order:** The workflow publishes **PyPI first**, then **npm**, so `uv tool run --from llm-lint` resolves after wheels exist. If the PyPI job fails, npm is skipped until you fix and re-run (or publish manually).

## 4. Manual checks (optional)

```bash
# Wheel build (needs maturin: pip/uv install maturin)
maturin build --release -o dist && ls dist/

# Local CLI
cargo run -- scan .
```

## 5. Forks and naming

Replace **`joshuagilley/llm-lint`** in **`pyproject.toml`**, **`npm/package.json`**, **`RELEASING.md`**, and templates with your GitHub URL. PyPI project name must match **`pyproject.toml`** `[project] name`.
