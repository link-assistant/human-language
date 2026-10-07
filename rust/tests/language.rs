use human_language::language::{detect, Language};

use human_language::language::{
    fallback_language, from_slug, language_for_concept_slug, language_name, registered_languages,
    surface_matches_language, uses_postpositions, word_order, Detector,
};

#[test]
fn latin_text_is_english() {
    assert_eq!(detect("Hello"), Language::English);
}

#[test]
fn cyrillic_text_is_russian() {
    assert_eq!(detect("Привет"), Language::Russian);
}

#[test]
fn devanagari_text_is_hindi() {
    assert_eq!(detect("नमस्ते"), Language::Hindi);
}

#[test]
fn cjk_text_is_chinese() {
    assert_eq!(detect("你好"), Language::Chinese);
}

#[test]
fn arabic_text_is_unknown() {
    assert_eq!(detect("لطفاً سلام بگو"), Language::Unknown);
}

#[test]
fn empty_prompt_defaults_to_english() {
    assert_eq!(detect(""), Language::English);
}

// Formal AI issue_706_any_language.rs detection corpus.
#[test]
fn formal_ai_spanish_and_mixed_script_corpus() {
    for (prompt, expected) in [
        ("¿Cómo estás?", Language::Spanish),
        ("hola, ¿quién eres?", Language::Spanish),
        ("gracias por favor", Language::Spanish),
        (
            "Formaliza este requisito: una comprobación debe rechazar la entrada",
            Language::Spanish,
        ),
        ("hello there", Language::English),
        ("что это такое", Language::Russian),
        ("क्या हाल है", Language::Hindi),
        ("你是谁", Language::Chinese),
        ("Расскажи о julián andrés quiñones?", Language::Russian),
        ("介绍一下 julián andrés quiñones?", Language::Chinese),
        ("julián andrés quiñones ¿quién es?", Language::Spanish),
    ] {
        assert_eq!(detect(prompt), expected, "{prompt:?}");
    }
}

#[test]
fn registry_and_metadata_cover_every_language() {
    let languages = registered_languages();
    assert_eq!(languages.len(), 5);
    for (slug, name, order, postpositions) in [
        ("en", "English", "SVO", false),
        ("ru", "Russian", "SVO", false),
        ("hi", "Hindi", "SOV", true),
        ("zh", "Chinese", "SVO", true),
        ("es", "Spanish", "SVO", false),
    ] {
        let language = from_slug(slug).unwrap();
        assert!(languages.contains(&language));
        assert_eq!(language.slug(), slug);
        assert_eq!(language_name(slug), Some(name));
        assert_eq!(word_order(slug), Some(order));
        assert_eq!(uses_postpositions(slug), postpositions);
        assert_eq!(
            language_for_concept_slug(&format!("language_{}", name.to_lowercase())),
            Some(language)
        );
    }
    assert_eq!(fallback_language(), Language::English);
    assert_eq!(from_slug("qq"), None);
    assert_eq!(language_name("qq"), None);
    assert_eq!(word_order("qq"), None);
    assert!(!uses_postpositions("qq"));
    assert_eq!(language_for_concept_slug("language_missing"), None);
    assert_eq!(language_for_concept_slug("spanish"), None);
}

#[test]
fn shared_script_surfaces_and_unknown_slugs() {
    assert!(surface_matches_language("manzana", "es"));
    assert!(surface_matches_language("яблоко", "ru"));
    assert!(!surface_matches_language("яблоко", "zh"));
    assert!(!surface_matches_language("abc", "unknown"));
    assert!(!surface_matches_language("123 [\\]^_`", "en"));
}

#[test]
fn lexical_cues_respect_word_starts_and_case() {
    assert_eq!(detect("Describe a function"), Language::English);
    assert_eq!(
        detect("Please describe this requirement"),
        Language::English
    );
    assert_eq!(detect("ESCRIBE una función"), Language::Spanish);
    assert_eq!(
        detect("describe first, then escribe una función"),
        Language::Spanish
    );
    assert_eq!(detect("这是什么"), Language::Chinese);
}

#[test]
fn mixed_scripts_preserve_formal_ai_routing() {
    for (text, expected) in [
        (
            "Расскажи о a_very_long_english_identifier",
            Language::Russian,
        ),
        ("क्या is a_very_long_english_identifier", Language::Hindi),
        ("这是什么 a_very_long_english_identifier", Language::Chinese),
        ("English Русский", Language::Russian),
        ("   123!? [\\]^_` 🦀", Language::English),
        ("سلام سلام hello", Language::Unknown),
    ] {
        assert_eq!(detect(text), expected, "{text:?}");
    }
}

#[test]
fn owned_overrides_are_independent_and_can_be_cleared() {
    let mut first = Detector::default();
    let second = Detector::default();
    assert_eq!(first.set_forced_language(Some(Language::Spanish)), None);
    assert_eq!(first.forced_language(), Some(Language::Spanish));
    assert_eq!(first.detect("你好"), Language::Spanish);
    assert_eq!(second.detect("你好"), Language::Chinese);
    assert_eq!(first.set_forced_language(None), Some(Language::Spanish));
    assert_eq!(first.detect("你好"), Language::Chinese);
}

#[cfg(feature = "std")]
#[test]
fn scoped_overrides_restore_nested_values_and_stay_on_their_thread() {
    use human_language::language::{forced_response_language_slug, set_forced_language};
    assert_eq!(forced_response_language_slug(), None);
    {
        let _outer = set_forced_language(Some(Language::Spanish));
        assert_eq!(detect("你好"), Language::Spanish);
        assert_eq!(forced_response_language_slug(), Some("es"));
        assert_eq!(Detector::default().detect("你好"), Language::Chinese);
        assert_eq!(
            std::thread::spawn(|| detect("你好")).join().unwrap(),
            Language::Chinese
        );
        {
            let _inner = set_forced_language(Some(Language::Hindi));
            assert_eq!(detect("你好"), Language::Hindi);
            {
                let _clear = set_forced_language(None);
                assert_eq!(detect("你好"), Language::Chinese);
            }
            assert_eq!(detect("你好"), Language::Hindi);
        }
        assert_eq!(detect("你好"), Language::Spanish);
    }
    assert_eq!(forced_response_language_slug(), None);
    assert_eq!(detect("你好"), Language::Chinese);
}

#[cfg(feature = "std")]
#[test]
fn scoped_override_restores_after_unwinding() {
    use human_language::language::{forced_response_language_slug, set_forced_language};
    let result = std::panic::catch_unwind(|| {
        let _guard = set_forced_language(Some(Language::Hindi));
        panic!("exercise guard cleanup");
    });
    assert!(result.is_err());
    assert_eq!(forced_response_language_slug(), None);
    assert_eq!(detect("hello"), Language::English);
}
