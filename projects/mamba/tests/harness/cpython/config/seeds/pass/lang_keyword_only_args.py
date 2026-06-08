# Operational AssertionPass seed for the keyword-only argument surface
# (PEP 3102 — bare `*` separator and `*args` boundary). Surface: a
# parameter that appears after a bare `*` or after `*args` is keyword-
# only and must be passed by name; the default-form `*, b: int = 100`
# makes the kw-only argument optional; a required kw-only parameter
# raises TypeError when omitted at the call site; `*args` collects
# extra positional args into a tuple and accepts the call-site
# `f(*list)` spread; `**kwargs` collects unbound keyword args into a
# dict; a fully named call (`q(a=1, b=2)`, `q(b=2, a=1)`) is order-
# free; a mixed call `q(1, b=2)` binds positional + named. Companion
# to lang_function_args (which covers positional/default) and
# lang_function_kwargs_defaults (which covers defaults).
_ledger: list[int] = []

# Bare-* separator — `b` is keyword-only
def f(a: int, *, b: int) -> int:
    return a + b

assert f(1, b=2) == 3; _ledger.append(1)
assert f(10, b=20) == 30; _ledger.append(1)

# Keyword-only with default — optional, override by name
def g(a: int, *, b: int = 100) -> int:
    return a + b

assert g(1) == 101; _ledger.append(1)
assert g(1, b=2) == 3; _ledger.append(1)
assert g(5, b=5) == 10; _ledger.append(1)

# Bare-* separator — multiple keyword-only parameters
def make_str(*, x: int, y: int) -> str:
    return str(x) + "," + str(y)

assert make_str(x=1, y=2) == "1,2"; _ledger.append(1)
assert make_str(y=10, x=20) == "20,10"; _ledger.append(1)

# Required keyword-only — TypeError when omitted, returns value when supplied
def m(*, key: str) -> str:
    return key

try:
    m()  # type: ignore[call-arg]
    _ledger.append(0)
except TypeError:
    _ledger.append(1)
assert m(key="hello") == "hello"; _ledger.append(1)

# **kwargs — collects unbound keyword args into a dict
def n(**kwargs) -> int:
    return len(kwargs)

assert n() == 0; _ledger.append(1)
assert n(a=1) == 1; _ledger.append(1)
assert n(a=1, b=2, c=3) == 3; _ledger.append(1)

# Order-free named call — same result for any permutation
def q(a: int, b: int) -> str:
    return str(a) + "-" + str(b)

assert q(1, 2) == "1-2"; _ledger.append(1)
assert q(a=1, b=2) == "1-2"; _ledger.append(1)
assert q(b=2, a=1) == "1-2"; _ledger.append(1)
assert q(1, b=2) == "1-2"; _ledger.append(1)

# *args — collects positional, accepts call-site spread
def r(*args: int) -> int:
    return sum(args)

assert r() == 0; _ledger.append(1)
assert r(1) == 1; _ledger.append(1)
assert r(1, 2, 3) == 6; _ledger.append(1)
assert r(*[1, 2, 3]) == 6; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_keyword_only_args {sum(_ledger)} asserts")
