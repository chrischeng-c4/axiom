# Operational AssertionPass seed for the bare `assert` statement.
# Surface: `assert True` / `assert <bool-expr>` are no-ops; `assert
# False` (and `assert <falsy-bool-expr>`) raises AssertionError that
# `except AssertionError` catches; the optional message form
# `assert cond, "msg"` propagates "msg" into AssertionError's str();
# assert inside a function body raises when called with bad args;
# assert composes with `and`/`or` (compound), `is`/`is not` (identity),
# `in` membership against tuple and list, function-call conditions,
# and `isinstance(...)` predicates. The catch path uses `as e` to
# verify the message text. Companion to lang_truthiness (which covers
# the truthiness predicate itself) — this one is about the statement
# form, control flow, and message propagation.
_ledger: list[int] = []

# Bare truthy assertions are no-ops
assert True; _ledger.append(1)
assert 1 == 1; _ledger.append(1)
assert not False; _ledger.append(1)

# Message form — message ignored when condition holds
assert True, "should not see this"; _ledger.append(1)
assert 1 + 1 == 2, "math broken"; _ledger.append(1)

# AssertionError raised on False, caught by except
try:
    assert False, "intentional"
    _ledger.append(0)  # unreachable
except AssertionError as e:
    _ledger.append(1)

try:
    assert False
    _ledger.append(0)
except AssertionError:
    _ledger.append(1)

try:
    assert 1 == 2
    _ledger.append(0)
except AssertionError:
    _ledger.append(1)

# Message text propagates into AssertionError str()
try:
    assert False, "msg42"
    _ledger.append(0)
except AssertionError as e:
    assert str(e) == "msg42"; _ledger.append(1)

# Compound boolean assertions
x = 5
assert x > 0; _ledger.append(1)
assert x < 10 and x > 0; _ledger.append(1)
assert x == 5 or x == 6; _ledger.append(1)

# Assert with a function-call condition
def is_pos(n: int) -> bool:
    return n > 0

assert is_pos(3); _ledger.append(1)
try:
    assert is_pos(-1)
    _ledger.append(0)
except AssertionError:
    _ledger.append(1)

# Assert with isinstance() predicate
assert isinstance(42, int); _ledger.append(1)
assert isinstance("abc", str); _ledger.append(1)
assert isinstance([1, 2], list); _ledger.append(1)

# Multiple asserts inside a function — first failing one raises
def check(v: int) -> bool:
    assert v > 0
    assert v < 100
    return True

assert check(50) == True; _ledger.append(1)
try:
    check(-1)
    _ledger.append(0)
except AssertionError:
    _ledger.append(1)
try:
    check(200)
    _ledger.append(0)
except AssertionError:
    _ledger.append(1)

# Assert with is / is not
assert None is None; _ledger.append(1)
assert 1 is not None; _ledger.append(1)

# Assert with `in` membership — tuple and list
assert 2 in (1, 2, 3); _ledger.append(1)
assert "a" in ["a", "b", "c"]; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_assert_statement {sum(_ledger)} asserts")
