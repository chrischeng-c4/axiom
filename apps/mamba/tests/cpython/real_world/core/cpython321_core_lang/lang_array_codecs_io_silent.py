# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_array_codecs_io_silent"
# subject = "cpython321.lang_array_codecs_io_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_array_codecs_io_silent.py"
# status = "filled"
# ///
"""cpython321.lang_array_codecs_io_silent: execute CPython 3.12 seed lang_array_codecs_io_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass seed for SILENT divergences across
# the `array.array` instance method surface +
# `array.array` instance class identity contract +
# `codecs` extra module identifier surface + `codecs.encode`
# value contract + `io` module identifier surface +
# `io.StringIO` / `io.BytesIO` instance class identity
# contract pinned by atomic 206: `array.array` (the
# documented instance-level method / attribute identifier
# hasattr surface — `append` / `buffer_info` / `byteswap`
# / `count` / `extend` / `frombytes` / `fromlist` /
# `index` / `insert` / `pop` / `remove` / `reverse` /
# `tobytes` / `tolist` / `typecode` / `itemsize` + the
# documented `type(array.array("i", [1, 2, 3, 4]))
# .__name__ == "array"` class-identity contract —
# mamba collapses to "int"), `codecs` (the documented
# extra helper / class identifier surface —
# `unregister` / `CodecInfo` / `make_identity_dict` /
# `make_encoding_map` + the documented
# `codecs.encode("abc", "rot13") == "nop"`
# transformation value contract — mamba leaves
# bytes echo of the input), and `io` (the documented
# class / sentinel identifier hasattr surface —
# `TextIOWrapper` / `BufferedReader` / `BufferedWriter`
# / `BufferedRandom` / `FileIO` / `RawIOBase` /
# `BufferedIOBase` / `TextIOBase` / `IOBase` / `open`
# / `UnsupportedOperation` / `DEFAULT_BUFFER_SIZE` /
# `SEEK_SET` / `SEEK_CUR` / `SEEK_END` + the documented
# `type(io.StringIO()).__name__ == "StringIO"` /
# `type(io.BytesIO()).__name__ == "BytesIO"` instance
# class identity contract — mamba collapses both to
# "dict").
#
# The matching subset (full bisect hasattr +
# binary-search/sorted-insert value contract, full
# heapq hasattr + min-heap value contract, full
# struct hasattr + pack/unpack/calcsize value
# contract, full errno hasattr + integer-constant
# value contract, full stat hasattr +
# integer-constant value contract) is covered by
# `test_bisect_heapq_struct_errno_stat_value_ops`;
# this fixture pins the CPython-only contracts that
# mamba currently elides.
#
# Surface (CPython AssertionPass; mamba diverges silently):
#   • hasattr(array.array("i", [1,2,3,4]), "append") is
#     True — documented instance method (mamba: False);
#   • hasattr(array.array("i", [1,2,3,4]), "buffer_info") is
#     True — documented instance method (mamba: False);
#   • hasattr(array.array("i", [1,2,3,4]), "byteswap") is
#     True — documented instance method (mamba: False);
#   • hasattr(array.array("i", [1,2,3,4]), "count") is
#     True — documented instance method (mamba: False);
#   • hasattr(array.array("i", [1,2,3,4]), "extend") is
#     True — documented instance method (mamba: False);
#   • hasattr(array.array("i", [1,2,3,4]), "frombytes") is
#     True — documented instance method (mamba: False);
#   • hasattr(array.array("i", [1,2,3,4]), "fromlist") is
#     True — documented instance method (mamba: False);
#   • hasattr(array.array("i", [1,2,3,4]), "index") is
#     True — documented instance method (mamba: False);
#   • hasattr(array.array("i", [1,2,3,4]), "insert") is
#     True — documented instance method (mamba: False);
#   • hasattr(array.array("i", [1,2,3,4]), "pop") is
#     True — documented instance method (mamba: False);
#   • hasattr(array.array("i", [1,2,3,4]), "remove") is
#     True — documented instance method (mamba: False);
#   • hasattr(array.array("i", [1,2,3,4]), "reverse") is
#     True — documented instance method (mamba: False);
#   • hasattr(array.array("i", [1,2,3,4]), "tobytes") is
#     True — documented instance method (mamba: False);
#   • hasattr(array.array("i", [1,2,3,4]), "tolist") is
#     True — documented instance method (mamba: False);
#   • hasattr(array.array("i", [1,2,3,4]), "typecode") is
#     True — documented instance attribute (mamba: False);
#   • hasattr(array.array("i", [1,2,3,4]), "itemsize") is
#     True — documented instance attribute (mamba: False);
#   • type(array.array("i", [1,2,3,4])).__name__ ==
#     "array" — documented class-identity contract
#     (mamba: "int");
#   • hasattr(codecs, "unregister") is True — documented
#     helper identifier (mamba: False);
#   • hasattr(codecs, "CodecInfo") is True — documented
#     class identifier (mamba: False);
#   • hasattr(codecs, "make_identity_dict") is True —
#     documented helper identifier (mamba: False);
#   • hasattr(codecs, "make_encoding_map") is True —
#     documented helper identifier (mamba: False);
#   • codecs.encode("abc", "rot13") == "nop" —
#     documented transformation value (mamba: False);
#   • hasattr(io, "TextIOWrapper") is True — documented
#     class identifier (mamba: False);
#   • hasattr(io, "BufferedReader") is True — documented
#     class identifier (mamba: False);
#   • hasattr(io, "BufferedWriter") is True — documented
#     class identifier (mamba: False);
#   • hasattr(io, "BufferedRandom") is True —
#     documented class identifier (mamba: False);
#   • hasattr(io, "FileIO") is True — documented
#     class identifier (mamba: False);
#   • hasattr(io, "RawIOBase") is True — documented
#     class identifier (mamba: False);
#   • hasattr(io, "BufferedIOBase") is True —
#     documented class identifier (mamba: False);
#   • hasattr(io, "TextIOBase") is True — documented
#     class identifier (mamba: False);
#   • hasattr(io, "IOBase") is True — documented
#     class identifier (mamba: False);
#   • hasattr(io, "open") is True — documented helper
#     identifier (mamba: False);
#   • hasattr(io, "UnsupportedOperation") is True —
#     documented exception identifier (mamba: False);
#   • hasattr(io, "DEFAULT_BUFFER_SIZE") is True —
#     documented sentinel identifier (mamba: False);
#   • hasattr(io, "SEEK_SET") is True — documented
#     sentinel identifier (mamba: False);
#   • hasattr(io, "SEEK_CUR") is True — documented
#     sentinel identifier (mamba: False);
#   • hasattr(io, "SEEK_END") is True — documented
#     sentinel identifier (mamba: False);
#   • type(io.StringIO()).__name__ == "StringIO" —
#     documented class-identity contract
#     (mamba: "dict");
#   • type(io.BytesIO()).__name__ == "BytesIO" —
#     documented class-identity contract
#     (mamba: "dict").
import array as _array_mod
import codecs as _codecs_mod
import io as _io_mod
from typing import Any

# Module bindings retyped as `Any` to bypass Pyright stub-driven
# narrowing — every spec contract below probes documented public
# class identity / module-attribute / instance-method
# identifier behavior that mamba's bundled type stubs do not
# surface accurately.
array: Any = _array_mod
codecs: Any = _codecs_mod
io: Any = _io_mod


_ledger: list[int] = []

# 1) array.array — instance method / attribute identifier surface
_a = array.array("i", [1, 2, 3, 4])
assert hasattr(_a, "append") == True; _ledger.append(1)
assert hasattr(_a, "buffer_info") == True; _ledger.append(1)
assert hasattr(_a, "byteswap") == True; _ledger.append(1)
assert hasattr(_a, "count") == True; _ledger.append(1)
assert hasattr(_a, "extend") == True; _ledger.append(1)
assert hasattr(_a, "frombytes") == True; _ledger.append(1)
assert hasattr(_a, "fromlist") == True; _ledger.append(1)
assert hasattr(_a, "index") == True; _ledger.append(1)
assert hasattr(_a, "insert") == True; _ledger.append(1)
assert hasattr(_a, "pop") == True; _ledger.append(1)
assert hasattr(_a, "remove") == True; _ledger.append(1)
assert hasattr(_a, "reverse") == True; _ledger.append(1)
assert hasattr(_a, "tobytes") == True; _ledger.append(1)
assert hasattr(_a, "tolist") == True; _ledger.append(1)
assert hasattr(_a, "typecode") == True; _ledger.append(1)
assert hasattr(_a, "itemsize") == True; _ledger.append(1)

# 2) array.array — class-identity contract
assert type(_a).__name__ == "array"; _ledger.append(1)

# 3) codecs — extra helper / class identifier surface
assert hasattr(codecs, "unregister") == True; _ledger.append(1)
assert hasattr(codecs, "CodecInfo") == True; _ledger.append(1)
assert hasattr(codecs, "make_identity_dict") == True; _ledger.append(1)
assert hasattr(codecs, "make_encoding_map") == True; _ledger.append(1)

# 4) codecs.encode — transformation value contract
assert codecs.encode("abc", "rot13") == "nop"; _ledger.append(1)

# 5) io — class / sentinel identifier surface
assert hasattr(io, "TextIOWrapper") == True; _ledger.append(1)
assert hasattr(io, "BufferedReader") == True; _ledger.append(1)
assert hasattr(io, "BufferedWriter") == True; _ledger.append(1)
assert hasattr(io, "BufferedRandom") == True; _ledger.append(1)
assert hasattr(io, "FileIO") == True; _ledger.append(1)
assert hasattr(io, "RawIOBase") == True; _ledger.append(1)
assert hasattr(io, "BufferedIOBase") == True; _ledger.append(1)
assert hasattr(io, "TextIOBase") == True; _ledger.append(1)
assert hasattr(io, "IOBase") == True; _ledger.append(1)
assert hasattr(io, "open") == True; _ledger.append(1)
assert hasattr(io, "UnsupportedOperation") == True; _ledger.append(1)
assert hasattr(io, "DEFAULT_BUFFER_SIZE") == True; _ledger.append(1)
assert hasattr(io, "SEEK_SET") == True; _ledger.append(1)
assert hasattr(io, "SEEK_CUR") == True; _ledger.append(1)
assert hasattr(io, "SEEK_END") == True; _ledger.append(1)

# 6) io.StringIO / io.BytesIO — class-identity contract
assert type(io.StringIO()).__name__ == "StringIO"; _ledger.append(1)
assert type(io.BytesIO()).__name__ == "BytesIO"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_array_codecs_io_silent {sum(_ledger)} asserts")
