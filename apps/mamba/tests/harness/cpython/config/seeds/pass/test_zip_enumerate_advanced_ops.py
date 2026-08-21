# Operational AssertionPass seed for advanced zip/enumerate/reversed
# surfaces beyond test_iter_builtins_ops.
# Surface: enumerate(start=N), zip truncates to shortest argument, zip
# accepts >2 iterables, reversed on a string, PEP 618 zip(strict=True)
# raises ValueError on length mismatch (CPython 3.10+).
_ledger: list[int] = []

# enumerate's `start` shifts the index
items = ["a", "b", "c"]
shifted = list(enumerate(items, start=10))
assert shifted == [(10, "a"), (11, "b"), (12, "c")]; _ledger.append(1)

# zip truncates: extra elements in the longer iterable are dropped
short = list(zip([1, 2, 3, 4], ["a", "b"]))
assert short == [(1, "a"), (2, "b")]; _ledger.append(1)

# zip across 3 iterables yields 3-tuples
three = list(zip([1, 2], ["a", "b"], [True, False]))
assert three == [(1, "a", True), (2, "b", False)]; _ledger.append(1)

# reversed on a string returns an iterator over characters in reverse
rs = list(reversed("abc"))
assert rs == ["c", "b", "a"]; _ledger.append(1)

# PEP 618 strict=True — successful equal-length zip
strict_ok = list(zip([1, 2], ["a", "b"], strict=True))
assert strict_ok == [(1, "a"), (2, "b")]; _ledger.append(1)

# PEP 618 strict=True — mismatched lengths raise ValueError
raised = False
try:
    list(zip([1, 2], ["a", "b", "c"], strict=True))
except ValueError:
    raised = True
assert raised; _ledger.append(1)

# enumerate over a non-list (str) — still yields (index, ch) pairs
chars = list(enumerate("xy"))
assert chars == [(0, "x"), (1, "y")]; _ledger.append(1)

# Empty zip — yields no tuples
empty = list(zip([], ["a"]))
assert empty == []; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_zip_enumerate_advanced_ops {sum(_ledger)} asserts")
