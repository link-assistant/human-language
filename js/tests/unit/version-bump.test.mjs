import { test } from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, mkdirSync, writeFileSync, rmSync, copyFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { spawnSync } from 'node:child_process';

// Exercise the actual release scripts in isolated histories, including multi-
// commit pushes and a failed publication retried without a version change.
for (const [name, script, manifest] of [
  ['npm', 'js/scripts/check-package-version.mjs', 'package.json'],
  ['Cargo', 'rust/scripts/check-cargo-version.mjs', 'rust/Cargo.toml'],
]) {
  test(`${name} release detection handles first release, push ranges, and retries`, { timeout: 15000 }, () => {
    const directory = mkdtempSync(join(tmpdir(), 'human-language-release-'));
    const scriptPath = join(directory, 'check-version.mjs');
    copyFileSync(resolve(script), scriptPath);
    mkdirSync(join(directory, 'rust'));
    function git(...args) {
      const result = spawnSync('git', args, { cwd: directory, encoding: 'utf8' });
      assert.equal(result.status, 0, result.stderr);
      return result.stdout.trim();
    }
    function version(value) {
      writeFileSync(join(directory, manifest), name === 'npm'
        ? JSON.stringify({ name: 'human-language', version: value })
        : `[package]\nname = "human-language"\nversion = "${value}"\n`);
    }
    function commit(message) {
      git('add', '.');
      git('commit', '-m', message);
      return git('rev-parse', 'HEAD');
    }
    function check(expected, extraEnv = {}) {
      const env = { ...process.env };
      for (const key of ['GITHUB_OUTPUT', 'PREVIOUS_SHA', 'FORCE_PUBLISH']) delete env[key];
      const result = spawnSync(process.execPath, [scriptPath], {
        cwd: directory, env: { ...env, ...extraEnv }, encoding: 'utf8',
      });
      assert.equal(result.status, 0, result.stderr);
      assert.match(result.stdout, new RegExp(`should_publish=${expected}\\b`));
    }
    try {
      git('init');
      git('config', 'user.name', 'Release Test');
      git('config', 'user.email', 'release@example.com');
      version('0.1.0');
      const first = commit('initial package');
      check(true);
      version('0.2.0');
      commit('bump version');
      writeFileSync(join(directory, 'followup.txt'), 'followup');
      commit('followup without a version change');
      check(false);
      check(true, { PREVIOUS_SHA: first });
      check(true, { FORCE_PUBLISH: 'true' });
      check(false, { FORCE_PUBLISH: 'false' });
    } finally {
      rmSync(directory, { recursive: true, force: true });
    }
  });
}
