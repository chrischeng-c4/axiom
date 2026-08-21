# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "behavior"
# case = "cookie_tests__test_domain_block"
# subject = "cpython.test_http_cookiejar.CookieTests.test_domain_block"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_http_cookiejar.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_http_cookiejar.py::CookieTests::test_domain_block
"""Auto-ported test: CookieTests::test_domain_block (CPython 3.12 oracle)."""


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
pol = DefaultCookiePolicy(rfc2965=True, blocked_domains=['.acme.com'])
c = CookieJar(policy=pol)
headers = ['Set-Cookie: CUSTOMER=WILE_E_COYOTE; path=/']
req = urllib.request.Request('http://www.acme.com/')
res = FakeResponse(headers, 'http://www.acme.com/')
c.extract_cookies(res, req)

assert len(c) == 0
p = pol.set_blocked_domains(['acme.com'])
c.extract_cookies(res, req)

assert len(c) == 1
c.clear()
req = urllib.request.Request('http://www.roadrunner.net/')
res = FakeResponse(headers, 'http://www.roadrunner.net/')
c.extract_cookies(res, req)

assert len(c) == 1
req = urllib.request.Request('http://www.roadrunner.net/')
c.add_cookie_header(req)

assert req.has_header('Cookie')

assert req.has_header('Cookie2')
c.clear()
pol.set_blocked_domains(['.acme.com'])
c.extract_cookies(res, req)

assert len(c) == 1
req = urllib.request.Request('http://www.acme.com/')
res = FakeResponse(headers, 'http://www.acme.com/')
cookies = c.make_cookies(res, req)
c.set_cookie(cookies[0])

assert len(c) == 2
c.add_cookie_header(req)

assert not req.has_header('Cookie')
c.clear()
pol.set_blocked_domains([])
req = urllib.request.Request('http://acme.com/')
res = FakeResponse(headers, 'http://acme.com/')
cookies = c.make_cookies(res, req)
c.extract_cookies(res, req)

assert len(c) == 1
req = urllib.request.Request('http://acme.com/')
c.add_cookie_header(req)

assert req.has_header('Cookie')
req = urllib.request.Request('http://badacme.com/')
c.add_cookie_header(req)

assert not pol.return_ok(cookies[0], req)

assert not req.has_header('Cookie')
p = pol.set_blocked_domains(['acme.com'])
req = urllib.request.Request('http://acme.com/')
c.add_cookie_header(req)

assert not req.has_header('Cookie')
req = urllib.request.Request('http://badacme.com/')
c.add_cookie_header(req)

assert not req.has_header('Cookie')
print("CookieTests::test_domain_block: ok")
