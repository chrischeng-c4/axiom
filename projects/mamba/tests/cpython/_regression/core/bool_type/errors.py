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


# bool is final: it cannot be used as a base class.
try:
    class _C(bool):  # type: ignore[misc]
        pass
    print("subclass: no_raise")
except TypeError as e:
    print("subclass:", type(e).__name__, str(e)[:60])


# bool() accepts at most one positional argument.
try:
    bool(42, 42)  # type: ignore[call-arg]
    print("two_args: no_raise")
except TypeError as e:
    print("two_args:", type(e).__name__, str(e)[:60])


# bool() takes no keyword arguments.
try:
    bool(x=10)  # type: ignore[call-arg]
    print("keyword: no_raise")
except TypeError as e:
    print("keyword:", type(e).__name__, str(e)[:60])


# int.__new__ refuses to build a bool: only bool.__new__ is safe.
try:
    int.__new__(bool, 0)
    print("int_new_bool: no_raise")
except TypeError as e:
    print("int_new_bool:", type(e).__name__, str(e)[:60])
