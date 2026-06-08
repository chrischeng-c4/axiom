# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_parse"
# dimension = "behavior"
# case = "url_parse_test_case__test_urlsplit_remove_unsafe_bytes"
# subject = "cpython.test_urlparse.UrlParseTestCase.test_urlsplit_remove_unsafe_bytes"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_urlparse.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_urlparse.py::UrlParseTestCase::test_urlsplit_remove_unsafe_bytes
"""Auto-ported test: UrlParseTestCase::test_urlsplit_remove_unsafe_bytes (CPython 3.12 oracle)."""


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
url = "http\t://www.python\n.org\t/java\nscript:\talert('msg\r\n')/?query\n=\tsomething#frag\nment"
p = urllib.parse.urlsplit(url)

assert p.scheme == 'http'

assert p.netloc == 'www.python.org'

assert p.path == "/javascript:alert('msg')/"

assert p.query == 'query=something'

assert p.fragment == 'fragment'

assert p.username == None

assert p.password == None

assert p.hostname == 'www.python.org'

assert p.port == None

assert p.geturl() == "http://www.python.org/javascript:alert('msg')/?query=something#fragment"
url = b"http\t://www.python\n.org\t/java\nscript:\talert('msg\r\n')/?query\n=\tsomething#frag\nment"
p = urllib.parse.urlsplit(url)

assert p.scheme == b'http'

assert p.netloc == b'www.python.org'

assert p.path == b"/javascript:alert('msg')/"

assert p.query == b'query=something'

assert p.fragment == b'fragment'

assert p.username == None

assert p.password == None

assert p.hostname == b'www.python.org'

assert p.port == None

assert p.geturl() == b"http://www.python.org/javascript:alert('msg')/?query=something#fragment"
url = "http://www.python.org/java\nscript:\talert('msg\r\n')/?query\n=\tsomething#frag\nment"
scheme = 'ht\ntp'
for _ in range(2):
    p = urllib.parse.urlsplit(url, scheme=scheme)

    assert p.scheme == 'http'

    assert p.geturl() == "http://www.python.org/javascript:alert('msg')/?query=something#fragment"
print("UrlParseTestCase::test_urlsplit_remove_unsafe_bytes: ok")
