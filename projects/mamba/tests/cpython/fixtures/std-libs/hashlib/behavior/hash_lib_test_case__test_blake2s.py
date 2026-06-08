# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "behavior"
# case = "hash_lib_test_case__test_blake2s"
# subject = "cpython.test_hashlib.HashLibTestCase.test_blake2s"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_hashlib.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_hashlib.py::HashLibTestCase::test_blake2s
"""Auto-ported test: HashLibTestCase::test_blake2s (CPython 3.12 oracle)."""


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

def _conditional_import_module(module_name):
    """Import a module and return a reference to it or None on failure."""
    try:
        return importlib.import_module(module_name)
    except ModuleNotFoundError as error:
        if self__warn_on_extension_import and module_name in builtin_hashes:
            warnings.warn(f'Did a C extension fail to compile? {error}')
    return None

def blake2_rfc7693(constructor, md_len, in_len):

    def selftest_seq(length, seed):
        mask = (1 << 32) - 1
        a = 3735899053 * seed & mask
        b = 1
        out = bytearray(length)
        for i in range(length):
            t = a + b & mask
            a, b = (b, t)
            out[i] = t >> 24 & 255
        return out
    outer = constructor(digest_size=32)
    for outlen in md_len:
        for inlen in in_len:
            indata = selftest_seq(inlen, inlen)
            key = selftest_seq(outlen, outlen)
            unkeyed = constructor(indata, digest_size=outlen)
            outer.update(unkeyed.digest())
            keyed = constructor(indata, key=key, digest_size=outlen)
            outer.update(keyed.digest())
    return outer.hexdigest()

def check(name, data, hexdigest, shake=False, **kwargs):
    length = len(hexdigest) // 2
    hexdigest = hexdigest.lower()
    constructors = self_constructors_to_test[name]

    assert len(constructors) >= 2
    for hash_object_constructor in constructors:
        m = hash_object_constructor(data, **kwargs)
        computed = m.hexdigest() if not shake else m.hexdigest(length)

        assert computed == hexdigest
        computed = m.digest() if not shake else m.digest(length)
        digest = bytes.fromhex(hexdigest)

        assert computed == digest
        if not shake:

            assert len(digest) == m.digest_size
    if not shake and kwargs.get('key') is None:
        check_file_digest(name, data, hexdigest)

def check_blake2(constructor, salt_size, person_size, key_size, digest_size, max_offset):

    assert constructor.SALT_SIZE == salt_size
    for i in range(salt_size + 1):
        constructor(salt=b'a' * i)
    salt = b'a' * (salt_size + 1)

    try:
        constructor(salt=salt)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass

    assert constructor.PERSON_SIZE == person_size
    for i in range(person_size + 1):
        constructor(person=b'a' * i)
    person = b'a' * (person_size + 1)

    try:
        constructor(person=person)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass

    assert constructor.MAX_DIGEST_SIZE == digest_size
    for i in range(1, digest_size + 1):
        constructor(digest_size=i)

    try:
        constructor(digest_size=-1)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass

    try:
        constructor(digest_size=0)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass

    try:
        constructor(digest_size=digest_size + 1)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass

    assert constructor.MAX_KEY_SIZE == key_size
    for i in range(key_size + 1):
        constructor(key=b'a' * i)
    key = b'a' * (key_size + 1)

    try:
        constructor(key=key)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass

    assert constructor().hexdigest() == constructor(key=b'').hexdigest()
    for i in range(0, 256):
        constructor(fanout=i)

    try:
        constructor(fanout=-1)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass

    try:
        constructor(fanout=256)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass
    for i in range(1, 256):
        constructor(depth=i)

    try:
        constructor(depth=-1)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass

    try:
        constructor(depth=0)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass

    try:
        constructor(depth=256)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass
    for i in range(0, 256):
        constructor(node_depth=i)

    try:
        constructor(node_depth=-1)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass

    try:
        constructor(node_depth=256)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass
    for i in range(0, digest_size + 1):
        constructor(inner_size=i)

    try:
        constructor(inner_size=-1)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass

    try:
        constructor(inner_size=digest_size + 1)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass
    constructor(leaf_size=0)
    constructor(leaf_size=(1 << 32) - 1)

    try:
        constructor(leaf_size=-1)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass

    try:
        constructor(leaf_size=1 << 32)
        raise AssertionError('expected OverflowError')
    except OverflowError:
        pass
    constructor(node_offset=0)
    constructor(node_offset=max_offset)

    try:
        constructor(node_offset=-1)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass

    try:
        constructor(node_offset=max_offset + 1)
        raise AssertionError('expected OverflowError')
    except OverflowError:
        pass

    try:
        constructor(data=b'')
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

    try:
        constructor(string=b'')
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

    try:
        constructor('')
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
    constructor(b'', key=b'', salt=b'', person=b'', digest_size=17, fanout=1, depth=1, leaf_size=256, node_offset=512, node_depth=1, inner_size=7, last_node=True)

def check_blocksize_name(name, block_size=0, digest_size=0, digest_length=None):
    constructors = self_constructors_to_test[name]
    for hash_object_constructor in constructors:
        m = hash_object_constructor(usedforsecurity=False)

        assert m.block_size == block_size

        assert m.digest_size == digest_size
        if digest_length:

            assert len(m.digest(digest_length)) == digest_length

            assert len(m.hexdigest(digest_length)) == 2 * digest_length
        else:

            assert len(m.digest()) == digest_size

            assert len(m.hexdigest()) == 2 * digest_size

        assert m.name == name

        assert name.split('_')[0] in repr(m).lower()

def check_file_digest(name, data, hexdigest):
    hexdigest = hexdigest.lower()
    try:
        hashlib.new(name)
    except ValueError:
        return
    digests = [name]
    digests.extend(self_constructors_to_test[name])
    with open(os_helper.TESTFN, 'wb') as f:
        f.write(data)
    try:
        for digest in digests:
            buf = io.BytesIO(data)
            buf.seek(0)

            assert hashlib.file_digest(buf, digest).hexdigest() == hexdigest
            with open(os_helper.TESTFN, 'rb') as f:
                digestobj = hashlib.file_digest(f, digest)

            assert digestobj.hexdigest() == hexdigest
    finally:
        os.unlink(os_helper.TESTFN)

def check_no_unicode(algorithm_name):
    constructors = self_constructors_to_test[algorithm_name]
    for hash_object_constructor in constructors:

        try:
            hash_object_constructor('spam')
            raise AssertionError('expected TypeError')
        except TypeError:
            pass

def check_sha3(name, capacity, rate, suffix):
    constructors = self_constructors_to_test[name]
    for hash_object_constructor in constructors:
        m = hash_object_constructor()
        if HASH is not None and isinstance(m, HASH):
            continue

        assert capacity + rate == 1600

        assert m._capacity_bits == capacity

        assert m._rate_bits == rate

        assert m._suffix == suffix

def hash_constructors():
    constructors = self_constructors_to_test.values()
    return itertools.chain.from_iterable(constructors)

def is_fips_mode():
    return get_fips_mode()
check_blake2(hashlib.blake2s, 8, 8, 32, 32, (1 << 48) - 1)
b2s_md_len = [16, 20, 28, 32]
b2s_in_len = [0, 3, 64, 65, 255, 1024]

assert blake2_rfc7693(hashlib.blake2s, b2s_md_len, b2s_in_len) == '6a411f08ce25adcdfb02aba641451cec53c598b24f4fc787fbdc88797f4c1dfe'
print("HashLibTestCase::test_blake2s: ok")
