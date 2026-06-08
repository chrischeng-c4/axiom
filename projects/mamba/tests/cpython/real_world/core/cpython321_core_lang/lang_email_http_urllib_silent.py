# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_email_http_urllib_silent"
# subject = "cpython321.lang_email_http_urllib_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_email_http_urllib_silent.py"
# status = "filled"
# ///
"""cpython321.lang_email_http_urllib_silent: execute CPython 3.12 seed lang_email_http_urllib_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass divergence-spec fixture for the silent
# value-contract divergences of `hasattr(email, 'message_from_string')`
# (the documented "email exposes the message_from_string parser entry"
# — mamba returns False — email is a dict), `hasattr(email.utils,
# 'parseaddr')` (the documented "email.utils exposes parseaddr" —
# mamba returns False — email.utils is None), `email.utils.parseaddr
# ('name <a@b>')` (the documented "parseaddr returns ('name', 'a@b')"
# — mamba returns ['', '']), `email.utils.formataddr(('name', 'a@b'))`
# (the documented "formataddr returns 'name <a@b>'" — mamba returns
# ''), `hasattr(http.client, 'HTTPConnection')` (the documented
# "http.client exposes the HTTPConnection class" — mamba returns
# False — http.client is None), `hasattr(http.client, 'OK')` (the
# documented "http.client exposes the OK status alias" — mamba
# returns False), `hasattr(http.client, 'responses')` (the
# documented "http.client exposes the responses status-text map" —
# mamba returns False), `http.HTTPStatus.OK.value` (the documented
# "HTTPStatus.OK.value is 200" — mamba returns None), `type(http.
# HTTPStatus).__name__` (the documented "HTTPStatus metaclass is
# EnumType" — mamba returns 'HTTPStatus' — HTTPStatus is an instance,
# not an enum class), and `hasattr(urllib.parse, 'urlparse')` (the
# documented "urllib.parse exposes urlparse" — mamba returns False —
# urllib.parse is None).
# Ten-pack pinned to atomic 277.
#
# Behavioral edges that CONFORM on mamba (urllib.parse — from-import
# urlparse/quote/unquote/urlencode/urljoin/quote_plus/unquote_plus all
# callable + not None. email.utils — from-import parseaddr/formataddr/
# formatdate/make_msgid all callable + not None. http.client —
# from-import HTTPConnection/HTTPResponse/HTTPSConnection all callable
# + not None. http — from-import HTTPStatus is not None + hasattr
# HTTPStatus.OK/CREATED/ACCEPTED/NO_CONTENT/MOVED_PERMANENTLY/BAD_
# REQUEST/UNAUTHORIZED/FORBIDDEN/NOT_FOUND/INTERNAL_SERVER_ERROR/BAD_
# GATEWAY/SERVICE_UNAVAILABLE) are covered in the matching pass
# fixture `test_email_http_urllib_value_ops`.
import email
import email.utils
import http
import http.client
import urllib
import urllib.parse


_ledger: list[int] = []

# 1) hasattr(email, 'message_from_string') — parser entry
#    (mamba: returns False — email is a dict)
assert hasattr(email, "message_from_string") == True; _ledger.append(1)

# 2) hasattr(email.utils, 'parseaddr') — RFC 5322 parser
#    (mamba: returns False — email.utils is None)
assert hasattr(email.utils, "parseaddr") == True; _ledger.append(1)

# 3) email.utils.parseaddr('name <a@b>') — ('name', 'a@b')
#    (mamba: returns ['', ''])
assert email.utils.parseaddr("name <a@b>") == ("name", "a@b"); _ledger.append(1)

# 4) email.utils.formataddr(('name', 'a@b')) — 'name <a@b>'
#    (mamba: returns '')
assert email.utils.formataddr(("name", "a@b")) == "name <a@b>"; _ledger.append(1)

# 5) hasattr(http.client, 'HTTPConnection') — connection class
#    (mamba: returns False — http.client is None)
assert hasattr(http.client, "HTTPConnection") == True; _ledger.append(1)

# 6) hasattr(http.client, 'OK') — status alias
#    (mamba: returns False)
assert hasattr(http.client, "OK") == True; _ledger.append(1)

# 7) hasattr(http.client, 'responses') — status-text map
#    (mamba: returns False)
assert hasattr(http.client, "responses") == True; _ledger.append(1)

# 8) http.HTTPStatus.OK.value == 200 — enum value
#    (mamba: returns None)
assert http.HTTPStatus.OK.value == 200; _ledger.append(1)

# 9) type(http.HTTPStatus).__name__ == 'EnumType' — metaclass
#    (mamba: returns 'HTTPStatus' — instance, not enum class)
assert type(http.HTTPStatus).__name__ == "EnumType"; _ledger.append(1)

# 10) hasattr(urllib.parse, 'urlparse') — URL parser
#     (mamba: returns False — urllib.parse is None)
assert hasattr(urllib.parse, "urlparse") == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_email_http_urllib_silent {sum(_ledger)} asserts")
