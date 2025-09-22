use deunicode::deunicode;
use html_escape::decode_html_entities;
use once_cell::sync::Lazy;
use regex::Regex;
use unicode_normalization::UnicodeNormalization;
use unicode_segmentation::UnicodeSegmentation;

/// Default separator used by slugify
pub const DEFAULT_SEPARATOR: &str = "-";

// Regex patterns (compiled once)
pub static CHAR_ENTITY_PATTERN: Lazy<Regex> = Lazy::new(|| {
    // Match named entities like &amp; &nbsp; etc. Full name-to-codepoint map
    // is handled by `html_escape::decode_html_entities` when decoding the
    // entire string. This pattern is kept if you want manual replace logic.
    Regex::new(r"&([A-Za-z0-9]+);").unwrap()
});

pub static DECIMAL_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"&#(\d+);").unwrap());
pub static HEX_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"&#x([\da-fA-F]+);").unwrap());
pub static QUOTE_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"'+").unwrap());
pub static DISALLOWED_CHARS_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"[^-A-Za-z0-9]+").unwrap());
// Rust `regex` supports `\W` (non-word). Keep underscore in the class as in Python.
pub static DISALLOWED_UNICODE_CHARS_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"[\W_]+").unwrap());
pub static DUPLICATE_DASH_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"-{2,}").unwrap());

// Note: Python used lookbehind/lookahead for numbers ("," between digits).
// Rust's `regex` crate doesn't support lookarounds, so we implement a helper
// `remove_commas_between_digits` below and use it instead of a regex.

pub fn slugify(
    text: &str,
    entities: bool,
    decimal: bool,
    hexadecimal: bool,
    max_length: usize,
    word_boundary: bool,
    separator: &str,
    save_order: bool,
    stopwords: &[&str],
    regex_pattern: Option<&str>,
    lowercase: bool,
    replacements: &[(&str, &str)],
    allow_unicode: bool,
) -> String {
    // Build options, if regex invalid fall back to previous behavior by ignoring pattern
    let opts = match SlugifyOptions::from_args(
        entities,
        decimal,
        hexadecimal,
        max_length,
        word_boundary,
        separator,
        save_order,
        stopwords,
        regex_pattern,
        lowercase,
        replacements,
        allow_unicode,
    ) {
        Ok(o) => o,
        Err(_) => {
            // fallback: ignore regex pattern and continue
            SlugifyOptions::from_args(
                entities,
                decimal,
                hexadecimal,
                max_length,
                word_boundary,
                separator,
                save_order,
                stopwords,
                None,
                lowercase,
                replacements,
                allow_unicode,
            )
            .unwrap()
        }
    };

    slugify_with_options(text, &opts)
}

// Configuration struct for slugify to improve readability and avoid long argument lists.
pub struct SlugifyOptions {
    pub entities: bool,
    pub decimal: bool,
    pub hexadecimal: bool,
    pub max_length: usize,
    pub word_boundary: bool,
    pub separator: String,
    pub save_order: bool,
    pub stopwords: Vec<String>,
    pub regex_pattern: Option<Regex>,
    pub lowercase: bool,
    pub replacements: Vec<(String, String)>,
    pub allow_unicode: bool,
}

#[derive(Debug)]
pub enum SlugifyError {
    InvalidRegex(String),
}

impl SlugifyOptions {
    pub fn from_args(
        entities: bool,
        decimal: bool,
        hexadecimal: bool,
        max_length: usize,
        word_boundary: bool,
        separator: &str,
        save_order: bool,
        stopwords: &[&str],
        regex_pattern: Option<&str>,
        lowercase: bool,
        replacements: &[(&str, &str)],
        allow_unicode: bool,
    ) -> Result<Self, SlugifyError> {
        let regex_compiled = if let Some(pat) = regex_pattern {
            match Regex::new(pat) {
                Ok(r) => Some(r),
                Err(_) => return Err(SlugifyError::InvalidRegex(pat.to_string())),
            }
        } else {
            None
        };

        Ok(SlugifyOptions {
            entities,
            decimal,
            hexadecimal,
            max_length,
            word_boundary,
            separator: separator.to_string(),
            save_order,
            stopwords: stopwords.iter().map(|s| s.to_string()).collect(),
            regex_pattern: regex_compiled,
            lowercase,
            replacements: replacements
                .iter()
                .map(|(a, b)| (a.to_string(), b.to_string()))
                .collect(),
            allow_unicode,
        })
    }
}

