# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "behavior"
# case = "cookie_tests__test_two_component_domain_ns"
# subject = "cpython.test_http_cookiejar.CookieTests.test_two_component_domain_ns"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_http_cookiejar.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_http_cookiejar.py::CookieTests::test_two_component_domain_ns
"""Auto-ported test: CookieTests::test_two_component_domain_ns (CPython 3.12 oracle)."""


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
c = CookieJar()
interact_netscape(c, 'http://foo.net/', 'ns=bar')

assert len(c) == 1

assert c._cookies['foo.net']['/']['ns'].value == 'bar'

assert interact_netscape(c, 'http://foo.net/') == 'ns=bar'

assert interact_netscape(c, 'http://www.foo.net/') == 'ns=bar'
pol = DefaultCookiePolicy(strict_ns_domain=DefaultCookiePolicy.DomainStrictNonDomain)
c.set_policy(pol)

assert interact_netscape(c, 'http://www.foo.net/') == ''
interact_netscape(c, 'http://foo.net/foo/', 'spam1=eggs; domain=foo.net')
interact_netscape(c, 'http://foo.net/foo/bar/', 'spam2=eggs; domain=.foo.net')

assert len(c) == 3

assert c._cookies['.foo.net']['/foo']['spam1'].value == 'eggs'

assert c._cookies['.foo.net']['/foo/bar']['spam2'].value == 'eggs'

assert interact_netscape(c, 'http://foo.net/foo/bar/') == 'spam2=eggs; spam1=eggs; ns=bar'
interact_netscape(c, 'http://foo.net/', 'nini="ni"; domain=.net')

assert len(c) == 3
interact_netscape(c, 'http://foo.co.uk', 'nasty=trick; domain=.co.uk')

assert len(c) == 4
print("CookieTests::test_two_component_domain_ns: ok")
