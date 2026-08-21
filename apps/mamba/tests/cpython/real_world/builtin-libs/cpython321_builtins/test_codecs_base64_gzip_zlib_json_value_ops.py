# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "cpython321_builtins"
# dimension = "real_world"
# case = "test_codecs_base64_gzip_zlib_json_value_ops"
# subject = "cpython321.test_codecs_base64_gzip_zlib_json_value_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_codecs_base64_gzip_zlib_json_value_ops.py"
# status = "filled"
# ///
"""cpython321.test_codecs_base64_gzip_zlib_json_value_ops: execute CPython 3.12 seed test_codecs_base64_gzip_zlib_json_value_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the value contract of five
# bootstrap stdlib modules used by every encode-decode /
# binary-to-ASCII / compression / structured-data path: `codecs`
# (the documented `encode` / `decode` / `lookup` / `getencoder`
# / `getdecoder` / `BOM_UTF8` / `BOM_UTF16` / `BOM_UTF32` /
# `register` / `register_error` attribute surface + the
# documented `encode("hello", "utf-8")` / `decode(b"hello",
# "utf-8")` UTF-8 contract + the documented `BOM_UTF8` byte-
# value contract), `base64` (the documented `b64encode` /
# `b64decode` / `b32encode` / `b16encode` / `urlsafe_b64encode`
# encoder surface + the documented byte-input round-trip
# contracts), `gzip` (the documented `compress` / `decompress`
# byte-input round-trip contract), `zlib` (the documented
# `compress` / `decompress` byte-input round-trip + `adler32` /
# `crc32` checksum integer-value contracts), and `json` (the
# documented `dumps` / `loads` nested container round-trip
# contract).
#
# The matching subset between mamba and CPython is the codecs
# UTF-8 encode/decode layer + BOM constant layer + module
# hasattr surface, the base64 full encoder layer, the gzip
# compress / decompress round-trip layer, the zlib full layer,
# the json nested-container layer, and the csv hasattr surface
# layer.
#
# Surface in this fixture:
#   • codecs — encode / decode / lookup / getencoder /
#     getdecoder / BOM_UTF8 / BOM_UTF16 / BOM_UTF32 / register /
#     register_error hasattr + encode("hello", "utf-8") /
#     decode(b"hello", "utf-8") + BOM_UTF8 byte value;
#   • base64 — b64encode / b64decode / b32encode / b16encode /
#     urlsafe_b64encode round-trip + byte-output value contract;
#   • gzip — compress + decompress round-trip;
#   • zlib — compress + decompress + adler32 + crc32 value
#     contract;
#   • json — dumps + loads nested container round-trip;
#   • csv — reader / writer / DictReader / DictWriter /
#     QUOTE_ALL / QUOTE_MINIMAL / QUOTE_NONE / QUOTE_NONNUMERIC
#     hasattr surface.
#
# Behavioral edges that DIVERGE on mamba (codecs.lookup(name).name
# returns None — lookup return doesn't surface the `.name`
# attribute, codecs.encode hex / base64 transform codecs return
# None — only utf-8/-16 surface, codecs.encode rot_13 returns
# the input bytes unchanged not the rot_13-shifted string,
# csv.writer(buf).writerow AttributeError 'str' object has no
# attribute 'writerow', csv.reader iteration returns the empty
# list) are covered in the matching spec fixture
# `lang_codecs_csv_silent`.
import codecs
import base64
import gzip
import zlib
import json
import csv


_ledger: list[int] = []

# 1) codecs — module attribute hasattr surface
assert hasattr(codecs, "encode") == True; _ledger.append(1)
assert hasattr(codecs, "decode") == True; _ledger.append(1)
assert hasattr(codecs, "lookup") == True; _ledger.append(1)
assert hasattr(codecs, "getencoder") == True; _ledger.append(1)
assert hasattr(codecs, "getdecoder") == True; _ledger.append(1)
assert hasattr(codecs, "BOM_UTF8") == True; _ledger.append(1)
assert hasattr(codecs, "BOM_UTF16") == True; _ledger.append(1)
assert hasattr(codecs, "BOM_UTF32") == True; _ledger.append(1)
assert hasattr(codecs, "register") == True; _ledger.append(1)
assert hasattr(codecs, "register_error") == True; _ledger.append(1)

# 2) codecs — utf-8 encode/decode + BOM value
assert codecs.encode("hello", "utf-8") == b"hello"; _ledger.append(1)
assert codecs.decode(b"hello", "utf-8") == "hello"; _ledger.append(1)
assert codecs.BOM_UTF8 == b"\xef\xbb\xbf"; _ledger.append(1)

# 3) base64 — encoder surface + round-trip
assert base64.b64encode(b"hi") == b"aGk="; _ledger.append(1)
assert base64.b64decode(b"aGk=") == b"hi"; _ledger.append(1)
assert base64.b32encode(b"hi") == b"NBUQ===="; _ledger.append(1)
assert base64.b16encode(b"hi") == b"6869"; _ledger.append(1)
assert base64.urlsafe_b64encode(b"hi") == b"aGk="; _ledger.append(1)

# 4) gzip — compress + decompress round-trip
assert gzip.decompress(gzip.compress(b"hello world")) == b"hello world"; _ledger.append(1)
assert gzip.decompress(gzip.compress(b"a" * 100)) == b"a" * 100; _ledger.append(1)

# 5) zlib — compress + decompress + checksums
assert zlib.decompress(zlib.compress(b"hello hello hello")) == b"hello hello hello"; _ledger.append(1)
assert zlib.adler32(b"hello") == 103547413; _ledger.append(1)
assert zlib.crc32(b"hello") == 907060870; _ledger.append(1)

# 6) json — nested container round-trip
assert json.dumps({"a": [1, 2, 3]}) == '{"a": [1, 2, 3]}'; _ledger.append(1)
assert json.loads('{"a":[1,{"b":[2,3]}]}') == {"a": [1, {"b": [2, 3]}]}; _ledger.append(1)
assert json.dumps([1, "two", 3.0, None, True]) == '[1, "two", 3.0, null, true]'; _ledger.append(1)
assert json.loads('[1, "two", 3.0, null, true]') == [1, "two", 3.0, None, True]; _ledger.append(1)

# 7) csv — module attribute hasattr surface
assert hasattr(csv, "reader") == True; _ledger.append(1)
assert hasattr(csv, "writer") == True; _ledger.append(1)
assert hasattr(csv, "DictReader") == True; _ledger.append(1)
assert hasattr(csv, "DictWriter") == True; _ledger.append(1)
assert hasattr(csv, "QUOTE_ALL") == True; _ledger.append(1)
assert hasattr(csv, "QUOTE_MINIMAL") == True; _ledger.append(1)
assert hasattr(csv, "QUOTE_NONE") == True; _ledger.append(1)
assert hasattr(csv, "QUOTE_NONNUMERIC") == True; _ledger.append(1)

# NB: codecs.lookup(name).name returns None — lookup return
# doesn't surface .name, codecs.encode hex / base64 transform
# codecs return None, codecs.encode rot_13 returns input bytes
# unchanged, csv.writer(buf).writerow AttributeError, csv.reader
# iteration returns the empty list — all DIVERGE on mamba —
# moved to the divergence-spec fixture.

print(f"MAMBA_ASSERTION_PASS: test_codecs_base64_gzip_zlib_json_value_ops {sum(_ledger)} asserts")
