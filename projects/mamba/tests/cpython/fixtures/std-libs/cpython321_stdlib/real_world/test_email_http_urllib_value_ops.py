# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_email_http_urllib_value_ops"
# subject = "cpython321.test_email_http_urllib_value_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_email_http_urllib_value_ops.py"
# status = "filled"
# ///
"""cpython321.test_email_http_urllib_value_ops: execute CPython 3.12 seed test_email_http_urllib_value_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Atomic 277 pass conformance — email/email.utils/email.message/email.parser/
# email.mime + http/http.client/http.server/http.cookies/http.cookiejar +
# urllib/urllib.parse/urllib.request/urllib.error/urllib.response import
# success + from-import callable surface (urlparse/quote/unquote/urlencode/
# urljoin/quote_plus/unquote_plus from urllib.parse + parseaddr/formataddr/
# formatdate/make_msgid from email.utils + HTTPConnection/HTTPResponse/
# HTTPSConnection from http.client + HTTPStatus from http) + HTTPStatus
# hasattr standard codes (OK/NOT_FOUND/CREATED/INTERNAL_SERVER_ERROR/
# BAD_REQUEST/UNAUTHORIZED/FORBIDDEN/ACCEPTED/MOVED_PERMANENTLY/
# SERVICE_UNAVAILABLE/NO_CONTENT/BAD_GATEWAY).
# All asserts match between CPython 3.12 and mamba.
import email
import email.utils
import email.message
import email.parser
import email.mime
import http
import http.client
import http.server
import http.cookies
import http.cookiejar
import urllib
import urllib.parse
import urllib.request
import urllib.error
import urllib.response

from urllib.parse import urlparse, quote, unquote, urlencode, urljoin, quote_plus, unquote_plus
from email.utils import parseaddr, formataddr, formatdate, make_msgid
from http.client import HTTPConnection, HTTPResponse, HTTPSConnection
from http import HTTPStatus


_ledger: list[int] = []

# 1) urllib.parse — from-import callable surface
assert callable(urlparse) == True; _ledger.append(1)
assert callable(quote) == True; _ledger.append(1)
assert callable(unquote) == True; _ledger.append(1)
assert callable(urlencode) == True; _ledger.append(1)
assert callable(urljoin) == True; _ledger.append(1)
assert callable(quote_plus) == True; _ledger.append(1)
assert callable(unquote_plus) == True; _ledger.append(1)

# 2) urllib.parse — from-import bindings are not None
assert (urlparse is not None) == True; _ledger.append(1)
assert (quote is not None) == True; _ledger.append(1)
assert (unquote is not None) == True; _ledger.append(1)
assert (urlencode is not None) == True; _ledger.append(1)
assert (urljoin is not None) == True; _ledger.append(1)
assert (quote_plus is not None) == True; _ledger.append(1)
assert (unquote_plus is not None) == True; _ledger.append(1)

# 3) email.utils — from-import callable surface
assert callable(parseaddr) == True; _ledger.append(1)
assert callable(formataddr) == True; _ledger.append(1)
assert callable(formatdate) == True; _ledger.append(1)
assert callable(make_msgid) == True; _ledger.append(1)

# 4) email.utils — from-import bindings are not None
assert (parseaddr is not None) == True; _ledger.append(1)
assert (formataddr is not None) == True; _ledger.append(1)
assert (formatdate is not None) == True; _ledger.append(1)
assert (make_msgid is not None) == True; _ledger.append(1)

# 5) http.client — from-import callable surface
assert callable(HTTPConnection) == True; _ledger.append(1)
assert callable(HTTPResponse) == True; _ledger.append(1)
assert callable(HTTPSConnection) == True; _ledger.append(1)

# 6) http.client — from-import bindings are not None
assert (HTTPConnection is not None) == True; _ledger.append(1)
assert (HTTPResponse is not None) == True; _ledger.append(1)
assert (HTTPSConnection is not None) == True; _ledger.append(1)

# 7) http — HTTPStatus from-import is not None
assert (HTTPStatus is not None) == True; _ledger.append(1)

# 8) http.HTTPStatus — hasattr standard 2xx codes
assert hasattr(HTTPStatus, "OK") == True; _ledger.append(1)
assert hasattr(HTTPStatus, "CREATED") == True; _ledger.append(1)
assert hasattr(HTTPStatus, "ACCEPTED") == True; _ledger.append(1)
assert hasattr(HTTPStatus, "NO_CONTENT") == True; _ledger.append(1)

# 9) http.HTTPStatus — hasattr standard 3xx/4xx codes
assert hasattr(HTTPStatus, "MOVED_PERMANENTLY") == True; _ledger.append(1)
assert hasattr(HTTPStatus, "BAD_REQUEST") == True; _ledger.append(1)
assert hasattr(HTTPStatus, "UNAUTHORIZED") == True; _ledger.append(1)
assert hasattr(HTTPStatus, "FORBIDDEN") == True; _ledger.append(1)
assert hasattr(HTTPStatus, "NOT_FOUND") == True; _ledger.append(1)

# 10) http.HTTPStatus — hasattr standard 5xx codes
assert hasattr(HTTPStatus, "INTERNAL_SERVER_ERROR") == True; _ledger.append(1)
assert hasattr(HTTPStatus, "BAD_GATEWAY") == True; _ledger.append(1)
assert hasattr(HTTPStatus, "SERVICE_UNAVAILABLE") == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_email_http_urllib_value_ops {sum(_ledger)} asserts")