// New internal API that takes the options struct. Keeps behavior identical.
fn slugify_with_options(input: &str, opts: &SlugifyOptions) -> String {
    // 1. Apply pre and post replacements, normalization, decoding and sanitization
    let after_replacements = apply_replacements(input, &opts.replacements);

    // 2. Replace quotes with separator early to avoid merging words
    let after_quotes = QUOTE_PATTERN.replace_all(&after_replacements, DEFAULT_SEPARATOR).to_string();

    // 3. Normalize / transliterate according to `allow_unicode`
    let normalized = normalize_text(&after_quotes, opts.allow_unicode);

    // 4. Optionally decode named entities
    let decoded_named = if opts.entities { decode_named_entities(&normalized) } else { normalized };

    // 5. Decode numeric references (decimal / hex) as requested
    let decoded_numeric = decode_numeric_refs(&decoded_named, opts.decimal, opts.hexadecimal);

    // 6. Re-normalize and apply lowercase if requested
    let renormalized = normalize_text(&decoded_numeric, opts.allow_unicode);
    let case_folded = if opts.lowercase { renormalized.to_lowercase() } else { renormalized };

    // 7. Remove quotes (now safe) and cleanup number commas
    let cleaned = QUOTE_PATTERN.replace_all(&case_folded, "").to_string();
    let cleaned = remove_commas_between_digits(&cleaned);

    // 8. Replace disallowed characters with separator using pattern or provided regex
    let sanitized = apply_pattern_replacement(&cleaned, opts);

    // 9. Collapse duplicate separators and trim leading/trailing separators
    let collapsed = DUPLICATE_DASH_PATTERN.replace_all(&sanitized, DEFAULT_SEPARATOR).to_string();
    let collapsed = collapsed.trim_matches('-').to_string();

    // 10. Remove stopwords if provided
    let without_stopwords = remove_stopwords(&collapsed, &opts.stopwords, opts.lowercase);

    // 11. Apply replacements again (post-processing)
    let finalized = apply_replacements(&without_stopwords, &opts.replacements);

    // 12. Truncate if requested
    let truncated = if opts.max_length > 0 {
        smart_truncate(&finalized, opts.max_length, opts.word_boundary, DEFAULT_SEPARATOR, opts.save_order)
    } else {
        finalized
    };

    // 13. Replace default separator with requested separator if different
    finalize_separator(&truncated, &opts.separator)
}

fn apply_replacements(input: &str, replacements: &[(String, String)]) -> String {
    if replacements.is_empty() {
        return input.to_string();
    }
    let mut out = input.to_string();
    for (old, new) in replacements.iter() {
        out = out.replace(old, new);
    }
    out
}

fn normalize_text(s: &str, allow_unicode: bool) -> String {
    if allow_unicode {
        s.nfkc().collect()
    } else {
        let decomposed: String = s.nfkd().collect();
        deunicode(&decomposed)
    }
}

fn decode_named_entities(s: &str) -> String {
    CHAR_ENTITY_PATTERN.replace_all(s, |caps: &regex::Captures| {
        let full = caps.get(0).map(|m| m.as_str()).unwrap_or("");
        decode_html_entities(full).to_string()
    }).to_string()
}

