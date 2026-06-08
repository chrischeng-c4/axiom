# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_parse"
# dimension = "behavior"
# case = "url_parse_test_case__test_invalid_bracketed_hosts"
# subject = "cpython.test_urlparse.UrlParseTestCase.test_invalid_bracketed_hosts"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_urlparse.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_urlparse.py::UrlParseTestCase::test_invalid_bracketed_hosts
"""Auto-ported test: UrlParseTestCase::test_invalid_bracketed_hosts (CPython 3.12 oracle)."""


import sys
import unicodedata
import unittest
import urllib.parse


RFC1808_BASE = 'http://a/b/c/d;p?q#f'

RFC2396_BASE = 'http://a/b/c/d;p?q'

RFC3986_BASE = 'http://a/b/c/d;p?q'

SIMPLE_BASE = 'http://a/b/c/d'

parse_qsl_test_cases = [('', []), ('&', []), ('&&', []), ('=', [('', '')]), ('=a', [('', 'a')]), ('a', [('a', '')]), ('a=', [('a', '')]), ('a=b=c', [('a', 'b=c')]), ('a%3Db=c', [('a=b', 'c')]), ('a=b&c=d', [('a', 'b'), ('c', 'd')]), ('a=b%26c=d', [('a', 'b&c=d')]), ('&a=b', [('a', 'b')]), ('a=a+b&b=b+c', [('a', 'a b'), ('b', 'b c')]), ('a=1&a=2', [('a', '1'), ('a', '2')]), (b'', []), (b'&', []), (b'&&', []), (b'=', [(b'', b'')]), (b'=a', [(b'', b'a')]), (b'a', [(b'a', b'')]), (b'a=', [(b'a', b'')]), (b'a=b=c', [(b'a', b'b=c')]), (b'a%3Db=c', [(b'a=b', b'c')]), (b'a=b&c=d', [(b'a', b'b'), (b'c', b'd')]), (b'a=b%26c=d', [(b'a', b'b&c=d')]), (b'&a=b', [(b'a', b'b')]), (b'a=a+b&b=b+c', [(b'a', b'a b'), (b'b', b'b c')]), (b'a=1&a=2', [(b'a', b'1'), (b'a', b'2')]), (';a=b', [(';a', 'b')]), ('a=a+b;b=b+c', [('a', 'a b;b=b c')]), (b';a=b', [(b';a', b'b')]), (b'a=a+b;b=b+c', [(b'a', b'a b;b=b c')]), ('Ł=é', [('Ł', 'é')]), ('%C5%81=%C3%A9', [('Ł', 'é')]), ('%81=%A9', [('�', '�')]), (b'\xc5\x81=\xc3\xa9', [(b'\xc5\x81', b'\xc3\xa9')]), (b'%C5%81=%C3%A9', [(b'\xc5\x81', b'\xc3\xa9')]), (b'\x81=\xa9', [(b'\x81', b'\xa9')]), (b'%81=%A9', [(b'\x81', b'\xa9')])]

parse_qs_test_cases = [('', {}), ('&', {}), ('&&', {}), ('=', {'': ['']}), ('=a', {'': ['a']}), ('a', {'a': ['']}), ('a=', {'a': ['']}), ('a=b=c', {'a': ['b=c']}), ('a%3Db=c', {'a=b': ['c']}), ('a=b&c=d', {'a': ['b'], 'c': ['d']}), ('a=b%26c=d', {'a': ['b&c=d']}), ('&a=b', {'a': ['b']}), ('a=a+b&b=b+c', {'a': ['a b'], 'b': ['b c']}), ('a=1&a=2', {'a': ['1', '2']}), (b'', {}), (b'&', {}), (b'&&', {}), (b'=', {b'': [b'']}), (b'=a', {b'': [b'a']}), (b'a', {b'a': [b'']}), (b'a=', {b'a': [b'']}), (b'a=b=c', {b'a': [b'b=c']}), (b'a%3Db=c', {b'a=b': [b'c']}), (b'a=b&c=d', {b'a': [b'b'], b'c': [b'd']}), (b'a=b%26c=d', {b'a': [b'b&c=d']}), (b'&a=b', {b'a': [b'b']}), (b'a=a+b&b=b+c', {b'a': [b'a b'], b'b': [b'b c']}), (b'a=1&a=2', {b'a': [b'1', b'2']}), (';a=b', {';a': ['b']}), ('a=a+b;b=b+c', {'a': ['a b;b=b c']}), (b';a=b', {b';a': [b'b']}), (b'a=a+b;b=b+c', {b'a': [b'a b;b=b c']}), (b'a=a%E2%80%99b', {b'a': [b'a\xe2\x80\x99b']}), ('Ł=é', {'Ł': ['é']}), ('%C5%81=%C3%A9', {'Ł': ['é']}), ('%81=%A9', {'�': ['�']}), (b'\xc5\x81=\xc3\xa9', {b'\xc5\x81': [b'\xc3\xa9']}), (b'%C5%81=%C3%A9', {b'\xc5\x81': [b'\xc3\xa9']}), (b'\x81=\xa9', {b'\x81': [b'\xa9']}), (b'%81=%A9', {b'\x81': [b'\xa9']})]


