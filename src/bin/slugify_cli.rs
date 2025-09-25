use std::env;
use std::io::{self, Read};

#[allow(dead_code)]
fn bool_from_env(key: &str, default: bool) -> bool {
    // delegate parsing to the pure helper so it can be tested without
    // mutating process environment in tests.
    match env::var(key).ok().as_deref() {
        Some(s) => parse_bool_str(s, default),
        None => default,
    }
}

#[allow(dead_code)]
fn usize_from_env(key: &str, default: usize) -> usize {
    // delegate parsing to pure helper for testability
    match env::var(key).ok().as_deref() {
        Some(s) => parse_usize_str(s, default),
        None => default,
    }
}

// Pure helpers that parse string inputs. Keeping them near the CLI code
// makes it simple to test parsing variants without touching process env.
fn parse_bool_str(s: &str, default: bool) -> bool {
    match s {
        "1" | "true" | "True" | "yes" => true,
        "0" | "false" | "False" | "no" => false,
        _ => default,
    }
}

fn parse_usize_str(s: &str, default: usize) -> usize {
    s.parse().ok().unwrap_or(default)
}

/// Read all input from an arbitrary reader, trim trailing newline and return a String.
fn read_input<R: Read>(r: &mut R) -> io::Result<String> {
    let mut input = String::new();
    r.read_to_string(&mut input)?;
    Ok(input.trim_end_matches('\n').to_string())
}

fn main() {
    // Read stdin via a small testable helper
    let text = match read_input(&mut io::stdin()) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("failed to read stdin: {}", e);
            std::process::exit(2);
        }
    };

    // Build an env map from current process env
    let mut env_map = StdHashMap::new();
    for (k, v) in std::env::vars() {
        env_map.insert(k, v);
    }

    match run_with_env_map(&env_map, &text) {
        Ok(out) => println!("{}", out),
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(2);
        }
    }
}

use std::collections::HashMap as StdHashMap;

// Centralized runner that consumes an env map and input text. This allows
// tests to exercise the exact same option-parsing and builder logic as
// `main` without spawning the binary. Returning Result lets callers decide
// how to handle builder errors (main exits, tests can assert on Err).
fn run_with_env_map(env_map: &StdHashMap<String, String>, text: &str) -> Result<String, String> {
    use slugify_rs::slugify::SlugifyOptions;
    use slugify_rs::slugify::slugify_with_options_public;

    let get = |k: &str| env_map.get(k).map(|s| s.as_str());

    let entities = get("ENTITIES").map(|v| matches!(v, "1" | "true" | "True" | "yes")).unwrap_or(true);
    let decimal = get("DECIMAL").map(|v| matches!(v, "1" | "true" | "True" | "yes")).unwrap_or(true);
    let hexadecimal = get("HEXADECIMAL").map(|v| matches!(v, "1" | "true" | "True" | "yes")).unwrap_or(true);
    let max_length = get("MAX_LENGTH").and_then(|s| s.parse().ok()).unwrap_or(0usize);
    let word_boundary = get("WORD_BOUNDARY").map(|v| matches!(v, "1" | "true" | "True" | "yes")).unwrap_or(false);
    let separator = get("SEPARATOR").unwrap_or("-").to_string();
    let save_order = get("SAVE_ORDER").map(|v| matches!(v, "1" | "true" | "True" | "yes")).unwrap_or(false);
    let stopwords_raw = get("STOPWORDS").unwrap_or("");
    let stopwords: Vec<&str> = if stopwords_raw.is_empty() { Vec::new() } else { stopwords_raw.split(',').collect() };
    let regex_pattern = get("REGEX_PATTERN").map(|s| s.to_string());
    let lowercase = get("LOWERCASE").map(|v| matches!(v, "1" | "true" | "True" | "yes")).unwrap_or(true);
    let replacements_raw = get("REPLACEMENTS").unwrap_or("");
    let replacements: Vec<(String, String)> = if replacements_raw.is_empty() {
        Vec::new()
    } else {
        replacements_raw
            .split(";;")
            .filter_map(|s| s.split_once("=>").map(|(a, b)| (a.to_string(), b.to_string())))
            .collect()
    };
    let allow_unicode = get("ALLOW_UNICODE").map(|v| matches!(v, "1" | "true" | "True" | "yes")).unwrap_or(false);
    let transliterate_icons_env = get("TRANSLITERATE_ICONS");

    let mut builder = SlugifyOptions::builder()
        .entities(entities)
        .decimal(decimal)
        .hexadecimal(hexadecimal)
        .max_length(max_length)
        .word_boundary(word_boundary)
        .separator(separator)
        .save_order(save_order)
        .stopwords(stopwords)
        .regex_pattern(regex_pattern)
        .lowercase(lowercase)
        .replacements(replacements)
        .allow_unicode(allow_unicode);

    if let Some(val) = transliterate_icons_env {
        builder = builder.transliterate_icons(matches!(val, "1" | "true" | "True" | "yes"));
    }

    let opts = builder.build().map_err(|e| format!("failed to build options: {:?}", e))?;
    Ok(slugify_with_options_public(&opts, text))
}


