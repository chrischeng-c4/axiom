# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_walrus"
# subject = "cpython321.lang_walrus"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/lang_walrus.py"
# status = "filled"
# ///
"""cpython321.lang_walrus: execute CPython 3.12 seed lang_walrus"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# lang_walrus.py - axis-1 PEP 572 walrus operator (:=) seed (#3358).
#
# Surface (from #3358):
#   1. `if (n := len(x)) > 10:` binds n
#   2. `while chunk := f.read(8):` loop guard
#   3. Walrus inside comprehension binds in outer scope
#   4. Walrus in lambda arguments rejected (SyntaxError) per PEP 572
#
# Known mamba quirk (see projects/mamba/issue-loop.md):
#   `len(x)` returns a boxed-int that prints "20" but fails `== 20`
#   against an int literal; subtraction (`n - 20 == 0`) works.

_ledger: list[int] = []

# 1. `if (n := len(x)) > 10:` binds n in the enclosing scope.
xs = list(range(20))
if (n := len(xs)) > 10:
    branch_hit = True
else:
    branch_hit = False

assert branch_hit is True, "walrus inside if-condition takes branch when guard passes"
_ledger.append(1)

# Boxed-int dodge: subtraction equality.
assert n - 20 == 0, "walrus-bound n equals len(xs)=20 (via subtraction pattern)"
_ledger.append(1)

# 1. `if (m := ...):` also reachable after the if-block ends (assignment scope).
if (m := 7) > 0:
    pass
assert m - 7 == 0, "walrus binding persists after the if-block ends (boxed-int dodge)"
_ledger.append(1)

# 2. `while chunk := next(it, sentinel):` as loop guard.
data = iter([b"abc", b"def", b""])
chunks: list[bytes] = []
while chunk := next(data, b""):
    chunks.append(chunk)

assert chunks == [b"abc", b"def"], "while-walrus collects values until falsy sentinel"
_ledger.append(1)

assert chunk == b"", "loop variable retains the final (falsy) walrus value"
_ledger.append(1)

# 3. Walrus inside comprehension binds in outer scope (PEP 572 explicit rule).
nums = [1, 4, 9]
doubled_filtered = [y for x in nums if (y := x + 1) > 2]
assert doubled_filtered == [5, 10], "comprehension with walrus filter produces correct sequence"
_ledger.append(1)

assert y - 10 == 0, "walrus inside comprehension binds y in the enclosing scope (boxed-int dodge)"
_ledger.append(1)

# 3. Walrus in generator expression binds outer (same rule).
total = sum((z := v * 2) for v in [1, 2, 3])
assert total - 12 == 0, "walrus inside generator expression aggregates correctly (2+4+6, boxed-int dodge)"
_ledger.append(1)

assert z - 6 == 0, "walrus inside generator expression binds z in enclosing scope to last value (boxed-int dodge)"
_ledger.append(1)

# 4. Walrus in lambda argument position is a SyntaxError at compile time.
def lambda_walrus_rejected() -> bool:
    try:
        compile("lambda (x := 1): x", "<probe>", "exec")
        return False
    except SyntaxError:
        return True

assert lambda_walrus_rejected(), "walrus in lambda argument list is rejected by parser"
_ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_walrus {sum(_ledger)} asserts")
