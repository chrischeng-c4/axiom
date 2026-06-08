# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "behavior"
# case = "lwp_cookie_tests__test_netscape_example_1"
# subject = "cpython.test_http_cookiejar.LWPCookieTests.test_netscape_example_1"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_http_cookiejar.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_http_cookiejar.py::LWPCookieTests::test_netscape_example_1
"""Auto-ported test: LWPCookieTests::test_netscape_example_1 (CPython 3.12 oracle)."""


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
year_plus_one = time.localtime()[0] + 1
headers = []
c = CookieJar(DefaultCookiePolicy(rfc2965=True))
req = urllib.request.Request('http://www.acme.com:80/', headers={'Host': 'www.acme.com:80'})
headers.append('Set-Cookie: CUSTOMER=WILE_E_COYOTE; path=/ ; expires=Wednesday, 09-Nov-%d 23:12:40 GMT' % year_plus_one)
res = FakeResponse(headers, 'http://www.acme.com/')
c.extract_cookies(res, req)
req = urllib.request.Request('http://www.acme.com/')
c.add_cookie_header(req)

assert req.get_header('Cookie') == 'CUSTOMER=WILE_E_COYOTE'

assert req.get_header('Cookie2') == '$Version="1"'
headers.append('Set-Cookie: PART_NUMBER=ROCKET_LAUNCHER_0001; path=/')
res = FakeResponse(headers, 'http://www.acme.com/')
c.extract_cookies(res, req)
req = urllib.request.Request('http://www.acme.com/foo/bar')
c.add_cookie_header(req)
h = req.get_header('Cookie')

assert 'PART_NUMBER=ROCKET_LAUNCHER_0001' in h

assert 'CUSTOMER=WILE_E_COYOTE' in h
headers.append('Set-Cookie: SHIPPING=FEDEX; path=/foo')
res = FakeResponse(headers, 'http://www.acme.com')
c.extract_cookies(res, req)
req = urllib.request.Request('http://www.acme.com/')
c.add_cookie_header(req)
h = req.get_header('Cookie')

assert 'PART_NUMBER=ROCKET_LAUNCHER_0001' in h

assert 'CUSTOMER=WILE_E_COYOTE' in h

assert 'SHIPPING=FEDEX' not in h
req = urllib.request.Request('http://www.acme.com/foo/')
c.add_cookie_header(req)
h = req.get_header('Cookie')

assert 'PART_NUMBER=ROCKET_LAUNCHER_0001' in h

assert 'CUSTOMER=WILE_E_COYOTE' in h

assert h.startswith('SHIPPING=FEDEX;')
print("LWPCookieTests::test_netscape_example_1: ok")
