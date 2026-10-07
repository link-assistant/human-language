#![cfg_attr(not(feature = "std"), no_std)]

//! Pure-function helpers shared by the npm package, the Docker microservice,
//! and the browser SPA. The contracts mirror the JavaScript modules under
//! `js/src/` so parity tests (`rust/tests/parity.rs`) can pin both sides.
//!
//! With default features disabled, the library uses only `core` and `alloc`.
//! The CLI and configuration codec require the default `std` feature.
//!
//! Modules:
//! - [`language`]: Data-driven language detection and language metadata.
//! - [`mod@tokenize`]: English-text tokenizer + n-gram generator + stop-word and
//!   property-indicator predicates.
//! - [`routing`]: `#mode=…&…` hash parser / serializer for the SPA.
//! - [`settings`]: Locale quote pairs + a (compact) flag map for the
//!   language switcher.
//! - [`lino`]: Renders Q/P sequences into the
//!   [Links Notation](https://github.com/link-foundation/links-notation)
//!   form used by the API and the CLI.

extern crate alloc;

pub mod language;
pub mod lino;
pub mod routing;
pub mod settings;
pub mod tokenize;

pub use language::{detect, Language};
pub use routing::{parse_hash, serialize_hash, ParsedHash, DEFAULT_MODE, MODES};
pub use settings::{flag_for_language, quotes_for_language, QuotePair};
pub use tokenize::{
    generate_ngrams, is_property_indicator, is_stop_word, tokenize, NGRAMS_DEFAULT_MAX,
};

/// Package name shipped on crates.io.
pub const NAME: &str = "human-language";

/// Package version. Matches `Cargo.toml`; bumped together with `package.json`
/// by the release pipeline.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
