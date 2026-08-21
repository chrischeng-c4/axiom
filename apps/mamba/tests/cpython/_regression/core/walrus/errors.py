# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""core/<area>: language-level error paths (CPython 3.12 oracle).

Catch-all error coverage: mixed-type ops, missing keys, missing
attributes, out-of-range index — the dominant TypeError / KeyError /
IndexError surface for language-level operations.
"""


# Mixed-type op raises TypeError.
try:
    _ = 1 + "a"  # type: ignore[operator]
    print("mixed: no_raise")
except TypeError as e:
    print("mixed:", type(e).__name__, str(e)[:60])


# Out-of-range index raises IndexError.
try:
    [1, 2][5]
    print("oor: no_raise")
except IndexError as e:
    print("oor:", type(e).__name__, str(e)[:60])


# Missing dict key raises KeyError.
try:
    {}["missing"]
    print("missing_key: no_raise")
except KeyError as e:
    print("missing_key:", type(e).__name__, str(e)[:60])


# Hashing an unhashable raises TypeError.
try:
    hash([1, 2])  # type: ignore[arg-type]
    print("unhashable: no_raise")
except TypeError as e:
    print("unhashable:", type(e).__name__, str(e)[:60])
