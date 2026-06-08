# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "behavior"
# case = "lwp_cookie_tests__test_mozilla"
# subject = "cpython.test_http_cookiejar.LWPCookieTests.test_mozilla"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_http_cookiejar.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_http_cookiejar.py::LWPCookieTests::test_mozilla
"""Auto-ported test: LWPCookieTests::test_mozilla (CPython 3.12 oracle)."""


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
filename = os_helper.TESTFN
c = MozillaCookieJar(filename, policy=DefaultCookiePolicy(rfc2965=True))
interact_2965(c, 'http://www.acme.com/', 'foo1=bar; max-age=100; Version=1')
interact_2965(c, 'http://www.acme.com/', 'foo2=bar; port="80"; max-age=100; Discard; Version=1')
interact_2965(c, 'http://www.acme.com/', 'foo3=bar; secure; Version=1')
expires = 'expires=09-Nov-%d 23:12:40 GMT' % (year_plus_one,)
interact_netscape(c, 'http://www.foo.com/', 'fooa=bar; %s' % expires)
interact_netscape(c, 'http://www.foo.com/', 'foob=bar; Domain=.foo.com; %s' % expires)
interact_netscape(c, 'http://www.foo.com/', 'fooc=bar; Domain=www.foo.com; %s' % expires)
for cookie in c:
    if cookie.name == 'foo1':
        cookie.set_nonstandard_attr('HTTPOnly', '')

def save_and_restore(cj, ignore_discard):
    try:
        cj.save(ignore_discard=ignore_discard)
        new_c = MozillaCookieJar(filename, DefaultCookiePolicy(rfc2965=True))
        new_c.load(ignore_discard=ignore_discard)
    finally:
        os_helper.unlink(filename)
    return new_c
new_c = save_and_restore(c, True)

assert len(new_c) == 6

assert "name='foo1', value='bar'" in repr(new_c)

assert "rest={'HTTPOnly': ''}" in repr(new_c)
new_c = save_and_restore(c, False)

assert len(new_c) == 4

assert "name='foo1', value='bar'" in repr(new_c)
print("LWPCookieTests::test_mozilla: ok")