fn decode_numeric_refs(s: &str, decimal: bool, hexadecimal: bool) -> String {
    let mut out = s.to_string();
    if decimal {
        out = DECIMAL_PATTERN.replace_all(&out, |caps: &regex::Captures| {
            caps.get(1).and_then(|m| m.as_str().parse::<u32>().ok())
                .and_then(std::char::from_u32)
                .map(|c| c.to_string())
                .unwrap_or_default()
        }).to_string();
    }
    if hexadecimal {
        out = HEX_PATTERN.replace_all(&out, |caps: &regex::Captures| {
            u32::from_str_radix(&caps[1], 16).ok()
                .and_then(std::char::from_u32)
                .map(|c| c.to_string())
                .unwrap_or_default()
        }).to_string();
    }
    out
}

fn apply_pattern_replacement(s: &str, opts: &SlugifyOptions) -> String {
    if let Some(ref rx) = opts.regex_pattern {
        rx.replace_all(s, DEFAULT_SEPARATOR).to_string()
    } else if opts.allow_unicode {
        DISALLOWED_UNICODE_CHARS_PATTERN.replace_all(s, DEFAULT_SEPARATOR).to_string()
    } else {
        DISALLOWED_CHARS_PATTERN.replace_all(s, DEFAULT_SEPARATOR).to_string()
    }
}

fn remove_stopwords(s: &str, stopwords: &[String], lowercase: bool) -> String {
    if stopwords.is_empty() {
        return s.to_string();
    }
    if lowercase {
        let lower_stop: Vec<String> = stopwords.iter().map(|w| w.to_lowercase()).collect();
        s.split(DEFAULT_SEPARATOR).filter(|w| !lower_stop.contains(&w.to_string())).collect::<Vec<&str>>().join(DEFAULT_SEPARATOR)
    } else {
        s.split(DEFAULT_SEPARATOR).filter(|w| !stopwords.contains(&w.to_string())).collect::<Vec<&str>>().join(DEFAULT_SEPARATOR)
    }
}

fn finalize_separator(s: &str, separator: &str) -> String {
    if separator != DEFAULT_SEPARATOR {
        s.replace(DEFAULT_SEPARATOR, separator)
    } else {
        s.to_string()
    }
}

// Note: Python used lookbehind/lookahead for numbers ("," between digits).
// Rust's `regex` crate doesn't support lookarounds, so we implement a small
// helper that removes commas between digits.
pub fn remove_commas_between_digits(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    let mut prev_is_digit = false;

    while let Some(c) = chars.next() {
        if c == ',' {
            let next_is_digit = chars.peek().map(|ch| ch.is_ascii_digit()).unwrap_or(false);
            if prev_is_digit && next_is_digit {
                // skip the comma
            } else {
                out.push(c);
                prev_is_digit = false;
            }
        } else {
            prev_is_digit = c.is_ascii_digit();
            out.push(c);
        }
    }
    out
}

/// Helper: normalize and transliterate when `allow_unicode == false`.
pub fn normalize_and_transliterate(s: &str, allow_unicode: bool) -> String {
    if allow_unicode {
        // Normalize to compatibility composed form (similar to NFKC in Python)
        s.nfkc().collect()
    } else {
        // NFKD then ascii transliteration using `deunicode`
        let decomposed: String = s.nfkd().collect();
        deunicode(&decomposed)
    }
}

