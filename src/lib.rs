pub mod slugify;

pub use slugify::{DEFAULT_SEPARATOR, slugify, smart_truncate};

// Include Python bindings when the `python` feature is enabled so that
// the PyO3 module (`python_slugify_pi`) is compiled and exports
// the `PyInit_python_slugify_pi` symbol required by Python imports.
#[cfg(feature = "python")]
pub mod lib_py;
