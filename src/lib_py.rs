#![cfg(feature = "python")]
use pyo3::prelude::*;
use pyo3::wrap_pyfunction;

use crate::slugify as slugify_mod;

#[pyfunction]
#[pyo3(signature=(
    text,
    entities=true,
    decimal=false,
    hexadecimal=false,
    max_length=0,
    word_boundary=true,
    separator=None,
    save_order=false,
    stopwords=None,
    regex_pattern=None,
    lowercase=true,
    replacements=None,
    allow_unicode=false,
    transliterate_icons=false
))]
fn slugify(
    text: &str,
    entities: bool,
    decimal: bool,
    hexadecimal: bool,
    max_length: usize,
    word_boundary: bool,
    separator: Option<&str>,
    save_order: bool,
    stopwords: Option<Vec<String>>,
    regex_pattern: Option<String>,
    lowercase: bool,
    replacements: Option<Vec<(String, String)>>,
    allow_unicode: bool,
    transliterate_icons: bool,
) -> PyResult<String> {
    let sep = separator.unwrap_or(slugify_mod::DEFAULT_SEPARATOR);

    let stop_vec: Vec<String> = stopwords.unwrap_or_default();
    let stop_refs: Vec<&str> = stop_vec.iter().map(|s| s.as_str()).collect();

    let repl_vec: Vec<(String, String)> = replacements.unwrap_or_default();
    let repl_refs: Vec<(&str, &str)> = repl_vec.iter().map(|(a, b)| (a.as_str(), b.as_str())).collect();

    Ok(slugify_mod::slugify_with_icons(
        text,
        entities,
        decimal,
        hexadecimal,
        max_length,
        word_boundary,
        sep,
        save_order,
        &stop_refs,
        regex_pattern.as_deref(),
        lowercase,
        &repl_refs,
        allow_unicode,
        transliterate_icons,
    ))
}

#[pymodule]
fn python_slugify_pi(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(slugify, m)?)?;
    Ok(())
}
