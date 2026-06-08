# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_parse"
# dimension = "behavior"
# case = "url_parse_test_case__test_parse_qsl_errors"
# subject = "cpython.test_urlparse.UrlParseTestCase.test_parse_qsl_errors"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_urlparse.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_urlparse.py::UrlParseTestCase::test_parse_qsl_errors
"""Auto-ported test: UrlParseTestCase::test_parse_qsl_errors (CPython 3.12 oracle)."""


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
    urllib.parse.parse_qsl(list(b'a=b'))
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    urllib.parse.parse_qsl(iter(b'a=b'))
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    urllib.parse.parse_qsl(1)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    urllib.parse.parse_qsl(object())
    raise AssertionError('expected TypeError')
except TypeError:
    pass
for separator in ('', b'', None, 0, 1, 0.0, 1.5):
    try:
        urllib.parse.parse_qsl('a=b', separator=separator)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass
try:
    urllib.parse.parse_qsl(b'a=b', separator='¦')
    raise AssertionError('expected UnicodeEncodeError')
except UnicodeEncodeError:
    pass
try:
    urllib.parse.parse_qsl('a=b', separator=b'\xa6')
    raise AssertionError('expected UnicodeDecodeError')
except UnicodeDecodeError:
    pass
print("UrlParseTestCase::test_parse_qsl_errors: ok")
