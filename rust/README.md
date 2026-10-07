# human-language (Rust)

Rust crate that mirrors the pure-function helpers of the JavaScript package
[`human-language`](https://www.npmjs.com/package/human-language). It powers the
Rust CLI, future WASM builds for the SPA, and any downstream consumer that
needs to tokenize text, parse the SPA hash, or render a Q/P sequence as
[Links Notation](https://github.com/link-foundation/links-notation)
without spinning up a Node runtime.

The release workflows publish the crate to crates.io and the JavaScript
package to npm when their versions advance on `main`. See
[`../docs/releases.md`](../docs/releases.md) for registry setup and retries.

## What ships

- `human_language::language::{detect, Language, Detector, registered_languages,
  from_slug, fallback_language, language_name, word_order, uses_postpositions,
  language_for_concept_slug, surface_matches_language}`
- `human_language::tokenize::{tokenize, generate_ngrams, is_stop_word, is_property_indicator}`
- `human_language::routing::{parse_hash, serialize_hash, ParsedHash, MODES, DEFAULT_MODE}`
- `human_language::settings::{quotes_for_language, flag_for_language, QuotePair}`
- `human_language::lino::{format_sequence_as_lino, SeqItem}`
- `human-language` binary with `tokenize`, `parse-hash`, `lino-sequence`,
  `version`, `help` subcommands.

## Language detection

```rust
use human_language::language::{detect, Language, word_order, uses_postpositions};

assert_eq!(detect("¿Cómo estás?"), Language::Spanish);
assert_eq!(detect("你好"), Language::Chinese);
assert_eq!(word_order("hi"), Some("SOV"));
assert!(uses_postpositions("hi"));
```

The registry covers English, Russian, Hindi, Chinese and Spanish. It preserves
Formal AI's script and lexical-cue behavior, including mixed prompts with Latin
identifiers. Unregistered dominant scripts produce `Language::Unknown`; blank
or punctuation-only input uses the declared fallback (English).

Detection is deterministic and heuristic. Shared scripts use their declared
default language; this is not a statistical detector for every language that
uses Cyrillic, Devanagari, Han or Latin. The adposition flags preserve Formal
AI's positional edit cues, including Chinese; word order describes a canonical
default rather than every grammatical construction.

Both data files are embedded and shipped inside the crate:

- `data/language-detection.lino`: one indented `rule` record per language, with
  `language`, `script`, hexadecimal `start`/`end`, optional `alphabetic-only yes`,
  quoted `markers (...)` and exactly one `fallback yes`.
- `data/languages.lino`: `language` records with `name`, `word-order` and
  `adposition` (`preposition` or `postposition`).

Add a language by editing the files and rebuilding. Slugs are data, so no new
Rust enum variant or branch is needed. The parser reads the documented
line-oriented Links Notation schema; quoted marker strings cannot contain
embedded quotes. Registry tests check every embedded record and a data-only
addition with a different fallback.

`Detector::default()` has an independent override set by
`detector.set_forced_language(Some(language))`; passing `None` clears it.
Use one detector per request in asynchronous or `no_std` applications.
With `std`, `set_forced_language` returns a guard that restores the previous
thread-local override when dropped. Nest guards and drop them in reverse order;
keep their scope synchronous, since an async task may move between threads.

## `no_std` + `alloc`

```toml
[dependencies]
human-language = { version = "0.2.2", default-features = false }
```

Disabling the default `std` feature retains detection, metadata, tokenization,
routing, quote/flag helpers and Q/P sequence rendering. The CLI, configuration
codec and scoped thread-local overrides require `std`. `wikidata-client`
enables `std` as well as its optional network dependencies.

```sh
cargo build --lib --no-default-features --target wasm32-unknown-unknown
cargo test --no-default-features
```

The library produces an `rlib`, suitable for linking into a consumer's WASM
worker. The previous unused `cdylib` output has been removed: a final `no_std`
WASM executable supplies its own allocator, panic handler and exported entry
points. There is no global mutable WASM override or cached allocation in the
`no_std` library, so workers may reset their allocator between calls.

## What's deferred

The network-backed parts of the transformer (Wikidata API client,
search/disambiguation, file/IndexedDB cache) currently live only in
JavaScript. Tracked under R3 in
`../docs/case-studies/issue-37/solution-plans.md`.

## Development

```sh
# from the repo root
cd rust
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

## Parity with JavaScript

`tests/parity.rs` pins the Rust output against the JavaScript test suite
under `../js/tests/unit/`. When either side changes a contract, update both
in the same commit.

## License

MIT — see [`../LICENSE`](../LICENSE).
