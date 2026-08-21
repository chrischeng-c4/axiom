# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# Positional-only parameters (PEP 570, Python 3.8+)
# `def f(a, b, /, c, d)` — a, b positional-only; c, d regular.

# Basic split: pos-only + regular
def f(a, b, /, c, d):
    return (a, b, c, d)

print(f(1, 2, 3, 4))
print(f(1, 2, c=3, d=4))
print(f(1, 2, 3, d=4))

# All positional-only
def g(a, b, /):
    return (a, b)

print(g(1, 2))

# Combined with defaults (pos-only can have defaults)
def h(a, b=10, /, c=20):
    return (a, b, c)

print(h(1))
print(h(1, 2))
print(h(1, 2, 3))
print(h(1, c=99))

# Positional-only + regular + kwargs
def k(a, b, /, c, **kw):
    return (a, b, c, sorted(kw.items()))

print(k(1, 2, 3))
print(k(1, 2, c=3))
print(k(1, 2, 3, x=10, y=20))
print(k(1, 2, c=3, x=99))
