# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "behavior"
# case = "compress_object_test_case__test_wbits"
# subject = "cpython.test_zlib.CompressObjectTestCase.test_wbits"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_zlib.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_zlib.py::CompressObjectTestCase::test_wbits
"""Auto-ported test: CompressObjectTestCase::test_wbits (CPython 3.12 oracle)."""


import unittest
from test import support
from test.support import import_helper
import binascii
import copy
import os
import pickle
import random
import sys
from test.support import bigmemtest, _1G, _4G, is_s390x


zlib = import_helper.import_module('zlib')

requires_Compress_copy = unittest.skipUnless(hasattr(zlib.compressobj(), 'copy'), 'requires Compress.copy()')

requires_Decompress_copy = unittest.skipUnless(hasattr(zlib.decompressobj(), 'copy'), 'requires Decompress.copy()')

def _zlib_runtime_version_tuple(zlib_version=zlib.ZLIB_RUNTIME_VERSION):
    v = zlib_version.split('-', 1)[0].split('.')
    if len(v) < 4:
        v.append('0')
    elif not v[-1].isnumeric():
        v[-1] = '0'
    return tuple(map(int, v))

ZLIB_RUNTIME_VERSION_TUPLE = _zlib_runtime_version_tuple()

HW_ACCELERATED = is_s390x

