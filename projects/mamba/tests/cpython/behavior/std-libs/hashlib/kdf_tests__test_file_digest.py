# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "behavior"
# case = "kdf_tests__test_file_digest"
# subject = "cpython.test_hashlib.KDFTests.test_file_digest"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_hashlib.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_hashlib.py::KDFTests::test_file_digest
"""Auto-ported test: KDFTests::test_file_digest (CPython 3.12 oracle)."""


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
pbkdf2_test_vectors = [(b'password', b'salt', 1, None), (b'password', b'salt', 2, None), (b'password', b'salt', 4096, None), (b'passwordPASSWORDpassword', b'saltSALTsaltSALTsaltSALTsaltSALTsalt', 4096, -1), (b'pass\x00word', b'sa\x00lt', 4096, 16)]
scrypt_test_vectors = [(b'', b'', 16, 1, 1, unhexlify('77d6576238657b203b19ca42c18a0497f16b4844e3074ae8dfdffa3fede21442fcd0069ded0948f8326a753a0fc81f17e8d3e0fb2e0d3628cf35e20c38d18906')), (b'password', b'NaCl', 1024, 8, 16, unhexlify('fdbabe1c9d3472007856e7190d01e9fe7c6ad7cbc8237830e77376634b3731622eaf30d92e22a3886ff109279d9830dac727afb94a83ee6d8360cbdfa2cc0640')), (b'pleaseletmein', b'SodiumChloride', 16384, 8, 1, unhexlify('7023bdcb3afd7348461c06cd81fd38ebfda8fbba904f8e3ea9b543f6545da1f2d5432955613f0fcf62d49705242a9af9e61e85dc0d651e40dfcf017b45575887'))]
pbkdf2_results = {'sha1': [(bytes.fromhex('0c60c80f961f0e71f3a9b524af6012062fe037a6'), None), (bytes.fromhex('ea6c014dc72d6f8ccd1ed92ace1d41f0d8de8957'), None), (bytes.fromhex('4b007901b765489abead49d926f721d065a429c1'), None), (bytes.fromhex('3d2eec4fe41c849b80c8d83662c0e44a8b291a964cf2f07038'), 25), (bytes.fromhex('56fa6aa75548099dcc37d7f03425e0c3'), None)], 'sha256': [(bytes.fromhex('120fb6cffcf8b32c43e7225256c4f837a86548c92ccc35480805987cb70be17b'), None), (bytes.fromhex('ae4d0c95af6b46d32d0adff928f06dd02a303f8ef3c251dfd6e2d85a95474c43'), None), (bytes.fromhex('c5e478d59288c841aa530db6845c4c8d962893a001ce4e11a4963873aa98134a'), None), (bytes.fromhex('348c89dbcbd32b2f32d814b8116e84cf2b17347ebc1800181c4e2a1fb8dd53e1c635518c7dac47e9'), 40), (bytes.fromhex('89b69d0516f829893c696226650a8687'), None)], 'sha512': [(bytes.fromhex('867f70cf1ade02cff3752599a3a53dc4af34c7a669815ae5d513554e1c8cf252c02d470a285a0501bad999bfe943c08f050235d7d68b1da55e63f73b60a57fce'), None), (bytes.fromhex('e1d9c16aa681708a45f5c7c4e215ceb66e011a2e9f0040713f18aefdb866d53cf76cab2868a39b9f7840edce4fef5a82be67335c77a6068e04112754f27ccf4e'), None), (bytes.fromhex('d197b1b33db0143e018b12f3d1d1479e6cdebdcc97c5c0f87f6902e072f457b5143f30602641b3d55cd335988cb36b84376060ecd532e039b742a239434af2d5'), None), (bytes.fromhex('8c0511f4c6e597c6ac6315d8f0362e225f3c501495ba23b868c005174dc4ee71115b59f9e60cd9532fa33e0f75aefe30225c583a186cd82bd4daea9724a3d3b8'), 64), (bytes.fromhex('9d9e9c4cd21fe4be24d5b8244c759665'), None)]}

def _test_pbkdf2_hmac(pbkdf2, supported):
    for digest_name, results in pbkdf2_results.items():
        if digest_name not in supported:
            continue
        for i, vector in enumerate(pbkdf2_test_vectors):
            password, salt, rounds, dklen = vector
            expected, overwrite_dklen = results[i]
            if overwrite_dklen:
                dklen = overwrite_dklen
            out = pbkdf2(digest_name, password, salt, rounds, dklen)

            assert out == expected
            out = pbkdf2(digest_name, memoryview(password), memoryview(salt), rounds, dklen)

            assert out == expected
            out = pbkdf2(digest_name, bytearray(password), bytearray(salt), rounds, dklen)

            assert out == expected
            if dklen is None:
                out = pbkdf2(digest_name, password, salt, rounds)

                assert out == expected
    try:
        pbkdf2('unknown', b'pass', b'salt', 1)
        raise AssertionError('expected ValueError')
    except ValueError as _aR_e:
        import re as _re_aR
        assert _re_aR.search('.*unsupported.*', str(_aR_e))
    if 'sha1' in supported:

        try:
            pbkdf2(b'sha1', b'pass', b'salt', 1)
            raise AssertionError('expected TypeError')
        except TypeError:
            pass

        try:
            pbkdf2('sha1', 'pass', 'salt', 1)
            raise AssertionError('expected TypeError')
        except TypeError:
            pass

        try:
            pbkdf2('sha1', b'pass', b'salt', 0)
            raise AssertionError('expected ValueError')
        except ValueError:
            pass

        try:
            pbkdf2('sha1', b'pass', b'salt', -1)
            raise AssertionError('expected ValueError')
        except ValueError:
            pass

        try:
            pbkdf2('sha1', b'pass', b'salt', 1, 0)
            raise AssertionError('expected ValueError')
        except ValueError:
            pass

        try:
            pbkdf2('sha1', b'pass', b'salt', 1, -1)
            raise AssertionError('expected ValueError')
        except ValueError:
            pass
        out = pbkdf2(hash_name='sha1', password=b'password', salt=b'salt', iterations=1, dklen=None)

        assert out == pbkdf2_results['sha1'][0][0]
data = b'a' * 65536
d1 = hashlib.sha256()
pass
with open(os_helper.TESTFN, 'wb') as f:
    for _ in range(10):
        d1.update(data)
        f.write(data)
with open(os_helper.TESTFN, 'rb') as f:
    d2 = hashlib.file_digest(f, hashlib.sha256)

assert d1.hexdigest() == d2.hexdigest()

assert d1.name == d2.name

assert type(d1) is type(d2)
try:
    hashlib.file_digest(None, 'sha256')
    raise AssertionError('expected ValueError')
except ValueError:
    pass
try:
    with open(os_helper.TESTFN, 'r') as f:
        hashlib.file_digest(f, 'sha256')
    raise AssertionError('expected ValueError')
except ValueError:
    pass
try:
    with open(os_helper.TESTFN, 'wb') as f:
        hashlib.file_digest(f, 'sha256')
    raise AssertionError('expected ValueError')
except ValueError:
    pass
print("KDFTests::test_file_digest: ok")