pub fn smart_truncate(
    input: &str,
    max_length: usize,
    word_boundary: bool,
    separator: &str,
    save_order: bool,
) -> String {
    // strip characters contained in `separator` from both ends (python semantics)
    let sep_chars: Vec<char> = separator.chars().collect();
    // Trims any leading or trailing characters from `input` that are present in `sep_chars`,
    // and assigns the resulting string to `s`.
    //
    // # Arguments
    //
    // * `input` - The original string to be trimmed.
    // * `sep_chars` - A collection of separator characters to be removed from both ends of `input`.
    //
    // # Returns
    //
    // A new `String` with the specified separator characters removed from the start and end.
    let s = input
        .trim_matches(|c: char| sep_chars.contains(&c))
        .to_string();

    if max_length == 0 {
        return s;
    }

    let char_count = s.chars().count();
    if char_count < max_length {
        return s;
    }

    if !word_boundary {
        return first_n_chars(&s, max_length)
            .trim_matches(|c: char| sep_chars.contains(&c))
            .to_string();
    }

    if !s.contains(separator) {
        return first_n_chars(&s, max_length);
    }

    let mut truncated = String::new();
    for word in s.split(separator) {
        if word.is_empty() {
            continue;
        }
        let next_len = truncated.chars().count() + word.chars().count();
        if next_len < max_length {
            truncated.push_str(word);
            truncated.push_str(separator);
        } else if next_len == max_length {
            truncated.push_str(word);
            break;
        } else {
            if save_order {
                break;
            } else {
                // when not save_order, continue to next word (Python picks different word order)
                // Python's original logic implicitly continues the loop; so we continue here.
                continue;
            }
        }
    }

    if truncated.is_empty() {
        truncated = first_n_chars(&s, max_length);
    }

    truncated
        .trim_matches(|c: char| sep_chars.contains(&c))
        .to_string()
}

