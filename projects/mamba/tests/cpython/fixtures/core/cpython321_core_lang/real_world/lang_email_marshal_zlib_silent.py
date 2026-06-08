# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_email_marshal_zlib_silent"
# subject = "cpython321.lang_email_marshal_zlib_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_email_marshal_zlib_silent.py"
# status = "filled"
# ///
"""cpython321.lang_email_marshal_zlib_silent: execute CPython 3.12 seed lang_email_marshal_zlib_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass seed for SILENT divergences across the
# email / template / marshal / zlib-constant / binascii-checksum
# quintet pinned by atomic 155: `email` (the documented
# EmailMessage / Message / Parser bare-class identity + the
# `EmailMessage().set_content / get_content` instance contract),
# `string` (the documented Template substitute / safe_substitute
# instance methods), `copyreg` (the documented `pickle` /
# `constructor` registry helpers), `shelve` (the documented
# Shelf / DbfilenameShelf bare-class identity), `marshal`
# (the documented `dumps` returning non-empty bytes + `loads`
# round-tripping the original value), `zlib` (the documented
# MAX_WBITS / DEF_BUF_SIZE / Z_DEFAULT_COMPRESSION integer
# constants + ZLIB_VERSION string + `error` exception class
# identity), and `binascii` (the documented `crc32` checksum
# helper).
#
# The matching subset (zlib.compress / decompress / adler32 /
# crc32 round-trip on "hello world", binascii.hexlify /
# unhexlify / b2a_base64 / a2b_base64 / b2a_hex / a2b_hex byte
# round-trip, email.message_from_string + message_from_bytes
# hasattr) is covered by `test_zlib_binascii_email_value_ops`;
# this fixture pins the CPython-only contracts that mamba
# currently elides.
#
# Surface (CPython AssertionPass; mamba diverges silently):
#   • email.message.EmailMessage.__name__ == "EmailMessage" —
#     bare class identity (mamba: returns None);
#   • email.message.Message.__name__ == "Message" (mamba:
#     None);
#   • email.parser.Parser.__name__ == "Parser" (mamba: None);
#   • EmailMessage().set_content("hello") + get_content() ==
#     "hello\n" — instance content round-trip (mamba:
#     AttributeError, 'dict' object has no attribute
#     'set_content');
#   • string.Template("Hello $name").substitute(name="World")
#     == "Hello World" — Template substitution (mamba:
#     AttributeError, 'dict' object has no attribute
#     'substitute');
#   • hasattr(copyreg, "pickle") is True — pickle registry
#     helper surface (mamba: returns False);
#   • hasattr(copyreg, "constructor") is True (mamba: False);
#   • shelve.Shelf.__name__ == "Shelf" — shelf-base class
#     identity (mamba: None);
#   • shelve.DbfilenameShelf.__name__ == "DbfilenameShelf"
#     (mamba: None);
#   • marshal.dumps(42) != b"" — non-empty serialization
#     (mamba: returns empty bytes b"");
#   • marshal.loads(marshal.dumps(42)) == 42 — round-trip
#     (mamba: returns None — both dumps and loads are
#     broken);
#   • marshal.loads(marshal.dumps([1, 2, 3])) == [1, 2, 3]
#     (mamba: None);
#   • zlib.MAX_WBITS == 15 — window-bits cap (mamba: None);
#   • zlib.DEF_BUF_SIZE == 16384 — default decompression
#     buffer (mamba: None);
#   • zlib.Z_DEFAULT_COMPRESSION == -1 — default-level
#     sentinel (mamba: None);
#   • type(zlib.ZLIB_VERSION).__name__ == "str" — version
#     string surface (mamba: NoneType);
#   • zlib.error.__name__ == "error" — compression-error
#     class identity (mamba: None);
#   • binascii.crc32(b"hello") == 907060870 — CRC-32 on
#     short payload (mamba: AttributeError, 'dict' object
#     has no attribute 'crc32').
import email.message as _email_message_mod
import email.parser as _email_parser_mod
import string as _string_mod
import copyreg as _copyreg_mod
import shelve as _shelve_mod
import marshal as _marshal_mod
import zlib as _zlib_mod
import binascii as _binascii_mod
from typing import Any

# Module bindings retyped as `Any` to bypass Pyright stub-driven
# narrowing — every spec contract below probes documented public
# class identifiers / module-level helpers / instance methods
# that mamba's bundled type stubs do not surface accurately.
email_message: Any = _email_message_mod
email_parser: Any = _email_parser_mod
string: Any = _string_mod
copyreg: Any = _copyreg_mod
shelve: Any = _shelve_mod
marshal: Any = _marshal_mod
zlib: Any = _zlib_mod
binascii: Any = _binascii_mod


_ledger: list[int] = []

# 1) email.message — bare class identity
assert email_message.EmailMessage.__name__ == "EmailMessage"; _ledger.append(1)
assert email_message.Message.__name__ == "Message"; _ledger.append(1)
assert email_parser.Parser.__name__ == "Parser"; _ledger.append(1)

# 2) EmailMessage — set_content / get_content round-trip
_msg = email_message.EmailMessage()
_msg.set_content("hello")
assert _msg.get_content() == "hello\n"; _ledger.append(1)

# 3) string.Template — substitute method
assert string.Template("Hello $name").substitute(name="World") == "Hello World"; _ledger.append(1)

# 4) copyreg — pickle-registry helper surface
assert hasattr(copyreg, "pickle") == True; _ledger.append(1)
assert hasattr(copyreg, "constructor") == True; _ledger.append(1)

# 5) shelve — shelf-base class identity
assert shelve.Shelf.__name__ == "Shelf"; _ledger.append(1)
assert shelve.DbfilenameShelf.__name__ == "DbfilenameShelf"; _ledger.append(1)

# 6) marshal — non-empty dumps + loads round-trip
assert marshal.dumps(42) != b""; _ledger.append(1)
assert marshal.loads(marshal.dumps(42)) == 42; _ledger.append(1)
assert marshal.loads(marshal.dumps([1, 2, 3])) == [1, 2, 3]; _ledger.append(1)

# 7) zlib — documented integer + string constants
assert zlib.MAX_WBITS == 15; _ledger.append(1)
assert zlib.DEF_BUF_SIZE == 16384; _ledger.append(1)
assert zlib.Z_DEFAULT_COMPRESSION == -1; _ledger.append(1)
assert type(zlib.ZLIB_VERSION).__name__ == "str"; _ledger.append(1)

# 8) zlib — compression-error class identity
assert zlib.error.__name__ == "error"; _ledger.append(1)

# 9) binascii.crc32 — CRC-32 on short payload
assert binascii.crc32(b"hello") == 907060870; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_email_marshal_zlib_silent {sum(_ledger)} asserts")
