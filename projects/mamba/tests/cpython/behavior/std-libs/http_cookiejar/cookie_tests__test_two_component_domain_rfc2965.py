# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "behavior"
# case = "cookie_tests__test_two_component_domain_rfc2965"
# subject = "cpython.test_http_cookiejar.CookieTests.test_two_component_domain_rfc2965"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_http_cookiejar.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_http_cookiejar.py::CookieTests::test_two_component_domain_rfc2965
"""Auto-ported test: CookieTests::test_two_component_domain_rfc2965 (CPython 3.12 oracle)."""


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
c = CookieJar(pol)
interact_2965(c, 'http://foo.net/', 'foo=bar; Version="1"')

assert len(c) == 1

assert c._cookies['foo.net']['/']['foo'].value == 'bar'

assert interact_2965(c, 'http://foo.net/') == '$Version=1; foo=bar'

assert interact_2965(c, 'http://www.foo.net/') == ''
interact_2965(c, 'http://foo.net/foo', 'spam=eggs; domain=foo.net; path=/foo; Version="1"')

assert len(c) == 1

assert interact_2965(c, 'http://foo.net/foo') == '$Version=1; foo=bar'
interact_2965(c, 'http://www.foo.net/foo/', 'spam=eggs; domain=foo.net; Version="1"')

assert c._cookies['.foo.net']['/foo/']['spam'].value == 'eggs'

assert len(c) == 2

assert interact_2965(c, 'http://foo.net/foo/') == '$Version=1; foo=bar'

assert interact_2965(c, 'http://www.foo.net/foo/') == '$Version=1; spam=eggs; $Domain="foo.net"'
interact_2965(c, 'http://foo.net/', 'ni="ni"; domain=".net"; Version="1"')

assert len(c) == 2
interact_2965(c, 'http://foo.co.uk/', 'nasty=trick; domain=.co.uk; Version="1"')

assert len(c) == 3
print("CookieTests::test_two_component_domain_rfc2965: ok")
