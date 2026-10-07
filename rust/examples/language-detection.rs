// Run: cargo run --manifest-path rust/Cargo.toml --example language-detection
use human_language::language::{detect, language_name, word_order, Detector, Language};

fn main() {
    for text in ["Hello", "Привет", "नमस्ते", "你好", "¿Cómo estás?"] {
        let language = detect(text);
        println!(
            "{text}: {} ({}, {})",
            language.slug(),
            language_name(language.slug()).unwrap_or("Unknown"),
            word_order(language.slug()).unwrap_or("unspecified")
        );
    }
    let mut detector = Detector::default();
    detector.set_forced_language(Some(Language::Spanish));
    assert_eq!(detector.detect("Hello"), Language::Spanish);
    detector.set_forced_language(None);
    assert_eq!(detector.detect("Hello"), Language::English);
}
