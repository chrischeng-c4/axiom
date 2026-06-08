# Operational AssertionPass seed for PEP 570 positional-only parameters.
# Surface: a `/` token in a function signature marks every parameter to
# its left as positional-only — callable only by position, never by
# keyword. `def f(a, b, /, c)` accepts (1, 2, 3) and (1, 2, c=3); the
# names `a` and `b` are not visible as keyword targets. Positional-only
# parameters compose freely with regular parameters, defaults, the
# `*`-separator that opens a keyword-only region, and `**kwargs`.
# A trailing `/` (with nothing after it) makes every parameter
# positional-only; a leading `*`-separator with no `*args` name forces
# every following parameter to be keyword-only. Defaults on the
# positional-only side fill in when the caller omits trailing
# positionals, and explicit `c=value` for keyword-region parameters
# still works through the slash boundary.
_ledger: list[int] = []

# Basic positional-only — left of `/` is pos-only, right is regular
def f(a, b, /, c):
    return (a, b, c)
assert f(1, 2, 3) == (1, 2, 3); _ledger.append(1)
assert f(1, 2, c=3) == (1, 2, 3); _ledger.append(1)

# Combined pos-only / regular / kw-only
def g(a, b, /, c, d, *, e, f):
    return (a, b, c, d, e, f)
assert g(1, 2, 3, 4, e=5, f=6) == (1, 2, 3, 4, 5, 6); _ledger.append(1)
assert g(1, 2, c=3, d=4, e=5, f=6) == (1, 2, 3, 4, 5, 6); _ledger.append(1)
assert g(1, 2, 3, d=4, e=5, f=6) == (1, 2, 3, 4, 5, 6); _ledger.append(1)

# Positional-only with defaults
def h(a, b=10, /, c=20):
    return (a, b, c)
assert h(1) == (1, 10, 20); _ledger.append(1)
assert h(1, 2) == (1, 2, 20); _ledger.append(1)
assert h(1, 2, 3) == (1, 2, 3); _ledger.append(1)
assert h(1, c=99) == (1, 10, 99); _ledger.append(1)

# Keyword-only with default
def k(a, *, b=5):
    return (a, b)
assert k(1) == (1, 5); _ledger.append(1)
assert k(1, b=10) == (1, 10); _ledger.append(1)

# **kwargs-only collector
def kwargs_only(**kw):
    return kw
assert kwargs_only() == {}; _ledger.append(1)
assert kwargs_only(a=1) == {"a": 1}; _ledger.append(1)

# *-separator keyword-only region
def star_sep(a, *, b):
    return (a, b)
assert star_sep(1, b=2) == (1, 2); _ledger.append(1)

# Trailing-`/` form makes every parameter positional-only
def slash_only(a, b, /):
    return (a, b)
assert slash_only(1, 2) == (1, 2); _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_pep570_positional_only {sum(_ledger)} asserts")
