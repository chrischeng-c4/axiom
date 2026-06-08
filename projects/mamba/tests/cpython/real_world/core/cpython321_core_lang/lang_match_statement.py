# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_match_statement"
# subject = "cpython321.lang_match_statement"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/lang_match_statement.py"
# status = "filled"
# ///
"""cpython321.lang_match_statement: execute CPython 3.12 seed lang_match_statement"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the structural pattern-matching
# surface (PEP 634/635/636 — `match` / `case`). Surface: literal
# patterns over int (`case 0:`, `case 1:`) and str (`case "hi":`)
# select on equality; the wildcard `case _:` catches the residue
# of any value not previously matched; sequence patterns match
# list/tuple by length (`case []:` empty, `case [x]:` single,
# `case [x, y]:` pair, with the tuple form `case (0, 0):` /
# `case (0, y):` capturing the unbound slot via name binding);
# or-pattern (`case "a" | "e" | "i":`) matches any alternative;
# guard clauses (`case x if x > 0:`) gate match acceptance on a
# runtime predicate; mapping patterns (`case {"type": "click"}:`)
# match dicts by required key/value pairs (allowing extra keys).
# Companion to lang_match_class_pattern (class destructuring) and
# lang_match_star_pattern (which exercises `[x, *_]`).
_ledger: list[int] = []

# Literal-int dispatch via `case 0:` / `case 1:` and `case _:`
def classify(x: int) -> str:
    match x:
        case 0:
            return "zero"
        case 1:
            return "one"
        case _:
            return "other"
assert classify(0) == "zero"; _ledger.append(1)
assert classify(1) == "one"; _ledger.append(1)
assert classify(99) == "other"; _ledger.append(1)
assert classify(-1) == "other"; _ledger.append(1)

# Literal-str dispatch
def label(s: str) -> str:
    match s:
        case "hi":
            return "greet"
        case "bye":
            return "farewell"
        case _:
            return "unknown"
assert label("hi") == "greet"; _ledger.append(1)
assert label("bye") == "farewell"; _ledger.append(1)
assert label("xx") == "unknown"; _ledger.append(1)

# Sequence patterns — length-discriminated dispatch on list
def first(L: list[int]) -> str:
    match L:
        case []:
            return "empty"
        case [_]:
            return "one"
        case [_, _]:
            return "two"
        case _:
            return "many"
assert first([]) == "empty"; _ledger.append(1)
assert first([1]) == "one"; _ledger.append(1)
assert first([1, 2]) == "two"; _ledger.append(1)
assert first([1, 2, 3]) == "many"; _ledger.append(1)

# Tuple pattern with literal + capture slots
def describe(p: tuple[int, int]) -> str:
    match p:
        case (0, 0):
            return "origin"
        case (0, _):
            return "y-axis"
        case (_, 0):
            return "x-axis"
        case _:
            return "elsewhere"
assert describe((0, 0)) == "origin"; _ledger.append(1)
assert describe((0, 5)) == "y-axis"; _ledger.append(1)
assert describe((5, 0)) == "x-axis"; _ledger.append(1)
assert describe((5, 5)) == "elsewhere"; _ledger.append(1)

# Or-pattern — multiple alternatives in a single case
def vowel(c: str) -> str:
    match c:
        case "a" | "e" | "i" | "o" | "u":
            return "vowel"
        case _:
            return "consonant"
assert vowel("a") == "vowel"; _ledger.append(1)
assert vowel("e") == "vowel"; _ledger.append(1)
assert vowel("b") == "consonant"; _ledger.append(1)
assert vowel("z") == "consonant"; _ledger.append(1)

# Guard clauses — `case <pat> if <expr>:`
def sign(n: int) -> str:
    match n:
        case x if x > 0:
            return "pos"
        case x if x < 0:
            return "neg"
        case _:
            return "zero"
assert sign(5) == "pos"; _ledger.append(1)
assert sign(-3) == "neg"; _ledger.append(1)
assert sign(0) == "zero"; _ledger.append(1)

# Mapping patterns — match required key/value, allow extras
def event(e: dict) -> str:
    match e:
        case {"type": "click"}:
            return "click"
        case {"type": "key"}:
            return "key"
        case _:
            return "other"
assert event({"type": "click"}) == "click"; _ledger.append(1)
assert event({"type": "key"}) == "key"; _ledger.append(1)
assert event({"type": "scroll"}) == "other"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_match_statement {sum(_ledger)} asserts")
