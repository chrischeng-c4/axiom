# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_typeerror_dict_type_constructor_silent"
# subject = "cpython321.lang_typeerror_dict_type_constructor_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_typeerror_dict_type_constructor_silent.py"
# status = "filled"
# ///
"""cpython321.lang_typeerror_dict_type_constructor_silent: execute CPython 3.12 seed lang_typeerror_dict_type_constructor_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Spec seed for CPython TypeError contract on the dict / type
# constructor argument-validation corners that mamba silently
# returns empty / no-op values from. Surface: CPython rejects (1)
# `{**non_mapping}` because `**`-unpack requires the right operand to
# implement the Mapping protocol (i.e. expose `keys()`); int and list
# don't, so it's TypeError — not silent empty dict; (2)
# `dict.fromkeys(non_iterable)` because the keys argument is iterated
# to build the new dict — int is not iterable, so TypeError — not
# silent empty dict; (3) `type(name, bad_bases, _)` because the
# bases argument must be a tuple, not int/list/dict — TypeError;
# (4) `type(name, _, bad_dict)` because the namespace argument must
# be a dict, not int/list/tuple — TypeError. Mamba 0.3.60 silently
# returns `{}` / `<type instance>` instead of dispatching the
# protocol → TypeError fallback.
#
# Probes (every form CPython rejects, mamba silently coerces):
#   • {**5}                          → mamba: {}             (TypeError)
#   • {**[1, 2]}                     → mamba: {}             (TypeError)
#   • {**(1, 2)}                     → mamba: {}             (TypeError)
#   • dict.fromkeys(5)               → mamba: {}             (TypeError)
#   • dict.fromkeys(5.5)             → mamba: {}             (TypeError)
#   • dict.fromkeys(None)            → mamba: {}             (TypeError)
#   • type("X", 5, {})               → mamba: <type instance>(TypeError)
#   • type("X", [object], {})        → mamba: <type instance>(TypeError)
#   • type("X", (object,), 5)        → mamba: <type instance>(TypeError)
#   • type("X", (object,), [1])      → mamba: <type instance>(TypeError)
#
# CPython contract:
#   {**non_mapping}        → TypeError("'<typename>' object is not a
#                                  mapping");
#   dict.fromkeys(non_iter)
#                          → TypeError("'<typename>' object is not
#                                  iterable");
#   type(name, bad_bases, dict)
#                          → TypeError("type.__new__() argument 2
#                                  must be tuple, not <typename>");
#   type(name, bases, bad_dict)
#                          → TypeError("type.__new__() argument 3
#                                  must be dict, not <typename>").
#
# `Any`-typed holders push the operand past static type-checkers
# (Pyright) and past mamba's compile-time argtype enforcement so the
# runtime divergence is what's exercised.
from typing import Any
_ledger: list[int] = []

_i: Any = 5
_f: Any = 5.5
_n: Any = None
_lst: Any = [1, 2]
_tup: Any = (1, 2)
_bases_int: Any = 5
_bases_list: Any = [object]
_dict_int: Any = 5
_dict_list: Any = [1]

# {**5} — int is not a mapping
try:
    _ = {**_i}
    raise AssertionError("{**5} must raise TypeError")
except TypeError:
    _ledger.append(1)

# {**[1, 2]} — list is not a mapping
try:
    _ = {**_lst}
    raise AssertionError("{**[1, 2]} must raise TypeError")
except TypeError:
    _ledger.append(1)

# {**(1, 2)} — tuple is not a mapping
try:
    _ = {**_tup}
    raise AssertionError("{**(1, 2)} must raise TypeError")
except TypeError:
    _ledger.append(1)

# dict.fromkeys(5) — int is not iterable
try:
    _ = dict.fromkeys(_i)
    raise AssertionError("dict.fromkeys(5) must raise TypeError")
except TypeError:
    _ledger.append(1)

# dict.fromkeys(5.5) — float is not iterable
try:
    _ = dict.fromkeys(_f)
    raise AssertionError("dict.fromkeys(5.5) must raise TypeError")
except TypeError:
    _ledger.append(1)

# dict.fromkeys(None) — None is not iterable
try:
    _ = dict.fromkeys(_n)
    raise AssertionError("dict.fromkeys(None) must raise TypeError")
except TypeError:
    _ledger.append(1)

# type("X", 5, {}) — bases must be tuple
try:
    _ = type("X", _bases_int, {})
    raise AssertionError("type('X', 5, {}) must raise TypeError")
except TypeError:
    _ledger.append(1)

# type("X", [object], {}) — bases must be tuple, not list
try:
    _ = type("X", _bases_list, {})
    raise AssertionError("type('X', [object], {}) must raise TypeError")
except TypeError:
    _ledger.append(1)

# type("X", (object,), 5) — namespace must be dict
try:
    _ = type("X", (object,), _dict_int)
    raise AssertionError("type('X', (object,), 5) must raise TypeError")
except TypeError:
    _ledger.append(1)

# type("X", (object,), [1]) — namespace must be dict, not list
try:
    _ = type("X", (object,), _dict_list)
    raise AssertionError("type('X', (object,), [1]) must raise TypeError")
except TypeError:
    _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_typeerror_dict_type_constructor_silent {sum(_ledger)} asserts")
