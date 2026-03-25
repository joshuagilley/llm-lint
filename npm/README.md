# llm-lint-cli

npm shim around the **[llm-lint](https://pypi.org/project/llm-lint/)** CLI. The real binary is installed on demand via **`uv tool run`** (same pattern as [slopsniff-cli](https://www.npmjs.com/package/slopsniff-cli)).

```bash
npx llm-lint scan .
```

Requires **uv** (bundled installer runs on macOS/Linux; on Windows install uv manually). PyPI package name: **`llm-lint`**.