# --- test body ---

try:
    urllib.parse.urlsplit('Scheme://user@[192.0.2.146]/Path?Query')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    urllib.parse.urlsplit('Scheme://user@[important.com:8000]/Path?Query')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    urllib.parse.urlsplit('Scheme://user@[v123r.IP]/Path?Query')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    urllib.parse.urlsplit('Scheme://user@[v12ae]/Path?Query')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    urllib.parse.urlsplit('Scheme://user@[v.IP]/Path?Query')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    urllib.parse.urlsplit('Scheme://user@[v123.]/Path?Query')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    urllib.parse.urlsplit('Scheme://user@[v]/Path?Query')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    urllib.parse.urlsplit('Scheme://user@[0439:23af::2309::fae7:1234]/Path?Query')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    urllib.parse.urlsplit('Scheme://user@[0439:23af:2309::fae7:1234:2342:438e:192.0.2.146]/Path?Query')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    urllib.parse.urlsplit('Scheme://user@]v6a.ip[/Path')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    urllib.parse.urlsplit('scheme://prefix.[v6a.ip]')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    urllib.parse.urlsplit('scheme://[v6a.ip].suffix')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    urllib.parse.urlsplit('scheme://prefix.[v6a.ip]/')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    urllib.parse.urlsplit('scheme://[v6a.ip].suffix/')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    urllib.parse.urlsplit('scheme://prefix.[v6a.ip]?')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    urllib.parse.urlsplit('scheme://[v6a.ip].suffix?')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    urllib.parse.urlsplit('scheme://prefix.[::1]')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    urllib.parse.urlsplit('scheme://[::1].suffix')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    urllib.parse.urlsplit('scheme://prefix.[::1]/')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    urllib.parse.urlsplit('scheme://[::1].suffix/')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    urllib.parse.urlsplit('scheme://prefix.[::1]?')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    urllib.parse.urlsplit('scheme://[::1].suffix?')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    urllib.parse.urlsplit('scheme://prefix.[::1]:a')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    urllib.parse.urlsplit('scheme://[::1].suffix:a')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    urllib.parse.urlsplit('scheme://prefix.[::1]:a1')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    urllib.parse.urlsplit('scheme://[::1].suffix:a1')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    urllib.parse.urlsplit('scheme://prefix.[::1]:1a')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    urllib.parse.urlsplit('scheme://[::1].suffix:1a')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    urllib.parse.urlsplit('scheme://prefix.[::1]:')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    urllib.parse.urlsplit('scheme://[::1].suffix:/')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    urllib.parse.urlsplit('scheme://prefix.[::1]:?')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    urllib.parse.urlsplit('scheme://user@prefix.[v6a.ip]')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    urllib.parse.urlsplit('scheme://user@[v6a.ip].suffix')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    urllib.parse.urlsplit('scheme://[v6a.ip')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    urllib.parse.urlsplit('scheme://v6a.ip]')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    urllib.parse.urlsplit('scheme://]v6a.ip[')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    urllib.parse.urlsplit('scheme://]v6a.ip')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    urllib.parse.urlsplit('scheme://v6a.ip[')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    urllib.parse.urlsplit('scheme://prefix.[v6a.ip')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    urllib.parse.urlsplit('scheme://v6a.ip].suffix')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    urllib.parse.urlsplit('scheme://prefix]v6a.ip[suffix')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    urllib.parse.urlsplit('scheme://prefix]v6a.ip')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    urllib.parse.urlsplit('scheme://v6a.ip[suffix')
    raise AssertionError('expected ValueError')
except ValueError:
    pass
print("UrlParseTestCase::test_invalid_bracketed_hosts: ok")
