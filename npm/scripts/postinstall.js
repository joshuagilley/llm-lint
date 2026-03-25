#!/usr/bin/env node
/* eslint-disable no-console */

const fs = require("node:fs");
const path = require("node:path");

function shouldSkip() {
  if (process.env.npm_config_global === "true") {
    return true;
  }
  if (!process.env.INIT_CWD) {
    return true;
  }
  return false;
}

function main() {
  if (shouldSkip()) {
    return;
  }

  const projectRoot = process.env.INIT_CWD;
  const targetPath = path.join(projectRoot, "llm-lint.toml");
  const templatePath = path.join(__dirname, "..", "templates", "llm-lint.toml");

  try {
    if (!fs.existsSync(targetPath)) {
      fs.copyFileSync(templatePath, targetPath);
      console.log("llm-lint-cli: created starter llm-lint.toml");
      return;
    }
  } catch (err) {
    console.warn(`llm-lint-cli: could not write llm-lint.toml (${err.message})`);
  }
}

main();
