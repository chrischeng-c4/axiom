# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "behavior"
# case = "hash_lib_test_case__test_unknown_hash"
# subject = "cpython.test_hashlib.HashLibTestCase.test_unknown_hash"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_hashlib.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_hashlib.py::HashLibTestCase::test_unknown_hash
"""Auto-ported test: HashLibTestCase::test_unknown_hash (CPython 3.12 oracle)."""


import array
from binascii import unhexlify
import hashlib
import importlib
import io
import itertools
import os
import sys
import sysconfig
import threading
import unittest
import warnings
from test import support
from test.support import _4G, bigmemtest
from test.support.import_helper import import_fresh_module
from test.support import os_helper
from test.support import requires_resource
from test.support import threading_helper
from http.client import HTTPException


default_builtin_hashes = {'md5', 'sha1', 'sha256', 'sha512', 'sha3', 'blake2'}

builtin_hashes = sysconfig.get_config_var('PY_BUILTIN_HASHLIB_HASHES')

if builtin_hashes is None:
    builtin_hashes = default_builtin_hashes
else:
    builtin_hashes = {m.strip() for m in builtin_hashes.strip('"').lower().split(',')}

openssl_hashlib = import_fresh_module('hashlib', fresh=['_hashlib'])

if builtin_hashes == default_builtin_hashes:
    builtin_hashlib = import_fresh_module('hashlib', blocked=['_hashlib'])
else:
    builtin_hashlib = None

try:
    from _hashlib import HASH, HASHXOF, openssl_md_meth_names, get_fips_mode
except ImportError:
    HASH = None
    HASHXOF = None
    openssl_md_meth_names = frozenset()

    def get_fips_mode():
        return 0

try:
    import _blake2
except ImportError:
    _blake2 = None

requires_blake2 = unittest.skipUnless(_blake2, 'requires _blake2')

SKIP_SHA3 = support.check_sanitizer(ub=True)

requires_sha3 = unittest.skipUnless(not SKIP_SHA3, 'requires _sha3')

def hexstr(s):
    assert isinstance(s, bytes), repr(s)
    h = '0123456789abcdef'
    r = ''
    for i in s:
        r += h[i >> 4 & 15] + h[i & 15]
    return r

URL = 'http://www.pythontest.net/hashlib/{}.txt'

def read_vectors(hash_name):
    url = URL.format(hash_name)
    try:
        testdata = support.open_urlresource(url, encoding='utf-8')
    except (OSError, HTTPException):
        raise unittest.SkipTest('Could not retrieve {}'.format(url))
    with testdata:
        for line in testdata:
            line = line.strip()
            if line.startswith('#') or not line:
                continue
            parts = line.split(',')
            parts[0] = bytes.fromhex(parts[0])
            yield parts


# --- test body ---
supported_hash_names = ('md5', 'MD5', 'sha1', 'SHA1', 'sha224', 'SHA224', 'sha256', 'SHA256', 'sha384', 'SHA384', 'sha512', 'SHA512', 'blake2b', 'blake2s', 'sha3_224', 'sha3_256', 'sha3_384', 'sha3_512', 'shake_128', 'shake_256')
shakes = {'shake_128', 'shake_256'}

try:
    hashlib.new('spam spam spam spam spam')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    hashlib.new(1)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("HashLibTestCase::test_unknown_hash: ok")
