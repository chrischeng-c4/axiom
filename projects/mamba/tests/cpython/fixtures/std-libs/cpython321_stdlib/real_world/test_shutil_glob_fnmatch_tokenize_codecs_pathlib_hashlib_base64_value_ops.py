# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_shutil_glob_fnmatch_tokenize_codecs_pathlib_hashlib_base64_value_ops"
# subject = "cpython321.test_shutil_glob_fnmatch_tokenize_codecs_pathlib_hashlib_base64_value_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_shutil_glob_fnmatch_tokenize_codecs_pathlib_hashlib_base64_value_ops.py"
# status = "filled"
# ///
"""cpython321.test_shutil_glob_fnmatch_tokenize_codecs_pathlib_hashlib_base64_value_ops: execute CPython 3.12 seed test_shutil_glob_fnmatch_tokenize_codecs_pathlib_hashlib_base64_value_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Atomic 242 pass conformance — shutil / glob / fnmatch / linecache / tokenize
# / token / keyword / codecs / json / pathlib class binding / re extended /
# hashlib / secrets / hmac / base64 / binascii partial / unicodedata partial /
# str.maketrans + translate surface + value ops that match between
# CPython 3.12 and mamba.
import shutil
import glob
import fnmatch
import linecache
import tokenize
import token
import keyword
import codecs
import json
import pathlib
import re
import hashlib
import secrets
import hmac
import base64
import binascii
import unicodedata


_ledger: list[int] = []

# 1) shutil hasattr surface
assert hasattr(shutil, "copy") == True; _ledger.append(1)
assert hasattr(shutil, "copy2") == True; _ledger.append(1)
assert hasattr(shutil, "copyfile") == True; _ledger.append(1)
assert hasattr(shutil, "copytree") == True; _ledger.append(1)
assert hasattr(shutil, "rmtree") == True; _ledger.append(1)
assert hasattr(shutil, "move") == True; _ledger.append(1)
assert hasattr(shutil, "disk_usage") == True; _ledger.append(1)
assert hasattr(shutil, "which") == True; _ledger.append(1)
assert hasattr(shutil, "make_archive") == True; _ledger.append(1)
assert hasattr(shutil, "unpack_archive") == True; _ledger.append(1)
assert hasattr(shutil, "get_archive_formats") == True; _ledger.append(1)
assert hasattr(shutil, "get_terminal_size") == True; _ledger.append(1)
assert hasattr(shutil, "SameFileError") == True; _ledger.append(1)
assert hasattr(shutil, "Error") == True; _ledger.append(1)
assert hasattr(shutil, "SpecialFileError") == True; _ledger.append(1)

# 2) glob hasattr + escape value op
assert hasattr(glob, "glob") == True; _ledger.append(1)
assert hasattr(glob, "iglob") == True; _ledger.append(1)
assert hasattr(glob, "escape") == True; _ledger.append(1)
assert hasattr(glob, "has_magic") == True; _ledger.append(1)
assert glob.escape("a*b") == "a[*]b"; _ledger.append(1)

# 3) fnmatch hasattr + value ops
assert hasattr(fnmatch, "fnmatch") == True; _ledger.append(1)
assert hasattr(fnmatch, "fnmatchcase") == True; _ledger.append(1)
assert hasattr(fnmatch, "translate") == True; _ledger.append(1)
assert hasattr(fnmatch, "filter") == True; _ledger.append(1)
assert fnmatch.fnmatch("abc.py", "*.py") == True; _ledger.append(1)
assert fnmatch.fnmatch("abc.txt", "*.py") == False; _ledger.append(1)
assert fnmatch.fnmatchcase("ABC.py", "*.py") == True; _ledger.append(1)
assert fnmatch.filter(["a.py", "b.txt", "c.py"], "*.py") == ["a.py", "c.py"]; _ledger.append(1)

# 4) linecache hasattr (cache module-level dict covered in spec)
assert hasattr(linecache, "getline") == True; _ledger.append(1)
assert hasattr(linecache, "getlines") == True; _ledger.append(1)
assert hasattr(linecache, "clearcache") == True; _ledger.append(1)
assert hasattr(linecache, "checkcache") == True; _ledger.append(1)

# 5) tokenize hasattr surface
assert hasattr(tokenize, "tokenize") == True; _ledger.append(1)
assert hasattr(tokenize, "untokenize") == True; _ledger.append(1)
assert hasattr(tokenize, "generate_tokens") == True; _ledger.append(1)
assert hasattr(tokenize, "TokenInfo") == True; _ledger.append(1)
assert hasattr(tokenize, "TokenError") == True; _ledger.append(1)
assert hasattr(tokenize, "open") == True; _ledger.append(1)
assert hasattr(tokenize, "detect_encoding") == True; _ledger.append(1)

# 6) token hasattr surface
assert hasattr(token, "NAME") == True; _ledger.append(1)
assert hasattr(token, "NUMBER") == True; _ledger.append(1)
assert hasattr(token, "STRING") == True; _ledger.append(1)
assert hasattr(token, "NEWLINE") == True; _ledger.append(1)
assert hasattr(token, "INDENT") == True; _ledger.append(1)
assert hasattr(token, "DEDENT") == True; _ledger.append(1)
assert hasattr(token, "ENDMARKER") == True; _ledger.append(1)
assert hasattr(token, "tok_name") == True; _ledger.append(1)
assert hasattr(token, "ISTERMINAL") == True; _ledger.append(1)
assert hasattr(token, "ISNONTERMINAL") == True; _ledger.append(1)
assert hasattr(token, "ISEOF") == True; _ledger.append(1)

# 7) keyword hasattr + value ops
assert hasattr(keyword, "iskeyword") == True; _ledger.append(1)
assert hasattr(keyword, "kwlist") == True; _ledger.append(1)
assert hasattr(keyword, "softkwlist") == True; _ledger.append(1)
assert keyword.iskeyword("if") == True; _ledger.append(1)
assert keyword.iskeyword("foo") == False; _ledger.append(1)

# 8) codecs hasattr + value ops
assert hasattr(codecs, "lookup") == True; _ledger.append(1)
assert hasattr(codecs, "getencoder") == True; _ledger.append(1)
assert hasattr(codecs, "getdecoder") == True; _ledger.append(1)
assert hasattr(codecs, "getreader") == True; _ledger.append(1)
assert hasattr(codecs, "getwriter") == True; _ledger.append(1)
assert hasattr(codecs, "encode") == True; _ledger.append(1)
assert hasattr(codecs, "decode") == True; _ledger.append(1)
assert hasattr(codecs, "register") == True; _ledger.append(1)
assert hasattr(codecs, "BOM_UTF8") == True; _ledger.append(1)
assert hasattr(codecs, "BOM_UTF16") == True; _ledger.append(1)
assert hasattr(codecs, "BOM_UTF32") == True; _ledger.append(1)
assert codecs.encode("hello", "utf-8") == b"hello"; _ledger.append(1)
assert codecs.decode(b"hello", "utf-8") == "hello"; _ledger.append(1)
assert codecs.encode("hello", "latin-1") == b"hello"; _ledger.append(1)

# 9) json round-trip value ops
assert json.loads('{"a": 1, "b": [2, 3]}') == {"a": 1, "b": [2, 3]}; _ledger.append(1)
assert json.loads('{"a": {"b": [1, 2, {"c": 3}]}}') == {"a": {"b": [1, 2, {"c": 3}]}}; _ledger.append(1)
assert json.dumps({"a": 1, "b": [2, 3]}, sort_keys=True) == '{"a": 1, "b": [2, 3]}'; _ledger.append(1)
assert json.dumps({"a": 1}, indent=2) == '{\n  "a": 1\n}'; _ledger.append(1)
assert json.loads("null") is None; _ledger.append(1)
assert json.loads("true") == True; _ledger.append(1)
assert json.dumps(None) == "null"; _ledger.append(1)
assert json.dumps(True) == "true"; _ledger.append(1)

# 10) pathlib class binding hasattr
assert hasattr(pathlib, "Path") == True; _ledger.append(1)
assert hasattr(pathlib, "PurePath") == True; _ledger.append(1)
assert hasattr(pathlib, "PurePosixPath") == True; _ledger.append(1)
assert hasattr(pathlib, "PureWindowsPath") == True; _ledger.append(1)
assert hasattr(pathlib, "PosixPath") == True; _ledger.append(1)
assert hasattr(pathlib, "WindowsPath") == True; _ledger.append(1)

# 11) re extended value ops
_m = re.fullmatch(r"\d+", "12345")
assert _m is not None; _ledger.append(1)
assert re.fullmatch(r"\d+", "12abc") is None; _ledger.append(1)
assert re.subn(r"\d", "X", "a1b2c3") == ("aXbXcX", 3); _ledger.append(1)
assert re.split(r"\s+", "hello world foo") == ["hello", "world", "foo"]; _ledger.append(1)
assert re.escape("hello.world+foo") == "hello\\.world\\+foo"; _ledger.append(1)

# 12) hashlib hexdigest value ops + blake2 hasattr
assert hashlib.md5(b"hello").hexdigest() == "5d41402abc4b2a76b9719d911017c592"; _ledger.append(1)
assert hashlib.sha1(b"hello").hexdigest() == "aaf4c61ddcc5e8a2dabede0f3b482cd9aea9434d"; _ledger.append(1)
assert hashlib.sha256(b"hello").hexdigest() == "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"; _ledger.append(1)
assert hashlib.new("sha256", b"hello").hexdigest() == "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"; _ledger.append(1)
assert hasattr(hashlib, "blake2b") == True; _ledger.append(1)
assert hasattr(hashlib, "blake2s") == True; _ledger.append(1)

# 13) secrets hasattr surface + token_hex len
assert hasattr(secrets, "token_bytes") == True; _ledger.append(1)
assert hasattr(secrets, "token_hex") == True; _ledger.append(1)
assert hasattr(secrets, "token_urlsafe") == True; _ledger.append(1)
assert hasattr(secrets, "choice") == True; _ledger.append(1)
assert hasattr(secrets, "randbelow") == True; _ledger.append(1)
assert hasattr(secrets, "randbits") == True; _ledger.append(1)
assert hasattr(secrets, "compare_digest") == True; _ledger.append(1)
assert hasattr(secrets, "SystemRandom") == True; _ledger.append(1)
assert len(secrets.token_hex(16)) == 32; _ledger.append(1)

# 14) hmac hasattr + value ops
assert hasattr(hmac, "new") == True; _ledger.append(1)
assert hasattr(hmac, "digest") == True; _ledger.append(1)
assert hasattr(hmac, "compare_digest") == True; _ledger.append(1)
assert hasattr(hmac, "HMAC") == True; _ledger.append(1)
assert hmac.new(b"key", b"msg", "sha256").hexdigest() == "2d93cbc1be167bcb1637a4a23cbff01a7878f0c50ee833954ea5221bb1b8c628"; _ledger.append(1)
assert hmac.digest(b"key", b"msg", "sha256").hex() == "2d93cbc1be167bcb1637a4a23cbff01a7878f0c50ee833954ea5221bb1b8c628"; _ledger.append(1)

# 15) base64 round-trip value ops
assert base64.b64encode(b"hello") == b"aGVsbG8="; _ledger.append(1)
assert base64.b64decode(b"aGVsbG8=") == b"hello"; _ledger.append(1)
assert base64.urlsafe_b64encode(b"hello") == b"aGVsbG8="; _ledger.append(1)
assert base64.urlsafe_b64decode(b"aGVsbG8=") == b"hello"; _ledger.append(1)
assert base64.standard_b64encode(b"hello") == b"aGVsbG8="; _ledger.append(1)
assert base64.b32encode(b"hello") == b"NBSWY3DP"; _ledger.append(1)
assert base64.b16encode(b"hello") == b"68656C6C6F"; _ledger.append(1)
assert hasattr(base64, "encodebytes") == True; _ledger.append(1)
assert hasattr(base64, "decodebytes") == True; _ledger.append(1)

# 16) binascii partial hasattr + hexlify/unhexlify value ops
assert hasattr(binascii, "hexlify") == True; _ledger.append(1)
assert hasattr(binascii, "unhexlify") == True; _ledger.append(1)
assert hasattr(binascii, "a2b_base64") == True; _ledger.append(1)
assert hasattr(binascii, "b2a_base64") == True; _ledger.append(1)
assert binascii.hexlify(b"hello") == b"68656c6c6f"; _ledger.append(1)
assert binascii.unhexlify(b"68656c6c6f") == b"hello"; _ledger.append(1)

# 17) unicodedata partial hasattr + category/normalize value ops
assert hasattr(unicodedata, "name") == True; _ledger.append(1)
assert hasattr(unicodedata, "category") == True; _ledger.append(1)
assert hasattr(unicodedata, "normalize") == True; _ledger.append(1)
assert hasattr(unicodedata, "decimal") == True; _ledger.append(1)
assert hasattr(unicodedata, "bidirectional") == True; _ledger.append(1)
assert unicodedata.category("A") == "Lu"; _ledger.append(1)
assert unicodedata.normalize("NFC", "ABC") == "ABC"; _ledger.append(1)

# 18) str.maketrans + translate
assert str.maketrans("abc", "xyz") == {97: 120, 98: 121, 99: 122}; _ledger.append(1)
assert "abcdef".translate(str.maketrans("abc", "xyz")) == "xyzdef"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_shutil_glob_fnmatch_tokenize_codecs_pathlib_hashlib_base64_value_ops {sum(_ledger)} asserts")
