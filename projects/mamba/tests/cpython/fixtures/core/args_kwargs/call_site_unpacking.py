# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# Call-site unpacking (PEP 448): `*iterable` from any iterable,
# `**mapping`, and multiple unpackings merged in a single call.


def three(a, b, c):
    return [a, b, c]


def collect(*args, **kwargs):
    return list(args), sorted(kwargs.items())


# *args accepts any iterable, not just list/tuple.
print(three(*range(3)))
print(three(*"xyz"))
print(collect(*{7})[0])          # one-element set -> deterministic


def gen():
    yield 1
    yield 2


print(three(*gen(), 3))


# **mapping unpacks dict keys as keyword args.
print(three(**{"a": 1, "b": 2, "c": 3}))


# Multiple positional unpackings concatenate left to right.
print(collect(*[1, 2], *[3, 4])[0])


# Multiple mapping unpackings merge when keys are disjoint.
print(collect(**{"a": 1, "b": 2}, **{"c": 3, "d": 4})[1])


# Mix literal + unpack on both sides in one call.
print(collect(0, *[1, 2], k=9, **{"m": 8}))
