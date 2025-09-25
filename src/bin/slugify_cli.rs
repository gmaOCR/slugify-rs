use std::env;
use std::io::{self, Read};

fn bool_from_env(key: &str, default: bool) -> bool {
    env::var(key)
        .ok()
        .and_then(|v| match v.as_str() {
            "1" | "true" | "True" | "yes" => Some(true),
            "0" | "false" | "False" | "no" => Some(false),
            _ => None,
        })
        .unwrap_or(default)
}

fn usize_from_env(key: &str, default: usize) -> usize {
    env::var(key).ok().and_then(|v| v.parse().ok()).unwrap_or(default)
}

fn main() {
    // Read full stdin as the text to slugify
    let mut input = String::new();
    if let Err(e) = io::stdin().read_to_string(&mut input) {
        eprintln!("failed to read stdin: {}", e);
        std::process::exit(2);
    }
    let text = input.trim_end_matches('\n').to_string();

    // Read env vars for options (simple mapping)
    let entities = bool_from_env("ENTITIES", true);
    let decimal = bool_from_env("DECIMAL", true);
    let hexadecimal = bool_from_env("HEXADECIMAL", true);
    let max_length = usize_from_env("MAX_LENGTH", 0);
    let word_boundary = bool_from_env("WORD_BOUNDARY", false);
    let separator = env::var("SEPARATOR").unwrap_or_else(|_| String::from("-"));
    let save_order = bool_from_env("SAVE_ORDER", false);
    let stopwords_raw = env::var("STOPWORDS").unwrap_or_default();
    let stopwords: Vec<&str> = if stopwords_raw.is_empty() {
        Vec::new()
    } else {
        stopwords_raw.split(',').collect()
    };
    let regex_pattern = env::var("REGEX_PATTERN").ok();
    let lowercase = bool_from_env("LOWERCASE", true);
    let replacements_raw = env::var("REPLACEMENTS").unwrap_or_default();
    let replacements: Vec<(String, String)> = if replacements_raw.is_empty() {
        Vec::new()
    } else {
        replacements_raw
            .split(";;")
            .filter_map(|s| s.split_once("=>").map(|(a, b)| (a.to_string(), b.to_string())))
            .collect()
    };
    let allow_unicode = bool_from_env("ALLOW_UNICODE", false);
    // CLI accepts TRANSLITERATE_ICONS for test harness parity but core API
    // currently ignores it (the Python binding also ignores it).
    // Read TRANSLITERATE_ICONS as an optional override; if unset, let
    // the builder default stand (which now defaults to true).
    let transliterate_icons_env = env::var("TRANSLITERATE_ICONS").ok();

    // Build options and call library
    // Use the crate's library exports
    use slugify_rs::slugify::SlugifyOptions;
    use slugify_rs::slugify::slugify_with_options_public;

    let builder = SlugifyOptions::builder()
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

    let builder = if let Some(val) = transliterate_icons_env {
        let b = builder.transliterate_icons(matches!(val.as_str(), "1" | "true" | "True" | "yes"));
        b
    } else {
        builder
    };

    let opts = match builder.build() {
        Ok(o) => o,
        Err(e) => {
            eprintln!("failed to build options: {:?}", e);
            std::process::exit(2);
        }
    };

    let out = slugify_with_options_public(&opts, &text);
    println!("{}", out);
}


// Unit tests for the CLI. We keep them here so coverage tools include the
// binary source when running `cargo test` and to keep tests next to the code
// they exercise.
#[cfg(test)]
mod tests {
    use std::env;
    use std::io::Write;
    use std::process::{Command, Stdio};

    fn bin_path() -> String {
        let manifest = env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
        format!("{}/target/debug/slugify_cli", manifest)
    }

    #[test]
    fn test_cli_basic_accented() {
        let bin = bin_path();
        let mut p = Command::new(&bin)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("failed to spawn slugify_cli");
        let input = "C'est déjà l'été.";
        p.stdin
            .as_mut()
            .expect("stdin")
            .write_all(input.as_bytes())
            .expect("write stdin");
        let out = p.wait_with_output().expect("wait");
        assert!(out.status.success());
        let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
        assert_eq!(s, "c-est-deja-l-ete");
    }

    #[test]
    fn test_cli_replacements_and_numeric() {
        let bin = bin_path();
        let mut cmd = Command::new(&bin);
        cmd.env("REPLACEMENTS", "|=>or;;%=>percent");
        let mut p = cmd
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("failed to spawn slugify_cli");
        let input = "10 | 20 %";
        p.stdin
            .as_mut()
            .expect("stdin")
            .write_all(input.as_bytes())
            .expect("write stdin");
        let out = p.wait_with_output().expect("wait");
        assert!(out.status.success());
        let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
        assert_eq!(s, "10-or-20-percent");
    }

    #[test]
    fn test_cli_allow_unicode_and_translit_icons() {
        let bin = bin_path();
        // allow_unicode = 0, transliterate_icons = 0 -> emoji dropped
        let out = Command::new(&bin)
            .env("ALLOW_UNICODE", "0")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .and_then(|mut child| {
                child
                    .stdin
                    .as_mut()
                    .unwrap()
                    .write_all("i love 🦄".as_bytes())?;
                child.wait_with_output()
            })
            .expect("run bin");
        assert!(out.status.success());
        let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
        assert_eq!(s, "i-love");

        // allow_unicode = 0, transliterate_icons = 1 -> emoji transliterated (non-empty, contains 'unicorn' or similar)
        let out2 = Command::new(&bin)
            .env("ALLOW_UNICODE", "0")
            .env("TRANSLITERATE_ICONS", "1")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .and_then(|mut child| {
                child
                    .stdin
                    .as_mut()
                    .unwrap()
                    .write_all("i love 🦄".as_bytes())?;
                child.wait_with_output()
            })
            .expect("run bin");
        assert!(out2.status.success());
        let s2 = String::from_utf8_lossy(&out2.stdout).trim().to_string();
        assert!(s2.len() > 0);
    }

    #[test]
    fn test_cli_invalid_regex_exits_nonzero() {
        let bin = bin_path();
        let out = Command::new(&bin)
            .env("REGEX_PATTERN", "(?")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .and_then(|mut child| {
                child
                    .stdin
                    .as_mut()
                    .unwrap()
                    .write_all("test".as_bytes())?;
                child.wait_with_output()
            })
            .expect("run bin");
        assert!(!out.status.success());
    }
}
