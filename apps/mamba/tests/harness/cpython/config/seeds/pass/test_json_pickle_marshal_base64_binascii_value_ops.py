# Operational AssertionPass seed for the value contract of the
# `json` / `pickle` / `marshal` / `base64` / `binascii`
# five-pack pinned to atomic 210: `json` (the documented
# full module-level helper / class / exception identifier
# hasattr surface — `dump` / `dumps` / `load` / `loads` /
# `JSONEncoder` / `JSONDecoder` / `JSONDecodeError` + the
# documented `type(json.dumps(...)).__name__ == "str"` /
# `json.loads(json.dumps(data)) == data` /
# `json.dumps({"a": 1}) == '{"a": 1}'` /
# `json.loads("[1,2,3]") == [1, 2, 3]` /
# `json.loads("true") == True` /
# `json.loads("null") is None` /
# `json.dumps({"a": 1}, indent=2).startswith("{")` /
# `json.dumps({"b": 2, "a": 1}, sort_keys=True) ==
# '{"a": 1, "b": 2}'` / `json.dumps({"a": 1, "b": 2},
# separators=(",", ":")) == '{"a":1,"b":2}'` round-trip
# / dump-formatting value contract), `pickle` (the
# documented partial module-level helper / class /
# exception / sentinel identifier hasattr surface —
# `dump` / `dumps` / `load` / `loads` / `Pickler` /
# `Unpickler` / `PicklingError` / `UnpicklingError` /
# `PickleError` / `DEFAULT_PROTOCOL` /
# `HIGHEST_PROTOCOL` / `STOP` + the documented
# `type(pickle.dumps(data)).__name__ == "bytes"` /
# `pickle.loads(pickle.dumps(data)) == data` /
# `pickle.HIGHEST_PROTOCOL >= 4` /
# `pickle.DEFAULT_PROTOCOL >= 0` round-trip /
# protocol-sentinel value contract), `marshal` (the
# documented full module-level helper / sentinel
# identifier hasattr surface — `dump` / `dumps` /
# `load` / `loads` / `version` + the documented
# `marshal.version > 0` integer-sentinel value
# contract), `base64` (the documented full
# module-level helper identifier hasattr surface —
# `b64encode` / `b64decode` / `standard_b64encode` /
# `standard_b64decode` / `urlsafe_b64encode` /
# `urlsafe_b64decode` / `b32encode` / `b32decode` /
# `b32hexencode` / `b32hexdecode` / `b16encode` /
# `b16decode` / `a85encode` / `a85decode` /
# `b85encode` / `b85decode` / `encode` / `decode` /
# `encodebytes` / `decodebytes` + the documented
# `b64encode(b"hello") == b"aGVsbG8="` /
# `b64decode(b"aGVsbG8=") == b"hello"` /
# `b32encode(b"hello") == b"NBSWY3DP"` /
# `b16encode(b"hello") == b"68656C6C6F"` /
# `urlsafe_b64encode(b"hello") == b"aGVsbG8="`
# base64-codec value contract), and `binascii` (the
# documented partial module-level helper identifier
# hasattr surface — `hexlify` / `unhexlify` /
# `a2b_base64` / `b2a_base64` / `a2b_hex` /
# `b2a_hex` + the documented
# `hexlify(b"\xab\xcd") == b"abcd"` /
# `unhexlify("abcd") == b"\xab\xcd"` hex-codec
# value contract).
#
# Behavioral edges that DIVERGE on mamba
# (hasattr(json, "encoder") / "decoder" False on mamba
# + type(json.JSONEncoder()).__name__ == "JSONEncoder"
# collapses to "int" on mamba +
# json.JSONEncoder().encode method unavailable on
# mamba + json.dumps("é") == "\"\\u00e9\""
# default-ensure-ascii collapses to '"é"' on mamba,
# type(marshal.dumps(data)).__name__ == "bytes"
# collapses to "str" on mamba + marshal.loads(marshal
# .dumps(data)) round-trip equality False on mamba,
# hasattr(binascii, "a2b_qp") / "b2a_qp" / "a2b_uu"
# / "b2a_uu" / "crc32" / "crc_hqx" / "Error" /
# "Incomplete" all False on mamba +
# binascii.crc32(b"hello") == 907060870 unavailable
# on mamba) are covered in the matching spec fixture
# `lang_json_marshal_binascii_silent`.
import json
import pickle
import marshal
import base64
import binascii


