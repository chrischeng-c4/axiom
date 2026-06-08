# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "behavior"
# case = "cookie_tests__test_ns_parser"
# subject = "cpython.test_http_cookiejar.CookieTests.test_ns_parser"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_http_cookiejar.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_http_cookiejar.py::CookieTests::test_ns_parser
"""Auto-ported test: CookieTests::test_ns_parser (CPython 3.12 oracle)."""


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
interact_netscape(c, 'http://www.acme.com/', 'spam=eggs; DoMain=.acme.com; port; blArgh="feep"')
interact_netscape(c, 'http://www.acme.com/', 'ni=ni; port=80,8080')
interact_netscape(c, 'http://www.acme.com:80/', 'nini=ni')
interact_netscape(c, 'http://www.acme.com:80/', 'foo=bar; expires=')
interact_netscape(c, 'http://www.acme.com:80/', 'spam=eggs; expires="Foo Bar 25 33:22:11 3022"')
interact_netscape(c, 'http://www.acme.com/', 'fortytwo=')
interact_netscape(c, 'http://www.acme.com/', '=unladenswallow')
interact_netscape(c, 'http://www.acme.com/', 'holyhandgrenade')
cookie = c._cookies['.acme.com']['/']['spam']

assert cookie.domain == '.acme.com'

assert cookie.domain_specified

assert cookie.port == DEFAULT_HTTP_PORT

assert not cookie.port_specified

assert cookie.has_nonstandard_attr('blArgh')

assert not cookie.has_nonstandard_attr('blargh')
cookie = c._cookies['www.acme.com']['/']['ni']

assert cookie.domain == 'www.acme.com'

assert not cookie.domain_specified

assert cookie.port == '80,8080'

assert cookie.port_specified
cookie = c._cookies['www.acme.com']['/']['nini']

assert cookie.port is None

assert not cookie.port_specified
foo = c._cookies['www.acme.com']['/']['foo']
spam = c._cookies['www.acme.com']['/']['foo']

assert foo.expires is None

assert spam.expires is None
cookie = c._cookies['www.acme.com']['/']['fortytwo']

assert cookie.value is not None

assert cookie.value == ''
cookie = c._cookies['www.acme.com']['/']['holyhandgrenade']

assert cookie.value is None
print("CookieTests::test_ns_parser: ok")