fn first_n_chars(s: &str, n: usize) -> String {
    // Use grapheme clusters so we don't split combined characters or emoji.
    s.graphemes(true).take(n).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_commas_between_digits() {
        assert_eq!(remove_commas_between_digits("1,234"), "1234");
        assert_eq!(remove_commas_between_digits("1,234,567"), "1234567");
        assert_eq!(remove_commas_between_digits("1,234 apples"), "1234 apples");
        assert_eq!(
            remove_commas_between_digits("apples, 1,234"),
            "apples, 1234"
        );
        assert_eq!(
            remove_commas_between_digits("no commas here"),
            "no commas here"
        );
        assert_eq!(
            remove_commas_between_digits(",leading comma"),
            ",leading comma"
        );
        assert_eq!(
            remove_commas_between_digits("trailing comma,"),
            "trailing comma,"
        );
        assert_eq!(
            remove_commas_between_digits("multiple,,commas"),
            "multiple,,commas"
        );
    }

    #[test]
    fn test_smart_truncate() {
        let start = std::time::Instant::now();
        let input = "This is a simple test string for truncation.";
        assert_eq!(smart_truncate(input, 10, false, " ", true), "This is a");
        assert_eq!(smart_truncate(input, 10, true, " ", true), "This is a");
        assert_eq!(
            smart_truncate(input, 0, true, " ", true),
            "This is a simple test string for truncation."
        );
        assert_eq!(
            smart_truncate(input, 50, true, " ", true),
            "This is a simple test string for truncation."
        );
        assert_eq!(
            smart_truncate(input, 15, true, " ", false),
            "This is a test"
        );
        let elapsed = start.elapsed();
        println!("[TIMER] test_smart_truncate: {} ms", elapsed.as_millis());
    }

    #[test]
    fn test_smart_truncate_no_max_length_python_case() {
        let start = std::time::Instant::now();
        let txt = "1,000 reasons you are #1";
        // default max_length == 0 should return the original string
        assert_eq!(smart_truncate(txt, 0, false, " ", false), txt);
        let elapsed = start.elapsed();
        println!(
            "[TIMER] test_smart_truncate_no_max_length_python_case: {} ms",
            elapsed.as_millis()
        );
    }

    #[test]
    fn test_smart_truncate_no_separator_python_case() {
        let start = std::time::Instant::now();
        let txt = "1,000 reasons you are #1";
        // separator '_' is not present in the input; with large max_length, should return original
        assert_eq!(smart_truncate(txt, 100, false, "_", false), txt);
        let elapsed = start.elapsed();
        println!(
            "[TIMER] test_smart_truncate_no_separator_python_case: {} ms",
            elapsed.as_millis()
        );
    }

    #[test]
    fn test_first_n_chars_combining_and_emoji() {
        // letter 'a' + combining acute accent (two codepoints but one grapheme)
        let s = "a\u{0301}bc"; // á b c (a + combining acute)
        assert_eq!(first_n_chars(s, 1), "a\u{0301}");
        // emoji family: man + zwj + heart + zwj + man etc. Use a multi-codepoint emoji
        let emoji = "👨‍👩‍👧‍👦abc"; // family emoji then abc
        assert_eq!(first_n_chars(emoji, 1), "👨‍👩‍👧‍👦");
        // taking more graphemes than present returns full string
        assert_eq!(first_n_chars("hi", 10), "hi");
    }

    // Helper wrappers to call slugify with convenient defaults

    fn s_default(text: &str) -> String {
        slugify(
            text,
            true,
            true,
            true,
            0,
            false,
            DEFAULT_SEPARATOR,
            false,
            &[],
            None,
            true,
            &[],
            false,
        )
    }

    fn s_unicode(text: &str) -> String {
        slugify(
            text,
            true,
            true,
            true,
            0,
            false,
            DEFAULT_SEPARATOR,
            false,
            &[],
            None,
            true,
            &[],
            true,
        )
    }

    #[test]
    fn test_slugify_extraneous_separators() {
        assert_eq!(s_default("This is a test ---"), "this-is-a-test");
        assert_eq!(s_default("___This is a test ---"), "this-is-a-test");
        assert_eq!(s_default("___This is a test___"), "this-is-a-test");
    }

    #[test]
    fn test_slugify_non_word_characters() {
        assert_eq!(s_default("This -- is a ## test ---"), "this-is-a-test");
    }

    #[test]
    fn test_slugify_accented_text() {
        let txt = "C'est déjà l'été.";
        assert_eq!(s_default(txt), "c-est-deja-l-ete");
    }

    #[test]
    fn test_slugify_cyrillic_text() {
        let txt = "Компьютер";
        assert_eq!(s_default(txt), "kompiuter");
    }

    #[test]
    fn test_slugify_max_length_and_word_boundary() {
        let txt = "jaja---lol-méméméoo--a";
        assert_eq!(
            slugify(
                txt,
                true,
                true,
                true,
                9,
                false,
                DEFAULT_SEPARATOR,
                false,
                &[],
                None,
                true,
                &[],
                false
            ),
            "jaja-lol"
        );
        assert_eq!(
            slugify(
                txt,
                true,
                true,
                true,
                15,
                true,
                DEFAULT_SEPARATOR,
                false,
                &[],
                None,
                true,
                &[],
                false
            ),
            "jaja-lol-a"
        );
    }

    #[test]
    fn test_slugify_custom_separator() {
        let txt = "jaja---lol-méméméoo--a";
        let r = slugify(
            txt,
            true,
            true,
            true,
            20,
            true,
            ".",
            false,
            &[],
            None,
            true,
            &[],
            false,
        );
        assert_eq!(r, "jaja.lol.mememeoo.a");
        let r2 = slugify(
            txt,
            true,
            true,
            true,
            20,
            true,
            "ZZZZZZ",
            false,
            &[],
            None,
            true,
            &[],
            false,
        );
        assert_eq!(r2, "jajaZZZZZZlolZZZZZZmememeooZZZZZZa");
    }

    #[test]
    fn test_slugify_save_order() {
        let txt = "one two three four five";
        assert_eq!(
            slugify(
                txt,
                true,
                true,
                true,
                13,
                true,
                DEFAULT_SEPARATOR,
                true,
                &[],
                None,
                true,
                &[],
                false
            ),
            "one-two-three"
        );
        assert_eq!(
            slugify(
                txt,
                true,
                true,
                true,
                12,
                true,
                DEFAULT_SEPARATOR,
                false,
                &[],
                None,
                true,
                &[],
                false
            ),
            "one-two-four"
        );
        assert_eq!(
            slugify(
                txt,
                true,
                true,
                true,
                12,
                true,
                DEFAULT_SEPARATOR,
                true,
                &[],
                None,
                true,
                &[],
                false
            ),
            "one-two"
        );
    }

    #[test]
    fn test_slugify_stopwords() {
        let txt = "this has a stopword";
        assert_eq!(
            slugify(
                txt,
                true,
                true,
                true,
                0,
                false,
                DEFAULT_SEPARATOR,
                false,
                &["stopword"],
                None,
                true,
                &[],
                false
            ),
            "this-has-a"
        );
        let txt2 = "thIs Has a stopword Stopword";
        assert_eq!(
            slugify(
                txt2,
                true,
                true,
                true,
                0,
                false,
                DEFAULT_SEPARATOR,
                false,
                &["Stopword"],
                None,
                false,
                &[],
                false
            ),
            "thIs-Has-a-stopword"
        );
    }

    #[test]
    fn test_slugify_entities_and_numeric_refs() {
        let txt = "foo &amp; bar";
        assert_eq!(s_default(txt), "foo-bar");
        assert_eq!(
            slugify(
                txt,
                false,
                true,
                true,
                0,
                false,
                DEFAULT_SEPARATOR,
                false,
                &[],
                None,
                true,
                &[],
                false
            ),
            "foo-amp-bar"
        );

        let dec = "&#381;";
        assert_eq!(
            slugify(
                dec,
                true,
                true,
                true,
                0,
                false,
                DEFAULT_SEPARATOR,
                false,
                &[],
                None,
                true,
                &[],
                false
            ),
            "z"
        );
        assert_eq!(
            slugify(
                dec,
                false,
                false,
                true,
                0,
                false,
                DEFAULT_SEPARATOR,
                false,
                &[],
                None,
                true,
                &[],
                false
            ),
            "381"
        );

        let hex = "&#x17D;";
        assert_eq!(
            slugify(
                hex,
                true,
                true,
                true,
                0,
                false,
                DEFAULT_SEPARATOR,
                false,
                &[],
                None,
                true,
                &[],
                false
            ),
            "z"
        );
        assert_eq!(
            slugify(
                hex,
                true,
                true,
                false,
                0,
                false,
                DEFAULT_SEPARATOR,
                false,
                &[],
                None,
                true,
                &[],
                false
            ),
            "x17d"
        );
    }

    #[test]
    fn test_slugify_numbers_behaviour() {
        let txt = "1,000 reasons you are #1";
        assert_eq!(s_default(txt), "1000-reasons-you-are-1");
        let txt2 = "404";
        assert_eq!(s_default(txt2), "404");
    }

    #[test]
    fn test_slugify_regex_pattern_and_replacements() {
        let txt = "___This is a test___";
        let r = slugify(
            txt,
            true,
            true,
            true,
            0,
            false,
            DEFAULT_SEPARATOR,
            false,
            &[],
            Some(r"[^-a-z0-9_]+"),
            true,
            &[],
            false,
        );
        assert_eq!(r, "___this-is-a-test___");

        let txt2 = "10 | 20 %";
        let r2 = slugify(
            txt2,
            true,
            true,
            true,
            0,
            false,
            DEFAULT_SEPARATOR,
            false,
            &[],
            None,
            true,
            &[("|", "or"), ("%", "percent")],
            false,
        );
        assert_eq!(r2, "10-or-20-percent");
    }

    #[test]
    fn test_slugify_emojis_and_unicode() {
        let txt = "i love 🦄";
        // default (allow_unicode=false) should drop emoji
        assert_eq!(s_unicode(txt), "i-love");
        // allow unicode true but regex_pattern can override to keep emoji
        let r = slugify(
            txt,
            true,
            true,
            true,
            0,
            false,
            DEFAULT_SEPARATOR,
            false,
            &[],
            Some(r"[^🦄]+"),
            true,
            &[],
            true,
        );
        assert_eq!(r, "🦄");
    }
}
