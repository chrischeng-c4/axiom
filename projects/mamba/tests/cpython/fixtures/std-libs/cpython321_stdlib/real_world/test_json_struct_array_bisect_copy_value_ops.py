# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_json_struct_array_bisect_copy_value_ops"
# subject = "cpython321.test_json_struct_array_bisect_copy_value_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_json_struct_array_bisect_copy_value_ops.py"
# status = "filled"
# ///
"""cpython321.test_json_struct_array_bisect_copy_value_ops: execute CPython 3.12 seed test_json_struct_array_bisect_copy_value_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the value contract of the
# `json` / `struct` / `array` / `bisect` / `copy` five-pack
# pinned to atomic 187: `json` (the documented full module-level
# helper hasattr surface — `dumps` / `loads` / `JSONDecoder` /
# `JSONEncoder` / `JSONDecodeError` / `load` / `dump` + the
# documented json.dumps / json.loads round-trip + the documented
# json.dumps indent / sort_keys / nested-list value contract),
# `struct` (the documented full module-level helper hasattr
# surface — `pack` / `unpack` / `calcsize` / `Struct` / `error` /
# `iter_unpack` / `pack_into` / `unpack_from` + the documented
# struct.pack / struct.unpack / struct.calcsize big-endian
# unsigned-int round-trip value contract), `array` (the
# documented partial module-level helper hasattr surface —
# `array` / `ArrayType` / `typecodes`), `bisect` (the documented
# full module-level helper hasattr surface — `bisect` /
# `bisect_left` / `bisect_right` / `insort` / `insort_left` /
# `insort_right` + the documented bisect.bisect_left / bisect_right
# / bisect value contract), and `copy` (the documented full
# module-level helper hasattr surface — `copy` / `deepcopy` /
# `Error` + the documented copy.deepcopy independence value
# contract).
#
# The matching subset between mamba and CPython is the full
# `json` module hasattr surface + json.dumps / json.loads value
# layer (including indent / sort_keys / nested-list emission),
# the full `struct` module hasattr surface + the struct.pack
# / unpack / calcsize value layer (the `struct.Struct` class
# instance .pack / .unpack methods DIVERGE), the partial `array`
# module hasattr surface (`array` / `ArrayType` / `typecodes` +
# the `.typecode` instance attribute — the class identity layer
# DIVERGES + the constructed-instance `.append` / `.tolist` /
# `.extend` / `.buffer_info` instance method layer DIVERGES +
# len + index access DIVERGE), the full `bisect` module hasattr
# surface + the bisect value layer, and the full `copy` module
# hasattr surface + the copy.copy / deepcopy independence value
# layer.
#
# Surface in this fixture:
#   • json — full module hasattr surface (dumps / loads /
#     JSONDecoder / JSONEncoder / JSONDecodeError / load /
#     dump);
#   • json.dumps / json.loads — round-trip value contract;
#   • json.dumps — indent / sort_keys / nested-list value
#     contract;
#   • struct — full module hasattr surface (pack / unpack /
#     calcsize / Struct / error / iter_unpack / pack_into /
#     unpack_from);
#   • struct.pack / unpack / calcsize — big-endian unsigned-int
#     value contract;
#   • array — partial module hasattr surface (array / ArrayType
#     / typecodes);
#   • array.array("i", [...]).typecode — typecode attribute
#     contract;
#   • bisect — full module hasattr surface (bisect /
#     bisect_left / bisect_right / insort / insort_left /
#     insort_right);
#   • bisect.bisect_left / bisect_right / bisect — index value
#     contract;
#   • copy — full module hasattr surface (copy / deepcopy /
#     Error);
#   • copy.deepcopy — nested-list independence value contract.
#
# Behavioral edges that DIVERGE on mamba
# (type(array.array("i", [1,2,3])).__name__ returns "int" not
# "array", len of constructed array is 0 not 3, indexed access
# returns None, hasattr(arr, "append") / "tolist" / "extend" /
# "buffer_info" all False on the constructed instance,
# struct.Struct(">I").pack(42) raises AttributeError because
# the instance lacks `.pack` / `.unpack` methods,
# type(json.JSONDecodeError).__name__ returns "str" not "type"
# — the exception class is rebound to a string placeholder)
# are covered in the matching spec fixture
# `lang_array_struct_json_silent`.
import json
import struct
import array
import bisect
import copy


_ledger: list[int] = []

# 1) json — full module hasattr surface
assert hasattr(json, "dumps") == True; _ledger.append(1)
assert hasattr(json, "loads") == True; _ledger.append(1)
assert hasattr(json, "JSONDecoder") == True; _ledger.append(1)
assert hasattr(json, "JSONEncoder") == True; _ledger.append(1)
assert hasattr(json, "JSONDecodeError") == True; _ledger.append(1)
assert hasattr(json, "load") == True; _ledger.append(1)
assert hasattr(json, "dump") == True; _ledger.append(1)

# 2) json.dumps / json.loads — round-trip value contract
assert json.dumps({"a": 1}) == '{"a": 1}'; _ledger.append(1)
assert json.loads('{"a": 1}') == {"a": 1}; _ledger.append(1)

# 3) json.dumps — indent / sort_keys / nested-list value contract
assert json.dumps({"a": 1}, indent=2) == '{\n  "a": 1\n}'; _ledger.append(1)
assert json.dumps({"b": 2, "a": 1}, sort_keys=True) == '{"a": 1, "b": 2}'; _ledger.append(1)
assert json.dumps([1, [2, [3]]]) == "[1, [2, [3]]]"; _ledger.append(1)

# 4) struct — full module hasattr surface
assert hasattr(struct, "pack") == True; _ledger.append(1)
assert hasattr(struct, "unpack") == True; _ledger.append(1)
assert hasattr(struct, "calcsize") == True; _ledger.append(1)
assert hasattr(struct, "Struct") == True; _ledger.append(1)
assert hasattr(struct, "error") == True; _ledger.append(1)
assert hasattr(struct, "iter_unpack") == True; _ledger.append(1)
assert hasattr(struct, "pack_into") == True; _ledger.append(1)
assert hasattr(struct, "unpack_from") == True; _ledger.append(1)

# 5) struct.pack / unpack / calcsize — big-endian unsigned-int value
assert struct.pack(">I", 42) == b'\x00\x00\x00\x2a'; _ledger.append(1)
assert struct.unpack(">I", b'\x00\x00\x00\x2a') == (42,); _ledger.append(1)
assert struct.calcsize(">I") == 4; _ledger.append(1)

# 6) array — partial module hasattr surface
#    (class identity + instance method layer DIVERGE — moved
#    to spec fixture)
assert hasattr(array, "array") == True; _ledger.append(1)
assert hasattr(array, "ArrayType") == True; _ledger.append(1)
assert hasattr(array, "typecodes") == True; _ledger.append(1)

# 7) array.array — typecode attribute contract
_arr = array.array("i", [1, 2, 3])
assert _arr.typecode == "i"; _ledger.append(1)

# 8) bisect — full module hasattr surface
assert hasattr(bisect, "bisect") == True; _ledger.append(1)
assert hasattr(bisect, "bisect_left") == True; _ledger.append(1)
assert hasattr(bisect, "bisect_right") == True; _ledger.append(1)
assert hasattr(bisect, "insort") == True; _ledger.append(1)
assert hasattr(bisect, "insort_left") == True; _ledger.append(1)
assert hasattr(bisect, "insort_right") == True; _ledger.append(1)

# 9) bisect — index value contract
assert bisect.bisect_left([1, 3, 5, 7], 4) == 2; _ledger.append(1)
assert bisect.bisect_right([1, 3, 5, 7], 5) == 3; _ledger.append(1)
assert bisect.bisect([1, 3, 5, 7], 4) == 2; _ledger.append(1)

# 10) copy — full module hasattr surface
assert hasattr(copy, "copy") == True; _ledger.append(1)
assert hasattr(copy, "deepcopy") == True; _ledger.append(1)
assert hasattr(copy, "Error") == True; _ledger.append(1)

# 11) copy.deepcopy — nested-list independence value contract
_src = [1, [2, 3]]
_dst = copy.deepcopy(_src)
_dst[1][0] = 99
assert _src[1][0] == 2; _ledger.append(1)
assert _dst[1][0] == 99; _ledger.append(1)
assert _src == [1, [2, 3]]; _ledger.append(1)

# NB: type(array.array("i", [...])).__name__ returns "int" on
# mamba, len of constructed array is 0 not 3, indexed access
# returns None, hasattr(arr, "append") / "tolist" / "extend" /
# "buffer_info" all False on the constructed instance,
# struct.Struct(">I").pack(42) raises AttributeError on mamba
# because the Struct instance lacks .pack / .unpack methods,
# type(json.JSONDecodeError).__name__ returns "str" on mamba —
# the exception class is rebound to a string placeholder — all
# DIVERGE on mamba — moved to the divergence-spec fixture.

print(f"MAMBA_ASSERTION_PASS: test_json_struct_array_bisect_copy_value_ops {sum(_ledger)} asserts")
