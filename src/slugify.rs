use deunicode::deunicode;
use html_escape::decode_html_entities;
use once_cell::sync::Lazy;
use regex::Regex;
use unicode_normalization::UnicodeNormalization;
use unicode_segmentation::UnicodeSegmentation;

/// Default separator used by slugify
pub const DEFAULT_SEPARATOR: &str = "-";

// `special` is now a crate-level module in `src/special.rs`.
pub use crate::special::apply_pre_translations;

// Regex patterns (compiled once)
#[allow(clippy::unwrap_used)]
pub static CHAR_ENTITY_PATTERN: Lazy<Regex> = Lazy::new(|| {
    // Match named entities like &amp; &nbsp; etc. Full name-to-codepoint map
    // is handled by `html_escape::decode_html_entities` when decoding the
    // entire string. This pattern is kept if you want manual replace logic.
    Regex::new(r"&([A-Za-z0-9]+);").unwrap()
});

#[allow(clippy::unwrap_used)]
pub static DECIMAL_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"&#(\d+);").unwrap());
#[allow(clippy::unwrap_used)]
pub static HEX_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"&#x([\da-fA-F]+);").unwrap());
#[allow(clippy::unwrap_used)]
pub static QUOTE_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"'+").unwrap());
#[allow(clippy::unwrap_used)]
pub static DISALLOWED_CHARS_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"[^-A-Za-z0-9]+").unwrap());
// Rust `regex` supports `\W` (non-word). Keep underscore in the class as in Python.
#[allow(clippy::unwrap_used)]
pub static DISALLOWED_UNICODE_CHARS_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"[\W_]+").unwrap());
#[allow(clippy::unwrap_used)]
pub static DUPLICATE_DASH_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"-{2,}").unwrap());

// Note: Python used lookbehind/lookahead for numbers ("," between digits).
// Rust's `regex` crate doesn't support lookarounds, so we implement a helper
// `remove_commas_between_digits` below and use it instead of a regex.

/// Public convenience wrapper that mirrors the original API. It builds a
/// `SlugifyOptions` from the long argument list and delegates to the
/// internal `slugify_with_options` implementation. This keeps external
/// callers working while the internal pipeline uses a single options struct.
#[allow(clippy::too_many_arguments)]
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
    // Try to construct options; on invalid regex fall back to ignoring the
    // provided pattern to preserve previous behavior.
    // Try to build with provided regex; if invalid, fall back to None.
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
        false,
    ) {
        Ok(o) => o,
        Err(_) => match SlugifyOptions::from_args(
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
            false,
        ) {
            Ok(o2) => o2,
            Err(e) => panic!("failed to build SlugifyOptions after fallback: {:?}", e),
        },
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
    pub transliterate_icons: bool,
}

#[derive(Debug)]
pub enum SlugifyError {
    InvalidRegex(String),
}

impl SlugifyOptions {
    #[allow(clippy::too_many_arguments)]
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
        transliterate_icons: bool,
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
            transliterate_icons,
        })
    }

    /// Return a builder for `SlugifyOptions` with sensible defaults.
    pub fn builder() -> SlugifyOptionsBuilder {
        SlugifyOptionsBuilder::default()
    }
}

/// Builder for `SlugifyOptions` to avoid long argument lists and improve ergonomics.
#[derive(Debug, Clone)]
pub struct SlugifyOptionsBuilder {
    entities: bool,
    decimal: bool,
    hexadecimal: bool,
    max_length: usize,
    word_boundary: bool,
    separator: String,
    save_order: bool,
    stopwords: Vec<String>,
    regex_pattern: Option<String>,
    lowercase: bool,
    replacements: Vec<(String, String)>,
    allow_unicode: bool,
    transliterate_icons: bool,
}

impl Default for SlugifyOptionsBuilder {
    fn default() -> Self {
        SlugifyOptionsBuilder {
            entities: true,
            decimal: true,
            hexadecimal: true,
            max_length: 0,
            word_boundary: false,
            separator: DEFAULT_SEPARATOR.to_string(),
            save_order: false,
            stopwords: Vec::new(),
            regex_pattern: None,
            lowercase: true,
            replacements: Vec::new(),
            allow_unicode: false,
            transliterate_icons: true,
        }
    }
}