_ledger: list[int] = []

# 1) json — full module hasattr surface
assert hasattr(json, "dump") == True; _ledger.append(1)
assert hasattr(json, "dumps") == True; _ledger.append(1)
assert hasattr(json, "load") == True; _ledger.append(1)
assert hasattr(json, "loads") == True; _ledger.append(1)
assert hasattr(json, "JSONEncoder") == True; _ledger.append(1)
assert hasattr(json, "JSONDecoder") == True; _ledger.append(1)
assert hasattr(json, "JSONDecodeError") == True; _ledger.append(1)

# 2) json — round-trip / dump-formatting value contract
_jdata = {"a": 1, "b": [2, 3], "c": "x"}
_js = json.dumps(_jdata)
assert type(_js).__name__ == "str"; _ledger.append(1)
assert json.loads(_js) == _jdata; _ledger.append(1)
assert json.dumps({"a": 1}) == '{"a": 1}'; _ledger.append(1)
assert json.loads("[1,2,3]") == [1, 2, 3]; _ledger.append(1)
assert json.loads("true") == True; _ledger.append(1)
assert json.loads("null") is None; _ledger.append(1)
assert json.dumps({"a": 1}, indent=2).startswith("{"); _ledger.append(1)
assert json.dumps({"b": 2, "a": 1}, sort_keys=True) == '{"a": 1, "b": 2}'; _ledger.append(1)
assert json.dumps({"a": 1, "b": 2}, separators=(",", ":")) == '{"a":1,"b":2}'; _ledger.append(1)

# 3) pickle — partial module hasattr surface
#    (PROTOCOL absent on both — drop from assertion set)
assert hasattr(pickle, "dump") == True; _ledger.append(1)
assert hasattr(pickle, "dumps") == True; _ledger.append(1)
assert hasattr(pickle, "load") == True; _ledger.append(1)
assert hasattr(pickle, "loads") == True; _ledger.append(1)
assert hasattr(pickle, "Pickler") == True; _ledger.append(1)
assert hasattr(pickle, "Unpickler") == True; _ledger.append(1)
assert hasattr(pickle, "PicklingError") == True; _ledger.append(1)
assert hasattr(pickle, "UnpicklingError") == True; _ledger.append(1)
assert hasattr(pickle, "PickleError") == True; _ledger.append(1)
assert hasattr(pickle, "DEFAULT_PROTOCOL") == True; _ledger.append(1)
assert hasattr(pickle, "HIGHEST_PROTOCOL") == True; _ledger.append(1)
assert hasattr(pickle, "STOP") == True; _ledger.append(1)

# 4) pickle — round-trip / protocol-sentinel value contract
_pdata = {"a": 1, "b": [2, 3], "c": "x"}
_pp = pickle.dumps(_pdata)
assert type(_pp).__name__ == "bytes"; _ledger.append(1)
assert pickle.loads(_pp) == _pdata; _ledger.append(1)
assert pickle.HIGHEST_PROTOCOL >= 4; _ledger.append(1)
assert pickle.DEFAULT_PROTOCOL >= 0; _ledger.append(1)

# 5) marshal — full module hasattr surface
#    (type(marshal.dumps(...)).__name__ == "bytes" + marshal
#    round-trip DIVERGE on mamba — moved to spec)
assert hasattr(marshal, "dump") == True; _ledger.append(1)
assert hasattr(marshal, "dumps") == True; _ledger.append(1)
assert hasattr(marshal, "load") == True; _ledger.append(1)
assert hasattr(marshal, "loads") == True; _ledger.append(1)
assert hasattr(marshal, "version") == True; _ledger.append(1)

# 6) marshal — integer-sentinel value contract
assert marshal.version > 0; _ledger.append(1)

