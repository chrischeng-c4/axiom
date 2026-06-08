# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_parse"
# dimension = "behavior"
# case = "url_parse_test_case__test_urlsplit_attributes"
# subject = "cpython.test_urlparse.UrlParseTestCase.test_urlsplit_attributes"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_urlparse.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_urlparse.py::UrlParseTestCase::test_urlsplit_attributes
"""Auto-ported test: UrlParseTestCase::test_urlsplit_attributes (CPython 3.12 oracle)."""


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
url = 'HTTP://WWW.PYTHON.ORG/doc/#frag'
p = urllib.parse.urlsplit(url)

assert p.scheme == 'http'

assert p.netloc == 'WWW.PYTHON.ORG'

assert p.path == '/doc/'

assert p.query == ''

assert p.fragment == 'frag'

assert p.username == None

assert p.password == None

assert p.hostname == 'www.python.org'

assert p.port == None

assert p.geturl()[4:] == url[4:]
url = 'http://User:Pass@www.python.org:080/doc/?query=yes#frag'
p = urllib.parse.urlsplit(url)

assert p.scheme == 'http'

assert p.netloc == 'User:Pass@www.python.org:080'

assert p.path == '/doc/'

assert p.query == 'query=yes'

assert p.fragment == 'frag'

assert p.username == 'User'

assert p.password == 'Pass'

assert p.hostname == 'www.python.org'

assert p.port == 80

assert p.geturl() == url
url = 'http://User@example.com:Pass@www.python.org:080/doc/?query=yes#frag'
p = urllib.parse.urlsplit(url)

assert p.scheme == 'http'

assert p.netloc == 'User@example.com:Pass@www.python.org:080'

assert p.path == '/doc/'

assert p.query == 'query=yes'

assert p.fragment == 'frag'

assert p.username == 'User@example.com'

assert p.password == 'Pass'

assert p.hostname == 'www.python.org'

assert p.port == 80

assert p.geturl() == url
url = b'HTTP://WWW.PYTHON.ORG/doc/#frag'
p = urllib.parse.urlsplit(url)

assert p.scheme == b'http'

assert p.netloc == b'WWW.PYTHON.ORG'

assert p.path == b'/doc/'

assert p.query == b''

assert p.fragment == b'frag'

assert p.username == None

assert p.password == None

assert p.hostname == b'www.python.org'

assert p.port == None

assert p.geturl()[4:] == url[4:]
url = b'http://User:Pass@www.python.org:080/doc/?query=yes#frag'
p = urllib.parse.urlsplit(url)

assert p.scheme == b'http'

assert p.netloc == b'User:Pass@www.python.org:080'

assert p.path == b'/doc/'

assert p.query == b'query=yes'

assert p.fragment == b'frag'

assert p.username == b'User'

assert p.password == b'Pass'

assert p.hostname == b'www.python.org'

assert p.port == 80

assert p.geturl() == url
url = b'http://User@example.com:Pass@www.python.org:080/doc/?query=yes#frag'
p = urllib.parse.urlsplit(url)

assert p.scheme == b'http'

assert p.netloc == b'User@example.com:Pass@www.python.org:080'

assert p.path == b'/doc/'

assert p.query == b'query=yes'

assert p.fragment == b'frag'

assert p.username == b'User@example.com'

assert p.password == b'Pass'

assert p.hostname == b'www.python.org'

assert p.port == 80

assert p.geturl() == url
url = b'HTTP://WWW.PYTHON.ORG:65536/doc/#frag'
p = urllib.parse.urlsplit(url)
try:
    p.port
    raise AssertionError('expected ValueError')
except ValueError as _aR_e:
    import re as _re_aR
    assert _re_aR.search('out of range', str(_aR_e))
print("UrlParseTestCase::test_urlsplit_attributes: ok")
