# Package releases

The Rust and JavaScript workflows validate pull requests and publish only from
`main`. PR #45 prepares Rust and npm version **0.2.2**. Merging a version bump
runs the release jobs; retries do not require a new version.

## Rust / crates.io

`.github/workflows/rust.yml` gates publication on formatting, Clippy with all
features, tests on Linux/macOS/Windows, native `no_std` tests, a WASM build,
and `cargo publish --dry-run`. The extracted package is also built with
`--no-default-features --target wasm32-unknown-unknown`, so missing embedded
`.lino` data cannot slip through.

Set `CARGO_REGISTRY_TOKEN` as a repository secret or in the `crates-io`
environment. Its account must have a verified crates.io email and permission
to publish `human-language`; the initial token must be able to create the crate.
The workflow fails with an explicit configuration error if the token is absent.
The library's MSRV is Rust 1.85, matching the default CLI dependencies.

## JavaScript / npm

`.github/workflows/js.yml` gates publication on syntax, unit/integration,
links and local browser checks. Its publish job installs dependencies,
checks `npm pack --dry-run` and publishes with provenance and public access.
Node 24's npm supports both an `NPM_TOKEN` secret and npm trusted publishing.

For the first publication, set `NPM_TOKEN` in the repository or the `npm`
environment using a granular token with permission to create/publish
`human-language` and bypass 2FA for CI publication. Alternatively configure
npm trusted publishing where package settings are available:

- Organization: `link-assistant`
- Repository: `human-language`
- Workflow filename: `js.yml`
- Environment: `npm`

The workflow grants `id-token: write`, so configured trusted publishing uses
OIDC; the token remains available as a fallback. For details see the
[npm trusted publishing documentation](https://docs.npmjs.com/trusted-publishers/).

## Retry a failed publication

Dispatch the relevant workflow from **main**, selecting `force_publish: true`:

```sh
gh workflow run rust.yml --ref main -f force_publish=true
gh workflow run js.yml --ref main -f force_publish=true
```

This reruns validation before publishing the unchanged, unpublished version.
The JS dispatch also retries its existing Docker and GitHub release jobs.
Do not force publication of a version already present in the registry; versions
cannot be overwritten. Ordinary push detection compares the manifest with
`github.event.before`, including version bumps earlier in a multi-commit push.
A missing previous manifest triggers the first release.

## Issue #44 publication evidence

The last main JS release run,
[31720758618](https://github.com/link-assistant/human-language/actions/runs/31720758618),
failed at `npm publish` with `ENEEDAUTH` on 2026-08-13. The downloaded log's
lines 4144–4146 report missing npm authentication. On 2026-10-07,
`gh secret list --repo link-assistant/human-language` returned no repository
secrets, and `npm view human-language version` returned E404. The crates.io API
was inaccessible from the implementation environment (HTTP 403), so public
crate availability could not be independently checked there.

This PR supplies the release gates, a version bump and retry controls. Registry
account configuration must be available for the main release to complete;
publication cannot be verified by PR checks, which intentionally do not publish.
