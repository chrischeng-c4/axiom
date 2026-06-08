# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_call_unpacking"
# subject = "cpython321.lang_call_unpacking"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/lang_call_unpacking.py"
# status = "filled"
# ///
"""cpython321.lang_call_unpacking: execute CPython 3.12 seed lang_call_unpacking"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for argument-unpacking at the call
# site.
# Surface: `*args` unpacks a sequence into positional parameters of
# a function call; PEP 448 `**` spreads dicts into dict-literal
# positions; chained dict spread merges left-to-right.
# Note: `**kwargs` call-unpacking into keyword parameters and `*` in
# list/set/tuple literals are NOT exercised here — both have known
# breakages on the current mamba runtime.
_ledger: list[int] = []

# * unpacks a list into positional args
def add3(a, b, c):
    return a + b + c

args = [1, 2, 3]
s = add3(*args)
assert s - 6 == 0; _ledger.append(1)

# * unpacks a tuple just as well
tup = (10, 20, 30)
s2 = add3(*tup)
assert s2 - 60 == 0; _ledger.append(1)

# * unpacks the iterable returned by range
def collect3(a, b, c):
    return (a, b, c)

c = collect3(*range(3))
assert c == (0, 1, 2); _ledger.append(1)

# PEP 448 dict spread: merge two dicts inside a literal
d1 = {"a": 1}
d2 = {"b": 2}
merged = {**d1, **d2}
assert merged == {"a": 1, "b": 2}; _ledger.append(1)

# Three-way spread plus an inline kv pair — left-to-right precedence
d3 = {"c": 3}
m2 = {**d1, **d2, **d3, "d": 4}
assert m2 == {"a": 1, "b": 2, "c": 3, "d": 4}; _ledger.append(1)

# Later spread overrides earlier on key collision
overlap = {**{"x": 1}, **{"x": 99}}
assert overlap == {"x": 99}; _ledger.append(1)

# Mixed: literal kv pair + dict spread + literal kv pair
mixed = {"head": 0, **{"mid": 1}, "tail": 2}
assert mixed == {"head": 0, "mid": 1, "tail": 2}; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: lang_call_unpacking {sum(_ledger)} asserts")