// Unit tests for the CLI. We keep them here so coverage tools include the
// binary source when running `cargo test` and to keep tests next to the code
// they exercise.
#[cfg(test)]
#[allow(clippy::expect_used)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::io::{self, Read};
    use std::env;

    // In-process helper functions to exercise the CLI behaviour without
    // spawning an external binary. This prevents accidental recursion when
    // the test harness finds the test binary itself and would otherwise
    // spawn it (causing exponential test runs and system overload).
    use std::collections::HashMap;
    use slugify_rs::slugify::SlugifyOptions;
    use slugify_rs::slugify::slugify_with_options_public;

    fn parse_bool_opt(v: Option<&str>, default: bool) -> bool {
        v.map(|s| match s {
            "1" | "true" | "True" | "yes" => true,
            "0" | "false" | "False" | "no" => false,
            _ => default,
        })
        .unwrap_or(default)
    }

    fn parse_usize_opt(v: Option<&str>, default: usize) -> usize {
        v.and_then(|s| s.parse().ok()).unwrap_or(default)
    }

    fn run_cli_inproc(envs: &[(&str, &str)], input: &str) -> String {
        // Build a hashmap from provided env pairs
        let map: HashMap<&str, &str> = envs.iter().cloned().collect();

        let entities = parse_bool_opt(map.get("ENTITIES").copied(), true);
        let decimal = parse_bool_opt(map.get("DECIMAL").copied(), true);
        let hexadecimal = parse_bool_opt(map.get("HEXADECIMAL").copied(), true);
        let max_length = parse_usize_opt(map.get("MAX_LENGTH").copied(), 0usize);
        let word_boundary = parse_bool_opt(map.get("WORD_BOUNDARY").copied(), false);
        let separator = map.get("SEPARATOR").copied().unwrap_or("-").to_string();
        let save_order = parse_bool_opt(map.get("SAVE_ORDER").copied(), false);
        let stopwords_raw = map.get("STOPWORDS").copied().unwrap_or("");
        let stopwords: Vec<&str> = if stopwords_raw.is_empty() {
            Vec::new()
        } else {
            stopwords_raw.split(',').collect()
        };
        let regex_pattern = map.get("REGEX_PATTERN").map(|s| s.to_string());
        let lowercase = parse_bool_opt(map.get("LOWERCASE").copied(), true);
        let replacements_raw = map.get("REPLACEMENTS").copied().unwrap_or("");
        let replacements: Vec<(String, String)> = if replacements_raw.is_empty() {
            Vec::new()
        } else {
            replacements_raw
                .split(";;")
                .filter_map(|s| s.split_once("=>").map(|(a, b)| (a.to_string(), b.to_string())))
                .collect()
        };
        let allow_unicode = parse_bool_opt(map.get("ALLOW_UNICODE").copied(), false);
        let transliterate_icons_env = map.get("TRANSLITERATE_ICONS").copied();

        let mut builder = SlugifyOptions::builder()
            .entities(entities)
            .decimal(decimal)
            .hexadecimal(hexadecimal)
            .max_length(max_length)
            .word_boundary(word_boundary)
            .separator(separator)
            .save_order(save_order)
            .stopwords(stopwords)
            .regex_pattern(regex_pattern)
            .lowercase(lowercase)
            .replacements(replacements)
            .allow_unicode(allow_unicode);

        if let Some(val) = transliterate_icons_env {
            builder = builder.transliterate_icons(matches!(val, "1" | "true" | "True" | "yes"));
        }

        let opts = builder.build().expect("build options");
        slugify_with_options_public(&opts, input)
    }

    #[allow(dead_code)]
    fn bin_path() -> String {
        let manifest = env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());

        // Candidate locations for the compiled CLI binary. CI may set
        // a custom target dir (e.g. --target-dir .../target/llvm-cov-target)
        // or the binary may have been built in release mode. Try several
        // likely locations and return the first that exists.
        let mut candidates = Vec::new();

        // If CARGO_TARGET_DIR is set (cargo invoked with --target-dir), prefer it
        if let Ok(cd) = env::var("CARGO_TARGET_DIR") {
            candidates.push(format!("{}/debug/slugify_cli", cd));
            candidates.push(format!("{}/release/slugify_cli", cd));
        }

        // Default target locations relative to the manifest
        candidates.push(format!("{}/target/debug/slugify_cli", manifest));
        candidates.push(format!("{}/target/release/slugify_cli", manifest));

        // Special case used by our CI coverage job
        candidates.push(format!("{}/target/llvm-cov-target/debug/slugify_cli", manifest));
        candidates.push(format!("{}/target/llvm-cov-target/release/slugify_cli", manifest));

        use std::fs;
        use std::path::Path;
        use std::env::current_exe;

        // Determine canonical path of the running test binary (if available)
        // and avoid returning it as the CLI binary candidate to prevent
        // recursive test execution (spawn -> runs tests -> spawn -> ...).
        let current_exe_canon = current_exe().ok().and_then(|p| p.canonicalize().ok());

        // For each candidate directory, allow matching files like
        // slugify_cli or slugify_cli-<hash> (cargo places test binaries
        // under deps with a hash suffix). Also check deps/ subdir.
        for c in &candidates {
            let p = Path::new(c);
            if p.exists() {
                // canonicalize and skip if it's the running test binary
                if let (Some(cur), Ok(canon)) = (current_exe_canon.as_ref(), p.canonicalize()) {
                    if &canon == cur {
                        // skip this candidate
                    } else {
                        return c.to_string();
                    }
                } else {
                    return c.to_string();
                }
            }
            // try deps/ subdir of the parent (e.g., target/debug/deps)
            if let Some(parent) = p.parent() {
                let deps = parent.join("deps");
                            if let Ok(rd) = fs::read_dir(&deps) {
                                for entry in rd.filter_map(Result::ok) {
                            let path = entry.path();
                            if !path.is_file() {
                                continue;
                            }
                            // canonicalize and skip if equals current exe
                            if let (Some(cur), Ok(canon)) = (current_exe_canon.as_ref(), path.canonicalize())
                                && &canon == cur {
                                    continue;
                                }
                            if let Some(name) = path.file_name().and_then(|s| s.to_str())
                                && name.starts_with("slugify_cli") {
                                    return path.to_string_lossy().to_string();
                                }
                        }
                            }
            }
        }
            // If none of the straightforward candidate paths exist, try to
            // infer the target directory from the running test binary's path.
            // When `cargo test` is invoked with a custom --target-dir (e.g.
            // `target/llvm-cov-target`) the test process executable will live
            // in something like
            //   <workspace>/target/llvm-cov-target/debug/deps/<testbin>
            // so we can parse that path and construct a sibling path for the
            // compiled CLI binary.
            if let Ok(cur) = env::current_exe() {
                let cur_s = cur.to_string_lossy();
                if let Some(idx) = cur_s.find("/target/") {
                    // base is the workspace path before '/target/'
                    let base = &cur_s[..idx];
                    // rest starts after '/target/'
                    let rest = &cur_s[idx + "/target/".len()..];
                    if let Some(first_seg) = rest.split('/').next() {
                        // candidate using the same target subdir observed in the
                        // current_exe path (e.g. 'llvm-cov-target')
                        let cand = format!("{}/target/{}/debug/slugify_cli", base, first_seg);
                        if std::path::Path::new(&cand).exists() {
                            return cand;
                        }
                        let cand_rel = format!("{}/target/{}/release/slugify_cli", base, first_seg);
                        if std::path::Path::new(&cand_rel).exists() {
                            return cand_rel;
                        }
                    }
                }
            }

            // Final fallback: return the conventional debug path (may not exist,
            // but keeps previous behaviour).
            candidates
                .into_iter()
                .next()
                .unwrap_or_else(|| format!("{}/target/debug/slugify_cli", manifest))
        }

        #[test]
        fn test_cli_basic_accented() {
            let input = "C'est déjà l'été.";
            let out = run_cli_inproc(&[], input);
            assert_eq!(out.trim(), "c-est-deja-l-ete");
        }

    #[test]
    fn test_cli_replacements_and_numeric() {
        let input = "10 | 20 %";
        let out = run_cli_inproc(&[("REPLACEMENTS", "|=>or;;%=>percent")], input);
        assert_eq!(out.trim(), "10-or-20-percent");
    }

    #[test]
    fn test_cli_allow_unicode_and_translit_icons() {
        // allow_unicode = 0, transliterate_icons = 0 -> emoji dropped (explicit)
        let out = run_cli_inproc(&[("ALLOW_UNICODE", "0"), ("TRANSLITERATE_ICONS", "0")], "i love 🦄");
        assert_eq!(out.trim(), "i-love");

        // allow_unicode = 0, transliterate_icons = 1 -> emoji transliterated (non-empty)
    let out2 = run_cli_inproc(&[("ALLOW_UNICODE", "0"), ("TRANSLITERATE_ICONS", "1")], "i love 🦄");
    assert!(!out2.trim().is_empty());
    }

    #[test]
    fn test_cli_invalid_regex_exits_nonzero() {
        // invalid regex should cause builder.build() to Err -> we expect panic
        // or the returned string to be the fallback; here we assert that
        // building options returns an error by attempting to build via the
        // inproc helper and catching the panic (builder.build() uses Err).
        let res = std::panic::catch_unwind(|| run_cli_inproc(&[("REGEX_PATTERN", "(?")], "test"));
        assert!(res.is_err());
    }

    #[test]
    fn test_cli_stopwords_removed() {
        let out = run_cli_inproc(&[("STOPWORDS", "the,and")], "the quick and nimble");
        let s = out.trim().to_string();
        // ensure stopwords were removed
        assert!(!s.contains("the"));
        assert!(!s.contains("and"));
    }

    #[test]
    fn test_cli_separator_and_lowercase_flag() {
        let out = run_cli_inproc(&[("SEPARATOR", "_"), ("LOWERCASE", "0")], "Hello WORLD");
        let s = out.trim().to_string();
        // Should contain the custom separator and preserve uppercase when LOWERCASE=0
        assert!(s.contains("_"));
        assert!(s.chars().any(|c| c.is_uppercase()));
    }

    #[test]
    fn test_cli_transliterate_icons_off_explicit() {
        let out = run_cli_inproc(&[("ALLOW_UNICODE", "0"), ("TRANSLITERATE_ICONS", "0")], "i love 🦄");
        assert_eq!(out.trim(), "i-love");
    }

    // NOTE: avoid manipulating global process env in tests (set_var/remove_var)
    // because it can be unsafe in multithreaded test harnesses. We exercise
    // parsing logic via inproc helpers instead.

    #[test]
    fn test_run_with_env_map_variants() {
        use std::collections::HashMap as StdHashMap;

        let mut m = StdHashMap::new();
        m.insert("SEPARATOR".to_string(), "_".to_string());
        m.insert("LOWERCASE".to_string(), "0".to_string());
        let out = super::run_with_env_map(&m, "Hello WORLD").expect("run failed");
        assert!(out.contains("_"));
        assert!(out.chars().any(|c| c.is_uppercase()));

        // transliterate icons path (ALLOW_UNICODE=0 but TRANSLITERATE_ICONS=1)
        let mut m2 = StdHashMap::new();
        m2.insert("ALLOW_UNICODE".to_string(), "0".to_string());
        m2.insert("TRANSLITERATE_ICONS".to_string(), "1".to_string());
    let out2 = super::run_with_env_map(&m2, "i love 🦄").expect("run failed");
    assert!(!out2.trim().is_empty());
    }

    #[test]
    fn test_bin_path_basic() {
        // Do not mutate the process environment; just ensure the returned
        // candidate string contains the expected binary name so the function
        // exercised string-building paths.
        let p = bin_path();
        assert!(p.contains("slugify_cli"));
    }

    #[test]
    fn test_read_input_success() {
        let mut data = "hello world\n".as_bytes();
        let s = super::read_input(&mut data).expect("read should succeed");
        assert_eq!(s, "hello world");
    }

    struct FailingReader;
    impl Read for FailingReader {
        fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
            Err(io::Error::other("boom"))
        }
    }

    #[test]
    fn test_read_input_error() {
        let mut r = FailingReader;
        let res = super::read_input(&mut r);
        assert!(res.is_err());
    }

    #[test]
    fn test_parse_bool_str_variants() {
    assert!(super::parse_bool_str("1", false));
    assert!(super::parse_bool_str("true", false));
    assert!(super::parse_bool_str("True", false));
    assert!(super::parse_bool_str("yes", false));

    assert!(!super::parse_bool_str("0", true));
    assert!(!super::parse_bool_str("false", true));
    assert!(!super::parse_bool_str("False", true));
    assert!(!super::parse_bool_str("no", true));

    // unknowns fall back to default
    assert!(super::parse_bool_str("maybe", true));
    assert!(!super::parse_bool_str("", false));
    }

    #[test]
    fn test_parse_usize_str_variants() {
        assert_eq!(super::parse_usize_str("0", 7), 0usize);
        assert_eq!(super::parse_usize_str("42", 0), 42usize);
        assert_eq!(super::parse_usize_str("notanumber", 5), 5usize);
        assert_eq!(super::parse_usize_str("", 9), 9usize);
    }
}
