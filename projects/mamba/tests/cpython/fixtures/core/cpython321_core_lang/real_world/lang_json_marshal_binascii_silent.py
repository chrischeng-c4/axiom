# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_json_marshal_binascii_silent"
# subject = "cpython321.lang_json_marshal_binascii_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_json_marshal_binascii_silent.py"
# status = "filled"
# ///
"""cpython321.lang_json_marshal_binascii_silent: execute CPython 3.12 seed lang_json_marshal_binascii_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass divergence-spec fixture for the
# silent value-contract divergence of the `json` / `marshal`
# / `binascii` three-pack pinned to atomic 210: `json` (the
# documented `hasattr(json, "encoder") == True` /
# `hasattr(json, "decoder") == True` sub-module hasattr
# surface + the documented
# `type(json.JSONEncoder()).__name__ == "JSONEncoder"` /
# `hasattr(json.JSONEncoder(), "encode") == True` /
# `json.JSONEncoder().encode("abc") == '"abc"'` encoder-
# class-identity value contract + the documented
# `json.dumps("é") == '"\\u00e9"'` default-ensure-ascii
# value contract), `marshal` (the documented
# `type(marshal.dumps(data)).__name__ == "bytes"` /
# `marshal.loads(marshal.dumps(data)) == data` round-trip
# value contract), and `binascii` (the documented
# `hasattr(binascii, "a2b_qp") / "b2a_qp" / "a2b_uu" /
# "b2a_uu" / "crc32" / "crc_hqx" / "Error" / "Incomplete"
# == True` extended hasattr surface + the documented
# `binascii.crc32(b"hello") == 907060870` /
# `type(binascii.crc32(b"hello")).__name__ == "int"`
# crc32 value contract).
#
# Behavioral edges that CONFORM on mamba
# (json `dump` / `dumps` / `load` / `loads` / `JSONEncoder`
# / `JSONDecoder` / `JSONDecodeError` hasattr surface +
# `type(json.dumps(...)).__name__ == "str"` /
# `json.loads(json.dumps(data)) == data` /
# `json.dumps({"a": 1}) == '{"a": 1}'` /
# `json.loads("[1,2,3]") == [1, 2, 3]` /
# `json.loads("true") == True` /
# `json.loads("null") is None` round-trip / dump-formatting
# value contract, marshal `dump` / `dumps` / `load` /
# `loads` / `version` hasattr surface + `marshal.version
# > 0` integer-sentinel value contract, binascii `hexlify`
# / `unhexlify` / `a2b_base64` / `b2a_base64` / `a2b_hex`
# / `b2a_hex` hasattr surface + `hexlify(b"\xab\xcd") ==
# b"abcd"` / `unhexlify("abcd") == b"\xab\xcd"` hex-codec
# value contract) are covered in the matching pass
# fixture `test_json_pickle_marshal_base64_binascii_value_ops`.
from typing import Any
import json as _json_mod
import marshal as _marshal_mod
import binascii as _binascii_mod

json: Any = _json_mod
marshal: Any = _marshal_mod
binascii: Any = _binascii_mod


_ledger: list[int] = []

# 1) json — sub-module hasattr surface
#    (mamba: hasattr(json, "encoder") / "decoder" both False)
assert hasattr(json, "encoder") == True; _ledger.append(1)
assert hasattr(json, "decoder") == True; _ledger.append(1)

# 2) json — JSONEncoder class-identity value contract
#    (mamba: type(json.JSONEncoder()).__name__ collapses to
#    "int" + JSONEncoder() has no `encode` method)
_je = json.JSONEncoder()
assert type(_je).__name__ == "JSONEncoder"; _ledger.append(1)
assert hasattr(_je, "encode") == True; _ledger.append(1)
assert _je.encode("abc") == '"abc"'; _ledger.append(1)

# 3) json — default ensure_ascii value contract
#    (mamba: json.dumps("é") collapses to '"é"' — default
#    ensure_ascii=True ignored)
assert json.dumps("é") == '"\\u00e9"'; _ledger.append(1)

# 4) marshal — bytes round-trip value contract
#    (mamba: type(marshal.dumps(data)).__name__ collapses
#    to "str" + marshal.loads(marshal.dumps(data)) round-
#    trip equality False)
_mdata = [1, 2, 3, "hi"]
_mm = marshal.dumps(_mdata)
assert type(_mm).__name__ == "bytes"; _ledger.append(1)
assert marshal.loads(_mm) == _mdata; _ledger.append(1)

# 5) binascii — extended module hasattr surface
#    (mamba: a2b_qp / b2a_qp / a2b_uu / b2a_uu / crc32 /
#    crc_hqx / Error / Incomplete all False)
assert hasattr(binascii, "a2b_qp") == True; _ledger.append(1)
assert hasattr(binascii, "b2a_qp") == True; _ledger.append(1)
assert hasattr(binascii, "a2b_uu") == True; _ledger.append(1)
assert hasattr(binascii, "b2a_uu") == True; _ledger.append(1)
assert hasattr(binascii, "crc32") == True; _ledger.append(1)
assert hasattr(binascii, "crc_hqx") == True; _ledger.append(1)
assert hasattr(binascii, "Error") == True; _ledger.append(1)
assert hasattr(binascii, "Incomplete") == True; _ledger.append(1)

# 6) binascii — crc32 value contract
#    (mamba: binascii.crc32(b"hello") unavailable — module
#    attribute lookup raises AttributeError)
assert binascii.crc32(b"hello") == 907060870; _ledger.append(1)
assert type(binascii.crc32(b"hello")).__name__ == "int"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_json_marshal_binascii_silent {sum(_ledger)} asserts")
