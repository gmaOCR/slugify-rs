use pyo3::prelude::*;
use pyo3::wrap_pyfunction;

use crate::slugify as slugify_mod;

#[pyfunction]
#[allow(clippy::too_many_arguments)]
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
    _transliterate_icons=false
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
    _transliterate_icons: bool,
) -> PyResult<String> {
    let sep = separator.unwrap_or(slugify_mod::DEFAULT_SEPARATOR);

    let stop_vec: Vec<String> = stopwords.unwrap_or_default();
    let stop_refs: Vec<&str> = stop_vec.iter().map(|s| s.as_str()).collect();

    let repl_vec: Vec<(String, String)> = replacements.unwrap_or_default();
    let repl_refs: Vec<(&str, &str)> = repl_vec
        .iter()
        .map(|(a, b)| (a.as_str(), b.as_str()))
        .collect();

    // The core Rust API does not currently support an explicit
    // `transliterate_icons` flag. We pass the supported parameters
    // to `slugify` and ignore `transliterate_icons` here to retain
    // compatibility with the Python tests which may pass it.
    // Build SlugifyOptions using the ergonomic builder API and call the
    // options-based public function.
    let builder = slugify_mod::SlugifyOptions::builder()
        .entities(entities)
        .decimal(decimal)
        .hexadecimal(hexadecimal)
        .max_length(max_length)
        .word_boundary(word_boundary)
        .separator(sep)
        .save_order(save_order)
        .stopwords(stop_refs)
        .regex_pattern(regex_pattern.as_deref().map(|s| s.to_string()))
        .lowercase(lowercase)
        .replacements(repl_refs)
        .allow_unicode(allow_unicode);

    let opts = builder
        .build()
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("invalid args: {:?}", e)))?;

    Ok(slugify_mod::slugify_with_options_public(&opts, text))
}

#[pymodule(name = "slugify_rs")]
fn python_slugify_pi(_py: Python, m: &Bound<PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(slugify, m)?)?;
    Ok(())
}
