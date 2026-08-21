# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "behavior"
# case = "header_tests__test_split_header_words"
# subject = "cpython.test_http_cookiejar.HeaderTests.test_split_header_words"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_http_cookiejar.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_http_cookiejar.py::HeaderTests::test_split_header_words
"""Auto-ported test: HeaderTests::test_split_header_words (CPython 3.12 oracle)."""


import os
import stat
import sys
import re
from test.support import os_helper
from test.support import warnings_helper
import time
import unittest
import urllib.request
from http.cookiejar import time2isoz, http2time, iso2time, time2netscape, parse_ns_headers, join_header_words, split_header_words, Cookie, CookieJar, DefaultCookiePolicy, LWPCookieJar, MozillaCookieJar, LoadError, lwp_cookie_str, DEFAULT_HTTP_PORT, escape_path, reach, is_HDN, domain_match, user_domain_match, request_path, request_port, request_host


'Tests for http/cookiejar.py.'

mswindows = sys.platform == 'win32'

class FakeResponse:

    def __init__(self, headers=[], url=None):
        """
        headers: list of RFC822-style 'Key: value' strings
        """
        import email
        self._headers = email.message_from_string('\n'.join(headers))
        self._url = url

    def info(self):
        return self._headers

def interact_2965(cookiejar, url, *set_cookie_hdrs):
    return _interact(cookiejar, url, set_cookie_hdrs, 'Set-Cookie2')

def interact_netscape(cookiejar, url, *set_cookie_hdrs):
    return _interact(cookiejar, url, set_cookie_hdrs, 'Set-Cookie')

def _interact(cookiejar, url, set_cookie_hdrs, hdr_name):
    """Perform a single request / response cycle, returning Cookie: header."""
    req = urllib.request.Request(url)
    cookiejar.add_cookie_header(req)
    cookie_hdr = req.get_header('Cookie', '')
    headers = []
    for hdr in set_cookie_hdrs:
        headers.append('%s: %s' % (hdr_name, hdr))
    res = FakeResponse(headers, url)
    cookiejar.extract_cookies(res, req)
    return cookie_hdr


# --- test body ---
tests = [('foo', [[('foo', None)]]), ('foo=bar', [[('foo', 'bar')]]), ('   foo   ', [[('foo', None)]]), ('   foo=   ', [[('foo', '')]]), ('   foo=', [[('foo', '')]]), ('   foo=   ; ', [[('foo', '')]]), ('   foo=   ; bar= baz ', [[('foo', ''), ('bar', 'baz')]]), ('foo=bar bar=baz', [[('foo', 'bar'), ('bar', 'baz')]]), ('foo= bar=baz', [[('foo', 'bar=baz')]]), ('foo=bar;bar=baz', [[('foo', 'bar'), ('bar', 'baz')]]), ('foo bar baz', [[('foo', None), ('bar', None), ('baz', None)]]), ('a, b, c', [[('a', None)], [('b', None)], [('c', None)]]), ('foo; bar=baz, spam=, foo="\\,\\;\\"", bar= ', [[('foo', None), ('bar', 'baz')], [('spam', '')], [('foo', ',;"')], [('bar', '')]])]
for arg, expect in tests:
    try:
        result = split_header_words([arg])
    except:
        import traceback, io
        f = io.StringIO()
        traceback.print_exc(None, f)
        result = '(error -- traceback follows)\n\n%s' % f.getvalue()

    assert result == expect
print("HeaderTests::test_split_header_words: ok")
