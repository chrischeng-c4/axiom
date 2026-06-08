# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_pickle_struct_itertools_inspect_difflib_enum_silent"
# subject = "cpython321.lang_pickle_struct_itertools_inspect_difflib_enum_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_pickle_struct_itertools_inspect_difflib_enum_silent.py"
# status = "filled"
# ///
"""cpython321.lang_pickle_struct_itertools_inspect_difflib_enum_silent: execute CPython 3.12 seed lang_pickle_struct_itertools_inspect_difflib_enum_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass divergence-spec fixture for the
# silent value-contract divergence of the
# `pickle.dumps` protocol byte format / `pickle.DEFAULT_PROTOCOL` /
# `struct.pack_into` / `struct.unpack_from` offset arg /
# `itertools.accumulate` initial+func args /
# `functools.cmp_to_key` sort key effect /
# `dis.Bytecode` / `inspect.Parameter / Signature /
# isfunction / signature(fn) render` / `difflib.SequenceMatcher`
# instance ops / class-form `enum.IntEnum` instance value
# contract / `enum.auto()` ten-pack pinned to atomic 244:
# `pickle.dumps({"a": 1}, protocol=0)` (the documented "pickle
# protocol 0 produces the ASCII pickle format starting with
# `(dp0\nVa\np1\nI1\ns.`" value contract — mamba silently
# returns the bespoke `D1;S1:a;I1` format regardless of the
# requested protocol), `pickle.DEFAULT_PROTOCOL` (the
# documented "4" default — mamba returns 5),
# `struct.pack_into("i", bytearray(4), 0, 42)` (the documented
# "write to the in-memory buffer in place" value contract —
# mamba silently returns an empty bytearray after pack_into),
# `struct.unpack_from("i", buf, offset=4)` (the documented
# "read from the buffer at the given byte offset" value
# contract — mamba silently ignores the offset and returns
# the first element), `itertools.accumulate([1, 2, 3], initial=10)`
# (the documented "seed the accumulator with the initial
# value" value contract — mamba silently returns `[1, None,
# None]`) and `itertools.accumulate([3, 1, 4, 1, 5, 9, 2, 6], max)`
# (the documented "use the binary func to fold" value contract —
# mamba silently emits `[3, None, None, None, ...]`),
# `sorted([3, 1, 2], key=functools.cmp_to_key(lambda a, b: a - b))`
# (the documented "cmp_to_key converts a comparator into a
# sort key" value contract — mamba silently returns the
# unsorted input), `dis.Bytecode` (the documented top-level
# class — mamba does not expose it), `inspect.Parameter /
# inspect.Signature` (the documented introspection class
# surface — mamba does not expose them) and
# `inspect.signature(fn).__str__() == "(a, b=2)"` and
# `inspect.isfunction(fn)` (the documented value contracts —
# mamba renders signature as `"()"` and reports isfunction as
# False for plain user-defined functions),
# `difflib.SequenceMatcher(None, a, b)` (the documented
# "constructor returns a SequenceMatcher instance with
# .ratio/.get_matching_blocks/.quick_ratio methods" value
# contract — mamba's constructor silently returns a bare
# float and every documented instance method raises
# AttributeError at the call site), class-form
# `enum.IntEnum`-derived `_Color.A` (the documented "member
# is the enum instance with .name / .value attrs and the
# class is iterable yielding the canonical members" value
# contract — mamba's class-form enum binds members to raw
# ints, `.name` and `.value` resolve to None, `__members__`
# is None, and iteration yields more entries than declared),
# and `enum.auto()` (the documented "auto() returns an enum
# value-placeholder sentinel" value contract — mamba raises
# AttributeError 'dict' object has no attribute 'auto' at
# the call site).
#
# Behavioral edges that CONFORM on mamba (pickle.HIGHEST_PROTOCOL
# constant + round-trip int/str/list/dict/tuple; struct
# pack/unpack/calcsize 3i/calcsize 2d/iter_unpack; itertools
# accumulate default sum + lambda mul; functools 3 hasattr;
# dis 5 hasattr + get_instructions callable; inspect signature/
# isclass/ismethod/getmembers hasattr; heapq nlargest/
# nsmallest basic; random choice/randint/randrange/random/
# sample/shuffle non-seeded; enum unique/Enum/IntEnum/Flag/
# IntFlag/auto hasattr surface) are covered in the matching
# pass fixture
# `test_pickle_struct_itertools_dis_heapq_random_enum_unique_value_ops`.
from typing import Any
import pickle as _pickle_mod
import struct as _struct_mod
import itertools as _itertools_mod
import functools as _functools_mod
import dis as _dis_mod
import inspect as _inspect_mod
import difflib as _difflib_mod
import enum as _enum_mod

pickle_mod: Any = _pickle_mod
struct_mod: Any = _struct_mod
it_mod: Any = _itertools_mod
functools_mod: Any = _functools_mod
dis_mod: Any = _dis_mod
inspect_mod: Any = _inspect_mod
difflib_mod: Any = _difflib_mod
enum_mod: Any = _enum_mod


class _Color(_enum_mod.IntEnum):
    A = 1
    B = 2
    C = 3


def _myfn(a, b=2):
    return a + b


_ledger: list[int] = []

# 1) pickle.dumps protocol byte format
#    (mamba: returns bespoke `D1;S1:a;I1` regardless of protocol)
assert pickle_mod.dumps({"a": 1}, protocol=0) == b"(dp0\nVa\np1\nI1\ns."; _ledger.append(1)

# 2) pickle.DEFAULT_PROTOCOL — documented "4"
#    (mamba: returns 5)
assert pickle_mod.DEFAULT_PROTOCOL == 4; _ledger.append(1)

# 3) struct.pack_into — in-place buffer write
#    (mamba: silently returns empty bytearray)
_buf = bytearray(4)
struct_mod.pack_into("i", _buf, 0, 42)
assert bytes(_buf) == b"*\x00\x00\x00"; _ledger.append(1)

# 4) struct.unpack_from offset arg
#    (mamba: silently ignores offset, returns first element)
assert struct_mod.unpack_from("i", struct_mod.pack("3i", 10, 20, 30), offset=4) == (20,); _ledger.append(1)

# 5) itertools.accumulate initial arg
#    (mamba: silently returns [1, None, None])
assert list(it_mod.accumulate([1, 2, 3], initial=10)) == [10, 11, 13, 16]; _ledger.append(1)

# 6) itertools.accumulate with binary func
#    (mamba: silently emits [3, None, None, None, ...])
assert list(it_mod.accumulate([3, 1, 4, 1, 5, 9, 2, 6], max)) == [3, 3, 4, 4, 5, 9, 9, 9]; _ledger.append(1)

# 7) functools.cmp_to_key sort key effect
#    (mamba: silently returns unsorted input)
assert sorted([3, 1, 2], key=functools_mod.cmp_to_key(lambda a, b: a - b)) == [1, 2, 3]; _ledger.append(1)

# 8) dis.Bytecode — top-level class
#    (mamba: missing)
assert hasattr(dis_mod, "Bytecode") == True; _ledger.append(1)

# 9) inspect.Parameter / Signature
#    (mamba: missing)
assert hasattr(inspect_mod, "Parameter") == True; _ledger.append(1)
assert hasattr(inspect_mod, "Signature") == True; _ledger.append(1)

# 10) inspect.signature render + isfunction
#     (mamba: signature renders '()' and isfunction reports False)
assert str(inspect_mod.signature(_myfn)) == "(a, b=2)"; _ledger.append(1)
assert inspect_mod.isfunction(_myfn) == True; _ledger.append(1)

# 11) difflib.SequenceMatcher instance ops
#     (mamba: constructor silently returns float — methods AttributeError)
_sm = difflib_mod.SequenceMatcher(None, "abc", "abd")
assert type(_sm).__name__ == "SequenceMatcher"; _ledger.append(1)
assert _sm.ratio() == 0.6666666666666666; _ledger.append(1)
assert _sm.quick_ratio() == 0.6666666666666666; _ledger.append(1)

# 12) class-form enum.IntEnum instance value contract
#     (mamba: members bind raw ints, .name/.value = None, __members__ = None)
assert _Color.A.name == "A"; _ledger.append(1)
assert _Color.A.value == 1; _ledger.append(1)
assert list(_Color.__members__.keys()) == ["A", "B", "C"]; _ledger.append(1)

# 13) enum.auto() call site
#     (mamba: raises AttributeError 'dict' object has no attribute 'auto')
assert type(enum_mod.auto()).__name__ == "auto"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_pickle_struct_itertools_inspect_difflib_enum_silent {sum(_ledger)} asserts")
