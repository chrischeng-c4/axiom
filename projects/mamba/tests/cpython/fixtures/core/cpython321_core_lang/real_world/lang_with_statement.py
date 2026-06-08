# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_with_statement"
# subject = "cpython321.lang_with_statement"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/lang_with_statement.py"
# status = "filled"
# ///
"""cpython321.lang_with_statement: execute CPython 3.12 seed lang_with_statement"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the `with` context-manager
# surface. Surface: `with CM() as v:` calls `CM.__enter__()` before
# the body, binds its return value to `v`, executes the body, and
# always calls `CM.__exit__(exc_type, exc_value, tb)` afterward —
# whether the body completed normally or raised; an `__exit__`
# returning False propagates the exception (caught by the
# surrounding try/except); an `__exit__` returning True swallows
# the exception (the surrounding try/except sees nothing); nested
# `with`s enter outer-then-inner and exit inner-then-outer in LIFO
# order; the multi-CM `with a, b, c:` form enters left-to-right
# and exits right-to-left; the `as` binding sees the
# `__enter__`-returned value (which may be an int, str, or the
# manager instance); the body can carry multiple statements.
# Companion to lang_with (which covers the broader surface).
_ledger: list[int] = []
_evt: list[str] = []

class CM:
    def __init__(self, name: str) -> None:
        self.name = name
    def __enter__(self):
        _evt.append("enter:" + self.name)
        return self
    def __exit__(self, exc_type, exc_value, tb) -> bool:
        _evt.append("exit:" + self.name)
        return False

# Basic enter/body/exit ordering
with CM("a") as cm:
    _evt.append("body:" + cm.name)
assert _evt == ["enter:a", "body:a", "exit:a"]; _ledger.append(1)

# Exception propagates and __exit__ still runs
_evt.clear()
try:
    with CM("b"):
        _evt.append("body:b")
        raise ValueError("boom")
except ValueError:
    _evt.append("caught")
assert _evt == ["enter:b", "body:b", "exit:b", "caught"]; _ledger.append(1)

# Nested with — outer enters first, inner exits first (LIFO)
_evt.clear()
with CM("outer"):
    with CM("inner"):
        _evt.append("body")
assert _evt == ["enter:outer", "enter:inner", "body", "exit:inner", "exit:outer"]; _ledger.append(1)

# Multi-CM single statement — left-to-right enter, right-to-left exit
_evt.clear()
with CM("x"), CM("y"):
    _evt.append("body")
assert _evt == ["enter:x", "enter:y", "body", "exit:y", "exit:x"]; _ledger.append(1)

# Three-way multi-CM — same ordering generalizes
_evt.clear()
with CM("M1"), CM("M2"), CM("M3"):
    _evt.append("body")
assert _evt == ["enter:M1", "enter:M2", "enter:M3", "body", "exit:M3", "exit:M2", "exit:M1"]; _ledger.append(1)

# `__exit__` returning True swallows the exception
class Swallow:
    def __enter__(self):
        return self
    def __exit__(self, t, v, tb) -> bool:
        return True

caught = False
try:
    with Swallow():
        raise RuntimeError("rt")
except RuntimeError:
    caught = True
assert caught == False; _ledger.append(1)

# `__enter__` returning an int / str — `as` binding sees that value
class Ctx:
    def __init__(self, v) -> None:
        self.v = v
    def __enter__(self):
        return self.v
    def __exit__(self, exc_type, exc_value, tb) -> bool:
        return False

with Ctx(42) as iv:
    assert iv == 42; _ledger.append(1)

with Ctx("hello") as sv:
    assert sv == "hello"; _ledger.append(1)

# `__exit__` always runs on the normal path too
_evt.clear()
with CM("p"):
    _evt.append("middle")
assert _evt == ["enter:p", "middle", "exit:p"]; _ledger.append(1)

# `as` binding visible to outer scope after exit
v_out = None
with Ctx(99) as v_out:
    pass
assert v_out == 99; _ledger.append(1)

# Manager binding readable inside the body
with Ctx("inner") as inner_val:
    assert inner_val == "inner"; _ledger.append(1)

# Two-deep nested with — both managers bind via `as`
_evt.clear()
with CM("L1") as c1:
    with CM("L2") as c2:
        _evt.append("body:" + c1.name + ":" + c2.name)
assert _evt == ["enter:L1", "enter:L2", "body:L1:L2", "exit:L2", "exit:L1"]; _ledger.append(1)

# Multi-statement body executes in source order
_evt.clear()
with CM("P"):
    _evt.append("op1")
    _evt.append("op2")
    _evt.append("op3")
assert _evt == ["enter:P", "op1", "op2", "op3", "exit:P"]; _ledger.append(1)

# `__enter__`-returned instance has its attributes readable in body
with CM("Q") as q:
    assert q.name == "Q"; _ledger.append(1)

# Re-raise after `__exit__` (return False) — exception reaches caller
_evt.clear()
raised = False
try:
    with CM("R"):
        raise TypeError("t")
except TypeError:
    raised = True
assert raised == True; _ledger.append(1)
assert "exit:R" in _evt; _ledger.append(1)

# Exception propagation event trace
_evt.clear()
try:
    with CM("S"):
        _evt.append("body")
        raise ValueError("v")
except ValueError:
    _evt.append("caught")
assert _evt == ["enter:S", "body", "exit:S", "caught"]; _ledger.append(1)

# Single-statement body — explicit `as` not always needed
_evt.clear()
with CM("Z"):
    pass
assert _evt == ["enter:Z", "exit:Z"]; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_with_statement {sum(_ledger)} asserts")
