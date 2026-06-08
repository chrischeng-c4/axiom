# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "behavior"
# case = "compress_object_test_case__test_decompress_incomplete_stream"
# subject = "cpython.test_zlib.CompressObjectTestCase.test_decompress_incomplete_stream"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_zlib.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_zlib.py::CompressObjectTestCase::test_decompress_incomplete_stream
"""Auto-ported test: CompressObjectTestCase::test_decompress_incomplete_stream (CPython 3.12 oracle)."""


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
x = b'x\x9cK\xcb\xcf\x07\x00\x02\x82\x01E'

assert zlib.decompress(x) == b'foo'

try:
    zlib.decompress(x[:-5])
    raise AssertionError('expected zlib.error')
except zlib.error:
    pass
dco = zlib.decompressobj()
y = dco.decompress(x[:-5])
y += dco.flush()

assert y == b'foo'
print("CompressObjectTestCase::test_decompress_incomplete_stream: ok")
