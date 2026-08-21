# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_urllib_parse_json_loads_float_binascii_crc32_silent"
# subject = "cpython321.lang_urllib_parse_json_loads_float_binascii_crc32_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_urllib_parse_json_loads_float_binascii_crc32_silent.py"
# status = "filled"
# ///
"""cpython321.lang_urllib_parse_json_loads_float_binascii_crc32_silent: execute CPython 3.12 seed lang_urllib_parse_json_loads_float_binascii_crc32_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass seed for SILENT divergences in
# `urllib.parse` (urlsplit returning a SplitResult-typed object,
# urlunsplit honouring the query slot, quote_from_bytes round-trip,
# ParseResult class identity), `json` (json.loads returning `float`
# for "1.5", JSONDecoder / JSONEncoder / JSONDecodeError class
# identity), `urllib.error` (URLError / HTTPError class identity),
# and `binascii` (crc32 module-level helper + binascii.Error class
# identity).
#
# The matching subset (urlparse / urlencode / parse_qs / parse_qsl /
# quote / unquote / quote_plus / unquote_plus / urljoin, json.dumps /
# loads byte-exact for scalars + containers, base64 / binascii
# hexlify / unhexlify round-trip) is covered by
# `test_urllib_parse_json_base64_value_ops`; this fixture pins the
# CPython-only contracts that mamba currently elides.
#
# Surface (CPython AssertionPass; mamba diverges silently):
#   • type(urlparse("x")).__name__ == "ParseResult" — bare class
#     identity, not the fully-qualified "urllib.parse.ParseResult"
#     stub-instance form that mamba surfaces;
#   • urlsplit / urlunsplit — urlsplit returns a SplitResult-typed
#     5-tuple (no `params` slot) and urlunsplit reproduces the input
#     URL (mamba: urlsplit returns the same ParseResult stub as
#     urlparse, and urlunsplit places the query at the params slot,
#     emitting "http://h/p;q=1");
#   • quote_from_bytes(b"\\x00\\xff") == "%00%FF" — percent-encodes
#     every non-ASCII byte (mamba: returns the empty string);
#   • json.loads("1.5") — returns `float` 1.5 (mamba: returns the
#     int64 raw bits of the IEEE-754 representation, 4609434218613702656);
#   • json.JSONDecoder.__name__ == "JSONDecoder" — class identity
#     (mamba: returns None);
#   • json.JSONEncoder.__name__ == "JSONEncoder" — class identity
#     (mamba: returns None);
#   • json.JSONDecodeError — exception class (type), not a string
#     (mamba: returns the literal string "ValueError");
#   • urllib.error.URLError.__name__ == "URLError" — exception-class
#     identity (mamba: returns None);
#   • urllib.error.HTTPError.__name__ == "HTTPError" — exception-class
#     identity (mamba: returns None);
#   • binascii.crc32(b"hi") — CRC-32 of the input bytes
#     (mamba: AttributeError, crc32 is not exposed);
#   • binascii.Error.__name__ == "Error" — exception-class identity
#     (mamba: hasattr returns False, value None).
import urllib.parse as _up_mod
import urllib.error as _ue_mod
import json as _json_mod
import binascii as _binascii_mod
from typing import Any

# Module bindings retyped as `Any` to bypass Pyright stub-driven
# narrowing — every spec contract below probes documented public
# constructors, class identifiers, or module-level helpers that
# mamba's bundled type stubs do not surface.
up: Any = _up_mod
ue: Any = _ue_mod
json: Any = _json_mod
binascii: Any = _binascii_mod

_ledger: list[int] = []

# 1) urlparse — bare class identity "ParseResult"
_p: Any = up.urlparse("http://h/p?q=1")
assert type(_p).__name__ == "ParseResult"; _ledger.append(1)

# 2) urlsplit — bare class identity "SplitResult"
_s: Any = up.urlsplit("http://h/p?q=1")
assert type(_s).__name__ == "SplitResult"; _ledger.append(1)

# 3) urlunsplit — round-trips a 5-tuple URL through the query slot
assert up.urlunsplit(("http", "h", "/p", "q=1", "")) == "http://h/p?q=1"; _ledger.append(1)

# 4) quote_from_bytes — percent-encodes every non-ASCII byte
assert up.quote_from_bytes(b"\x00\xff") == "%00%FF"; _ledger.append(1)
assert up.quote_from_bytes(b"hi") == "hi"; _ledger.append(1)

# 5) json.loads("1.5") — returns a `float`, not the int64 bit pattern
_f: Any = json.loads("1.5")
assert isinstance(_f, float); _ledger.append(1)
assert _f == 1.5; _ledger.append(1)
_f2: Any = json.loads("3.14")
assert isinstance(_f2, float); _ledger.append(1)
assert _f2 == 3.14; _ledger.append(1)

# 6) json.JSONDecoder / JSONEncoder — class identity
assert json.JSONDecoder.__name__ == "JSONDecoder"; _ledger.append(1)
assert json.JSONEncoder.__name__ == "JSONEncoder"; _ledger.append(1)

# 7) json.JSONDecodeError — exception class (type), not a string
assert type(json.JSONDecodeError).__name__ == "type"; _ledger.append(1)

# 8) urllib.error — URLError / HTTPError class identity
assert ue.URLError.__name__ == "URLError"; _ledger.append(1)
assert ue.HTTPError.__name__ == "HTTPError"; _ledger.append(1)

# 9) binascii.crc32 — CRC-32 module-level helper
_c: Any = binascii.crc32(b"hi")
assert isinstance(_c, int); _ledger.append(1)
assert _c == 3633523372; _ledger.append(1)
# Bit-width: CRC-32 result fits in 32 bits
assert 0 <= _c < (1 << 32); _ledger.append(1)

# 10) binascii.Error — exception class
assert binascii.Error.__name__ == "Error"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_urllib_parse_json_loads_float_binascii_crc32_silent {sum(_ledger)} asserts")
