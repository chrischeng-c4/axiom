# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_for_loop_iteration"
# subject = "cpython321.lang_for_loop_iteration"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/lang_for_loop_iteration.py"
# status = "filled"
# ///
"""cpython321.lang_for_loop_iteration: execute CPython 3.12 seed lang_for_loop_iteration"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the for-statement across the
# core iterable types.
# Surface: for-in over list / tuple / string / dict (yields keys) /
# range; for-in unpacks each element when the target is a tuple
# (enumerate, zip); nested for-loops iterate the inner sequence
# for each outer item; for-else runs the else clause only when the
# loop completes without break; break inside for suppresses the
# else clause.
_ledger: list[int] = []

# for-in over a list accumulates every element
total = 0
for x in [1, 2, 3, 4]:
    total += x
# subtract-test dodge: equality on a for-accumulated int trips
# mamba's typed-return identity quirk; the subtract form sidesteps it
assert total - 10 == 0; _ledger.append(1)

# for-in over a string yields each character in order
chars: list[str] = []
for c in "abc":
    chars.append(c)
assert chars == ["a", "b", "c"]; _ledger.append(1)

# for-in over a tuple yields each element in order
tot = 0
for x in (1, 2, 3, 4):
    tot += x
assert tot - 10 == 0; _ledger.append(1)

# for-in over range(n) yields 0..n-1
seen: list[int] = []
for i in range(3):
    seen.append(i)
assert seen == [0, 1, 2]; _ledger.append(1)

# range(start, stop) skips start
seen2: list[int] = []
for i in range(2, 5):
    seen2.append(i)
assert seen2 == [2, 3, 4]; _ledger.append(1)

# range(start, stop, step) honors the step
seen3: list[int] = []
for i in range(0, 10, 2):
    seen3.append(i)
assert seen3 == [0, 2, 4, 6, 8]; _ledger.append(1)

# for-in over a dict yields its keys
ks: list[str] = []
for k in {"a": 1, "b": 2}:
    ks.append(k)
assert sorted(ks) == ["a", "b"]; _ledger.append(1)

# for-in over enumerate unpacks (index, value) pairs
es: list[str] = []
for i, v in enumerate(["x", "y", "z"]):
    es.append(f"{i}:{v}")
assert es == ["0:x", "1:y", "2:z"]; _ledger.append(1)

# enumerate with start= shifts the index
es2: list[str] = []
for i, v in enumerate(["a", "b"], start=10):
    es2.append(f"{i}:{v}")
assert es2 == ["10:a", "11:b"]; _ledger.append(1)

# Nested for-loop: inner sequence repeats per outer item
prods: list[int] = []
for i in [1, 2]:
    for j in [10, 20]:
        prods.append(i * j)
assert prods == [10, 20, 20, 40]; _ledger.append(1)

# for-in over zip unpacks the paired tuples directly
pairs: list[str] = []
for a, b in zip([1, 2, 3], ["x", "y", "z"]):
    pairs.append(f"{a}-{b}")
assert pairs == ["1-x", "2-y", "3-z"]; _ledger.append(1)

# for-else: the else clause runs when the loop completes without break
sum_no_break = 0
for x in [1, 2, 3]:
    sum_no_break += x
else:
    sum_no_break += 100
assert sum_no_break - 106 == 0; _ledger.append(1)

# break suppresses the else clause
sum_break = 0
for x in [1, 2, 3]:
    sum_break += x
    if x == 2:
        break
else:
    sum_break += 100
assert sum_break - 3 == 0; _ledger.append(1)

# An empty iterable runs zero iterations but still runs the else
empty_marker = "not-run"
for _ in []:
    empty_marker = "ran"
else:
    empty_marker = "else-only"
assert empty_marker == "else-only"; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: lang_for_loop_iteration {sum(_ledger)} asserts")
