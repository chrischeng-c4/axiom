# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_pickle_marshal_base64_binascii_shutil_value_ops"
# subject = "cpython321.test_pickle_marshal_base64_binascii_shutil_value_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_pickle_marshal_base64_binascii_shutil_value_ops.py"
# status = "filled"
# ///
"""cpython321.test_pickle_marshal_base64_binascii_shutil_value_ops: execute CPython 3.12 seed test_pickle_marshal_base64_binascii_shutil_value_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the value contract of the
# `pickle` / `marshal` / `binascii` / `base64` / `quopri` /
# `string` / `shutil` seven-pack pinned to atomic 184: `pickle`
# (the documented full module-level helper hasattr surface —
# `dumps` / `loads` / `dump` / `load` / `Pickler` / `Unpickler`
# / `HIGHEST_PROTOCOL` / `DEFAULT_PROTOCOL` / `PickleError` /
# `UnpicklingError` + the documented pickle.dumps/loads round-
# trip value contract), `marshal` (the documented full module-
# level helper hasattr surface — `dumps` / `loads` / `dump` /
# `load` / `version`), `binascii` (the documented partial
# module-level helper hasattr surface — `a2b_hex` / `b2a_hex`
# / `a2b_base64` / `b2a_base64` / `hexlify` / `unhexlify` +
# the documented binascii.hexlify / unhexlify value contract),
# `base64` (the documented full module-level helper hasattr
# surface — `b64encode` / `b64decode` / `b32encode` /
# `b32decode` / `b16encode` / `b16decode` / `urlsafe_b64encode`
# / `urlsafe_b64decode` / `standard_b64encode` /
# `standard_b64decode` / `encodebytes` / `decodebytes` + the
# documented base64.b64encode / b64decode / b32encode /
# b16encode / urlsafe_b64encode value contract), `quopri` (the
# documented `encodestring` / `decodestring` helper
# identifiers), `string` (the documented `Formatter` /
# `Template` class identifiers), and `shutil` (the documented
# full module-level helper hasattr surface — `copy` / `copy2`
# / `copyfile` / `copytree` / `move` / `rmtree` / `which` /
# `disk_usage` / `get_terminal_size` / `make_archive` /
# `unpack_archive` + the documented shutil.get_terminal_size()
# terminal_size return-type contract).
#
# The matching subset between mamba and CPython is the full
# `pickle` module hasattr surface + the pickle.dumps/loads
# round-trip layer (the produced byte-stream length differs
# but round-trip holds), the full `marshal` module hasattr
# surface (the marshal.dumps return-type / round-trip
# DIVERGES), the partial `binascii` module hasattr surface
# (a2b_hex / b2a_hex / a2b_base64 / b2a_base64 / hexlify /
# unhexlify — crc32 / Error / Incomplete DIVERGE) + the
# binascii.hexlify / unhexlify value layer, the full
# `base64` module hasattr surface + the full base64 encode/
# decode value layer (b64 / b32 / b16 / urlsafe), the
# `quopri` encodestring / decodestring hasattr layer, the
# `string.Formatter` / `string.Template` hasattr layer (the
# Formatter instance .format value contract DIVERGES + the
# textwrap.TextWrapper instance construction DIVERGES), and
# the full `shutil` module hasattr surface + the
# shutil.get_terminal_size() terminal_size return-type
# layer.
#
# Surface in this fixture:
#   • pickle — full module hasattr surface (dumps / loads /
#     dump / load / Pickler / Unpickler / HIGHEST_PROTOCOL /
#     DEFAULT_PROTOCOL / PickleError / UnpicklingError);
#   • pickle.dumps + pickle.loads — round-trip value
#     contract;
#   • marshal — full module hasattr surface (dumps / loads
#     / dump / load / version);
#   • binascii — partial module hasattr surface (a2b_hex /
#     b2a_hex / a2b_base64 / b2a_base64 / hexlify /
#     unhexlify);
#   • binascii.hexlify / unhexlify — value contract;
#   • base64 — full module hasattr surface (b64encode /
#     b64decode / b32encode / b32decode / b16encode /
#     b16decode / urlsafe_b64encode / urlsafe_b64decode /
#     standard_b64encode / standard_b64decode / encodebytes
#     / decodebytes);
#   • base64.b64encode / b64decode / b32encode / b16encode
#     / urlsafe_b64encode — value contract;
#   • quopri — partial module hasattr surface (encodestring
#     / decodestring);
#   • string — partial module hasattr surface (Formatter /
#     Template);
#   • shutil — full module hasattr surface (copy / copy2 /
#     copyfile / copytree / move / rmtree / which /
#     disk_usage / get_terminal_size / make_archive /
#     unpack_archive);
#   • shutil.get_terminal_size() — terminal_size return-
#     type contract.
#
# Behavioral edges that DIVERGE on mamba
# (type(marshal.dumps(...)).__name__ returns "str" not "bytes",
# marshal.loads(marshal.dumps(x)) returns None not the
# original value, hasattr(binascii, "crc32") / "Error" /
# "Incomplete" all False, binascii.crc32(...) raises
# AttributeError 'dict' object has no attribute 'crc32',
# type(string.Formatter()).__name__ returns "dict" not
# "Formatter", string.Formatter().format(...) raises
# AttributeError, textwrap.TextWrapper(width=15) construction
# raises AttributeError on .TextWrapper accessor) are covered
# in the matching spec fixture
# `lang_marshal_binascii_formatter_textwrap_silent`.
import pickle
import marshal
import binascii
import base64
import quopri
import string
import shutil


_ledger: list[int] = []

# 1) pickle — full module hasattr surface
assert hasattr(pickle, "dumps") == True; _ledger.append(1)
assert hasattr(pickle, "loads") == True; _ledger.append(1)
assert hasattr(pickle, "dump") == True; _ledger.append(1)
assert hasattr(pickle, "load") == True; _ledger.append(1)
assert hasattr(pickle, "Pickler") == True; _ledger.append(1)
assert hasattr(pickle, "Unpickler") == True; _ledger.append(1)
assert hasattr(pickle, "HIGHEST_PROTOCOL") == True; _ledger.append(1)
assert hasattr(pickle, "DEFAULT_PROTOCOL") == True; _ledger.append(1)
assert hasattr(pickle, "PickleError") == True; _ledger.append(1)
assert hasattr(pickle, "UnpicklingError") == True; _ledger.append(1)

# 2) pickle.dumps + pickle.loads — round-trip value
_data_p = [1, 2, 3, "hello", {"a": 1}]
_b_p = pickle.dumps(_data_p)
assert type(_b_p).__name__ == "bytes"; _ledger.append(1)
assert pickle.loads(_b_p) == _data_p; _ledger.append(1)

# 3) marshal — full module hasattr surface
assert hasattr(marshal, "dumps") == True; _ledger.append(1)
assert hasattr(marshal, "loads") == True; _ledger.append(1)
assert hasattr(marshal, "dump") == True; _ledger.append(1)
assert hasattr(marshal, "load") == True; _ledger.append(1)
assert hasattr(marshal, "version") == True; _ledger.append(1)

# 4) binascii — partial module hasattr surface
#    (crc32 / Error / Incomplete DIVERGE — moved to spec fixture)
assert hasattr(binascii, "a2b_hex") == True; _ledger.append(1)
assert hasattr(binascii, "b2a_hex") == True; _ledger.append(1)
assert hasattr(binascii, "a2b_base64") == True; _ledger.append(1)
assert hasattr(binascii, "b2a_base64") == True; _ledger.append(1)
assert hasattr(binascii, "hexlify") == True; _ledger.append(1)
assert hasattr(binascii, "unhexlify") == True; _ledger.append(1)

# 5) binascii.hexlify / unhexlify — value contract
assert binascii.hexlify(b"abc") == b"616263"; _ledger.append(1)
assert binascii.unhexlify(b"616263") == b"abc"; _ledger.append(1)

# 6) base64 — full module hasattr surface
assert hasattr(base64, "b64encode") == True; _ledger.append(1)
assert hasattr(base64, "b64decode") == True; _ledger.append(1)
assert hasattr(base64, "b32encode") == True; _ledger.append(1)
assert hasattr(base64, "b32decode") == True; _ledger.append(1)
assert hasattr(base64, "b16encode") == True; _ledger.append(1)
assert hasattr(base64, "b16decode") == True; _ledger.append(1)
assert hasattr(base64, "urlsafe_b64encode") == True; _ledger.append(1)
assert hasattr(base64, "urlsafe_b64decode") == True; _ledger.append(1)
assert hasattr(base64, "standard_b64encode") == True; _ledger.append(1)
assert hasattr(base64, "standard_b64decode") == True; _ledger.append(1)
assert hasattr(base64, "encodebytes") == True; _ledger.append(1)
assert hasattr(base64, "decodebytes") == True; _ledger.append(1)

# 7) base64 — value contract
assert base64.b64encode(b"hello") == b"aGVsbG8="; _ledger.append(1)
assert base64.b64decode(b"aGVsbG8=") == b"hello"; _ledger.append(1)
assert base64.b32encode(b"hello") == b"NBSWY3DP"; _ledger.append(1)
assert base64.b16encode(b"hello") == b"68656C6C6F"; _ledger.append(1)
assert base64.urlsafe_b64encode(b"hi") == b"aGk="; _ledger.append(1)

# 8) quopri — partial module hasattr surface
assert hasattr(quopri, "encodestring") == True; _ledger.append(1)
assert hasattr(quopri, "decodestring") == True; _ledger.append(1)

# 9) string — partial module hasattr surface
assert hasattr(string, "Formatter") == True; _ledger.append(1)
assert hasattr(string, "Template") == True; _ledger.append(1)

# 10) shutil — full module hasattr surface
assert hasattr(shutil, "copy") == True; _ledger.append(1)
assert hasattr(shutil, "copy2") == True; _ledger.append(1)
assert hasattr(shutil, "copyfile") == True; _ledger.append(1)
assert hasattr(shutil, "copytree") == True; _ledger.append(1)
assert hasattr(shutil, "move") == True; _ledger.append(1)
assert hasattr(shutil, "rmtree") == True; _ledger.append(1)
assert hasattr(shutil, "which") == True; _ledger.append(1)
assert hasattr(shutil, "disk_usage") == True; _ledger.append(1)
assert hasattr(shutil, "get_terminal_size") == True; _ledger.append(1)
assert hasattr(shutil, "make_archive") == True; _ledger.append(1)
assert hasattr(shutil, "unpack_archive") == True; _ledger.append(1)

# 11) shutil.get_terminal_size() — terminal_size return type
assert type(shutil.get_terminal_size()).__name__ == "terminal_size"; _ledger.append(1)

# NB: type(marshal.dumps(...)).__name__ returns "str" not
# "bytes" on mamba, marshal.loads(marshal.dumps(x)) returns
# None not the original value, hasattr(binascii, "crc32") /
# "Error" / "Incomplete" all False, binascii.crc32 call
# raises AttributeError, type(string.Formatter()).__name__
# returns "dict" not "Formatter", string.Formatter().format
# raises AttributeError, textwrap.TextWrapper construction
# raises AttributeError — all DIVERGE on mamba — moved to
# the divergence-spec fixture.

print(f"MAMBA_ASSERTION_PASS: test_pickle_marshal_base64_binascii_shutil_value_ops {sum(_ledger)} asserts")
