# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "behavior"
# case = "lwp_cookie_tests__test_rejection"
# subject = "cpython.test_http_cookiejar.LWPCookieTests.test_rejection"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_http_cookiejar.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_http_cookiejar.py::LWPCookieTests::test_rejection
"""Auto-ported test: LWPCookieTests::test_rejection (CPython 3.12 oracle)."""


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
pol = DefaultCookiePolicy(rfc2965=True)
c = LWPCookieJar(policy=pol)
max_age = 'max-age=3600'
cookie = interact_2965(c, 'http://www.acme.com', 'foo=bar; domain=".com"; version=1')

assert not c
cookie = interact_2965(c, 'http://www.acme.com', 'ping=pong; domain="acme.com"; version=1')

assert len(c) == 1
cookie = interact_2965(c, 'http://www.a.acme.com', 'whiz=bang; domain="acme.com"; version=1')

assert len(c) == 1
cookie = interact_2965(c, 'http://www.a.acme.com', 'wow=flutter; domain=".a.acme.com"; version=1')

assert len(c) == 2
cookie = interact_2965(c, 'http://125.125.125.125', 'zzzz=ping; domain="125.125.125"; version=1')

assert len(c) == 2
cookie = interact_2965(c, 'http://www.sol.no', 'blah=rhubarb; domain=".sol.no"; path="/foo"; version=1')

assert len(c) == 2
cookie = interact_2965(c, 'http://www.sol.no/foo/bar', 'bing=bong; domain=".sol.no"; path="/foo"; version=1')

assert len(c) == 3
cookie = interact_2965(c, 'http://www.sol.no', 'whiz=ffft; domain=".sol.no"; port="90,100"; version=1')

assert len(c) == 3
cookie = interact_2965(c, 'http://www.sol.no', 'bang=wallop; version=1; domain=".sol.no"; port="90,100, 80,8080"; max-age=100; Comment = "Just kidding! (\\"|\\\\\\\\) "')

assert len(c) == 4
cookie = interact_2965(c, 'http://www.sol.no', 'foo9=bar; version=1; domain=".sol.no"; port; max-age=100;')

assert len(c) == 5
cookie = interact_2965(c, 'http://www.sol.no/<oo/', 'foo8=bar; version=1; path="/%3coo"')

assert len(c) == 6
filename = os_helper.TESTFN
try:
    c.save(filename, ignore_discard=True)
    old = repr(c)
    c = LWPCookieJar(policy=pol)
    c.load(filename, ignore_discard=True)
finally:
    os_helper.unlink(filename)

assert old == repr(c)
print("LWPCookieTests::test_rejection: ok")
