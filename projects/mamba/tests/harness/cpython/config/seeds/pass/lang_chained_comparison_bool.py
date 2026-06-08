# Operational AssertionPass seed for chained comparisons, boolean
# short-circuit, ternary, and membership / identity operators.
# Surface: a < b < c, a > b > c, mixed-direction chain (< then >),
# equality chain (==), 4-operand mixed chain; `or` short-circuits
# before evaluating the right side, same for `and`; `or` chain
# returns the first truthy value; `and` chain returns the last
# truthy value; `not` on each falsy singleton (True, False, 0, "",
# [], None); ternary (if/else) expression; nested ternary; `in`
# over list / str / dict; `not in`; `is None` and `is not None`.
_ledger: list[int] = []

# Chained comparison treats `a < b < c` as `a < b and b < c`
assert 1 < 2 < 3; _ledger.append(1)
assert 3 > 2 > 1; _ledger.append(1)
# Mixed-direction chain: `a < b > c` reads as `a < b and b > c`
assert 1 < 2 > 1; _ledger.append(1)
# Equality chain: 1 == 1 == 1 is True (all neighbors equal)
assert 1 == 1 == 1; _ledger.append(1)
# Longer chain mixing < and >
assert 1 < 2 < 4 > 3 > 0; _ledger.append(1)
# A failing middle short-circuits the rest of the chain to False
assert not (1 < 0 < 2); _ledger.append(1)

# `or` short-circuits — `True or X` never evaluates X
def _explode():
    raise RuntimeError("should never run")
    return 0  # unreachable

assert (True or _explode()) == True; _ledger.append(1)
# `and` short-circuits — `False and X` never evaluates X
assert (False and _explode()) == False; _ledger.append(1)

# `or` chain returns the first truthy value (or the last if all falsy)
assert (0 or "" or [] or "found") == "found"; _ledger.append(1)
# `and` chain returns the last value when all are truthy
assert (1 and 2 and 3) == 3; _ledger.append(1)

# `not` on each canonical falsy / truthy — use parentheses to make
# precedence explicit (not binds looser than `is` in plain Python)
assert (not True) == False; _ledger.append(1)
assert (not False) == True; _ledger.append(1)
assert (not 0) == True; _ledger.append(1)
assert (not "") == True; _ledger.append(1)
assert (not []) == True; _ledger.append(1)
assert (not None) == True; _ledger.append(1)

# Ternary if/else expression picks the value matching the predicate
assert ("yes" if 5 > 0 else "no") == "yes"; _ledger.append(1)
assert ("yes" if 5 < 0 else "no") == "no"; _ledger.append(1)

# Nested ternary dispatches on three buckets
def _cls(n):
    return "neg" if n < 0 else ("zero" if n == 0 else "pos")

assert _cls(-1) == "neg"; _ledger.append(1)
assert _cls(0) == "zero"; _ledger.append(1)
assert _cls(1) == "pos"; _ledger.append(1)

# `in` operator over list / str / dict
assert 2 in [1, 2, 3]; _ledger.append(1)
assert "a" in "abc"; _ledger.append(1)
assert "b" in {"a": 1, "b": 2}; _ledger.append(1)

# `not in` is the negation
assert 5 not in [1, 2, 3]; _ledger.append(1)

# `is None` / `is not None` identity checks against the interned None
assert None is None; _ledger.append(1)
assert 5 is not None; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: lang_chained_comparison_bool {sum(_ledger)} asserts")
