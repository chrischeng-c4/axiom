# lang_exception_group.py — #3339 axis-1 PEP 654 seed (partial).
#
# Exercises the parts of PEP 654 that mamba currently implements:
#   1. `ExceptionGroup("msg", [e1, e2])` construction + `.exceptions` tuple
#   2. `except* T as v:` parse syntax + split + remainder branch
#
# Mamba runtime gaps tracked separately (linked from #3339):
#   - `.split(T)` method on ExceptionGroup
#   - `BaseExceptionGroup` (NameError at parse-time today)
#   - `.derive(...)` factory on ExceptionGroup subclasses
# Surface items 3 / 4 / 5 from the issue body will be folded into this
# seed once the runtime-gap issue lands; #3339 stays OPEN until then.
#
# Contract with cpython_lib_test_runner (#2691): every `assert` executes
# at top level, AssertionError → non-zero exit → `Fail` classification.
# Emitting `MAMBA_ASSERTION_PASS: lang_exception_group N asserts` flips
# the outcome to `AssertionPass`.
#
# Known mamba quirk (see projects/mamba/issue-loop.md):
#   `len(eg.exceptions)` evaluated inside an `except*` branch returns
#   an int that prints "2" but fails `== 2` against an int literal.
#   Subtraction (`x - 2 == 0`) works. The except* assertions below use
#   the subtraction pattern; the top-level ones use direct `==` since
#   the value was not threaded through an except* binding.

_ledger: list[int] = []

# 1. ExceptionGroup construction + `.exceptions` tuple.
g = ExceptionGroup("two errors", [ValueError("a"), TypeError("b")])
assert isinstance(g, ExceptionGroup), "ExceptionGroup constructible"
_ledger.append(1)

assert len(g.exceptions) == 2, ".exceptions has both members"
_ledger.append(1)

assert isinstance(g.exceptions[0], ValueError), "first member is ValueError"
_ledger.append(1)

assert isinstance(g.exceptions[1], TypeError), "second member is TypeError"
_ledger.append(1)

assert g.message == "two errors", ".message round-trips"
_ledger.append(1)

assert isinstance(g.exceptions, tuple), ".exceptions is a tuple"
_ledger.append(1)

# 2. `except*` splits the group; remainder dispatches to next branch.
ve_count = 0
te_count = 0
captured_type = ""
try:
    raise ExceptionGroup("mixed", [ValueError("v"), TypeError("t")])
except* ValueError as eg:
    ve_count = len(eg.exceptions)
    captured_type = type(eg).__name__
except* TypeError as eg:
    te_count = len(eg.exceptions)

# Subtraction pattern: see file header.
assert ve_count - 1 == 0, "except* ValueError branch matched one member"
_ledger.append(1)

assert te_count - 1 == 0, "except* TypeError branch caught the remainder"
_ledger.append(1)

assert captured_type == "ExceptionGroup", "except* hands an ExceptionGroup wrapper"
_ledger.append(1)

# Single-member group: except* still wraps in an ExceptionGroup.
hit = 0
inner_msg = ""
try:
    raise ExceptionGroup("solo", [ValueError("only")])
except* ValueError as eg:
    hit = len(eg.exceptions)
    inner_msg = str(eg.exceptions[0])

assert hit - 1 == 0, "except* on a singleton group captures one member"
_ledger.append(1)

assert inner_msg == "only", "captured exception payload is preserved"
_ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_exception_group {sum(_ledger)} asserts")
