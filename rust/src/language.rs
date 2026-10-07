//! Deterministic language detection driven by embedded Links Notation data.
//!
//! Script ranges, lexical markers and the fallback language live in
//! `data/language-detection.lino`; names, word order and adpositions live in
//! `data/languages.lino`. Adding a language requires only editing these files.
//! The algorithm and initial corpus come from link-assistant/formal-ai.
//!
//! With `std`, `set_forced_language` provides a scoped thread-local override.
//! [`Detector`] provides an independent override on every target, including
//! `no_std` + `alloc`, without shared mutable state.

use alloc::string::String;
use alloc::vec::Vec;

/// The detection registry, embedded so the browser worker (`no_std` + `alloc`,
/// no filesystem) reads exactly the same rules as the native build.
const LANGUAGE_DETECTION: &str = include_str!("../data/language-detection.lino");

/// Detected language, identified by the slug the registry gives it.
///
/// A newtype rather than an enum: the set of languages is data, so the type
/// cannot enumerate it. Slugs are `&'static str` slices of the embedded
/// registry (or of the constants below), which keeps the type `Copy` and
/// usable in the `no_std` worker.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Language(&'static str);

#[allow(non_upper_case_globals)]
impl Language {
    /// English — also the registry's declared fallback.
    pub const English: Self = Self("en");
    /// Russian.
    pub const Russian: Self = Self("ru");
    /// Hindi.
    pub const Hindi: Self = Self("hi");
    /// Chinese.
    pub const Chinese: Self = Self("zh");
    /// Spanish.
    pub const Spanish: Self = Self("es");
    /// No registered language matched the prompt's dominant script.
    pub const Unknown: Self = Self("unknown");

    /// The slug used inside `language:<slug>` evidence links.
    #[must_use]
    pub const fn slug(self) -> &'static str {
        self.0
    }

    /// Wrap an already-static slug, e.g. one sliced out of the registry.
    #[must_use]
    pub const fn from_static_slug(slug: &'static str) -> Self {
        Self(slug)
    }
}

impl core::fmt::Debug for Language {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str(self.0)
    }
}

// Native overrides are thread-local; no_std callers use an owned Detector.
#[cfg(feature = "std")]
mod forced_language {
    use super::Language;
    use std::cell::Cell;

    thread_local! {
        static FORCED_LANGUAGE: Cell<Option<Language>> = const { Cell::new(None) };
    }

    pub(super) fn replace(language: Option<Language>) -> Option<Language> {
        FORCED_LANGUAGE.with(|slot| slot.replace(language))
    }

    pub(super) fn set(language: Option<Language>) {
        FORCED_LANGUAGE.with(|slot| slot.set(language));
    }

    pub(super) fn get() -> Option<Language> {
        FORCED_LANGUAGE.with(Cell::get)
    }
}

/// Force [`detect`] to return `language` until the returned guard is dropped.
///
/// The guard restores the previous forced value on drop, so nested replays
/// stay balanced. Passing `None` clears the override (a plain, non-forced
/// solve).
#[must_use]
#[cfg(feature = "std")]
pub fn set_forced_language(language: Option<Language>) -> ForcedLanguageGuard {
    let previous = forced_language::replace(language);
    ForcedLanguageGuard {
        previous,
        _thread: core::marker::PhantomData,
    }
}

/// The slug of the language forced onto this solve, if any.
///
/// Handlers that cannot derive their output language from [`detect`] (their
/// report language is a fixed default, not a detection) read this instead, so a
/// forced response language still reaches them.
#[must_use]
#[cfg(feature = "std")]
pub fn forced_response_language_slug() -> Option<&'static str> {
    forced_language::get().map(Language::slug)
}

/// Resolve a language slug to a [`Language`], accepting every slug the
/// detection registry declares.
///
/// Returns `None` for slugs the registry does not know, so callers keep a
/// single place to notice an unregistered language.
#[must_use]
pub fn from_slug(slug: &str) -> Option<Language> {
    with_rules(|rules| {
        rules
            .iter()
            .find(|rule| rule.language == slug)
            .map(|rule| Language(rule.language))
    })
}

/// Every language slug the detection registry declares, in registry order.
#[must_use]
pub fn registered_languages() -> Vec<Language> {
    with_rules(|rules| rules.iter().map(|rule| Language(rule.language)).collect())
}

