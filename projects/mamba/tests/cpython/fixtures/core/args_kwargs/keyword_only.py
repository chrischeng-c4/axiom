# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# Keyword-only arguments — `def f(a, b, *, c, d=0)` and `def f(*, name, **extra)`
# Covers PEP 3102 (Python 3+) keyword-only-after-star syntax.
# The `def f(*args, kw)` form is a known compiler gap and omitted here.

# Basic keyword-only with defaults
def g(a, b, *, c=10, d=20):
    return (a, b, c, d)

print(g(1, 2))
print(g(1, 2, c=100))
print(g(1, 2, c=100, d=200))
print(g(1, 2, d=200))

# Keyword-only without defaults (must be supplied)
def h(a, *, x):
    return (a, x)

print(h(1, x=99))
print(h(a=1, x=99))
print(h(1, x="hello"))

# Keyword-only + **kwargs
def j(*, name, **extra):
    return (name, sorted(extra.items()))

print(j(name="A"))
print(j(name="B", x=1, y=2))

# Multiple kw-only, mixed with/without defaults
def k(a, *, b, c=10, d):
    return (a, b, c, d)

print(k(1, b=2, d=4))
print(k(1, b=2, c=30, d=4))
print(k(a=1, b=2, d=4))
