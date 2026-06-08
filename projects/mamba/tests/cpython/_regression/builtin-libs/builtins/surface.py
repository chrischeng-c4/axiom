# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""builtin-libs/builtins: surface probes (CPython 3.12 oracle)."""

# Probes for documented builtin attributes / methods. Each `assert`
# verifies that the API surface is in place.

import builtins

# Core builtin functions exist.
for name in ("print", "len", "range", "open", "input", "int", "float",
             "str", "bytes", "bytearray", "memoryview", "complex",
             "list", "tuple", "dict", "set", "frozenset", "slice",
             "type", "isinstance", "issubclass", "callable",
             "iter", "next", "abs", "min", "max", "sum",
             "any", "all", "sorted", "reversed", "map", "filter", "zip",
             "enumerate", "object", "super", "property",
             "hash", "id", "repr", "ascii", "format",
             "bin", "oct", "hex", "chr", "ord",
             "divmod", "pow", "round",
             "eval", "exec", "compile", "__import__",
             "getattr", "setattr", "delattr", "hasattr",
             "dir", "vars", "globals", "locals",
             "staticmethod", "classmethod"):
    assert hasattr(builtins, name), name

# A removed Python-2 builtin must not reappear.
assert not hasattr(builtins, "cmp")

print("surface OK")
