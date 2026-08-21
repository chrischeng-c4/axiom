# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_for_else_break"
# subject = "cpython321.lang_for_else_break"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/lang_for_else_break.py"
# status = "filled"
# ///
"""cpython321.lang_for_else_break: execute CPython 3.12 seed lang_for_else_break"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for for-loop control surfaces beyond
# lang_for_loop_iteration (basic iteration) and lang_while_break_continue.
# Surface: for-else — the `else` clause runs ONLY when the loop
# completes without hitting `break`. Specifically: clean completion
# fires `else`; an interior `break` skips `else`; an EMPTY iterable
# still fires `else` (because zero iterations also count as completing
# without break); `continue` does NOT skip `else`; the "search loop"
# idiom — `for x in items: if matches(x): return found`-style with a
# trailing return-not-found — is the canonical use case; break with a
# carrying boolean flag works just like for-else; for-loop range
# iteration with break + continue interleaved.
_ledger: list[int] = []

# for-else — clean completion fires else
ledger1: list[int] = []
for x in [1, 2, 3]:
    ledger1.append(x)
else:
    ledger1.append(99)
assert ledger1 == [1, 2, 3, 99]; _ledger.append(1)

# for-else — interior break SKIPS else
ledger2: list[int] = []
for x in [1, 2, 3, 4]:
    if x == 3:
        break
    ledger2.append(x)
else:
    ledger2.append(99)
assert ledger2 == [1, 2]; _ledger.append(1)

# for-else — empty iterable still fires else (zero iterations count as
# "completed without break")
ledger3: list[int] = []
for x in []:
    ledger3.append(x)
else:
    ledger3.append(99)
assert ledger3 == [99]; _ledger.append(1)

# for-else — `continue` does NOT skip the else clause
ledger4: list[int] = []
for x in [1, 2, 3, 4]:
    if x % 2 == 0:
        continue
    ledger4.append(x)
else:
    ledger4.append(99)
assert ledger4 == [1, 3, 99]; _ledger.append(1)

# Canonical search idiom — for-else as "not found" branch via a
# string sentinel return
def find(haystack: list[int], needle: int) -> str:
    for x in haystack:
        if x == needle:
            return "FOUND"
    return "NOT_FOUND"
assert find([1, 2, 3], 2) == "FOUND"; _ledger.append(1)
assert find([1, 2, 3], 99) == "NOT_FOUND"; _ledger.append(1)
assert find([], 5) == "NOT_FOUND"; _ledger.append(1)

# Same search idiom written with for-else
def find_else(haystack: list[int], needle: int) -> str:
    for x in haystack:
        if x == needle:
            return "FOUND"
    else:
        return "NOT_FOUND"
    return "UNREACHABLE"
assert find_else([1, 2, 3], 2) == "FOUND"; _ledger.append(1)
assert find_else([1, 2, 3], 99) == "NOT_FOUND"; _ledger.append(1)

# Break with a flag — equivalent restatement of for-else without
# using the else clause
def find_flag(haystack: list[int], needle: int) -> str:
    found = False
    for x in haystack:
        if x == needle:
            found = True
            break
    if found:
        return "FOUND"
    return "NOT_FOUND"
assert find_flag([1, 2, 3], 2) == "FOUND"; _ledger.append(1)
assert find_flag([1, 2, 3], 99) == "NOT_FOUND"; _ledger.append(1)

# Break stops the loop immediately
out: list[int] = []
for x in [1, 2, 3, 4, 5]:
    if x == 4:
        break
    out.append(x)
assert out == [1, 2, 3]; _ledger.append(1)

# Continue skips just the current iteration
out2: list[int] = []
for x in range(5):
    if x % 2 == 0:
        continue
    out2.append(x)
assert out2 == [1, 3]; _ledger.append(1)

# Interleaved break + continue
out3: list[int] = []
for x in range(10):
    if x == 7:
        break
    if x % 2 == 0:
        continue
    out3.append(x)
assert out3 == [1, 3, 5]; _ledger.append(1)

# for over enumerate inside a function with a return-on-match
def first_index(items: list[int], needle: int) -> int:
    for i, v in enumerate(items):
        if v == needle:
            return i
    return -1
assert (first_index([10, 20, 30], 20) - 1) == 0; _ledger.append(1)
assert (first_index([10, 20, 30], 99) - (-1)) == 0; _ledger.append(1)

# for-else on a range iterable (range is an iterable, so for-else
# applies just like for a list)
ledger5: list[int] = []
for i in range(3):
    ledger5.append(i)
else:
    ledger5.append(-1)
assert ledger5 == [0, 1, 2, -1]; _ledger.append(1)

# for-else with break on a range
ledger6: list[int] = []
for i in range(10):
    if i == 4:
        break
    ledger6.append(i)
else:
    ledger6.append(-1)
assert ledger6 == [0, 1, 2, 3]; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: lang_for_else_break {sum(_ledger)} asserts")