class BaseCompressTestCase(object):

    def check_big_compress_buffer(self, size, compress_func):
        _1M = 1024 * 1024
        data = random.randbytes(_1M * 10)
        data = data * (size // len(data) + 1)
        try:
            compress_func(data)
        finally:
            data = None

    def check_big_decompress_buffer(self, size, decompress_func):
        data = b'x' * size
        try:
            compressed = zlib.compress(data, 1)
        finally:
            data = None
        data = decompress_func(compressed)
        try:
            self.assertEqual(len(data), size)
            self.assertEqual(len(data.strip(b'x')), 0)
        finally:
            data = None

def choose_lines(source, number, seed=None, generator=random):
    """Return a list of number lines randomly chosen from the source"""
    if seed is not None:
        generator.seed(seed)
    sources = source.split('\n')
    return [generator.choice(sources) for n in range(number)]

HAMLET_SCENE = b"\nLAERTES\n\n       O, fear me not.\n       I stay too long: but here my father comes.\n\n       Enter POLONIUS\n\n       A double blessing is a double grace,\n       Occasion smiles upon a second leave.\n\nLORD POLONIUS\n\n       Yet here, Laertes! aboard, aboard, for shame!\n       The wind sits in the shoulder of your sail,\n       And you are stay'd for. There; my blessing with thee!\n       And these few precepts in thy memory\n       See thou character. Give thy thoughts no tongue,\n       Nor any unproportioned thought his act.\n       Be thou familiar, but by no means vulgar.\n       Those friends thou hast, and their adoption tried,\n       Grapple them to thy soul with hoops of steel;\n       But do not dull thy palm with entertainment\n       Of each new-hatch'd, unfledged comrade. Beware\n       Of entrance to a quarrel, but being in,\n       Bear't that the opposed may beware of thee.\n       Give every man thy ear, but few thy voice;\n       Take each man's censure, but reserve thy judgment.\n       Costly thy habit as thy purse can buy,\n       But not express'd in fancy; rich, not gaudy;\n       For the apparel oft proclaims the man,\n       And they in France of the best rank and station\n       Are of a most select and generous chief in that.\n       Neither a borrower nor a lender be;\n       For loan oft loses both itself and friend,\n       And borrowing dulls the edge of husbandry.\n       This above all: to thine ownself be true,\n       And it must follow, as the night the day,\n       Thou canst not then be false to any man.\n       Farewell: my blessing season this in thee!\n\nLAERTES\n\n       Most humbly do I take my leave, my lord.\n\nLORD POLONIUS\n\n       The time invites you; go; your servants tend.\n\nLAERTES\n\n       Farewell, Ophelia; and remember well\n       What I have said to you.\n\nOPHELIA\n\n       'Tis in my memory lock'd,\n       And you yourself shall keep the key of it.\n\nLAERTES\n\n       Farewell.\n"

class CustomInt:

    def __index__(self):
        return 100


# --- test body ---
def check_big_compress_buffer(size, compress_func):
    _1M = 1024 * 1024
    data = random.randbytes(_1M * 10)
    data = data * (size // len(data) + 1)
    try:
        compress_func(data)
    finally:
        data = None

def check_big_decompress_buffer(size, decompress_func):
    data = b'x' * size
    try:
        compressed = zlib.compress(data, 1)
    finally:
        data = None
    data = decompress_func(compressed)
    try:

        assert len(data) == size

        assert len(data.strip(b'x')) == 0
    finally:
        data = None
supports_wbits_0 = ZLIB_RUNTIME_VERSION_TUPLE >= (1, 2, 3, 5)
co = zlib.compressobj(level=1, wbits=15)
zlib15 = co.compress(HAMLET_SCENE) + co.flush()

assert zlib.decompress(zlib15, 15) == HAMLET_SCENE
if supports_wbits_0:

    assert zlib.decompress(zlib15, 0) == HAMLET_SCENE

assert zlib.decompress(zlib15, 32 + 15) == HAMLET_SCENE
try:
    zlib.decompress(zlib15, 14)
    raise AssertionError('expected zlib.error')
except zlib.error as _aR_e:
    import re as _re_aR
    assert _re_aR.search('invalid window size', str(_aR_e))
dco = zlib.decompressobj(wbits=32 + 15)

assert dco.decompress(zlib15) == HAMLET_SCENE
dco = zlib.decompressobj(wbits=14)
try:
    dco.decompress(zlib15)
    raise AssertionError('expected zlib.error')
except zlib.error as _aR_e:
    import re as _re_aR
    assert _re_aR.search('invalid window size', str(_aR_e))
co = zlib.compressobj(level=1, wbits=9)
zlib9 = co.compress(HAMLET_SCENE) + co.flush()

assert zlib.decompress(zlib9, 9) == HAMLET_SCENE

assert zlib.decompress(zlib9, 15) == HAMLET_SCENE
if supports_wbits_0:

    assert zlib.decompress(zlib9, 0) == HAMLET_SCENE

assert zlib.decompress(zlib9, 32 + 9) == HAMLET_SCENE
dco = zlib.decompressobj(wbits=32 + 9)

assert dco.decompress(zlib9) == HAMLET_SCENE
co = zlib.compressobj(level=1, wbits=-15)
deflate15 = co.compress(HAMLET_SCENE) + co.flush()

assert zlib.decompress(deflate15, -15) == HAMLET_SCENE
dco = zlib.decompressobj(wbits=-15)

assert dco.decompress(deflate15) == HAMLET_SCENE
co = zlib.compressobj(level=1, wbits=-9)
deflate9 = co.compress(HAMLET_SCENE) + co.flush()

assert zlib.decompress(deflate9, -9) == HAMLET_SCENE

assert zlib.decompress(deflate9, -15) == HAMLET_SCENE
dco = zlib.decompressobj(wbits=-9)

assert dco.decompress(deflate9) == HAMLET_SCENE
co = zlib.compressobj(level=1, wbits=16 + 15)
gzip = co.compress(HAMLET_SCENE) + co.flush()

assert zlib.decompress(gzip, 16 + 15) == HAMLET_SCENE

assert zlib.decompress(gzip, 32 + 15) == HAMLET_SCENE
dco = zlib.decompressobj(32 + 15)

assert dco.decompress(gzip) == HAMLET_SCENE
for wbits in (-15, 15, 31):
    expected = HAMLET_SCENE
    actual = zlib.decompress(zlib.compress(HAMLET_SCENE, wbits=wbits), wbits=wbits)

    assert expected == actual
print("CompressObjectTestCase::test_wbits: ok")