impl SlugifyOptionsBuilder {
    pub fn entities(mut self, v: bool) -> Self {
        self.entities = v;
        self
    }
    pub fn decimal(mut self, v: bool) -> Self {
        self.decimal = v;
        self
    }
    pub fn hexadecimal(mut self, v: bool) -> Self {
        self.hexadecimal = v;
        self
    }
    pub fn max_length(mut self, v: usize) -> Self {
        self.max_length = v;
        self
    }
    pub fn word_boundary(mut self, v: bool) -> Self {
        self.word_boundary = v;
        self
    }
    pub fn separator<S: Into<String>>(mut self, s: S) -> Self {
        self.separator = s.into();
        self
    }
    pub fn save_order(mut self, v: bool) -> Self {
        self.save_order = v;
        self
    }
    pub fn stopwords<I, S>(mut self, words: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        self.stopwords = words.into_iter().map(|s| s.as_ref().to_string()).collect();
        self
    }
    pub fn regex_pattern<S: Into<String>>(mut self, pat: Option<S>) -> Self {
        self.regex_pattern = pat.map(|s| s.into());
        self
    }
    pub fn lowercase(mut self, v: bool) -> Self {
        self.lowercase = v;
        self
    }
    pub fn replacements<I, A, B>(mut self, reps: I) -> Self
    where
        I: IntoIterator<Item = (A, B)>,
        A: AsRef<str>,
        B: AsRef<str>,
    {
        self.replacements = reps
            .into_iter()
            .map(|(a, b)| (a.as_ref().to_string(), b.as_ref().to_string()))
            .collect();
        self
    }
    pub fn allow_unicode(mut self, v: bool) -> Self {
        self.allow_unicode = v;
        self
    }
    pub fn transliterate_icons(mut self, v: bool) -> Self {
        self.transliterate_icons = v;
        self
    }

    /// Build the `SlugifyOptions`, validating the regex if present.
    pub fn build(self) -> Result<SlugifyOptions, SlugifyError> {
        let regex_compiled = if let Some(pat) = self.regex_pattern.as_deref() {
            match Regex::new(pat) {
                Ok(r) => Some(r),
                Err(_) => return Err(SlugifyError::InvalidRegex(pat.to_string())),
            }
        } else {
            None
        };

        Ok(SlugifyOptions {
            entities: self.entities,
            decimal: self.decimal,
            hexadecimal: self.hexadecimal,
            max_length: self.max_length,
            word_boundary: self.word_boundary,
            separator: self.separator,
            save_order: self.save_order,
            stopwords: self.stopwords,
            regex_pattern: regex_compiled,
            lowercase: self.lowercase,
            replacements: self.replacements,
            allow_unicode: self.allow_unicode,
            transliterate_icons: self.transliterate_icons,
        })
    }
}

