# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_abc_copy_enum_pprint_textwrap_value_ops"
# subject = "cpython321.test_abc_copy_enum_pprint_textwrap_value_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_abc_copy_enum_pprint_textwrap_value_ops.py"
# status = "filled"
# ///
"""cpython321.test_abc_copy_enum_pprint_textwrap_value_ops: execute CPython 3.12 seed test_abc_copy_enum_pprint_textwrap_value_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the value contract of the
# `abc` / `copy` / `enum` / `pprint` / `textwrap` five-pack
# pinned to atomic 201: `abc` (the documented full module-level
# class / decorator identifier hasattr surface — `ABC` /
# `ABCMeta` / `abstractmethod` / `abstractclassmethod` /
# `abstractstaticmethod` / `abstractproperty` /
# `get_cache_token` / `update_abstractmethods` + the documented
# `issubclass(class(abc.ABC), object)` MRO contract), `copy`
# (the documented partial module-level helper / exception
# identifier hasattr surface — `copy` / `deepcopy` / `Error` +
# the documented shallow / deep round-trip identity contract —
# shallow shares nested children, deep does not), `enum` (the
# documented partial module-level class / decorator
# identifier hasattr surface — `Enum` / `IntEnum` / `Flag` /
# `IntFlag` / `StrEnum` / `auto` / `unique` / `EnumType`),
# `pprint` (the documented partial module-level helper
# identifier hasattr surface — `pprint` / `pformat`), and
# `textwrap` (the documented partial module-level helper
# identifier hasattr surface — `wrap` / `fill` / `dedent` /
# `indent` / `shorten`).
#
# Behavioral edges that DIVERGE on mamba
# (type(class(abc.ABC)).__name__ collapses to "str" on mamba
# instead of "ABCMeta", hasattr(class(abc.ABC),
# "__abstractmethods__") False on mamba, hasattr(enum,
# "ReprEnum") / "EnumMeta" / "FlagBoundary" / "STRICT" /
# "CONFORM" / "EJECT" / "KEEP" / "verify" / "EnumCheck" /
# "global_enum" / "member" / "nonmember" /
# "show_flag_values" / "property" all False on mamba,
# type(enum.Enum-subclass).__name__ collapses to "str",
# enum-member type collapses to "int", .value / .name return
# None, len(enum-class) returns 5 instead of 3, hasattr(pprint,
# "PrettyPrinter") / "pp" / "saferepr" / "isreadable" /
# "isrecursive" all False on mamba, pprint.pformat([1,2,3])
# expands to multi-line on mamba, hasattr(textwrap,
# "TextWrapper") False on mamba, textwrap.dedent /
# .indent strip trailing newline, textwrap.wrap returns a
# single-element list instead of multi-line wrap) are
# covered in the matching spec fixture
# `lang_abc_enum_pprint_textwrap_silent`.
import abc
import copy
import enum
import pprint
import textwrap


class _Animal(abc.ABC):
    @abc.abstractmethod
    def sound(self): ...


_ledger: list[int] = []

# 1) abc — full module hasattr surface
assert hasattr(abc, "ABC") == True; _ledger.append(1)
assert hasattr(abc, "ABCMeta") == True; _ledger.append(1)
assert hasattr(abc, "abstractmethod") == True; _ledger.append(1)
assert hasattr(abc, "abstractclassmethod") == True; _ledger.append(1)
assert hasattr(abc, "abstractstaticmethod") == True; _ledger.append(1)
assert hasattr(abc, "abstractproperty") == True; _ledger.append(1)
assert hasattr(abc, "get_cache_token") == True; _ledger.append(1)
assert hasattr(abc, "update_abstractmethods") == True; _ledger.append(1)

# 2) abc — MRO contract (type(class(abc.ABC)).__name__
#    collapses to "str" on mamba; hasattr(class(abc.ABC),
#    "__abstractmethods__") False on mamba — moved to spec)
assert issubclass(_Animal, object) == True; _ledger.append(1)

# 3) copy — partial module hasattr surface
assert hasattr(copy, "copy") == True; _ledger.append(1)
assert hasattr(copy, "deepcopy") == True; _ledger.append(1)
assert hasattr(copy, "Error") == True; _ledger.append(1)

# 4) copy — shallow / deep round-trip identity contract
_src = [1, 2, [3, 4]]
_shallow = copy.copy(_src)
_deep = copy.deepcopy(_src)
assert (_shallow == _src) == True; _ledger.append(1)
assert (_shallow is _src) == False; _ledger.append(1)
assert (_shallow[2] is _src[2]) == True; _ledger.append(1)
assert (_deep == _src) == True; _ledger.append(1)
assert (_deep is _src) == False; _ledger.append(1)
assert (_deep[2] is _src[2]) == False; _ledger.append(1)

# 5) enum — partial module hasattr surface
#    (ReprEnum / EnumMeta / FlagBoundary / STRICT / CONFORM /
#    EJECT / KEEP / verify / EnumCheck / global_enum / member
#    / nonmember / show_flag_values / property all DIVERGE on
#    mamba — moved to spec)
assert hasattr(enum, "Enum") == True; _ledger.append(1)
assert hasattr(enum, "IntEnum") == True; _ledger.append(1)
assert hasattr(enum, "Flag") == True; _ledger.append(1)
assert hasattr(enum, "IntFlag") == True; _ledger.append(1)
assert hasattr(enum, "StrEnum") == True; _ledger.append(1)
assert hasattr(enum, "auto") == True; _ledger.append(1)
assert hasattr(enum, "unique") == True; _ledger.append(1)
assert hasattr(enum, "EnumType") == True; _ledger.append(1)

# 6) pprint — partial module hasattr surface
#    (PrettyPrinter / pp / saferepr / isreadable /
#    isrecursive all DIVERGE on mamba — moved to spec)
assert hasattr(pprint, "pprint") == True; _ledger.append(1)
assert hasattr(pprint, "pformat") == True; _ledger.append(1)

# 7) textwrap — partial module hasattr surface
#    (TextWrapper DIVERGES on mamba — moved to spec)
assert hasattr(textwrap, "wrap") == True; _ledger.append(1)
assert hasattr(textwrap, "fill") == True; _ledger.append(1)
assert hasattr(textwrap, "dedent") == True; _ledger.append(1)
assert hasattr(textwrap, "indent") == True; _ledger.append(1)
assert hasattr(textwrap, "shorten") == True; _ledger.append(1)

# NB: type(class(abc.ABC)).__name__ collapses to "str" on mamba
# instead of "ABCMeta", hasattr(class(abc.ABC),
# "__abstractmethods__") False on mamba, hasattr(enum,
# "ReprEnum") / "EnumMeta" / "FlagBoundary" / "STRICT" /
# "CONFORM" / "EJECT" / "KEEP" / "verify" / "EnumCheck" /
# "global_enum" / "member" / "nonmember" /
# "show_flag_values" / "property" all False on mamba,
# type(enum.Enum-subclass).__name__ collapses to "str",
# enum-member type collapses to "int", .value / .name return
# None, len(enum-class) returns 5 instead of 3, hasattr(pprint,
# "PrettyPrinter") / "pp" / "saferepr" / "isreadable" /
# "isrecursive" all False on mamba, pprint.pformat([1,2,3])
# expands to multi-line on mamba, hasattr(textwrap,
# "TextWrapper") False on mamba, textwrap.dedent /
# .indent strip trailing newline, textwrap.wrap returns a
# single-element list instead of multi-line wrap — all
# DIVERGE on mamba — moved to the divergence-spec fixture.

print(f"MAMBA_ASSERTION_PASS: test_abc_copy_enum_pprint_textwrap_value_ops {sum(_ledger)} asserts")
