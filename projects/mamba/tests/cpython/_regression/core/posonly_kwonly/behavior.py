# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""core/posonly_kwonly: language-area behavior asserts (CPython 3.12 oracle)."""

# Generic language behavior checks.
assert 1 + 1 == 2
assert type(()) is tuple
assert list(range(3)) == [0, 1, 2]

# Lambdas honor the `/` marker just like def.
add = lambda a, /, b: a + b
assert add(1, 2) == 3
assert add(1, b=2) == 3          # `b` is still pos-or-keyword

# Defaults may straddle the `/` marker: a, b are positional-only, c is
# pos-or-keyword, all three carry defaults.
def straddle(a, b=10, /, c=100):
    return a + b + c

assert straddle(1, 2, 3) == 6
assert straddle(1, 2, c=3) == 6  # only c may be named
assert straddle(1, 2) == 103
assert straddle(1, c=2) == 13    # b falls back to its default

# Generators accept positional-only params; binding happens at the
# call that creates the generator, not at the first next().
def gen(a=1, /, b=2):
    yield (a, b)

assert next(gen(5, 6)) == (5, 6)
assert next(gen(5, b=6)) == (5, 6)
assert next(gen()) == (1, 2)

# Closures: an inner function may declare its own `/` segment and still
# read enclosing-scope variables.
def outer(x, /, y):
    def inner(p, /, q):
        return x + y + p + q
    return inner

assert outer(1, 2)(3, 4) == 10

# __defaults__ is a live, mutable attribute on the function object; the
# positional-only tail picks up replacement defaults.
def mutable(a, b=2, /, c=3):
    return a + b + c

assert mutable.__defaults__ == (2, 3)
mutable.__defaults__ = (1, 2, 3)
assert mutable(1, 2, 3) == 6     # explicit args still win
assert mutable() == 6            # now every param has a default

# Positional-only call satisfied entirely by iterable unpacking.
def pair(a, b, /):
    return a + b

assert pair(*[1, 2]) == 3

print("behavior OK")
