#!/usr/bin/env node
/* eslint-disable no-console */

const { spawnSync } = require("node:child_process");
const os = require("node:os");

function commandExists(cmd) {
  const checker = process.platform === "win32" ? "where" : "which";
  const found = spawnSync(checker, [cmd], { stdio: "ignore" });
  return found.status === 0;
}

function run(cmd, args, options = {}) {
  const result = spawnSync(cmd, args, {
    stdio: "inherit",
    env: process.env,
    ...options,
  });
  return result.status ?? 1;
}

function ensureUv() {
  if (commandExists("uv")) {
    return true;
  }

  if (process.platform === "win32") {
    console.error(
      "llm-lint-cli requires uv on Windows for now. Install from https://docs.astral.sh/uv/getting-started/installation/ and retry."
    );
    return false;
  }

  console.log("uv not found; installing uv (includes Python management)...");
  const install = spawnSync("sh", ["-c", "curl -LsSf https://astral.sh/uv/install.sh | sh"], {
    stdio: "inherit",
    env: process.env,
  });
  if ((install.status ?? 1) !== 0) {
    console.error("Failed to install uv automatically.");
    return false;
  }

  const home = os.homedir();
  process.env.PATH = `${home}/.local/bin:${home}/.cargo/bin:${process.env.PATH}`;
  return commandExists("uv");
}

function main() {
  if (!ensureUv()) {
    process.exit(1);
  }

  const args = process.argv.slice(2);
  const status = run("uv", ["tool", "run", "--from", "llm-lint", "llm-lint", ...args]);
  process.exit(status);
}

main();
