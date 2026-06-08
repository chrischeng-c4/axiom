# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "behavior"
# case = "lwp_cookie_tests__test_session_cookies"
# subject = "cpython.test_http_cookiejar.LWPCookieTests.test_session_cookies"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_http_cookiejar.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_http_cookiejar.py::LWPCookieTests::test_session_cookies
"""Auto-ported test: LWPCookieTests::test_session_cookies (CPython 3.12 oracle)."""


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
req = urllib.request.Request('http://www.perlmeister.com/scripts')
headers = []
headers.append('Set-Cookie: s1=session;Path=/scripts')
headers.append('Set-Cookie: p1=perm; Domain=.perlmeister.com;Path=/;expires=Fri, 02-Feb-%d 23:24:20 GMT' % year_plus_one)
headers.append('Set-Cookie: p2=perm;Path=/;expires=Fri, 02-Feb-%d 23:24:20 GMT' % year_plus_one)
headers.append('Set-Cookie: s2=session;Path=/scripts;Domain=.perlmeister.com')
headers.append('Set-Cookie2: s3=session;Version=1;Discard;Path="/"')
res = FakeResponse(headers, 'http://www.perlmeister.com/scripts')
c = CookieJar()
c.extract_cookies(res, req)
counter = {'session_after': 0, 'perm_after': 0, 'session_before': 0, 'perm_before': 0}
for cookie in c:
    key = '%s_before' % cookie.value
    counter[key] = counter[key] + 1
c.clear_session_cookies()
for cookie in c:
    key = '%s_after' % cookie.value
    counter[key] = counter[key] + 1

assert counter['perm_after'] == counter['perm_before']

assert counter['session_after'] == 0

assert counter['session_before'] != 0
print("LWPCookieTests::test_session_cookies: ok")
