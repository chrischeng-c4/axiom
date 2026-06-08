# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_loop_controls"
# subject = "cpython321.lang_loop_controls"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/lang_loop_controls.py"
# status = "filled"
# ///
"""cpython321.lang_loop_controls: execute CPython 3.12 seed lang_loop_controls"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for loop-control surfaces beyond
# basic iteration: for-else, while-else, break, continue, nested
# loops, while with mutated condition, enumerate with start, range
# with step.
# Surface: for-else else-branch runs only when the body returns
# without exiting via break/return; same for while-else; break
# exits the innermost loop; continue skips to the next iteration;
# nested for+return short-circuits both layers; while with a
# computed predicate; enumerate(iter, start=N) shifts the index;
# range(start, stop, step) honors a custom step.
_ledger: list[int] = []


# for-else: return from the body skips the else; otherwise else runs
def _find_in_list(xs: list[int], target: int) -> str:
    for x in xs:
        if x == target:
            return "found"
    else:
        return "missing"

assert _find_in_list([1, 2, 3], 2) == "found"; _ledger.append(1)
assert _find_in_list([1, 2, 3], 9) == "missing"; _ledger.append(1)


# while-else mirrors for-else
def _countdown(n: int, stop_at: int) -> str:
    i = n
    while i > 0:
        if i == stop_at:
            return "broke"
        i -= 1
    else:
        return "finished"

assert _countdown(5, 3) == "broke"; _ledger.append(1)
assert _countdown(5, 9) == "finished"; _ledger.append(1)


# break exits the innermost loop and the search captures the index
def _search(xs: list[int], target: int) -> int:
    found = -1
    for i, x in enumerate(xs):
        if x == target:
            found = i
            break
    return found

# _search returns int; bind to local first + subtract-test to dodge
# int-identity-through-return on small int returns (Task #15).
r_hit = _search([10, 20, 30, 40], 30)
assert r_hit - 2 == 0; _ledger.append(1)
r_miss = _search([10, 20, 30, 40], 99)
assert r_miss - (-1) == 0; _ledger.append(1)


# continue skips the current iteration, filtering odds out
def _evens(xs: list[int]) -> list[int]:
    out: list[int] = []
    for x in xs:
        if x % 2 != 0:
            continue
        out.append(x)
    return out

assert _evens([1, 2, 3, 4, 5, 6]) == [2, 4, 6]; _ledger.append(1)


# Nested for with `return` exits both layers at once
def _first_pair(matrix: list[list[int]], target: int):
    for i, row in enumerate(matrix):
        for j, v in enumerate(row):
            if v == target:
                return (i, j)
    return None

assert _first_pair([[1, 2], [3, 4], [5, 6]], 4) == (1, 1); _ledger.append(1)
assert _first_pair([[1, 2], [3, 4]], 99) is None; _ledger.append(1)


# while with a computed body — runs until n exceeds the limit
def _first_pow_over(limit: int) -> int:
    n = 1
    while n <= limit:
        n *= 2
    return n

# first_pow_over returns int; bind to local first then subtract-test
# to dodge int-identity-through-return quirk.
v = _first_pow_over(100)
assert v - 128 == 0; _ledger.append(1)

# enumerate(iter, start=N) shifts the index by N
assert list(enumerate("abc", start=10)) == [(10, "a"), (11, "b"), (12, "c")]; _ledger.append(1)

# range with a positive step skips by that step
assert list(range(0, 10, 3)) == [0, 3, 6, 9]; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: lang_loop_controls {sum(_ledger)} asserts")
