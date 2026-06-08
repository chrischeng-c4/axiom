# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""core/args_kwargs: argument-binding surface probes (CPython 3.12 oracle)."""


def collect(*args, **kwargs):
    return args, kwargs


# *args binds to a tuple; **kwargs binds to a dict.
a, kw = collect(1, 2, x=3)
assert type(a) is tuple, type(a)
assert type(kw) is dict, type(kw)

# Empty call yields an empty tuple and empty dict (never None).
a0, kw0 = collect()
assert a0 == ()
assert kw0 == {}


def only_pos(a, b, /):
    return a + b


def only_kw(*, a, b):
    return a + b


# Both special parameter forms are accepted by the compiler and callable.
assert only_pos(2, 3) == 5
assert only_kw(a=2, b=3) == 5

# Call-site unpacking operators exist for both sequences and mappings.
assert only_pos(*[2, 3]) == 5
assert only_kw(**{"a": 2, "b": 3}) == 5

print("surface OK")
