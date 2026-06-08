# Operational AssertionPass seed for the function-argument surface:
# default args, *args collector, **kwargs collector, keyword-only
# parameters (PEP 3102), positional-only parameters (PEP 570), and
# mixed forms. The `*args` collector materialises as a list on mamba
# rather than a tuple — assertions exercise len()/indexing semantics
# that both shapes satisfy.
_ledger: list[int] = []

# Default-valued parameters
def f(a, b=10, c=20):
    return a + b + c

r1 = f(1)
r2 = f(1, 2)
r3 = f(1, 2, 3)
r4 = f(1, c=99)
assert r1 == 31; _ledger.append(1)
assert r2 == 23; _ledger.append(1)
assert r3 == 6; _ledger.append(1)
assert r4 == 110; _ledger.append(1)

# Keyword-only parameters (PEP 3102): the bare `*` requires b/c by name
def g(a, *, b, c=10):
    return a + b + c

g1 = g(1, b=2)
g2 = g(1, b=2, c=99)
assert g1 == 13; _ledger.append(1)
assert g2 == 102; _ledger.append(1)

# *args collector
def h(*args):
    return sum(args)

assert h(1, 2, 3) == 6; _ledger.append(1)
assert h() == 0; _ledger.append(1)
assert h(42) == 42; _ledger.append(1)

# **kwargs collector
def k(**kwargs):
    return kwargs

result = k(a=1, b=2)
assert result["a"] == 1; _ledger.append(1)
assert result["b"] == 2; _ledger.append(1)
assert len(result) == 2; _ledger.append(1)

# Positional-only parameters (PEP 570): a, b before `/` cannot be
# passed by keyword
def p(a, b, /, c):
    return a + b + c

assert p(1, 2, 3) == 6; _ledger.append(1)
assert p(1, 2, c=3) == 6; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: lang_function_args {sum(_ledger)} asserts")
