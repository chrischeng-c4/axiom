# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "test_lang_comprehensions_match_walrus_value_ops"
# subject = "cpython321.test_lang_comprehensions_match_walrus_value_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_lang_comprehensions_match_walrus_value_ops.py"
# status = "filled"
# ///
"""cpython321.test_lang_comprehensions_match_walrus_value_ops: execute CPython 3.12 seed test_lang_comprehensions_match_walrus_value_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Atomic 322 pass conformance — PEP-level language features:
# comprehensions (list/dict/set/generator, with filter, nested),
# f-strings (PEP 498) basic interpolation and format-spec, walrus :=
# (PEP 572), str.format basic, match/case (PEP 634) with literal/
# sequence/mapping/OR/capture patterns, exception handling (raise/
# except/multi/as), context managers (with), and generators (yield/
# yield from). All asserts match between CPython 3.12 and mamba.

_ledger: list[int] = []

# 1) list/dict/set/gen comprehensions
assert [x * 2 for x in range(3)] == [0, 2, 4]; _ledger.append(1)
assert [x for x in range(5) if x % 2] == [1, 3]; _ledger.append(1)
assert {x: x * 2 for x in range(3)} == {0: 0, 1: 2, 2: 4}; _ledger.append(1)
assert {x for x in [1, 2, 2, 3]} == {1, 2, 3}; _ledger.append(1)
assert list(x * 2 for x in range(3)) == [0, 2, 4]; _ledger.append(1)
assert [[i * j for j in range(2)] for i in range(2)] == [[0, 0], [0, 1]]; _ledger.append(1)
assert [(i, j) for i in range(2) for j in range(2)] == [(0, 0), (0, 1), (1, 0), (1, 1)]; _ledger.append(1)

# 2) f-strings basic
assert f"hello {1 + 1}" == "hello 2"; _ledger.append(1)
assert f"{1:.2f}" == "1.00"; _ledger.append(1)
assert f"{42:04d}" == "0042"; _ledger.append(1)
assert f"{[1, 2]!r}" == "[1, 2]"; _ledger.append(1)
assert f"{'hi'!a}" == "'hi'"; _ledger.append(1)
assert f"{42:>5}" == "   42"; _ledger.append(1)
assert f"{42:#x}" == "0x2a"; _ledger.append(1)
assert f"{3.14:.1f}" == "3.1"; _ledger.append(1)

# 3) walrus := (PEP 572)
if (n := 5) > 0:
    assert n == 5; _ledger.append(1)
data = [1, 2, 3, 4]
assert [y for y in data if (sq := y * y) > 4] == [3, 4]; _ledger.append(1)
total = 0
i = 0
while (i := i + 1) < 4:
    total += i
assert total == 6; _ledger.append(1)

# 4) str.format basic
assert "{0} {1}".format("a", "b") == "a b"; _ledger.append(1)
assert "{:.2f}".format(3.14159) == "3.14"; _ledger.append(1)
assert "{x}".format(x="hi") == "hi"; _ledger.append(1)
assert "{:>5}".format("a") == "    a"; _ledger.append(1)
assert "{:<5}".format("a") == "a    "; _ledger.append(1)
assert "{:^5}".format("a") == "  a  "; _ledger.append(1)

# 5) match/case (PEP 634)
def m_literal(v):
    match v:
        case 1:
            return "one"
        case 2:
            return "two"
        case _:
            return "other"
assert m_literal(1) == "one"; _ledger.append(1)
assert m_literal(2) == "two"; _ledger.append(1)
assert m_literal(99) == "other"; _ledger.append(1)

def m_seq(v):
    match v:
        case [1, 2, *rest]:
            return rest
        case _:
            return None
assert m_seq([1, 2, 3, 4]) == [3, 4]; _ledger.append(1)
assert m_seq([5, 6]) is None; _ledger.append(1)

def m_map(v):
    match v:
        case {"name": n, "age": a}:
            return (n, a)
        case _:
            return None
assert m_map({"name": "x", "age": 5}) == ("x", 5); _ledger.append(1)

def m_or(v):
    match v:
        case 1 | 2 | 3:
            return "small"
        case _:
            return "big"
assert m_or(2) == "small"; _ledger.append(1)
assert m_or(10) == "big"; _ledger.append(1)

# 6) exception handling
def raise_value():
    raise ValueError("v")

caught = None
try:
    raise_value()
except ValueError as e:
    caught = str(e)
assert caught == "v"; _ledger.append(1)

def raise_key():
    raise KeyError("k")

caught2 = None
try:
    raise_key()
except (KeyError, IndexError) as e:
    caught2 = type(e).__name__
assert caught2 == "KeyError"; _ledger.append(1)

caught3 = None
try:
    raise ValueError("v")
except Exception as e:
    caught3 = type(e).__name__
assert caught3 == "ValueError"; _ledger.append(1)

# 7) context manager
class CM:
    def __init__(self):
        self.entered = False
        self.exited = False
    def __enter__(self):
        self.entered = True
        return self
    def __exit__(self, exc_type, exc_val, exc_tb):
        self.exited = True
        return False

cm_obj = CM()
with cm_obj as v:
    assert v.entered == True; _ledger.append(1)
assert cm_obj.exited == True; _ledger.append(1)

# 8) generators
def gen():
    yield 1
    yield 2
    yield 3

assert list(gen()) == [1, 2, 3]; _ledger.append(1)

def gen2():
    yield from [1, 2, 3]

assert list(gen2()) == [1, 2, 3]; _ledger.append(1)

def gen_filter():
    for x in range(5):
        if x % 2 == 0:
            yield x

assert list(gen_filter()) == [0, 2, 4]; _ledger.append(1)

# 9) function call with annotations (callable behavior, not inspection)
def annotated(x: int, y: int = 0) -> int:
    return x + y

assert annotated(1, 2) == 3; _ledger.append(1)
assert annotated(5) == 5; _ledger.append(1)

# 10) chained comparison
assert 1 < 2 < 3; _ledger.append(1)
assert not (1 < 2 < 2); _ledger.append(1)
assert 1 == 1 == 1; _ledger.append(1)
assert 1 < 3 > 2; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_lang_comprehensions_match_walrus_value_ops {sum(_ledger)} asserts")
