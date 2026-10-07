#!/usr/bin/env node
// Detect package.json version changes across the pre-push SHA or parent.
// GITHUB_OUTPUT receives should_publish and version for the release jobs.
// FORCE_PUBLISH=true retries a failed release without another version bump.
// A missing previous manifest triggers the first publication.

import { spawnSync } from 'node:child_process';
import { readFileSync, appendFileSync } from 'node:fs';

function readCurrentVersion() {
  const pkg = JSON.parse(readFileSync('package.json', 'utf8'));
  if (!pkg.version) throw new Error('package.json missing `version`');
  return pkg.version;
}

function readPreviousVersion() {
  const result = spawnSync('git', ['show', `${process.env.PREVIOUS_SHA || 'HEAD~1'}:package.json`], { encoding: 'utf8' });
  if (result.status !== 0) return null;
  try {
    return JSON.parse(result.stdout).version || null;
  } catch {
    return null;
  }
}

function setOutput(key, value) {
  const path = process.env.GITHUB_OUTPUT;
  const line = `${key}=${value}\n`;
  if (path) appendFileSync(path, line);
  else process.stdout.write(line);
}

const current = readCurrentVersion();
setOutput('version', current);

if (process.env.FORCE_PUBLISH === 'true') {
  console.log(`Force publish requested for ${current}.`);
  setOutput('should_publish', 'true');
  process.exit(0);
}

const previous = readPreviousVersion();
if (previous && previous !== current) {
  console.log(`npm package version: ${previous} -> ${current}; publishing.`);
  setOutput('should_publish', 'true');
} else if (!previous) {
  console.log(`npm package version: ${current} (first release); publishing.`);
  setOutput('should_publish', 'true');
} else {
  console.log(`npm package version unchanged at ${current}; skipping publish.`);
  setOutput('should_publish', 'false');
}
