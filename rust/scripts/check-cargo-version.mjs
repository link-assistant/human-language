#!/usr/bin/env node
// Detect rust/Cargo.toml version changes across the pre-push SHA or parent.
// GITHUB_OUTPUT receives should_publish and version for the release jobs.
// FORCE_PUBLISH=true retries a failed release without another version bump.
// A missing previous manifest triggers the first publication.

import { spawnSync } from 'node:child_process';
import { readFileSync, appendFileSync } from 'node:fs';

function readCurrentVersion() {
  const text = readFileSync('rust/Cargo.toml', 'utf8');
  const m = text.match(/^\s*\[package\][^[]*?^\s*version\s*=\s*"([^"]+)"/ms);
  if (!m) throw new Error('Could not find [package].version in rust/Cargo.toml');
  return m[1];
}

function readPreviousVersion() {
  const result = spawnSync('git', ['show', `${process.env.PREVIOUS_SHA || 'HEAD~1'}:rust/Cargo.toml`], { encoding: 'utf8' });
  if (result.status !== 0) return null;
  const m = result.stdout.match(/^\s*\[package\][^[]*?^\s*version\s*=\s*"([^"]+)"/ms);
  return m ? m[1] : null;
}

function setOutput(key, value) {
  const path = process.env.GITHUB_OUTPUT;
  const line = `${key}=${value}\n`;
  if (path) appendFileSync(path, line);
  else process.stdout.write(line);
}

const current = readCurrentVersion();
const previous = readPreviousVersion();

setOutput('version', current);
if (process.env.FORCE_PUBLISH === 'true') {
  console.log(`Force publish requested for ${current}.`);
  setOutput('should_publish', 'true');
  process.exit(0);
}
if (previous && previous !== current) {
  console.log(`Rust crate version: ${previous} -> ${current}; publishing.`);
  setOutput('should_publish', 'true');
} else if (!previous) {
  console.log(`Rust crate version: ${current} (first release); publishing.`);
  setOutput('should_publish', 'true');
} else {
  console.log(`Rust crate version unchanged at ${current}; skipping publish.`);
  setOutput('should_publish', 'false');
}
