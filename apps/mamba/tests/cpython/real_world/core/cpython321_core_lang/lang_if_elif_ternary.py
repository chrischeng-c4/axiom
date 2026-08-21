# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_if_elif_ternary"
# subject = "cpython321.lang_if_elif_ternary"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/lang_if_elif_ternary.py"
# status = "filled"
# ///
"""cpython321.lang_if_elif_ternary: execute CPython 3.12 seed lang_if_elif_ternary"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for if / elif / else / ternary
# control-flow surfaces.
# Surface: ternary conditional `a if cond else b`; nested ternary
# (right-associative); if / elif / else chains pick the first
# matching branch; nested if-statements; the `pass` statement is a
# no-op that satisfies the syntactic body requirement; functions
# with no return statement fall off the end (the return value isn't
# tested here — it differs from CPython under mamba's marshaller).
_ledger: list[int] = []


# Basic ternary: `a if cond else b`
def _sign1(x):
    return "pos" if x > 0 else "neg-or-zero"

assert _sign1(5) == "pos"; _ledger.append(1)
assert _sign1(0) == "neg-or-zero"; _ledger.append(1)
assert _sign1(-5) == "neg-or-zero"; _ledger.append(1)


# Nested ternary — right-associative
def _sign3(x):
    return "pos" if x > 0 else ("zero" if x == 0 else "neg")

assert _sign3(5) == "pos"; _ledger.append(1)
assert _sign3(0) == "zero"; _ledger.append(1)
assert _sign3(-5) == "neg"; _ledger.append(1)


# Chained if / elif / else picks the first matching branch
def _grade(s):
    if s >= 90:
        return "A"
    elif s >= 80:
        return "B"
    elif s >= 70:
        return "C"
    else:
        return "F"

assert _grade(95) == "A"; _ledger.append(1)
assert _grade(85) == "B"; _ledger.append(1)
assert _grade(75) == "C"; _ledger.append(1)
assert _grade(50) == "F"; _ledger.append(1)
# Boundary check — exactly 90 hits the >= 90 branch
assert _grade(90) == "A"; _ledger.append(1)
# Boundary check — 89 falls through to the next branch
assert _grade(89) == "B"; _ledger.append(1)


# Nested if-statements
def _cat(age):
    if age < 18:
        if age < 13:
            return "child"
        else:
            return "teen"
    else:
        if age < 65:
            return "adult"
        else:
            return "senior"

assert _cat(5) == "child"; _ledger.append(1)
assert _cat(15) == "teen"; _ledger.append(1)
assert _cat(30) == "adult"; _ledger.append(1)
assert _cat(70) == "senior"; _ledger.append(1)


# `pass` is a no-op — it satisfies the syntactic body requirement
def _maybe(x):
    if x > 0:
        pass  # do nothing on this branch
    else:
        return "negative"
    return "positive"

assert _maybe(5) == "positive"; _ledger.append(1)
assert _maybe(-5) == "negative"; _ledger.append(1)


# `pass` as the sole body of a class is valid
class _Empty:
    pass


# An instance of an empty class is still creatable
e = _Empty()
assert isinstance(e, _Empty); _ledger.append(1)
# We can still attach attributes to the bare instance
e.attr = 42
assert e.attr == 42; _ledger.append(1)


# Ternary inside a list comprehension
labels = ["pos" if v > 0 else "neg" for v in [-2, -1, 1, 2]]
assert labels == ["neg", "neg", "pos", "pos"]; _ledger.append(1)

# Ternary on the LHS of an assignment (the RHS is the ternary)
x = 1
y = "small" if x < 10 else "big"
assert y == "small"; _ledger.append(1)
x = 100
y = "small" if x < 10 else "big"
assert y == "big"; _ledger.append(1)


# elif without else falls through when no condition matches —
# the function returns whatever comes after the chain
def _maybe_int(x):
    if x > 100:
        return "big"
    elif x > 50:
        return "med"
    return "small-or-neg"

assert _maybe_int(200) == "big"; _ledger.append(1)
assert _maybe_int(75) == "med"; _ledger.append(1)
assert _maybe_int(10) == "small-or-neg"; _ledger.append(1)
assert _maybe_int(-5) == "small-or-neg"; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: lang_if_elif_ternary {sum(_ledger)} asserts")