// New internal API that takes the options struct. Keeps behavior identical.
fn slugify_with_options(input: &str, opts: &SlugifyOptions) -> String {
    // 1. Apply user replacements first (match python-slugify behavior).
    // Note: pre-translations are available via `crate::special::apply_pre_translations`
    // but are NOT applied by default to preserve original Python semantics.
    let after_replacements = apply_replacements(input, &opts.replacements);

    // 2. Replace quotes with separator early to avoid merging words
    let after_quotes = QUOTE_PATTERN
        .replace_all(&after_replacements, DEFAULT_SEPARATOR)
        .to_string();

    // 3. Normalize / transliterate according to `allow_unicode`
    let normalized = normalize_text(&after_quotes, opts.allow_unicode, opts.transliterate_icons);

    // 4. Optionally decode named entities
    let decoded_named = if opts.entities {
        decode_named_entities(&normalized)
    } else {
        normalized
    };

    // 5. Decode numeric references (decimal / hex) as requested
    let decoded_numeric = decode_numeric_refs(&decoded_named, opts.decimal, opts.hexadecimal);

    // 6. Re-normalize and apply lowercase if requested
    let renormalized = normalize_text(&decoded_numeric, opts.allow_unicode, opts.transliterate_icons);
    let case_folded = if opts.lowercase {
        renormalized.to_lowercase()
    } else {
        renormalized
    };

    // 7. Remove quotes (now safe) and cleanup number commas
    let cleaned = QUOTE_PATTERN.replace_all(&case_folded, "").to_string();
    let cleaned = remove_commas_between_digits(&cleaned);

    // 8. Replace disallowed characters with separator using pattern or provided regex
    let sanitized = apply_pattern_replacement(&cleaned, opts);

    // 9. Collapse duplicate separators and trim leading/trailing separators
    let collapsed = DUPLICATE_DASH_PATTERN
        .replace_all(&sanitized, DEFAULT_SEPARATOR)
        .to_string();
    let collapsed = collapsed.trim_matches('-').to_string();

    // 10. Remove stopwords if provided
    let without_stopwords = remove_stopwords(&collapsed, &opts.stopwords, opts.lowercase);

    // 11. Apply replacements again (post-processing)
    let finalized = apply_replacements(&without_stopwords, &opts.replacements);

    // 12. Truncate if requested
    let truncated = if opts.max_length > 0 {
        smart_truncate(
            &finalized,
            opts.max_length,
            opts.word_boundary,
            DEFAULT_SEPARATOR,
            opts.save_order,
        )
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

fn is_emoji(c: char) -> bool {
    // Heuristic ranges covering most common emoji/pictographs
    let cp = c as u32;
    matches!(cp,
        0x1F300..=0x1F5FF | // Misc Symbols and Pictographs
        0x1F600..=0x1F64F | // Emoticons
        0x1F680..=0x1F6FF | // Transport & Map
        0x1F700..=0x1F77F | // Alchemical Symbols
        0x1F900..=0x1F9FF | // Supplemental Symbols and Pictographs
        0x2600..=0x26FF   | // Misc symbols
        0x2700..=0x27BF   | // Dingbats
        0xFE00..=0xFE0F   | // Variation Selectors
        0x1F1E6..=0x1F1FF   // Regional indicator symbols (flags)
    )
}

fn normalize_text(s: &str, allow_unicode: bool, transliterate_icons: bool) -> String {
    if allow_unicode {
        s.nfkc().collect()
    } else {
        // If transliterate_icons is disabled we remove emoji early.
        // If enabled, perform a small, explicit mapping for common
        // pictographs (heart, rocket, unicorn) to ASCII words so
        // `deunicode` can handle the rest similar to python-slugify.
        let filtered: String = if !transliterate_icons {
            s.chars().filter(|c| !is_emoji(*c)).collect()
        } else {
            // Replace a few known icons with ASCII words separated by spaces
            // so later normalization and pattern replacement will turn
            // them into words in the final slug.
            let mut out = String::with_capacity(s.len() * 4);
            for c in s.chars() {
                match c {
                    '♥' => out.push_str(" hearts "),
                    '🚀' => out.push_str(" rocket "),
                    '🦄' => out.push_str(" unicorn "),
                    // fall back to keeping the character for other codepoints
                    other => out.push(other),
                }
            }
            out
        };
        let decomposed: String = filtered.nfkd().collect();
        deunicode(&decomposed)
    }
}

fn decode_named_entities(s: &str) -> String {
    CHAR_ENTITY_PATTERN
        .replace_all(s, |caps: &regex::Captures| {
            let full = caps.get(0).map(|m| m.as_str()).unwrap_or("");
            decode_html_entities(full).to_string()
        })
        .to_string()
}

fn decode_numeric_refs(s: &str, decimal: bool, hexadecimal: bool) -> String {
    let mut out = s.to_string();
    if decimal {
        out = DECIMAL_PATTERN
            .replace_all(&out, |caps: &regex::Captures| {
                caps.get(1)
                    .and_then(|m| m.as_str().parse::<u32>().ok())
                    .and_then(std::char::from_u32)
                    .map(|c| c.to_string())
                    .unwrap_or_default()
            })
            .to_string();
    }
    if hexadecimal {
        out = HEX_PATTERN
            .replace_all(&out, |caps: &regex::Captures| {
                u32::from_str_radix(&caps[1], 16)
                    .ok()
                    .and_then(std::char::from_u32)
                    .map(|c| c.to_string())
                    .unwrap_or_default()
            })
            .to_string();
    }
    out
}

fn apply_pattern_replacement(s: &str, opts: &SlugifyOptions) -> String {
    if let Some(ref rx) = opts.regex_pattern {
        rx.replace_all(s, DEFAULT_SEPARATOR).to_string()
    } else if opts.allow_unicode {
        DISALLOWED_UNICODE_CHARS_PATTERN
            .replace_all(s, DEFAULT_SEPARATOR)
            .to_string()
    } else {
        DISALLOWED_CHARS_PATTERN
            .replace_all(s, DEFAULT_SEPARATOR)
            .to_string()
    }
}

fn remove_stopwords(s: &str, stopwords: &[String], lowercase: bool) -> String {
    if stopwords.is_empty() {
        return s.to_string();
    }
    if lowercase {
        let lower_stop: Vec<String> = stopwords.iter().map(|w| w.to_lowercase()).collect();
        s.split(DEFAULT_SEPARATOR)
            .filter(|w| !lower_stop.contains(&w.to_string()))
            .collect::<Vec<&str>>()
            .join(DEFAULT_SEPARATOR)
    } else {
        s.split(DEFAULT_SEPARATOR)
            .filter(|w| !stopwords.contains(&w.to_string()))
            .collect::<Vec<&str>>()
            .join(DEFAULT_SEPARATOR)
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
        } else if save_order {
            break;
        } else {
            // when not save_order, continue to next word (Python picks different word order)
            // Python's original logic implicitly continues the loop; so we continue here.
            continue;
        }
    }

    if truncated.is_empty() {
        truncated = first_n_chars(&s, max_length);
    }

    truncated
        .trim_matches(|c: char| sep_chars.contains(&c))
        .to_string()
}

/// Public API that accepts an options struct. Prefer this for programmatic use
/// to avoid long argument lists and improve readability.
pub fn slugify_with_options_public(opts: &SlugifyOptions, text: &str) -> String {
    slugify_with_options(text, opts)
}

fn first_n_chars(s: &str, n: usize) -> String {
    // Use grapheme clusters so we don't split combined characters or emoji.
    s.graphemes(true).take(n).collect()
}

#[cfg(test)]
#[allow(clippy::expect_used)]
#[allow(clippy::unwrap_used)]
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

    #[test]
    fn test_from_args_invalid_regex() {
        let res = SlugifyOptions::from_args(
            true, true, true, 0, false, DEFAULT_SEPARATOR, false, &[], Some("(?"), true, &[], false, false,
        );
        match res {
            Err(SlugifyError::InvalidRegex(_)) => {}
            _ => panic!("expected invalid regex error"),
        }
    }

    #[test]
    fn test_apply_pattern_replacement_regex_and_unicode_branches() {
        // regex provided should be used
        let opts = SlugifyOptions::builder()
            .entities(true)
            .decimal(true)
            .hexadecimal(true)
            .max_length(0)
            .word_boundary(false)
            .separator(DEFAULT_SEPARATOR)
            .save_order(false)
            .stopwords(Vec::<&str>::new())
            .regex_pattern(Some(String::from(r"[^a-z\s]+")))
            .lowercase(true)
            .replacements(Vec::<(&str, &str)>::new())
            .allow_unicode(false)
            .transliterate_icons(false)
            .build()
            .unwrap();
        let out = apply_pattern_replacement("hello -- world!!!", &opts);
        assert!(out.contains("hello"));

        // if regex is None and allow_unicode true, use unicode-disallowed pattern
        let opts2 = SlugifyOptions::builder()
            .entities(true)
            .decimal(true)
            .hexadecimal(true)
            .max_length(0)
            .word_boundary(false)
            .separator(DEFAULT_SEPARATOR)
            .save_order(false)
            .stopwords(Vec::<&str>::new())
            .regex_pattern(None::<String>)
            .lowercase(true)
            .replacements(Vec::<(&str, &str)>::new())
            .allow_unicode(true)
            .transliterate_icons(false)
            .build()
            .unwrap();
        let out2 = apply_pattern_replacement("hello 🦄 world!!!", &opts2);
    // since allow_unicode=true, the emoji is not replaced by the ascii pattern; ensure non-empty
    assert!(!out2.is_empty());
    }

    #[test]
    fn test_normalize_and_transliterate_function() {
        // allow_unicode true preserves characters
        let s1 = normalize_and_transliterate("abcÄ", true);
        assert!(s1.contains("Ä") || s1.contains("A"));

        // allow_unicode false transliterates to ascii
        let s2 = normalize_and_transliterate("Ä", false);
        assert!(s2.to_lowercase().contains("a"));
    }

    // Helper wrappers to call slugify with convenient defaults

    fn s_default(text: &str) -> String {
        let opts = SlugifyOptions::builder()
            .entities(true)
            .decimal(true)
            .hexadecimal(true)
            .max_length(0)
            .word_boundary(false)
            .separator(DEFAULT_SEPARATOR)
            .save_order(false)
            .stopwords(Vec::<&str>::new())
            .regex_pattern(None::<String>)
            .lowercase(true)
            .replacements(Vec::<(&str, &str)>::new())
            .allow_unicode(false)
            .transliterate_icons(false)
            .build()
            .unwrap();
        slugify_with_options_public(&opts, text)
    }

    fn s_unicode(text: &str) -> String {
        let opts = SlugifyOptions::builder()
            .entities(true)
            .decimal(true)
            .hexadecimal(true)
            .max_length(0)
            .word_boundary(false)
            .separator(DEFAULT_SEPARATOR)
            .save_order(false)
            .stopwords(Vec::<&str>::new())
            .regex_pattern(None::<String>)
            .lowercase(true)
            .replacements(Vec::<(&str, &str)>::new())
            .allow_unicode(true)
            .transliterate_icons(false)
            .build()
            .unwrap();
        slugify_with_options_public(&opts, text)
    }

    fn s_args_with_opts(text: &str, opts: SlugifyOptions) -> String {
        slugify_with_options_public(&opts, text)
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
            {
                let opts = SlugifyOptions::builder()
                    .entities(true)
                    .decimal(true)
                    .hexadecimal(true)
                    .max_length(9)
                    .word_boundary(false)
                    .separator(DEFAULT_SEPARATOR)
                    .save_order(false)
                    .stopwords(Vec::<&str>::new())
                    .regex_pattern(None::<String>)
                    .lowercase(true)
                    .replacements(Vec::<(&str, &str)>::new())
                    .allow_unicode(false)
                    .transliterate_icons(false)
                    .build()
                    .unwrap();
                slugify_with_options_public(&opts, txt)
            },
            "jaja-lol"
        );
        assert_eq!(
            {
                let opts = SlugifyOptions::builder()
                    .entities(true)
                    .decimal(true)
                    .hexadecimal(true)
                    .max_length(15)
                    .word_boundary(true)
                    .separator(DEFAULT_SEPARATOR)
                    .save_order(false)
                    .stopwords(Vec::<&str>::new())
                    .regex_pattern(None::<String>)
                    .lowercase(true)
                    .replacements(Vec::<(&str, &str)>::new())
                    .allow_unicode(false)
                    .build()
                    .unwrap();
                slugify_with_options_public(&opts, txt)
            },
            "jaja-lol-a"
        );
    }

    #[test]
    fn test_slugify_custom_separator() {
        let txt = "jaja---lol-méméméoo--a";
        let opts = SlugifyOptions::builder()
            .entities(true)
            .decimal(true)
            .hexadecimal(true)
            .max_length(20)
            .word_boundary(true)
            .separator(".")
            .save_order(false)
            .stopwords(Vec::<&str>::new())
            .regex_pattern(None::<String>)
            .lowercase(true)
            .replacements(Vec::<(&str, &str)>::new())
            .allow_unicode(false)
            .transliterate_icons(false)
            .build()
            .unwrap();
        let r = s_args_with_opts(txt, opts);
        assert_eq!(r, "jaja.lol.mememeoo.a");
        let opts2 = SlugifyOptions::builder()
            .entities(true)
            .decimal(true)
            .hexadecimal(true)
            .max_length(20)
            .word_boundary(true)
            .separator("ZZZZZZ")
            .save_order(false)
            .stopwords(Vec::<&str>::new())
            .regex_pattern(None::<String>)
            .lowercase(true)
            .replacements(Vec::<(&str, &str)>::new())
            .allow_unicode(false)
            .transliterate_icons(false)
            .build()
            .unwrap();
        let r2 = s_args_with_opts(txt, opts2);
        assert_eq!(r2, "jajaZZZZZZlolZZZZZZmememeooZZZZZZa");
    }


    #[test]
    fn test_slugify_save_order() {
        let txt = "one two three four five";
        let opts_a = SlugifyOptions::builder()
            .entities(true)
            .decimal(true)
            .hexadecimal(true)
            .max_length(13)
            .word_boundary(true)
            .separator(DEFAULT_SEPARATOR)
            .save_order(true)
            .stopwords(Vec::<&str>::new())
            .regex_pattern(None::<String>)
            .lowercase(true)
            .replacements(Vec::<(&str, &str)>::new())
            .allow_unicode(false)
            .transliterate_icons(false)
            .build()
            .unwrap();
        let opts_b = SlugifyOptions::builder()
            .entities(true)
            .decimal(true)
            .hexadecimal(true)
            .max_length(12)
            .word_boundary(true)
            .separator(DEFAULT_SEPARATOR)
            .save_order(false)
            .stopwords(Vec::<&str>::new())
            .regex_pattern(None::<String>)
            .lowercase(true)
            .replacements(Vec::<(&str, &str)>::new())
            .allow_unicode(false)
            .transliterate_icons(false)
            .build()
            .unwrap();
        let opts_c = SlugifyOptions::builder()
            .entities(true)
            .decimal(true)
            .hexadecimal(true)
            .max_length(12)
            .word_boundary(true)
            .separator(DEFAULT_SEPARATOR)
            .save_order(true)
            .stopwords(Vec::<&str>::new())
            .regex_pattern(None::<String>)
            .lowercase(true)
            .replacements(Vec::<(&str, &str)>::new())
            .allow_unicode(false)
            .transliterate_icons(false)
            .build()
            .unwrap();
        assert_eq!(s_args_with_opts(txt, opts_a), "one-two-three");
        assert_eq!(s_args_with_opts(txt, opts_b), "one-two-four");
        assert_eq!(s_args_with_opts(txt, opts_c), "one-two");
    }

    #[test]
    fn test_slugify_stopwords() {
        let txt = "this has a stopword";
        let opts_stop = SlugifyOptions::builder()
            .entities(true)
            .decimal(true)
            .hexadecimal(true)
            .max_length(0)
            .word_boundary(false)
            .separator(DEFAULT_SEPARATOR)
            .save_order(false)
            .stopwords(vec!["stopword" as &str])
            .regex_pattern(None::<String>)
            .lowercase(true)
            .replacements(Vec::<(&str, &str)>::new())
            .allow_unicode(false)
            .transliterate_icons(false)
            .build()
            .unwrap();
        assert_eq!(s_args_with_opts(txt, opts_stop), "this-has-a");
        let txt2 = "thIs Has a stopword Stopword";
        let opts_stop2 = SlugifyOptions::builder()
            .entities(true)
            .decimal(true)
            .hexadecimal(true)
            .max_length(0)
            .word_boundary(false)
            .separator(DEFAULT_SEPARATOR)
            .save_order(false)
            .stopwords(vec!["Stopword" as &str])
            .regex_pattern(None::<String>)
            .lowercase(false)
            .replacements(Vec::<(&str, &str)>::new())
            .allow_unicode(false)
            .transliterate_icons(false)
            .build()
            .unwrap();
        assert_eq!(s_args_with_opts(txt2, opts_stop2), "thIs-Has-a-stopword");
    }

    #[test]
    fn test_slugify_entities_and_numeric_refs() {
        let txt = "foo &amp; bar";
        assert_eq!(s_default(txt), "foo-bar");
        let opts_entities = SlugifyOptions::builder()
            .entities(false)
            .decimal(true)
            .hexadecimal(true)
            .max_length(0)
            .word_boundary(false)
            .separator(DEFAULT_SEPARATOR)
            .save_order(false)
            .stopwords(Vec::<&str>::new())
            .regex_pattern(None::<String>)
            .lowercase(true)
            .replacements(Vec::<(&str, &str)>::new())
            .allow_unicode(false)
            .transliterate_icons(false)
            .build()
            .unwrap();
        assert_eq!(s_args_with_opts(txt, opts_entities), "foo-amp-bar");

        let dec = "&#381;";
        let opts_dec = SlugifyOptions::builder()
            .entities(true)
            .decimal(true)
            .hexadecimal(true)
            .max_length(0)
            .word_boundary(false)
            .separator(DEFAULT_SEPARATOR)
            .save_order(false)
            .stopwords(Vec::<&str>::new())
            .regex_pattern(None::<String>)
            .lowercase(true)
            .replacements(Vec::<(&str, &str)>::new())
            .allow_unicode(false)
            .transliterate_icons(false)
            .build()
            .unwrap();
        assert_eq!(s_args_with_opts(dec, opts_dec), "z");
        let opts_dec2 = SlugifyOptions::builder()
            .entities(false)
            .decimal(false)
            .hexadecimal(true)
            .max_length(0)
            .word_boundary(false)
            .separator(DEFAULT_SEPARATOR)
            .save_order(false)
            .stopwords(Vec::<&str>::new())
            .regex_pattern(None::<String>)
            .lowercase(true)
            .replacements(Vec::<(&str, &str)>::new())
            .allow_unicode(false)
            .transliterate_icons(false)
            .build()
            .unwrap();
        assert_eq!(s_args_with_opts(dec, opts_dec2), "381");

        let hex = "&#x17D;";
        let opts_hex = SlugifyOptions::builder()
            .entities(true)
            .decimal(true)
            .hexadecimal(true)
            .max_length(0)
            .word_boundary(false)
            .separator(DEFAULT_SEPARATOR)
            .save_order(false)
            .stopwords(Vec::<&str>::new())
            .regex_pattern(None::<String>)
            .lowercase(true)
            .replacements(Vec::<(&str, &str)>::new())
            .allow_unicode(false)
            .transliterate_icons(false)
            .build()
            .unwrap();
        assert_eq!(s_args_with_opts(hex, opts_hex), "z");
        let opts_hex2 = SlugifyOptions::builder()
            .entities(true)
            .decimal(true)
            .hexadecimal(false)
            .max_length(0)
            .word_boundary(false)
            .separator(DEFAULT_SEPARATOR)
            .save_order(false)
            .stopwords(Vec::<&str>::new())
            .regex_pattern(None::<String>)
            .lowercase(true)
            .replacements(Vec::<(&str, &str)>::new())
            .allow_unicode(false)
            .transliterate_icons(false)
            .build()
            .unwrap();
        assert_eq!(s_args_with_opts(hex, opts_hex2), "x17d");
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
        let opts_pattern = SlugifyOptions::builder()
            .entities(true)
            .decimal(true)
            .hexadecimal(true)
            .max_length(0)
            .word_boundary(false)
            .separator(DEFAULT_SEPARATOR)
            .save_order(false)
            .stopwords(Vec::<&str>::new())
            .regex_pattern(Some(r"[^-a-z0-9_]+".to_string()))
            .lowercase(true)
            .replacements(Vec::<(&str, &str)>::new())
            .allow_unicode(false)
            .transliterate_icons(false)
            .build()
            .unwrap();
        let r = s_args_with_opts(txt, opts_pattern);
        assert_eq!(r, "___this-is-a-test___");

        let txt2 = "10 | 20 %";
        let opts_repl = SlugifyOptions::builder()
            .entities(true)
            .decimal(true)
            .hexadecimal(true)
            .max_length(0)
            .word_boundary(false)
            .separator(DEFAULT_SEPARATOR)
            .save_order(false)
            .stopwords(Vec::<&str>::new())
            .regex_pattern(None::<String>)
            .lowercase(true)
            .replacements(vec![("|".to_string(), "or".to_string()), ("%".to_string(), "percent".to_string())])
            .allow_unicode(false)
            .transliterate_icons(false)
            .build()
            .unwrap();
        let r2 = s_args_with_opts(txt2, opts_repl);
        assert_eq!(r2, "10-or-20-percent");
    }

    #[test]
    fn test_slugify_emojis_and_unicode() {
        let txt = "i love 🦄";
        // default (allow_unicode=false) should drop emoji
        assert_eq!(s_unicode(txt), "i-love");
        // allow unicode true but regex_pattern can override to keep emoji
        let opts_emoji = SlugifyOptions::builder()
            .entities(true)
            .decimal(true)
            .hexadecimal(true)
            .max_length(0)
            .word_boundary(false)
            .separator(DEFAULT_SEPARATOR)
            .save_order(false)
            .stopwords(Vec::<&str>::new())
            .regex_pattern(Some(r"[^🦄]+".to_string()))
            .lowercase(true)
            .replacements(Vec::<(&str, &str)>::new())
            .allow_unicode(true)
            .transliterate_icons(false)
            .build()
            .unwrap();
        let r = s_args_with_opts(txt, opts_emoji);
        assert_eq!(r, "🦄");
    }
}
