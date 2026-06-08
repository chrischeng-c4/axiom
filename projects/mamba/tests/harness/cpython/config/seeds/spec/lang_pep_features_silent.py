# Operational AssertionPass divergence-spec fixture for the silent
# value-contract divergences of `f"{x = }"` debug-repr expansion (the
# documented "PEP 501 debug-equals f-string renders 'name = value'" —
# mamba renders just 'value'), function `__annotations__` (the
# documented "PEP 526 annotation dict exposed on function objects" —
# mamba returns None), `int | str` union type expression (the
# documented "PEP 604 returns a types.UnionType" — mamba returns
# None), `isinstance(1, int | str)` (the documented "PEP 604 union
# accepted by isinstance" — mamba returns False because union expr
# evaluates to None), `eval("'{:d}'.format('x')")` (the documented
# "format-spec mismatch raises ValueError" — mamba returns None
# silently), `eval("f'{1.5:d}'")` (the documented "f-string format
# 'd' on float raises ValueError" — mamba returns None silently),
# `eval("f'{1:s}'")` (the documented "f-string format 's' on int
# raises ValueError" — mamba returns None silently), `eval("'{:.2f}'.
# format('x')")` (the documented "format-spec '.2f' on str raises
# ValueError" — mamba returns None silently), `eval("'{:b}'.format(1.
# 5)")` (the documented "format-spec 'b' on float raises ValueError"
# — mamba returns None silently), and `eval("'{:x}'.format('a')")`
# (the documented "format-spec 'x' on str raises ValueError" — mamba
# returns None silently).
# Ten-pack pinned to atomic 322.
#
# Behavioral edges that CONFORM on mamba (list/dict/set/generator
# comprehensions with filter and nesting. F-string basic interpola-
# tion and format spec for matching types. Walrus := in if/compre-
# hension/while. str.format basic with positional/keyword/alignment.
# match/case literal/sequence/mapping/OR/capture patterns. Exception
# raise/except/multi-except/as/Exception-base. Context manager with/
# as. Generators yield/yield from. Function annotations CALLED but
# not inspected. Chained comparisons.) are covered in the matching
# pass fixture `test_lang_comprehensions_match_walrus_value_ops`.


_ledger: list[int] = []

# 1) f"{x = }" PEP 501 debug-equals f-string renders "x = 10"
#    (mamba: renders just "10" — no "x = " prefix)
def _dbg():
    x = 10
    return f"{x = }"
assert _dbg() == "x = 10"; _ledger.append(1)

# 2) function __annotations__ is a dict mapping param/return to types
#    (mamba: returns None)
def _ann(x: int, y: int = 0) -> int:
    return x + y
assert _ann.__annotations__ is not None; _ledger.append(1)

# 3) int | str returns a types.UnionType
#    (mamba: returns None)
assert (int | str) is not None; _ledger.append(1)

# 4) isinstance(1, int | str) returns True (PEP 604)
#    (mamba: returns False because int | str evaluates to None)
assert isinstance(1, int | str) == True; _ledger.append(1)

# 5) eval("'{:d}'.format('x')") raises ValueError
#    (mamba: returns None silently)
try:
    eval("'{:d}'.format('x')")
    raise AssertionError("expected ValueError")
except ValueError:
    _ledger.append(1)

# 6) eval("f'{1.5:d}'") raises ValueError
#    (mamba: returns None silently)
try:
    eval("f'{1.5:d}'")
    raise AssertionError("expected ValueError")
except ValueError:
    _ledger.append(1)

# 7) eval("f'{1:s}'") raises ValueError
#    (mamba: returns None silently)
try:
    eval("f'{1:s}'")
    raise AssertionError("expected ValueError")
except ValueError:
    _ledger.append(1)

# 8) eval("'{:.2f}'.format('x')") raises ValueError
#    (mamba: returns None silently)
try:
    eval("'{:.2f}'.format('x')")
    raise AssertionError("expected ValueError")
except ValueError:
    _ledger.append(1)

# 9) eval("'{:b}'.format(1.5)") raises ValueError
#    (mamba: returns None silently)
try:
    eval("'{:b}'.format(1.5)")
    raise AssertionError("expected ValueError")
except ValueError:
    _ledger.append(1)

# 10) eval("'{:x}'.format('a')") raises ValueError
#     (mamba: returns None silently)
try:
    eval("'{:x}'.format('a')")
    raise AssertionError("expected ValueError")
except ValueError:
    _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_pep_features_silent {sum(_ledger)} asserts")