/// The fallback language declared by the registry (`fallback yes`).
#[must_use]
pub fn fallback_language() -> Language {
    with_rules(fallback_of)
}

/// Per-language metadata, embedded next to the detection registry.
const LANGUAGE_LEDGER: &str = include_str!("../data/languages.lino");

/// A language's English name, read from the metadata file.
#[must_use]
pub fn language_name(slug: &str) -> Option<&'static str> {
    metadata_field(slug, "name")
}

/// Canonical word order (`SVO`, `SOV`, …), or `None` for an unlisted language.
/// These describe the defaults, not every construction a language allows.
#[must_use]
pub fn word_order(slug: &str) -> Option<&'static str> {
    metadata_field(slug, "word-order")
}

/// Whether positional cues follow the literal they modify. The initial
/// flags preserve Formal AI's handling of Hindi and Chinese edit instructions.
#[must_use]
pub fn uses_postpositions(slug: &str) -> bool {
    metadata_field(slug, "adposition") == Some("postposition")
}

fn metadata_field(slug: &str, field: &str) -> Option<&'static str> {
    let mut current = None;
    for line in LANGUAGE_LEDGER.lines() {
        let Some((key, value)) = line.trim().split_once(' ') else {
            continue;
        };
        if key == "language" {
            current = Some(unquote(value));
        } else if key == field && current == Some(slug) {
            return Some(unquote(value));
        }
    }
    None
}

/// Resolve a `language_*` concept slug (`language_spanish`) to its registered
/// language (`es`) by matching the ledger's English name.
///
/// The concept graph names languages in words; the translation pipeline and the
/// Wiktionary client address them by slug. This is the bridge, and it is data,
/// not a table of four hard-coded pairs.
#[must_use]
pub fn language_for_concept_slug(concept_slug: &str) -> Option<Language> {
    let name = concept_slug.strip_prefix("language_")?;
    with_rules(|rules| {
        rules
            .iter()
            .find(|rule| {
                language_name(rule.language).is_some_and(|declared| {
                    declared.len() == name.len()
                        && declared
                            .chars()
                            .zip(name.chars())
                            .all(|(left, right)| left.to_ascii_lowercase() == right)
                })
            })
            .map(|rule| Language(rule.language))
    })
}

/// Whether `surface` contains at least one character the registry attributes to
/// `slug`'s script (or, for the fallback language, at least one letter of the
/// fallback script's range).
///
/// Importers use this to reject a surface form filed under the wrong language
/// without enumerating Unicode ranges per language.
#[must_use]
pub fn surface_matches_language(surface: &str, slug: &str) -> bool {
    with_rules(|rules| {
        let Some(rule) = rules.iter().find(|rule| rule.language == slug) else {
            return false;
        };
        // A language sharing the fallback script (Spanish and English are both
        // Latin) matches any character of that script, not only its accented
        // sub-range, so its plain-ASCII surfaces are not rejected.
        let script = rule.script;
        rules
            .iter()
            .filter(|candidate| candidate.script == script)
            .any(|candidate| {
                surface.chars().any(|character| {
                    let codepoint = u32::from(character);
                    (candidate.start..=candidate.end).contains(&codepoint)
                        && (!candidate.alphabetic_only || character.is_alphabetic())
                })
            })
    })
}

/// RAII guard that restores the previous forced language when dropped.
#[cfg(feature = "std")]
pub struct ForcedLanguageGuard {
    previous: Option<Language>,
    // A guard must be dropped on the thread whose override it changed.
    _thread: core::marker::PhantomData<alloc::rc::Rc<()>>,
}

#[cfg(feature = "std")]
impl Drop for ForcedLanguageGuard {
    fn drop(&mut self) {
        forced_language::set(self.previous);
    }
}

/// One parsed `rule` record from `data/language-detection.lino`.
///
/// Borrowed from the embedded registry, so a rule costs no allocation beyond
/// its marker vector.
struct Rule {
    language: &'static str,
    script: &'static str,
    start: u32,
    end: u32,
    /// Count only `char::is_alphabetic` code points inside the range. The
    /// Latin fallback range spans six ASCII punctuation code points that are
    /// not letters and must not inflate the Latin count.
    alphabetic_only: bool,
    fallback: bool,
    /// Tokens that vote for this language when its script alone cannot decide
    /// the prompt — either because the script is shared (Spanish and English
    /// are both Latin) or because the prompt mixes scripts.
    markers: Vec<&'static str>,
}

