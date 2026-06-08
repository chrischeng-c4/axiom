# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""builtin-libs/string_methods: surface probes (CPython 3.12 oracle)."""

# The documented str-method surface: every name below must exist as a
# callable attribute on the str type.
str_methods = [
    "upper", "lower", "title", "capitalize", "swapcase", "casefold",
    "strip", "lstrip", "rstrip", "split", "rsplit", "splitlines",
    "join", "partition", "rpartition", "replace", "translate",
    "maketrans", "find", "rfind", "index", "rindex", "count",
    "startswith", "endswith", "center", "ljust", "rjust", "zfill",
    "expandtabs", "format", "format_map", "encode", "removeprefix",
    "removesuffix", "isalpha", "isdigit", "isalnum", "isspace",
    "isupper", "islower", "istitle", "isascii", "isidentifier",
    "isnumeric", "isdecimal", "isprintable",
]
for name in str_methods:
    assert hasattr(str, name), name
    assert callable(getattr(str, name)), name

# The string module exposes its classic character-class constants and
# the Template / Formatter helpers.
import string
for const in ["ascii_letters", "digits", "punctuation", "whitespace",
              "hexdigits", "octdigits", "printable"]:
    assert isinstance(getattr(string, const), str), const
assert hasattr(string, "capwords")
assert hasattr(string, "Template")
assert hasattr(string, "Formatter")

print("surface OK")
