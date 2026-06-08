# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_array_struct_json_silent"
# subject = "cpython321.lang_array_struct_json_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_array_struct_json_silent.py"
# status = "filled"
# ///
"""cpython321.lang_array_struct_json_silent: execute CPython 3.12 seed lang_array_struct_json_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass seed for SILENT divergences across the
# array.array constructed-instance class-identity / length / index
# / instance-method contract + struct.Struct instance-method
# (.pack / .unpack) contract + json.JSONDecodeError class-identity
# contract pinned by atomic 187: `array` (the documented
# `type(array.array("i", [1,2,3])).__name__ == "array"` class-
# identity contract + the documented `len(...) == 3` length
# contract + the documented `[0] == 1` indexed-access contract +
# the documented `.append` / `.tolist` / `.extend` /
# `.buffer_info` instance-method identifier surface), `struct`
# (the documented `struct.Struct(">I").pack(42)` /
# `struct.Struct(">I").unpack(...)` instance-method layer + the
# documented `.size` attribute layer), and `json` (the documented
# `type(json.JSONDecodeError).__name__ == "type"` class-identity
# contract for the JSONDecodeError exception).
#
# The matching subset (full json module hasattr + dumps/loads
# round-trip + indent/sort_keys/nested-list value, full struct
# module hasattr + pack/unpack/calcsize value, partial array
# module hasattr + .typecode attribute, full bisect module
# hasattr + value, full copy module hasattr + deepcopy
# independence) is covered by
# `test_json_struct_array_bisect_copy_value_ops`; this fixture
# pins the CPython-only contracts that mamba currently elides.
#
# Surface (CPython AssertionPass; mamba diverges silently):
#   • type(array.array("i", [1, 2, 3])).__name__ == "array" —
#     documented class identity (mamba: returns "int" — the
#     constructor short-circuits to a placeholder integer);
#   • len(array.array("i", [1, 2, 3])) == 3 — documented
#     length contract (mamba: returns 0);
#   • array.array("i", [1, 2, 3])[0] == 1 — documented
#     indexed-access contract (mamba: returns None);
#   • hasattr(array.array("i", []), "append") is True —
#     documented instance-method identifier (mamba: False);
#   • hasattr(array.array("i", []), "tolist") is True —
#     documented instance-method identifier (mamba: False);
#   • hasattr(array.array("i", []), "extend") is True —
#     documented instance-method identifier (mamba: False);
#   • hasattr(array.array("i", []), "buffer_info") is True —
#     documented instance-method identifier (mamba: False);
#   • hasattr(struct.Struct(">I"), "pack") is True —
#     documented instance-method identifier (mamba: False);
#   • hasattr(struct.Struct(">I"), "unpack") is True —
#     documented instance-method identifier (mamba: False);
#   • struct.Struct(">I").pack(42) == b'\x00\x00\x00\x2a' —
#     documented instance-method value contract (mamba: raises
#     AttributeError on .pack because the Struct instance
#     lacks the bound method);
#   • type(json.JSONDecodeError).__name__ == "type" —
#     documented class-identity contract for the JSONDecodeError
#     exception class (mamba: returns "str" — the exception
#     class is rebound to a string placeholder).
import array as _array_mod
import struct as _struct_mod
import json as _json_mod
from typing import Any

# Module bindings retyped as `Any` to bypass Pyright stub-driven
# narrowing — every spec contract below probes documented public
# class identifiers / instance-method / value-contract behavior
# that mamba's bundled type stubs do not surface accurately.
array: Any = _array_mod
struct: Any = _struct_mod
json: Any = _json_mod


_ledger: list[int] = []

# 1) array.array — class identity + length + index contract
_arr = array.array("i", [1, 2, 3])
assert type(_arr).__name__ == "array"; _ledger.append(1)
assert len(_arr) == 3; _ledger.append(1)
assert _arr[0] == 1; _ledger.append(1)

# 2) array.array — instance-method identifier surface
assert hasattr(_arr, "append") == True; _ledger.append(1)
assert hasattr(_arr, "tolist") == True; _ledger.append(1)
assert hasattr(_arr, "extend") == True; _ledger.append(1)
assert hasattr(_arr, "buffer_info") == True; _ledger.append(1)

# 3) struct.Struct — instance-method identifier surface
_s = struct.Struct(">I")
assert hasattr(_s, "pack") == True; _ledger.append(1)
assert hasattr(_s, "unpack") == True; _ledger.append(1)

# 4) struct.Struct — instance-method value contract
assert _s.pack(42) == b'\x00\x00\x00\x2a'; _ledger.append(1)
assert _s.unpack(b'\x00\x00\x00\x2a') == (42,); _ledger.append(1)

# 5) json.JSONDecodeError — class-identity contract
assert type(json.JSONDecodeError).__name__ == "type"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_array_struct_json_silent {sum(_ledger)} asserts")
