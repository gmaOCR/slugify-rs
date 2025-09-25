// Static PRE_TRANSLATIONS in the exact order expected by the Python tests.
pub static PRE_TRANSLATIONS: &[(&str, &str)] = &[
    ("Ю", "U"),
    ("Щ", "Sch"),
    ("У", "Y"),
    ("Х", "H"),
    ("Я", "Ya"),
    ("Ё", "E"),
    ("ё", "e"),
    ("я", "ya"),
    ("х", "h"),
    ("у", "y"),
    ("щ", "sch"),
    ("ю", "u"),
    ("Ü", "Ue"),
    ("Ö", "Oe"),
    ("Ä", "Ae"),
    ("ä", "ae"),
    ("ö", "oe"),
    ("ü", "ue"),
    ("Ϋ́", "Y"),
    ("Ϋ", "Y"),
    ("Ύ", "Y"),
    ("Υ", "Y"),
    ("Χ", "Ch"),
    ("χ", "ch"),
    ("Ξ", "X"),
    ("ϒ", "Y"),
    ("υ", "y"),
    ("ύ", "y"),
    ("ϋ", "y"),
    ("ΰ", "y"),
];

pub fn pre_translations() -> &'static [(&'static str, &'static str)] {
    PRE_TRANSLATIONS
}

use aho_corasick::{AhoCorasick, AhoCorasickBuilder, MatchKind};
use once_cell::sync::Lazy;

#[allow(clippy::expect_used)]
static AC_AUTOMATON: Lazy<AhoCorasick> = Lazy::new(|| {
    let pats: Vec<&str> = PRE_TRANSLATIONS.iter().map(|(s, _)| *s).collect();
    match AhoCorasickBuilder::new()
        .match_kind(MatchKind::LeftmostLongest)
        .build(&pats)
    {
        Ok(a) => a,
        Err(e) => panic!("failed to build aho-corasick automaton: {:?}", e),
    }
});

pub fn apply_pre_translations(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut last = 0usize;
    for mat in AC_AUTOMATON.find_iter(s) {
        let (start, end) = (mat.start(), mat.end());
        if start > last {
            out.push_str(&s[last..start]);
        }
        let idx = mat.pattern();
        let repl = PRE_TRANSLATIONS[idx].1;
        out.push_str(repl);
        last = end;
    }
    if last < s.len() {
        out.push_str(&s[last..]);
    }
    out
}

#[cfg(test)]
#[allow(clippy::expect_used)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    

    #[test]
    fn test_pre_translations_exact_sequence() {
        let expected: Vec<(&str, &str)> = vec![
            ("Ю", "U"),
            ("Щ", "Sch"),
            ("У", "Y"),
            ("Х", "H"),
            ("Я", "Ya"),
            ("Ё", "E"),
            ("ё", "e"),
            ("я", "ya"),
            ("х", "h"),
            ("у", "y"),
            ("щ", "sch"),
            ("ю", "u"),
            ("Ü", "Ue"),
            ("Ö", "Oe"),
            ("Ä", "Ae"),
            ("ä", "ae"),
            ("ö", "oe"),
            ("ü", "ue"),
            ("Ϋ́", "Y"),
            ("Ϋ", "Y"),
            ("Ύ", "Y"),
            ("Υ", "Y"),
            ("Χ", "Ch"),
            ("χ", "ch"),
            ("Ξ", "X"),
            ("ϒ", "Y"),
            ("υ", "y"),
            ("ύ", "y"),
            ("ϋ", "y"),
            ("ΰ", "y"),
        ];
        let pre = pre_translations();
        assert_eq!(pre.len(), expected.len());
        for (i, &(a, b)) in expected.iter().enumerate() {
            assert_eq!(pre[i].0, a);
            assert_eq!(pre[i].1, b);
        }
    }

    #[test]
    fn test_apply_pre_translations_basic() {
        let input = "ё Test ÜBER Χχ";
        let out = apply_pre_translations(input);
        assert!(out.contains("e Test"));
        assert!(out.contains("UeBER") || out.contains("UeBER"));
        assert!(out.contains("Chch") || out.contains("Chch"));
    }

    #[test]
    fn test_apply_pre_translations_integration_with_slugify() {
        let input = "ё ÜBER";
        let pre = apply_pre_translations(input);
        let opts = crate::slugify::SlugifyOptions::builder()
            .entities(true)
            .decimal(true)
            .hexadecimal(true)
            .max_length(0)
            .word_boundary(false)
            .separator(crate::slugify::DEFAULT_SEPARATOR)
            .save_order(false)
            .stopwords(Vec::<&str>::new())
            .regex_pattern(None::<String>)
            .lowercase(true)
            .replacements(Vec::<(&str, &str)>::new())
            .allow_unicode(false)
            .transliterate_icons(false)
            .build()
            .unwrap();
        let out = crate::slugify::slugify_with_options_public(&opts, &pre);
        assert_eq!(out, "e-ueber");
    }
}
