# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""core/args_kwargs: argument-binding behavior asserts (CPython 3.12 oracle)."""


def f(a, b=2, *args, c, d=4, **kwargs):
    return a, b, args, c, d, kwargs


# Positional fills a then b; surplus positionals land in *args.
assert f(1, c=3) == (1, 2, (), 3, 4, {})
assert f(1, 9, 10, 11, c=3) == (1, 9, (10, 11), 3, 4, {})

# Keyword-only args bind by name; surplus keywords land in **kwargs.
assert f(1, c=3, d=40, e=5) == (1, 2, (), 3, 40, {"e": 5})

# A keyword can target a normally-positional parameter.
assert f(a=1, b=2, c=3) == (1, 2, (), 3, 4, {})


def g(a, b, c):
    return (a, b, c)


# Positional-unpack a sequence; keyword-unpack a mapping; mix both.
assert g(*[1, 2, 3]) == (1, 2, 3)
assert g(**{"a": 1, "b": 2, "c": 3}) == (1, 2, 3)
assert g(1, **{"b": 2, "c": 3}) == (1, 2, 3)

# *args preserves order; ** preserves the dict's insertion order.
def echo_kw(**kw):
    return list(kw)


assert echo_kw(z=1, a=2, m=3) == ["z", "a", "m"]

print("behavior OK")