fn unquote(value: &'static str) -> &'static str {
    value
        .strip_prefix('"')
        .and_then(|rest| rest.strip_suffix('"'))
        .unwrap_or(value)
}

fn parse_codepoint(value: &str) -> u32 {
    let digits = value
        .strip_prefix("0x")
        .or_else(|| value.strip_prefix("0X"));
    digits.map_or(0, |digits| u32::from_str_radix(digits, 16).unwrap_or(0))
}

/// Split a `("a" "b" "c")` list into its quoted items.
fn parse_quoted_list(value: &'static str) -> Vec<&'static str> {
    let mut items = Vec::new();
    let mut rest = value;
    while let Some(open) = rest.find('"') {
        let after = &rest[open + 1..];
        let Some(close) = after.find('"') else { break };
        items.push(&after[..close]);
        rest = &after[close + 1..];
    }
    items
}

fn parse_rules() -> Vec<Rule> {
    parse_rules_from(LANGUAGE_DETECTION)
}

fn parse_rules_from(notation: &'static str) -> Vec<Rule> {
    let mut rules: Vec<Rule> = Vec::new();
    for line in notation.lines() {
        let trimmed = line.trim_start();
        if trimmed.is_empty() {
            continue;
        }
        let (key, value) = trimmed.split_once(' ').unwrap_or((trimmed, ""));
        if key == "rule" {
            rules.push(Rule {
                language: "",
                script: "",
                start: 0,
                end: 0,
                alphabetic_only: false,
                fallback: false,
                markers: Vec::new(),
            });
            continue;
        }
        let Some(rule) = rules.last_mut() else {
            continue;
        };
        match key {
            "language" => rule.language = unquote(value),
            "script" => rule.script = unquote(value),
            "label" if rule.script.is_empty() => rule.script = unquote(value),
            "start" => rule.start = parse_codepoint(value),
            "end" => rule.end = parse_codepoint(value),
            "alphabetic-only" => rule.alphabetic_only = value == "yes",
            "fallback" => rule.fallback = value == "yes",
            "markers" => rule.markers = parse_quoted_list(value),
            _ => {}
        }
    }
    rules.retain(|rule| !rule.language.is_empty());
    rules
}

// Native callers reuse immutable rules. no_std callers parse within the
// current allocation lifetime, avoiding cached allocations in WASM workers
// that reset their allocator between calls.
#[cfg(feature = "std")]
fn with_rules<R>(action: impl FnOnce(&[Rule]) -> R) -> R {
    use std::sync::OnceLock;
    static RULES: OnceLock<Vec<Rule>> = OnceLock::new();
    action(RULES.get_or_init(parse_rules))
}

#[cfg(not(feature = "std"))]
fn with_rules<R>(action: impl FnOnce(&[Rule]) -> R) -> R {
    action(&parse_rules())
}

fn fallback_of(rules: &[Rule]) -> Language {
    rules
        .iter()
        .find(|rule| rule.fallback)
        .map_or(Language::English, |rule| Language(rule.language))
}

fn fallback_script_of(rules: &[Rule]) -> &'static str {
    rules
        .iter()
        .find(|rule| rule.fallback)
        .map_or("", |rule| rule.script)
}

/// The language a script defaults to: the first rule declaring that script.
fn default_language_of(rules: &[Rule], script: &str) -> Language {
    rules
        .iter()
        .find(|rule| rule.script == script && rule.fallback)
        .or_else(|| rules.iter().find(|rule| rule.script == script))
        .map_or_else(|| fallback_of(rules), |rule| Language(rule.language))
}

/// Per-script character counts, keyed by script name in registry order.
struct ScriptCounts {
    counts: Vec<(&'static str, usize)>,
    other: usize,
    first: Option<&'static str>,
}

impl ScriptCounts {
    fn of(&self, script: &str) -> usize {
        self.counts
            .iter()
            .find(|(name, _)| *name == script)
            .map_or(0, |(_, count)| *count)
    }

    fn bump(&mut self, script: &'static str) {
        if let Some(entry) = self.counts.iter_mut().find(|(name, _)| *name == script) {
            entry.1 += 1;
        } else {
            self.counts.push((script, 1));
        }
        self.first.get_or_insert(script);
    }

    /// The highest count among scripts other than `script`.
    fn max_excluding(&self, script: &str) -> usize {
        self.counts
            .iter()
            .filter(|(name, _)| *name != script)
            .map(|(_, count)| *count)
            .max()
            .unwrap_or(0)
    }

    fn total(&self) -> usize {
        self.counts.iter().map(|(_, count)| *count).sum::<usize>() + self.other
    }
}

fn count_scripts(prompt: &str, rules: &[Rule]) -> ScriptCounts {
    let mut counts = ScriptCounts {
        counts: Vec::new(),
        other: 0,
        first: None,
    };
    for character in prompt.chars() {
        let codepoint = u32::from(character);
        let matched = rules.iter().find(|rule| {
            (rule.start..=rule.end).contains(&codepoint)
                && (!rule.alphabetic_only || character.is_alphabetic())
        });
        match matched {
            Some(rule) => counts.bump(rule.script),
            None if character.is_alphabetic() => {
                counts.other += 1;
                counts.first.get_or_insert("");
            }
            None => {}
        }
    }
    counts
}

/// Whether a marker occurs in `normalized` at a position that counts.
///
/// A marker written in the shared fallback script only counts where it begins
/// a word: Spanish "escribe" must not claim English "describe". Markers in a
/// script of their own may sit inside native morphology — Chinese "什么"
/// following "是" — so they match anywhere, the way their languages write them.
fn marker_present(normalized: &str, marker: &str, rules: &[Rule], fallback_script: &str) -> bool {
    let Some(first) = marker.chars().next() else {
        return false;
    };
    let needs_word_start = rules.iter().any(|rule| {
        rule.script == fallback_script
            && (rule.start..=rule.end).contains(&u32::from(first))
            && (!rule.alphabetic_only || first.is_alphabetic())
    });
    let mut from = 0;
    while let Some(at) = normalized[from..].find(marker) {
        let start = from + at;
        let word_start = normalized[..start]
            .chars()
            .next_back()
            .is_none_or(|previous| !previous.is_alphabetic());
        if word_start || !needs_word_start {
            return true;
        }
        from = start + marker.len();
    }
    false
}

/// The language whose markers appear in the prompt, preferring the one whose
/// script carries the most characters.
///
/// Markers are weaker evidence than a script: they are ordinary tokens that can
/// appear inside a foreign proper name ("Расскажи о julián andrés quiñones"
/// carries Spanish markers but is Russian). So a marker rule only votes when no
/// *rival* script — a registered script other than the rule's own and other than
/// the fallback script every language may borrow identifiers from — appears in
/// the prompt; otherwise the script rules below decide.
fn marker_language(
    prompt: &str,
    rules: &[Rule],
    counts: &ScriptCounts,
    fallback_script: &str,
) -> Option<Language> {
    let normalized: String = prompt.to_lowercase();
    let mut best: Option<(usize, Language)> = None;
    for rule in rules.iter().filter(|rule| !rule.markers.is_empty()) {
        let count = counts.of(rule.script);
        if count == 0 {
            continue;
        }
        let contested = rules.iter().any(|other| {
            other.script != rule.script
                && other.script != fallback_script
                && counts.of(other.script) > 0
        });
        if contested {
            continue;
        }
        if !rule.markers.iter().any(|marker| {
            let lowered = marker.to_lowercase();
            marker_present(&normalized, &lowered, rules, fallback_script)
        }) {
            continue;
        }
        match best {
            Some((best_count, _)) if count <= best_count => {}
            _ => best = Some((count, Language(rule.language))),
        }
    }
    best.map(|(_, language)| language)
}

/// Detect the dominant language of a prompt.
///
/// Counts characters per registered script. Text written only in the fallback
/// script resolves to the fallback language unless another language's markers
/// claim it; when a registered non-fallback script opens the prompt or its
/// markers appear alongside fallback-script identifiers, that language wins.
/// Text dominated by a script no rule claims returns [`Language::Unknown`] so
/// the loop can record an explicit `language:unknown` event.
#[must_use]
pub fn detect(prompt: &str) -> Language {
    #[cfg(feature = "std")]
    if let Some(forced) = forced_language::get() {
        return forced;
    }
    with_rules(|rules| detect_with(prompt, rules))
}

fn detect_with(prompt: &str, rules: &[Rule]) -> Language {
    let fallback = fallback_of(rules);
    let fallback_script = fallback_script_of(rules);
    let counts = count_scripts(prompt, rules);

    if counts.total() == 0 {
        return fallback;
    }

    let fallback_count = counts.of(fallback_script);
    if counts.other > fallback_count && counts.other >= counts.max_excluding(fallback_script) {
        return Language::Unknown;
    }

    if fallback_count > 0 {
        if let Some(language) = marker_language(prompt, rules, &counts, fallback_script) {
            return language;
        }
        if let Some(first) = counts
            .first
            .filter(|first| !first.is_empty() && *first != fallback_script)
        {
            let rival = counts
                .counts
                .iter()
                .filter(|(name, _)| *name != fallback_script && *name != first)
                .map(|(_, count)| *count)
                .max()
                .unwrap_or(0);
            if counts.of(first) >= rival {
                return default_language_of(rules, first);
            }
        }
    }

    for rule in rules.iter().filter(|rule| !rule.fallback) {
        let count = counts.of(rule.script);
        if count > 0 && count >= counts.max_excluding(rule.script) {
            return default_language_of(rules, rule.script);
        }
    }
    fallback
}

/// An independent detection context with an optional forced language.
///
/// Use one instance per solve/request in `no_std` or asynchronous code.
/// It never reads the thread-local override used by the free [`detect`] function.
///
/// ```
/// use human_language::language::{Detector, Language};
/// let mut detector = Detector::default();
/// assert_eq!(detector.detect("你好"), Language::Chinese);
/// detector.set_forced_language(Some(Language::Spanish));
/// assert_eq!(detector.detect("你好"), Language::Spanish);
/// detector.set_forced_language(None);
/// assert_eq!(detector.detect("你好"), Language::Chinese);
/// ```
#[derive(Debug, Default, Clone)]
pub struct Detector {
    forced: Option<Language>,
}

impl Detector {
    /// Replace the override, returning its previous value.
    pub fn set_forced_language(&mut self, language: Option<Language>) -> Option<Language> {
        core::mem::replace(&mut self.forced, language)
    }

    /// The language currently forced in this context.
    #[must_use]
    pub const fn forced_language(&self) -> Option<Language> {
        self.forced
    }

    /// Detect text using this context's override and the embedded registry.
    #[must_use]
    pub fn detect(&self, prompt: &str) -> Language {
        self.forced
            .unwrap_or_else(|| with_rules(|rules| detect_with(prompt, rules)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_registry_has_complete_unambiguous_records() {
        let rules = parse_rules();
        assert_eq!(rules.iter().filter(|rule| rule.fallback).count(), 1);
        for (index, rule) in rules.iter().enumerate() {
            assert!(!rule.script.is_empty());
            assert!(rule.start > 0 && rule.start <= rule.end);
            assert!(char::from_u32(rule.start).is_some());
            assert!(char::from_u32(rule.end).is_some());
            assert!(!rules[..index]
                .iter()
                .any(|other| other.language == rule.language));
            assert!(language_name(rule.language).is_some());
            assert!(word_order(rule.language).is_some());
            assert!(metadata_field(rule.language, "adposition").is_some());
        }
    }

    #[test]
    fn a_new_language_and_fallback_need_only_data_changes() {
        let rules = parse_rules_from(
            "language_detection\n\
             rule arabic\n\
               language ar\n\
               script Arabic\n\
               start 0x0600\n\
               end 0x06FF\n\
               fallback yes\n\
             rule latin\n\
               language la\n\
               script Latin\n\
               start 0x0041\n\
               end 0x007A\n\
               alphabetic-only yes\n\
               markers (\"salve\")\n",
        );
        assert_eq!(detect_with("مرحبا", &rules).slug(), "ar");
        assert_eq!(detect_with("", &rules).slug(), "ar");
        assert_eq!(detect_with("salve", &rules).slug(), "la");
    }
}
