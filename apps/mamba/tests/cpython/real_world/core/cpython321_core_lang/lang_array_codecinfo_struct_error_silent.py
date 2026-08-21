# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_array_codecinfo_struct_error_silent"
# subject = "cpython321.lang_array_codecinfo_struct_error_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_array_codecinfo_struct_error_silent.py"
# status = "filled"
# ///
"""cpython321.lang_array_codecinfo_struct_error_silent: execute CPython 3.12 seed lang_array_codecinfo_struct_error_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass seed for SILENT divergences in `array`
# (array.array() returning a real array instance — len / subscript /
# iteration / list-conversion all reproduce the input sequence),
# `codecs` (codecs.lookup returning a `CodecInfo`-typed object with
# the documented `.name == "utf-8"` attribute, not a bare 7-tuple
# with `.name == None`), and `struct` (struct.error being a type
# (class), not an exception instance — i.e. type(struct.error) is
# `type`).
#
# The matching subset (bisect.bisect_left / bisect_right / insort,
# struct.pack / unpack / calcsize byte-exact, struct.error class
# name identity, codecs.encode / decode utf-8 round-trip, BOM
# constants byte-exact, array.typecodes / array.array.typecode /
# itemsize attribute access) is covered by
# `test_bisect_struct_codecs_value_ops`; this fixture pins the
# CPython-only contracts that mamba currently elides.
#
# Surface (CPython AssertionPass; mamba diverges silently):
#   • type(array.array('i', [1, 2, 3])).__name__ == "array" — the
#     constructor returns an instance, not an integer handle
#     (mamba: returns the int handle 17592186044416 — only typecode
#     and itemsize survive on the stub);
#   • len(array.array('i', [1, 2, 3])) == 3 — array carries the
#     input length (mamba: returns 0);
#   • array.array('i', [1, 2, 3])[0] == 1 — subscript returns the
#     stored value (mamba: returns None);
#   • list(array.array('i', [1, 2, 3])) == [1, 2, 3] — array is
#     iterable as the input sequence (mamba: raises TypeError
#     "object is not iterable", list() returns []);
#   • array.array('d', [1.5, 2.5])[1] == 2.5 — floating-point
#     storage (mamba: returns None);
#   • codecs.lookup('utf-8').name == "utf-8" — CodecInfo carries
#     the codec name (mamba: returns the bare tuple
#     ('utf-8', None, None, None, None, None, None) where .name
#     resolves to None);
#   • type(codecs.lookup('utf-8')).__name__ == "CodecInfo" — class
#     identity of the lookup result (mamba: returns "tuple");
#   • type(struct.error).__name__ == "type" — struct.error is a
#     class object (mamba: returns "error" — the binding is an
#     instance, not the class itself).
import array as _array_mod
import codecs as _codecs_mod
import struct as _struct_mod
from typing import Any

# Module bindings retyped as `Any` to bypass Pyright stub-driven
# narrowing — every spec contract below probes the runtime-level
# constructor / class / lookup-result surface that mamba's bundled
# type stubs do not surface accurately.
array: Any = _array_mod
codecs: Any = _codecs_mod
struct: Any = _struct_mod

_ledger: list[int] = []

# 1) array.array — constructor returns a real array instance
_ai: Any = array.array('i', [1, 2, 3])
assert type(_ai).__name__ == "array"; _ledger.append(1)

# 2) array.array — length carries the input length
assert len(_ai) == 3; _ledger.append(1)

# 3) array.array — subscript returns the stored value
assert _ai[0] == 1; _ledger.append(1)
assert _ai[1] == 2; _ledger.append(1)
assert _ai[2] == 3; _ledger.append(1)

# 4) array.array — iterable as the input sequence
assert list(_ai) == [1, 2, 3]; _ledger.append(1)

# 5) array.array — floating-point storage round-trip
_ad: Any = array.array('d', [1.5, 2.5, 3.5])
assert type(_ad).__name__ == "array"; _ledger.append(1)
assert len(_ad) == 3; _ledger.append(1)
assert _ad[1] == 2.5; _ledger.append(1)
assert list(_ad) == [1.5, 2.5, 3.5]; _ledger.append(1)

# 6) array.array — byte storage round-trip
_ab: Any = array.array('B', [10, 20, 30])
assert type(_ab).__name__ == "array"; _ledger.append(1)
assert len(_ab) == 3; _ledger.append(1)
assert _ab[2] == 30; _ledger.append(1)

# 7) codecs.lookup — CodecInfo class identity
_ci: Any = codecs.lookup('utf-8')
assert type(_ci).__name__ == "CodecInfo"; _ledger.append(1)

# 8) codecs.lookup — .name attribute carries the codec name
assert _ci.name == "utf-8"; _ledger.append(1)

# 9) codecs.lookup — multiple codecs all carry the right name
assert codecs.lookup('ascii').name == "ascii"; _ledger.append(1)
assert codecs.lookup('latin-1').name == "iso8859-1"; _ledger.append(1)

# 10) struct.error — type identity (class, not instance)
assert type(struct.error).__name__ == "type"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_array_codecinfo_struct_error_silent {sum(_ledger)} asserts")
