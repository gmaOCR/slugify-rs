import python_slugify_pi


def test_import_and_slugify():
    assert hasattr(python_slugify_pi, "slugify")
    out = python_slugify_pi.slugify("Hello, Äpfel & Öl -- 123")
    assert isinstance(out, str)
    assert "hello" in out
