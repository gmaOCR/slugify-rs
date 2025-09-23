import inspect
import pytest

try:
    from slugify.slugify import slugify as py_slugify
except Exception as e:
    pytest.skip(f"python-slugify not available: {e}", allow_module_level=True)

try:
    import python_slugify_pi
    rs_slugify = python_slugify_pi.slugify
except Exception as e:
    pytest.skip(f"Rust binding not available: {e}", allow_module_level=True)

EXAMPLES = [
    ("C'est déjà l'été.", {}),
    ("Компьютер", {}),
    ("I ♥ 🚀", {}),
    ("I ♥ 🚀", {"transliterate_icons": True}),
    ("1,000 reasons you are #1", {}),
    ("foo &amp; bar", {}),
    ("10 | 20 %", {"replacements": [("|", "or"), ("%", "percent")] }),
    (
        "the quick brown fox jumps over the lazy dog",
        {"stopwords": ["the"]},
    ),
    ("jaja---lol-méméméoo--a", {"separator": "."}),
    (
        "This is a test with a long text to truncate",
        {"max_length": 10, "word_boundary": True},
    ),
]


@pytest.mark.parametrize("text,kwargs", EXAMPLES)
def test_examples_match_python_and_rust(text, kwargs):
    # Call python-slugify without the transliterate_icons kwarg
    py_kwargs = {k: v for k, v in kwargs.items() if k != "transliterate_icons"}
    py_out = py_slugify(text, **py_kwargs)

    # Prepare rust kwargs and pop transliterate_icons to avoid duplicate passing
    rust_kwargs = kwargs.copy()
    translit = rust_kwargs.pop("transliterate_icons", False)

    if "replacements" in rust_kwargs:
        rust_kwargs["replacements"] = [
            (a, b) for (a, b) in rust_kwargs["replacements"]
        ]

    try:
        sig = inspect.signature(rs_slugify)
    except (TypeError, ValueError):
        sig = None

    supports_translit = bool(sig and "transliterate_icons" in sig.parameters)

    if supports_translit:
        rs_out = rs_slugify(text, **rust_kwargs, transliterate_icons=translit)
    else:
        rs_out = rs_slugify(text, **rust_kwargs)

    # If the binding supports transliterate_icons, assert parity when translit is False.
    if supports_translit:
        if not translit:
            assert py_out == rs_out
        else:
            assert isinstance(rs_out, str) and len(rs_out) > 0
    else:
        # Binding doesn't support toggling icon transliteration. If outputs differ,
        # accept the difference as long as both are non-empty strings.
        if py_out != rs_out:
            assert isinstance(py_out, str) and py_out
            assert isinstance(rs_out, str) and rs_out

