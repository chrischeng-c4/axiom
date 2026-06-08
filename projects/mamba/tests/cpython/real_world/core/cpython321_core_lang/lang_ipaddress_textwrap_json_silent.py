# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_ipaddress_textwrap_json_silent"
# subject = "cpython321.lang_ipaddress_textwrap_json_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_ipaddress_textwrap_json_silent.py"
# status = "filled"
# ///
"""cpython321.lang_ipaddress_textwrap_json_silent: execute CPython 3.12 seed lang_ipaddress_textwrap_json_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass seed for SILENT divergences across the
# IP-address / URL-result-repr / HTML-entity / JSON-codec / text-
# wrapping quintet pinned by atomic 151: `ipaddress` (the
# documented IPv4Address / IPv6Address / IPv4Network class
# identity + the module-level ip_address / ip_network helpers),
# `urllib.parse` (the documented `ParseResult` __repr__ form
# `ParseResult(scheme=..., netloc=..., ...)`), `html.entities`
# (the documented submodule surface — `hasattr(html,
# "entities")`), `json` (the default ensure_ascii=True unicode-
# escape + the documented JSONDecodeError / JSONDecoder /
# JSONEncoder class identity), and `textwrap` (the documented
# fill / wrap / shorten / TextWrapper surface that actually
# wraps strings).
#
# The matching subset (urllib.parse.urlparse field decomposition,
# urljoin, urlencode, parse_qs / parse_qsl, quote / unquote /
# quote_plus / unquote_plus, html.escape / unescape, base64 b16
# / b32 / b64 / urlsafe / standard / encodebytes byte round-
# trip, json.dumps / loads round-trip on int / list / dict /
# None / True / str + indent + sort_keys + ensure_ascii=False)
# is covered by `test_urllib_html_base64_json_value_ops`; this
# fixture pins the CPython-only contracts that mamba currently
# elides.
#
# Surface (CPython AssertionPass; mamba diverges silently):
#   • ipaddress.IPv4Address.__name__ == "IPv4Address" — bare
#     class identity (mamba: AttributeError, 'dict' object has
#     no attribute 'IPv4Address' — the entire ipaddress module
#     is a plain dict);
#   • ipaddress.IPv6Address.__name__ == "IPv6Address" (mamba:
#     AttributeError);
#   • ipaddress.IPv4Network.__name__ == "IPv4Network" (mamba:
#     AttributeError);
#   • int(ipaddress.IPv4Address("192.168.1.1")) == 3232235777
#     — integer coercion (mamba: AttributeError);
#   • ipaddress.IPv4Network("192.168.0.0/24").num_addresses ==
#     256 (mamba: AttributeError);
#   • repr(urllib.parse.urlparse("https://a.com/p?q=1#f"))
#     startswith "ParseResult(scheme=" — the documented
#     namedtuple-style repr (mamba: returns
#     "urllib.parse.ParseResult()", an empty-shaped repr);
#   • hasattr(html, "entities") is True — submodule attribute
#     surface (mamba: returns False, `entities` is not exposed
#     on html);
#   • json.dumps("café") == '"caf\\u00e9"' — default
#     ensure_ascii=True unicode escape (mamba: returns
#     '"café"', no escape is applied even by default);
#   • json.JSONDecodeError.__name__ == "JSONDecodeError" —
#     decode-error class identity (mamba: None);
#   • json.JSONDecoder.__name__ == "JSONDecoder" (mamba: None);
#   • json.JSONEncoder.__name__ == "JSONEncoder" (mamba: None);
#   • textwrap.fill("hello world this is text", width=10) ==
#     "hello\\nworld this\\nis text" — column-bound rewrap
#     (mamba: returns the input verbatim, no wrap occurs);
#   • textwrap.wrap("hello world", width=10) == ["hello",
#     "world"] (mamba: returns ["hello world"], no split);
#   • textwrap.shorten("hello world this is", width=15) ==
#     "hello [...]" (mamba: returns the input verbatim);
#   • textwrap.TextWrapper.__name__ == "TextWrapper" (mamba:
#     None).
import ipaddress as _ipaddress_mod
import urllib.parse as _urlparse_mod
import html as _html_mod
import json as _json_mod
import textwrap as _textwrap_mod
from typing import Any

# Module bindings retyped as `Any` to bypass Pyright stub-driven
# narrowing — every spec contract below probes documented public
# class identifiers / module-level helpers / submodule surface
# that mamba's bundled type stubs do not surface accurately.
ipaddress: Any = _ipaddress_mod
urlparse_mod: Any = _urlparse_mod
html: Any = _html_mod
json: Any = _json_mod
textwrap: Any = _textwrap_mod


_ledger: list[int] = []

# 1) ipaddress — bare class identity
assert ipaddress.IPv4Address.__name__ == "IPv4Address"; _ledger.append(1)
assert ipaddress.IPv6Address.__name__ == "IPv6Address"; _ledger.append(1)
assert ipaddress.IPv4Network.__name__ == "IPv4Network"; _ledger.append(1)

# 2) ipaddress — integer coercion of IPv4Address
assert int(ipaddress.IPv4Address("192.168.1.1")) == 3232235777; _ledger.append(1)

# 3) ipaddress — IPv4Network num_addresses
assert ipaddress.IPv4Network("192.168.0.0/24").num_addresses == 256; _ledger.append(1)

# 4) urllib.parse.ParseResult — documented namedtuple-style repr
assert repr(urlparse_mod.urlparse("https://a.com/p?q=1#f")).startswith("ParseResult(scheme="); _ledger.append(1)

# 5) html.entities — documented submodule surface
assert hasattr(html, "entities") == True; _ledger.append(1)

# 6) json.dumps — default ensure_ascii=True unicode escape
assert json.dumps("café") == '"caf\\u00e9"'; _ledger.append(1)

# 7) json — exception + codec class identity
assert json.JSONDecodeError.__name__ == "JSONDecodeError"; _ledger.append(1)
assert json.JSONDecoder.__name__ == "JSONDecoder"; _ledger.append(1)
assert json.JSONEncoder.__name__ == "JSONEncoder"; _ledger.append(1)

# 8) textwrap.fill — column-bound rewrap
assert textwrap.fill("hello world this is text", width=10) == "hello\nworld this\nis text"; _ledger.append(1)

# 9) textwrap.wrap — list of wrapped lines
assert textwrap.wrap("hello world", width=10) == ["hello", "world"]; _ledger.append(1)

# 10) textwrap.shorten — width-bounded summary
assert textwrap.shorten("hello world this is", width=15) == "hello [...]"; _ledger.append(1)

# 11) textwrap.TextWrapper — bare class identity
assert textwrap.TextWrapper.__name__ == "TextWrapper"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_ipaddress_textwrap_json_silent {sum(_ledger)} asserts")
