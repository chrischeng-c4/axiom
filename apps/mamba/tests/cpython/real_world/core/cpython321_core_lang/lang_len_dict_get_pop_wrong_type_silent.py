# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_len_dict_get_pop_wrong_type_silent"
# subject = "cpython321.lang_len_dict_get_pop_wrong_type_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_len_dict_get_pop_wrong_type_silent.py"
# status = "filled"
# ///
"""cpython321.lang_len_dict_get_pop_wrong_type_silent: execute CPython 3.12 seed lang_len_dict_get_pop_wrong_type_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Spec seed for CPython TypeError contract on the `len(obj)` /
# `dict.get(unhashable)` / `dict.pop(unhashable)` corners where
# mamba silently returns `0` / `None` / the supplied default
# instead of raising TypeError.
#
# Surface: CPython rejects (1) `len(non_sized)` for any object
# whose type does not implement `__len__` — `int`, `None`, `float`,
# `bool`, function, `type` itself — TypeError("object of type
# '<type>' has no len()"); (2) `dict.get(unhashable, default)` /
# `dict.pop(unhashable, default)` because the lookup routes
# through `__hash__` and `list` / `dict` / `set` / `bytearray`
# don't define `__hash__` — TypeError("unhashable type: '<name>'").
#
# Mamba accepts every form and silently returns `0` for `len()`
# (so `if len(maybe_iter):` silently takes the False branch when
# `maybe_iter` is `None`, masking a missing-iterable bug) and
# silently returns `None` / the supplied default for
# `dict.get(unhashable)` / `dict.pop(unhashable, default)` (so
# `cache.get(meta_dict_used_as_key)` silently returns `None`
# whenever the lookup key was accidentally a dict, rather than
# failing loud at the hash-check).
#
# Existing lang_typeerror_unhashable.py covers
# `set.add / .discard / .remove(unhashable)` and `dict[unhashable]
# = v` + `dict.setdefault(unhashable, ...)` + `unhashable in dict
# / set` + `hash(unhashable)` + `{unhashable: 1}` / `{unhashable}`
# literals. Existing lang_collection_mutator_non_iterable_silent.py
# covers `dict.update(non_iter)` / `list.extend(non_iter)` /
# `set.union(non_iter)`. Existing lang_iter_non_iterable_silent.py
# covers `iter(non_iter)` / `reversed(non_iter)`. This seed covers
# the FRESH divergence family — `len(non_sized)` (which routes
# through `__len__`, not `__iter__` or `__hash__`) and
# `dict.get / .pop(unhashable, default)` (which short-circuit the
# missing-key check on mamba so the default fires instead of the
# hash-check raising TypeError).
#
# Probes (every form CPython rejects, mamba silently coerces):
#   • len(5)                       → mamba: 0          (TypeError)
#   • len(None)                    → mamba: 0          (TypeError)
#   • len(3.14)                    → mamba: 0          (TypeError)
#   • len(True)                    → mamba: 0          (TypeError)
#   • len(lambda x: x)             → mamba: 0          (TypeError)
#   • {}.get([1,2])                → mamba: None       (TypeError)
#   • {}.get({"k":1})              → mamba: None       (TypeError)
#   • {}.get({1,2})                → mamba: None       (TypeError)
#   • {}.get(bytearray(b"a"))      → mamba: None       (TypeError)
#   • {}.get([1,2], 99)            → mamba: 99         (TypeError)
#   • {}.pop([1,2], 99)            → mamba: 99         (TypeError)
#   • {}.pop({"k":1}, 99)          → mamba: 99         (TypeError)
#
# CPython contract (uniform across every form):
#   len(non_sized)
#       → TypeError("object of type '<type>' has no len()");
#   dict.get(unhashable, default) / dict.pop(unhashable, default)
#       → TypeError("unhashable type: '<name>'").
#
# `Any`-typed holders push the operand past static type-checkers
# (Pyright) and past mamba's compile-time argtype enforcement so
# the runtime divergence is what's exercised.
from typing import Any
_ledger: list[int] = []

_int: Any = 5
_none: Any = None
_flt: Any = 3.14
_bool: Any = True
_fn: Any = lambda x: x
_lst: Any = [1, 2]
_dct: Any = {"k": 1}
_st: Any = {1, 2}
_ba: Any = bytearray(b"a")

# len(int) — int has no __len__
try:
    _ = len(_int)
    raise AssertionError("len(5) must raise TypeError")
except TypeError:
    _ledger.append(1)

# len(None)
try:
    _ = len(_none)
    raise AssertionError("len(None) must raise TypeError")
except TypeError:
    _ledger.append(1)

# len(float)
try:
    _ = len(_flt)
    raise AssertionError("len(3.14) must raise TypeError")
except TypeError:
    _ledger.append(1)

# len(bool)
try:
    _ = len(_bool)
    raise AssertionError("len(True) must raise TypeError")
except TypeError:
    _ledger.append(1)

# len(function)
try:
    _ = len(_fn)
    raise AssertionError("len(lambda) must raise TypeError")
except TypeError:
    _ledger.append(1)

# dict.get(list) — unhashable key routes through __hash__
_d_get: Any = {"a": 1}
try:
    _ = _d_get.get(_lst)
    raise AssertionError("dict.get([1,2]) must raise TypeError")
except TypeError:
    _ledger.append(1)

# dict.get(dict)
try:
    _ = _d_get.get(_dct)
    raise AssertionError("dict.get({'k':1}) must raise TypeError")
except TypeError:
    _ledger.append(1)

# dict.get(set)
try:
    _ = _d_get.get(_st)
    raise AssertionError("dict.get({1,2}) must raise TypeError")
except TypeError:
    _ledger.append(1)

# dict.get(bytearray)
try:
    _ = _d_get.get(_ba)
    raise AssertionError("dict.get(bytearray) must raise TypeError")
except TypeError:
    _ledger.append(1)

# dict.get(list, default) — even with default, hash-check fires first
try:
    _ = _d_get.get(_lst, 99)
    raise AssertionError("dict.get([1,2], 99) must raise TypeError")
except TypeError:
    _ledger.append(1)

# dict.pop(list, default)
_d_pop: Any = {"a": 1}
try:
    _ = _d_pop.pop(_lst, 99)
    raise AssertionError("dict.pop([1,2], 99) must raise TypeError")
except TypeError:
    _ledger.append(1)

# dict.pop(dict, default)
_d_pop2: Any = {"a": 1}
try:
    _ = _d_pop2.pop(_dct, 99)
    raise AssertionError("dict.pop({'k':1}, 99) must raise TypeError")
except TypeError:
    _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_len_dict_get_pop_wrong_type_silent {sum(_ledger)} asserts")
