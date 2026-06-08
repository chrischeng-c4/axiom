# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_match_sequence_mapping"
# subject = "cpython321.lang_match_sequence_mapping"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/lang_match_sequence_mapping.py"
# status = "filled"
# ///
"""cpython321.lang_match_sequence_mapping: execute CPython 3.12 seed lang_match_sequence_mapping"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# lang_match_sequence_mapping.py - axis-1 PEP 634 match: sequence + mapping patterns seed (#3345).
#
# Surface (from #3345):
#   1. `case [a, b, *rest]:` star pattern binds remainder
#   2. `case {"key": v}:` mapping pattern with capture
#   3. `case {"a": 1, **rest}:` double-star mapping binds leftover keys
#   4. Nested sequence inside class pattern

_ledger: list[int] = []


class Box:
    __match_args__ = ("items",)

    def __init__(self, items):
        self.items = items


def seq_split(v):
    match v:
        case [a, b, *rest]:
            return ("split", a, b, rest)
        case _:
            return "other"


def map_pick(v):
    match v:
        case {"a": 1, **rest}:
            return ("a1", rest)
        case {"key": x}:
            return ("k", x)
        case _:
            return "other"


def nested(v):
    match v:
        case Box(items=[head, *tail]):
            return ("box-cons", head, tail)
        case Box(items=[]):
            return "box-empty"
        case _:
            return "other"


# 1. Star pattern: `[a, b, *rest]` binds first two + rest.
res = seq_split([1, 2, 3, 4])
assert res == ("split", 1, 2, [3, 4]), "star pattern binds head/head + remainder list"
_ledger.append(1)

res2 = seq_split([10, 20])
assert res2 == ("split", 10, 20, []), "star pattern accepts empty remainder"
_ledger.append(1)

# 1. Sequence pattern fails on non-sequence (falls to wildcard).
assert seq_split("not-a-list") == "other", "sequence pattern does not match a string"
_ledger.append(1)

# 2. Mapping pattern: `{"key": v}` captures value at key.
assert map_pick({"key": "v"}) == ("k", "v"), "mapping pattern captures value at named key"
_ledger.append(1)

# 3. Double-star mapping: `{"a": 1, **rest}` binds remaining keys.
res3 = map_pick({"a": 1, "b": 2, "c": 3})
assert res3 == ("a1", {"b": 2, "c": 3}), "double-star mapping binds leftover keys into rest"
_ledger.append(1)

# 3. Double-star with empty leftover.
res4 = map_pick({"a": 1})
assert res4 == ("a1", {}), "double-star mapping yields empty dict when no leftover keys"
_ledger.append(1)

# 2/3. Mapping pattern fails when key absent -> falls to wildcard.
assert map_pick({"other": 99}) == "other", "mapping pattern does not match when required key missing"
_ledger.append(1)

# 4. Nested sequence inside class pattern.
b = Box([10, 20, 30])
assert nested(b) == ("box-cons", 10, [20, 30]), "nested sequence pattern destructures inside class pattern"
_ledger.append(1)

assert nested(Box([])) == "box-empty", "nested empty sequence inside class pattern matches []"
_ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_match_sequence_mapping {sum(_ledger)} asserts")
