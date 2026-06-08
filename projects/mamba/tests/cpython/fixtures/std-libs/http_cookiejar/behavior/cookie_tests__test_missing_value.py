# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "behavior"
# case = "cookie_tests__test_missing_value"
# subject = "cpython.test_http_cookiejar.CookieTests.test_missing_value"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_http_cookiejar.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_http_cookiejar.py::CookieTests::test_missing_value
"""Auto-ported test: CookieTests::test_missing_value (CPython 3.12 oracle)."""


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
filename = os_helper.TESTFN
c = MozillaCookieJar(filename)
interact_netscape(c, 'http://www.acme.com/', 'eggs')
interact_netscape(c, 'http://www.acme.com/', '"spam"; path=/foo/')
cookie = c._cookies['www.acme.com']['/']['eggs']

assert cookie.value is None

assert cookie.name == 'eggs'
cookie = c._cookies['www.acme.com']['/foo/']['"spam"']

assert cookie.value is None

assert cookie.name == '"spam"'

assert lwp_cookie_str(cookie) == '"spam"; path="/foo/"; domain="www.acme.com"; path_spec; discard; version=0'
old_str = repr(c)
c.save(ignore_expires=True, ignore_discard=True)
try:
    c = MozillaCookieJar(filename)
    c.revert(ignore_expires=True, ignore_discard=True)
finally:
    os_helper.unlink(c.filename)

assert repr(c) == re.sub('path_specified=%s' % True, 'path_specified=%s' % False, old_str)

assert interact_netscape(c, 'http://www.acme.com/foo/') == '"spam"; eggs'
print("CookieTests::test_missing_value: ok")
