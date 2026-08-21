"""Surface contract for language pattern matching (match/case, PEP 634).

# type-regime: monomorphic

Probes: literal patterns, capture patterns, wildcard, sequence patterns,
mapping patterns, OR patterns, guard clauses.
CPython 3.12 is the oracle.
"""

# Literal pattern
def _classify_int(n: int) -> str:
    match n:
        case 0:
            return "zero"
        case 1:
            return "one"
        case _:
            return "other"

assert _classify_int(0) == "zero", f"0 = {_classify_int(0)!r}"
assert _classify_int(1) == "one", f"1 = {_classify_int(1)!r}"
assert _classify_int(99) == "other", f"99 = {_classify_int(99)!r}"

# Capture pattern
def _first_item(lst: list):
    match lst:
        case [first, *_]:
            return first
        case []:
            return None

assert _first_item([10, 20, 30]) == 10, f"first = {_first_item([10,20,30])!r}"
assert _first_item([]) is None, f"empty = {_first_item([])!r}"

# Sequence pattern — exact length
def _head_tail(lst: list):
    match lst:
        case [a, b]:
            return (a, b)
        case [a, b, c]:
            return (a, b, c)
        case _:
            return None

assert _head_tail([1, 2]) == (1, 2), f"pair = {_head_tail([1,2])!r}"
assert _head_tail([1, 2, 3]) == (1, 2, 3), f"triple = {_head_tail([1,2,3])!r}"
assert _head_tail([1]) is None, f"single = {_head_tail([1])!r}"

# Mapping pattern
def _get_name(d: dict) -> str:
    match d:
        case {"name": str(n)}:
            return n
        case _:
            return "unknown"

assert _get_name({"name": "Alice"}) == "Alice", f"name = {_get_name({'name':'Alice'})!r}"
assert _get_name({"age": 30}) == "unknown", f"no name = {_get_name({'age':30})!r}"

# OR pattern
def _is_vowel(ch: str) -> bool:
    match ch:
        case "a" | "e" | "i" | "o" | "u":
            return True
        case _:
            return False

assert _is_vowel("a") == True, "a is vowel"
assert _is_vowel("b") == False, "b not vowel"
assert _is_vowel("e") == True, "e is vowel"

# Guard clause (if condition in case)
def _positive_or_neg(n: int) -> str:
    match n:
        case x if x > 0:
            return "positive"
        case x if x < 0:
            return "negative"
        case _:
            return "zero"

assert _positive_or_neg(5) == "positive", f"5 = {_positive_or_neg(5)!r}"
assert _positive_or_neg(-3) == "negative", f"-3 = {_positive_or_neg(-3)!r}"
assert _positive_or_neg(0) == "zero", f"0 = {_positive_or_neg(0)!r}"

print("surface OK")
