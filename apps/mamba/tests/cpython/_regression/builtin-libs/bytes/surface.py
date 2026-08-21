# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""builtin-libs/bytes: surface probes (CPython 3.12 oracle)."""

# Probes for documented builtin attributes / methods. Each `assert`
# verifies that the API surface is in place.

# Core bytes methods.
for name in ("decode", "split", "rsplit", "strip", "lstrip", "rstrip",
             "startswith", "endswith", "replace", "join", "find",
             "rfind", "index", "rindex", "count", "lower", "upper",
             "title", "capitalize", "swapcase", "hex", "fromhex",
             "partition", "rpartition", "center", "ljust", "rjust",
             "translate", "maketrans", "expandtabs", "splitlines",
             "isalpha", "isdigit", "isspace", "removeprefix",
             "removesuffix", "zfill"):
    assert hasattr(b"", name), name

# bytearray adds the mutating-sequence methods on top of the bytes surface.
for name in ("append", "extend", "insert", "remove", "pop", "reverse",
             "clear", "copy"):
    assert hasattr(bytearray(), name), name

# bytes is immutable; it must NOT expose the mutators above.
for name in ("append", "extend", "insert", "remove", "clear"):
    assert not hasattr(b"", name), name

print("surface OK")