# 7) base64 — full module hasattr surface
assert hasattr(base64, "b64encode") == True; _ledger.append(1)
assert hasattr(base64, "b64decode") == True; _ledger.append(1)
assert hasattr(base64, "standard_b64encode") == True; _ledger.append(1)
assert hasattr(base64, "standard_b64decode") == True; _ledger.append(1)
assert hasattr(base64, "urlsafe_b64encode") == True; _ledger.append(1)
assert hasattr(base64, "urlsafe_b64decode") == True; _ledger.append(1)
assert hasattr(base64, "b32encode") == True; _ledger.append(1)
assert hasattr(base64, "b32decode") == True; _ledger.append(1)
assert hasattr(base64, "b32hexencode") == True; _ledger.append(1)
assert hasattr(base64, "b32hexdecode") == True; _ledger.append(1)
assert hasattr(base64, "b16encode") == True; _ledger.append(1)
assert hasattr(base64, "b16decode") == True; _ledger.append(1)
assert hasattr(base64, "a85encode") == True; _ledger.append(1)
assert hasattr(base64, "a85decode") == True; _ledger.append(1)
assert hasattr(base64, "b85encode") == True; _ledger.append(1)
assert hasattr(base64, "b85decode") == True; _ledger.append(1)
assert hasattr(base64, "encode") == True; _ledger.append(1)
assert hasattr(base64, "decode") == True; _ledger.append(1)
assert hasattr(base64, "encodebytes") == True; _ledger.append(1)
assert hasattr(base64, "decodebytes") == True; _ledger.append(1)

# 8) base64 — base64-codec value contract
assert base64.b64encode(b"hello") == b"aGVsbG8="; _ledger.append(1)
assert base64.b64decode(b"aGVsbG8=") == b"hello"; _ledger.append(1)
assert base64.b32encode(b"hello") == b"NBSWY3DP"; _ledger.append(1)
assert base64.b16encode(b"hello") == b"68656C6C6F"; _ledger.append(1)
assert base64.urlsafe_b64encode(b"hello") == b"aGVsbG8="; _ledger.append(1)

# 9) binascii — partial module hasattr surface
#    (a2b_qp / b2a_qp / a2b_uu / b2a_uu / crc32 /
#    crc_hqx / Error / Incomplete all DIVERGE on
#    mamba — moved to spec)
assert hasattr(binascii, "hexlify") == True; _ledger.append(1)
assert hasattr(binascii, "unhexlify") == True; _ledger.append(1)
assert hasattr(binascii, "a2b_base64") == True; _ledger.append(1)
assert hasattr(binascii, "b2a_base64") == True; _ledger.append(1)
assert hasattr(binascii, "a2b_hex") == True; _ledger.append(1)
assert hasattr(binascii, "b2a_hex") == True; _ledger.append(1)

# 10) binascii — hex-codec value contract
assert binascii.hexlify(b"\xab\xcd") == b"abcd"; _ledger.append(1)
assert binascii.unhexlify("abcd") == b"\xab\xcd"; _ledger.append(1)

# NB: hasattr(json, "encoder") / "decoder" False on mamba
# + type(json.JSONEncoder()).__name__ == "JSONEncoder"
# collapses to "int" on mamba +
# json.JSONEncoder().encode method unavailable on
# mamba + json.dumps("é") == "\"\\u00e9\""
# default-ensure-ascii collapses to '"é"' on mamba,
# type(marshal.dumps(data)).__name__ == "bytes"
# collapses to "str" on mamba + marshal.loads(marshal
# .dumps(data)) round-trip equality False on mamba,
# hasattr(binascii, "a2b_qp") / "b2a_qp" / "a2b_uu"
# / "b2a_uu" / "crc32" / "crc_hqx" / "Error" /
# "Incomplete" all False on mamba +
# binascii.crc32(b"hello") == 907060870 unavailable
# on mamba — all DIVERGE on mamba — moved to the
# divergence-spec fixture.

print(f"MAMBA_ASSERTION_PASS: test_json_pickle_marshal_base64_binascii_value_ops {sum(_ledger)} asserts")
