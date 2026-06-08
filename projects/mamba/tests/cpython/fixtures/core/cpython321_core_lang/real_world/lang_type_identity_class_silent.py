# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_type_identity_class_silent"
# subject = "cpython321.lang_type_identity_class_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_type_identity_class_silent.py"
# status = "filled"
# ///
"""cpython321.lang_type_identity_class_silent: execute CPython 3.12 seed lang_type_identity_class_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass divergence-spec fixture for the
# silent value-contract divergence of `type(42) is int` /
# `type(42) == int` (the documented "type() returns the canonical
# type object that is the same as the built-in" identity contract —
# mamba silently returns a fresh non-identical type each call, so
# both `is` and `==` against the canonical built-in evaluate to
# False), `type("hi") is str` / `type([]) is list` / `type({}) is
# dict` / `type(()) is tuple` / `type(set()) is set` /
# `type(True) is bool` / `type(None) is type(None)` (the same
# documented identity contract across primitive built-in types —
# mamba returns False for every one of them), and `_m.__class__
# is _MyCls` (the documented "instance.__class__ is the class
# object that constructed it" identity contract — mamba's user-
# class instances silently bind `.__class__` to a placeholder
# that does not match the original class object). Ten-pack pinned
# to atomic 249.
#
# Behavioral edges that CONFORM on mamba (the entire str method
# surface — split/rsplit/splitlines/strip/lstrip/rstrip/join/
# replace/find/rfind/index/count/upper/lower/title/capitalize/
# swapcase/casefold/isalpha/isdigit/isspace/isalnum/isupper/
# islower/istitle/startswith/endswith/center/ljust/rjust/zfill/
# partition/rpartition/expandtabs/removeprefix/removesuffix/
# encode + maketrans/translate; str format specs :b /:o /:#x /:, /
# :+d /:%; list mutation methods append/extend/insert/remove/
# reverse/sort/clear + count/index/copy; dict keys/values/items/
# get/get-default/update/setdefault + in/not-in; set add/remove/
# discard/union/intersection/difference/symmetric_difference/
# issubset/issuperset/isdisjoint; isinstance int/str/list/tuple
# +multi; issubclass bool-int/list-list/dict-object; repr dict/
# list/tuple insertion order; OOP instance-attr/classmethod/
# staticmethod/property/super; dict constructor variants; list/
# dict/set/gen comprehensions; type(...).__name__) are covered
# in the matching pass fixture
# `test_str_list_dict_set_methods_oop_comprehensions_value_ops`.
from typing import Any


class _MyCls:
    cv = 99


_ledger: list[int] = []

# 1) type(42) is int — built-in primitive identity contract
#    (mamba: type() returns a non-identical fresh object each call)
assert (type(42) is int) == True; _ledger.append(1)

# 2) type(42) == int — built-in primitive equality contract
#    (mamba: returns False)
assert (type(42) == int) == True; _ledger.append(1)

# 3) type("hi") is str
#    (mamba: returns False)
assert (type("hi") is str) == True; _ledger.append(1)

# 4) type([]) is list
#    (mamba: returns False)
assert (type([]) is list) == True; _ledger.append(1)

# 5) type({}) is dict
#    (mamba: returns False)
assert (type({}) is dict) == True; _ledger.append(1)

# 6) type(()) is tuple
#    (mamba: returns False)
assert (type(()) is tuple) == True; _ledger.append(1)

# 7) type(set()) is set
#    (mamba: returns False)
assert (type(set()) is set) == True; _ledger.append(1)

# 8) type(True) is bool
#    (mamba: returns False)
assert (type(True) is bool) == True; _ledger.append(1)

# 9) type(None) is type(None) — even two type-of calls don't return same
#    (mamba: returns False)
assert (type(None) is type(None)) == True; _ledger.append(1)

# 10) _m.__class__ is _MyCls — user-class instance __class__ identity
#     (mamba: returns False; instance __class__ binds to placeholder)
_m: Any = _MyCls()
assert (_m.__class__ is _MyCls) == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_type_identity_class_silent {sum(_ledger)} asserts")
