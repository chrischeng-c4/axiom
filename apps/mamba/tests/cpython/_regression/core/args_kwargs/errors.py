# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""core/<area>: function-machinery error paths (CPython 3.12 oracle)."""


def f(a, b, *, c):
    return (a, b, c)


# Missing required kw-only raises TypeError.
try:
    f(1, 2)  # type: ignore[call-arg]
    print("missing_kw: no_raise")
except TypeError as e:
    print("missing_kw:", type(e).__name__, str(e)[:60])


# Too many positional args raises TypeError.
def g(a, b):
    return a + b


try:
    g(1, 2, 3)  # type: ignore[call-arg]
    print("too_many: no_raise")
except TypeError as e:
    print("too_many:", type(e).__name__, str(e)[:60])


# Duplicate keyword arg raises TypeError.
try:
    g(1, a=2)  # type: ignore[call-arg]
    print("dup_kw: no_raise")
except TypeError as e:
    print("dup_kw:", type(e).__name__, str(e)[:60])
