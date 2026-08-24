use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/zlib/adler32_exact_values_and_seed_chaining.py`.
#[test]
fn test_gen_behavior_std_libs_zlib_adler32_exact_values_and_seed_chaining() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "behavior"
# case = "adler32_exact_values_and_seed_chaining"
# subject = "zlib.adler32"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zlib.adler32: adler32 returns the mod-65521 unsigned 32-bit value, seeds with 1, passes a seed through for empty input, matches documented values for explicit seeds, chains incrementally to equal the one-shot over the concatenation, and stays unsigned for a large seed"""
import zlib

# Exact values (unsigned) and known small inputs.
assert zlib.adler32(b"abcdefghijklmnop" * 2) == 3573550353, "adler32 abc..p x2"
assert zlib.adler32(b"spam") == 72286642, "adler32 spam"
assert zlib.adler32(b"hello") == 103547413, "adler32 hello"
assert zlib.adler32(b"\x01") == 131074, "adler32 soh"

# Start value is the identity/seed: adler32 seeds with 1.
assert zlib.adler32(b"") == 1, "adler32 empty default = 1"
assert zlib.adler32(b"") == zlib.adler32(b"", 1), "adler32 default seed is 1"
# Empty input returns the supplied seed unchanged.
assert zlib.adler32(b"", 432) == 432, "adler32 empty passes seed through"

# Explicit seed produces documented values.
assert zlib.adler32(b"penguin", 0) == 198116086, "adler32 penguin seed 0"
assert zlib.adler32(b"penguin", 1) == 198574839, "adler32 penguin seed 1"

# Seed chaining equals one-shot over the concatenation.
_part = zlib.adler32(b"hel")
assert zlib.adler32(b"lo", _part) == zlib.adler32(b"hello"), "adler32 incremental"

# Large seed (0xFFFFFFFF) is accepted and result stays unsigned 32-bit.
assert 0 <= zlib.adler32(b"abc", 4294967295) <= 0xFFFFFFFF, "adler32 big seed unsigned"

print("adler32_exact_values_and_seed_chaining OK")
"###);
    assert_output(&out, r###"adler32_exact_values_and_seed_chaining OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/zlib/checksum_test_case__test_adler32empty.py`.
#[test]
fn test_gen_behavior_std_libs_zlib_checksum_test_case__test_adler32empty() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "behavior"
# case = "checksum_test_case__test_adler32empty"
# subject = "cpython.test_zlib.ChecksumTestCase.test_adler32empty"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_zlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_zlib.py::ChecksumTestCase::test_adler32empty
"""Auto-ported test: ChecksumTestCase::test_adler32empty (CPython 3.12 oracle)."""


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

assert zlib.adler32(b'', 0) == 0

assert zlib.adler32(b'', 1) == 1

assert zlib.adler32(b'', 432) == 432
print("ChecksumTestCase::test_adler32empty: ok")
"###);
    assert_output(&out, r###"ChecksumTestCase::test_adler32empty: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/zlib/checksum_test_case__test_adler32start.py`.
#[test]
fn test_gen_behavior_std_libs_zlib_checksum_test_case__test_adler32start() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "behavior"
# case = "checksum_test_case__test_adler32start"
# subject = "cpython.test_zlib.ChecksumTestCase.test_adler32start"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_zlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_zlib.py::ChecksumTestCase::test_adler32start
"""Auto-ported test: ChecksumTestCase::test_adler32start (CPython 3.12 oracle)."""


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

assert zlib.adler32(b'') == zlib.adler32(b'', 1)

assert zlib.adler32(b'abc', 4294967295)
print("ChecksumTestCase::test_adler32start: ok")
"###);
    assert_output(&out, r###"ChecksumTestCase::test_adler32start: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/zlib/checksum_test_case__test_crc32_adler32_unsigned.py`.
#[test]
fn test_gen_behavior_std_libs_zlib_checksum_test_case__test_crc32_adler32_unsigned() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "behavior"
# case = "checksum_test_case__test_crc32_adler32_unsigned"
# subject = "cpython.test_zlib.ChecksumTestCase.test_crc32_adler32_unsigned"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_zlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_zlib.py::ChecksumTestCase::test_crc32_adler32_unsigned
"""Auto-ported test: ChecksumTestCase::test_crc32_adler32_unsigned (CPython 3.12 oracle)."""


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
foo = b'abcdefghijklmnop'

assert zlib.crc32(foo) == 2486878355

assert zlib.crc32(b'spam') == 1138425661

assert zlib.adler32(foo + foo) == 3573550353

assert zlib.adler32(b'spam') == 72286642
print("ChecksumTestCase::test_crc32_adler32_unsigned: ok")
"###);
    assert_output(&out, r###"ChecksumTestCase::test_crc32_adler32_unsigned: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/zlib/checksum_test_case__test_crc32empty.py`.
#[test]
fn test_gen_behavior_std_libs_zlib_checksum_test_case__test_crc32empty() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "behavior"
# case = "checksum_test_case__test_crc32empty"
# subject = "cpython.test_zlib.ChecksumTestCase.test_crc32empty"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_zlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_zlib.py::ChecksumTestCase::test_crc32empty
"""Auto-ported test: ChecksumTestCase::test_crc32empty (CPython 3.12 oracle)."""


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

assert zlib.crc32(b'', 0) == 0

assert zlib.crc32(b'', 1) == 1

assert zlib.crc32(b'', 432) == 432
print("ChecksumTestCase::test_crc32empty: ok")
"###);
    assert_output(&out, r###"ChecksumTestCase::test_crc32empty: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/zlib/checksum_test_case__test_crc32start.py`.
#[test]
fn test_gen_behavior_std_libs_zlib_checksum_test_case__test_crc32start() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "behavior"
# case = "checksum_test_case__test_crc32start"
# subject = "cpython.test_zlib.ChecksumTestCase.test_crc32start"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_zlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_zlib.py::ChecksumTestCase::test_crc32start
"""Auto-ported test: ChecksumTestCase::test_crc32start (CPython 3.12 oracle)."""


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

assert zlib.crc32(b'') == zlib.crc32(b'', 0)

assert zlib.crc32(b'abc', 4294967295)
print("ChecksumTestCase::test_crc32start: ok")
"###);
    assert_output(&out, r###"ChecksumTestCase::test_crc32start: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/zlib/checksum_test_case__test_penguins.py`.
#[test]
fn test_gen_behavior_std_libs_zlib_checksum_test_case__test_penguins() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "behavior"
# case = "checksum_test_case__test_penguins"
# subject = "cpython.test_zlib.ChecksumTestCase.test_penguins"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_zlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_zlib.py::ChecksumTestCase::test_penguins
"""Auto-ported test: ChecksumTestCase::test_penguins (CPython 3.12 oracle)."""


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

assert zlib.crc32(b'penguin', 0) == 3854672160

assert zlib.crc32(b'penguin', 1) == 1136044692

assert zlib.adler32(b'penguin', 0) == 198116086

assert zlib.adler32(b'penguin', 1) == 198574839

assert zlib.crc32(b'penguin') == zlib.crc32(b'penguin', 0)

assert zlib.adler32(b'penguin') == zlib.adler32(b'penguin', 1)
print("ChecksumTestCase::test_penguins: ok")
"###);
    assert_output(&out, r###"ChecksumTestCase::test_penguins: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/zlib/checksum_test_case__test_same_as_binascii_crc32.py`.
#[test]
fn test_gen_behavior_std_libs_zlib_checksum_test_case__test_same_as_binascii_crc32() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "behavior"
# case = "checksum_test_case__test_same_as_binascii_crc32"
# subject = "cpython.test_zlib.ChecksumTestCase.test_same_as_binascii_crc32"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_zlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_zlib.py::ChecksumTestCase::test_same_as_binascii_crc32
"""Auto-ported test: ChecksumTestCase::test_same_as_binascii_crc32 (CPython 3.12 oracle)."""


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
foo = b'abcdefghijklmnop'
crc = 2486878355

assert binascii.crc32(foo) == crc

assert zlib.crc32(foo) == crc

assert binascii.crc32(b'spam') == zlib.crc32(b'spam')
print("ChecksumTestCase::test_same_as_binascii_crc32: ok")
"###);
    assert_output(&out, r###"ChecksumTestCase::test_same_as_binascii_crc32: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/zlib/compress_decompress_roundtrips_varied_payloads.py`.
#[test]
fn test_gen_behavior_std_libs_zlib_compress_decompress_roundtrips_varied_payloads() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "behavior"
# case = "compress_decompress_roundtrips_varied_payloads"
# subject = "zlib.decompress"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zlib.decompress: compress then decompress round-trips a range of payloads byte-for-byte: empty, single byte, the full 0..255 byte range, a 10000-byte repeat, and repeated text"""
import zlib

_payloads = [
    b"",
    b"x",
    bytes(range(256)),
    b"a" * 10000,
    b"hello world " * 100,
]
for _p in _payloads:
    _rt = zlib.decompress(zlib.compress(_p))
    assert _rt == _p, f"round-trip len={len(_p)}"

print("compress_decompress_roundtrips_varied_payloads OK")
"###);
    assert_output(&out, r###"compress_decompress_roundtrips_varied_payloads OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/zlib/compress_is_deterministic_per_level.py`.
#[test]
fn test_gen_behavior_std_libs_zlib_compress_is_deterministic_per_level() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "behavior"
# case = "compress_is_deterministic_per_level"
# subject = "zlib.compress"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zlib.compress: two compress calls on the same data at the same level produce byte-identical streams"""
import zlib

assert zlib.compress(b"test", level=6) == zlib.compress(b"test", level=6), "deterministic"
_data = b"deterministic payload " * 64
assert zlib.compress(_data, level=9) == zlib.compress(_data, level=9), "deterministic level 9"

print("compress_is_deterministic_per_level OK")
"###);
    assert_output(&out, r###"compress_is_deterministic_per_level OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/zlib/compress_object_test_case__test_badcompresscopy.py`.
#[test]
fn test_gen_behavior_std_libs_zlib_compress_object_test_case__test_badcompresscopy() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "behavior"
# case = "compress_object_test_case__test_badcompresscopy"
# subject = "cpython.test_zlib.CompressObjectTestCase.test_badcompresscopy"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_zlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_zlib.py::CompressObjectTestCase::test_badcompresscopy
"""Auto-ported test: CompressObjectTestCase::test_badcompresscopy (CPython 3.12 oracle)."""


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
c = zlib.compressobj()
c.compress(HAMLET_SCENE)
c.flush()

try:
    c.copy()
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    copy.copy(c)
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    copy.deepcopy(c)
    raise AssertionError('expected ValueError')
except ValueError:
    pass
print("CompressObjectTestCase::test_badcompresscopy: ok")
"###);
    assert_output(&out, r###"CompressObjectTestCase::test_badcompresscopy: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/zlib/compress_object_test_case__test_baddecompresscopy.py`.
#[test]
fn test_gen_behavior_std_libs_zlib_compress_object_test_case__test_baddecompresscopy() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "behavior"
# case = "compress_object_test_case__test_baddecompresscopy"
# subject = "cpython.test_zlib.CompressObjectTestCase.test_baddecompresscopy"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_zlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_zlib.py::CompressObjectTestCase::test_baddecompresscopy
"""Auto-ported test: CompressObjectTestCase::test_baddecompresscopy (CPython 3.12 oracle)."""


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
data = zlib.compress(HAMLET_SCENE)
d = zlib.decompressobj()
d.decompress(data)
d.flush()

try:
    d.copy()
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    copy.copy(d)
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    copy.deepcopy(d)
    raise AssertionError('expected ValueError')
except ValueError:
    pass
print("CompressObjectTestCase::test_baddecompresscopy: ok")
"###);
    assert_output(&out, r###"CompressObjectTestCase::test_baddecompresscopy: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/zlib/compress_object_test_case__test_clear_unconsumed_tail.py`.
#[test]
fn test_gen_behavior_std_libs_zlib_compress_object_test_case__test_clear_unconsumed_tail() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "behavior"
# case = "compress_object_test_case__test_clear_unconsumed_tail"
# subject = "cpython.test_zlib.CompressObjectTestCase.test_clear_unconsumed_tail"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_zlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_zlib.py::CompressObjectTestCase::test_clear_unconsumed_tail
"""Auto-ported test: CompressObjectTestCase::test_clear_unconsumed_tail (CPython 3.12 oracle)."""


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
cdata = b'x\x9cKLJ\x06\x00\x02M\x01'
dco = zlib.decompressobj()
ddata = dco.decompress(cdata, 1)
ddata += dco.decompress(dco.unconsumed_tail)

assert dco.unconsumed_tail == b''
print("CompressObjectTestCase::test_clear_unconsumed_tail: ok")
"###);
    assert_output(&out, r###"CompressObjectTestCase::test_clear_unconsumed_tail: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/zlib/compress_object_test_case__test_compresscopy.py`.
#[test]
fn test_gen_behavior_std_libs_zlib_compress_object_test_case__test_compresscopy() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "behavior"
# case = "compress_object_test_case__test_compresscopy"
# subject = "cpython.test_zlib.CompressObjectTestCase.test_compresscopy"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_zlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_zlib.py::CompressObjectTestCase::test_compresscopy
"""Auto-ported test: CompressObjectTestCase::test_compresscopy (CPython 3.12 oracle)."""


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
data0 = HAMLET_SCENE
data1 = bytes(str(HAMLET_SCENE, 'ascii').swapcase(), 'ascii')
for func in (lambda c: c.copy(), copy.copy, copy.deepcopy):
    c0 = zlib.compressobj(zlib.Z_BEST_COMPRESSION)
    bufs0 = []
    bufs0.append(c0.compress(data0))
    c1 = func(c0)
    bufs1 = bufs0[:]
    bufs0.append(c0.compress(data0))
    bufs0.append(c0.flush())
    s0 = b''.join(bufs0)
    bufs1.append(c1.compress(data1))
    bufs1.append(c1.flush())
    s1 = b''.join(bufs1)

    assert zlib.decompress(s0) == data0 + data0

    assert zlib.decompress(s1) == data0 + data1
print("CompressObjectTestCase::test_compresscopy: ok")
"###);
    assert_output(&out, r###"CompressObjectTestCase::test_compresscopy: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/zlib/compress_object_test_case__test_compressincremental.py`.
#[test]
fn test_gen_behavior_std_libs_zlib_compress_object_test_case__test_compressincremental() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "behavior"
# case = "compress_object_test_case__test_compressincremental"
# subject = "cpython.test_zlib.CompressObjectTestCase.test_compressincremental"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_zlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_zlib.py::CompressObjectTestCase::test_compressincremental
"""Auto-ported test: CompressObjectTestCase::test_compressincremental (CPython 3.12 oracle)."""


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
data = HAMLET_SCENE * 128
co = zlib.compressobj()
bufs = []
for i in range(0, len(data), 256):
    bufs.append(co.compress(data[i:i + 256]))
bufs.append(co.flush())
combuf = b''.join(bufs)
dco = zlib.decompressobj()
y1 = dco.decompress(b''.join(bufs))
y2 = dco.flush()

assert data == y1 + y2
print("CompressObjectTestCase::test_compressincremental: ok")
"###);
    assert_output(&out, r###"CompressObjectTestCase::test_compressincremental: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/zlib/compress_object_test_case__test_compressoptions.py`.
#[test]
fn test_gen_behavior_std_libs_zlib_compress_object_test_case__test_compressoptions() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "behavior"
# case = "compress_object_test_case__test_compressoptions"
# subject = "cpython.test_zlib.CompressObjectTestCase.test_compressoptions"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_zlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_zlib.py::CompressObjectTestCase::test_compressoptions
"""Auto-ported test: CompressObjectTestCase::test_compressoptions (CPython 3.12 oracle)."""


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
level = 2
method = zlib.DEFLATED
wbits = -12
memLevel = 9
strategy = zlib.Z_FILTERED
co = zlib.compressobj(level, method, wbits, memLevel, strategy)
x1 = co.compress(HAMLET_SCENE)
x2 = co.flush()
dco = zlib.decompressobj(wbits)
y1 = dco.decompress(x1 + x2)
y2 = dco.flush()

assert HAMLET_SCENE == y1 + y2
print("CompressObjectTestCase::test_compressoptions: ok")
"###);
    assert_output(&out, r###"CompressObjectTestCase::test_compressoptions: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/zlib/compress_object_test_case__test_compresspickle.py`.
#[test]
fn test_gen_behavior_std_libs_zlib_compress_object_test_case__test_compresspickle() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "behavior"
# case = "compress_object_test_case__test_compresspickle"
# subject = "cpython.test_zlib.CompressObjectTestCase.test_compresspickle"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_zlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_zlib.py::CompressObjectTestCase::test_compresspickle
"""Auto-ported test: CompressObjectTestCase::test_compresspickle (CPython 3.12 oracle)."""


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
for proto in range(pickle.HIGHEST_PROTOCOL + 1):
    try:
        pickle.dumps(zlib.compressobj(zlib.Z_BEST_COMPRESSION), proto)
        raise AssertionError('expected (TypeError, pickle.PicklingError)')
    except (TypeError, pickle.PicklingError):
        pass
print("CompressObjectTestCase::test_compresspickle: ok")
"###);
    assert_output(&out, r###"CompressObjectTestCase::test_compresspickle: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/zlib/compress_object_test_case__test_decompress_eof.py`.
#[test]
fn test_gen_behavior_std_libs_zlib_compress_object_test_case__test_decompress_eof() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "behavior"
# case = "compress_object_test_case__test_decompress_eof"
# subject = "cpython.test_zlib.CompressObjectTestCase.test_decompress_eof"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_zlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_zlib.py::CompressObjectTestCase::test_decompress_eof
"""Auto-ported test: CompressObjectTestCase::test_decompress_eof (CPython 3.12 oracle)."""


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
dco = zlib.decompressobj()

assert not dco.eof
dco.decompress(x[:-5])

assert not dco.eof
dco.decompress(x[-5:])

assert dco.eof
dco.flush()

assert dco.eof
print("CompressObjectTestCase::test_decompress_eof: ok")
"###);
    assert_output(&out, r###"CompressObjectTestCase::test_decompress_eof: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/zlib/compress_object_test_case__test_decompress_eof_incomplete_stream.py`.
#[test]
fn test_gen_behavior_std_libs_zlib_compress_object_test_case__test_decompress_eof_incomplete_stream() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "behavior"
# case = "compress_object_test_case__test_decompress_eof_incomplete_stream"
# subject = "cpython.test_zlib.CompressObjectTestCase.test_decompress_eof_incomplete_stream"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_zlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_zlib.py::CompressObjectTestCase::test_decompress_eof_incomplete_stream
"""Auto-ported test: CompressObjectTestCase::test_decompress_eof_incomplete_stream (CPython 3.12 oracle)."""


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
dco = zlib.decompressobj()

assert not dco.eof
dco.decompress(x[:-5])

assert not dco.eof
dco.flush()

assert not dco.eof
print("CompressObjectTestCase::test_decompress_eof_incomplete_stream: ok")
"###);
    assert_output(&out, r###"CompressObjectTestCase::test_decompress_eof_incomplete_stream: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/zlib/compress_object_test_case__test_decompress_incomplete_stream.py`.
#[test]
fn test_gen_behavior_std_libs_zlib_compress_object_test_case__test_decompress_incomplete_stream() {
    let out = jit_capture(r###"# /// script
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
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_zlib.py"
# status = "filled"
# ///
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
"###);
    assert_output(&out, r###"CompressObjectTestCase::test_decompress_incomplete_stream: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/zlib/compress_object_test_case__test_decompress_raw_with_dictionary.py`.
#[test]
fn test_gen_behavior_std_libs_zlib_compress_object_test_case__test_decompress_raw_with_dictionary() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "behavior"
# case = "compress_object_test_case__test_decompress_raw_with_dictionary"
# subject = "cpython.test_zlib.CompressObjectTestCase.test_decompress_raw_with_dictionary"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_zlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_zlib.py::CompressObjectTestCase::test_decompress_raw_with_dictionary
"""Auto-ported test: CompressObjectTestCase::test_decompress_raw_with_dictionary (CPython 3.12 oracle)."""


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
zdict = b'abcdefghijklmnopqrstuvwxyz'
co = zlib.compressobj(wbits=-zlib.MAX_WBITS, zdict=zdict)
comp = co.compress(zdict) + co.flush()
dco = zlib.decompressobj(wbits=-zlib.MAX_WBITS, zdict=zdict)
uncomp = dco.decompress(comp) + dco.flush()

assert zdict == uncomp
print("CompressObjectTestCase::test_decompress_raw_with_dictionary: ok")
"###);
    assert_output(&out, r###"CompressObjectTestCase::test_decompress_raw_with_dictionary: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/zlib/compress_object_test_case__test_decompress_unused_data.py`.
#[test]
fn test_gen_behavior_std_libs_zlib_compress_object_test_case__test_decompress_unused_data() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "behavior"
# case = "compress_object_test_case__test_decompress_unused_data"
# subject = "cpython.test_zlib.CompressObjectTestCase.test_decompress_unused_data"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_zlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_zlib.py::CompressObjectTestCase::test_decompress_unused_data
"""Auto-ported test: CompressObjectTestCase::test_decompress_unused_data (CPython 3.12 oracle)."""


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
source = b'abcdefghijklmnopqrstuvwxyz'
remainder = b'0123456789'
y = zlib.compress(source)
x = y + remainder
for maxlen in (0, 1000):
    for step in (1, 2, len(y), len(x)):
        dco = zlib.decompressobj()
        data = b''
        for i in range(0, len(x), step):
            if i < len(y):

                assert dco.unused_data == b''
            if maxlen == 0:
                data += dco.decompress(x[i:i + step])

                assert dco.unconsumed_tail == b''
            else:
                data += dco.decompress(dco.unconsumed_tail + x[i:i + step], maxlen)
        data += dco.flush()

        assert dco.eof

        assert data == source

        assert dco.unconsumed_tail == b''

        assert dco.unused_data == remainder
print("CompressObjectTestCase::test_decompress_unused_data: ok")
"###);
    assert_output(&out, r###"CompressObjectTestCase::test_decompress_unused_data: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/zlib/compress_object_test_case__test_decompresscopy.py`.
#[test]
fn test_gen_behavior_std_libs_zlib_compress_object_test_case__test_decompresscopy() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "behavior"
# case = "compress_object_test_case__test_decompresscopy"
# subject = "cpython.test_zlib.CompressObjectTestCase.test_decompresscopy"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_zlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_zlib.py::CompressObjectTestCase::test_decompresscopy
"""Auto-ported test: CompressObjectTestCase::test_decompresscopy (CPython 3.12 oracle)."""


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
data = HAMLET_SCENE
comp = zlib.compress(data)

assert isinstance(comp, bytes)
for func in (lambda c: c.copy(), copy.copy, copy.deepcopy):
    d0 = zlib.decompressobj()
    bufs0 = []
    bufs0.append(d0.decompress(comp[:32]))
    d1 = func(d0)
    bufs1 = bufs0[:]
    bufs0.append(d0.decompress(comp[32:]))
    s0 = b''.join(bufs0)
    bufs1.append(d1.decompress(comp[32:]))
    s1 = b''.join(bufs1)

    assert s0 == s1

    assert s0 == data
print("CompressObjectTestCase::test_decompresscopy: ok")
"###);
    assert_output(&out, r###"CompressObjectTestCase::test_decompresscopy: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/zlib/compress_object_test_case__test_decompresspickle.py`.
#[test]
fn test_gen_behavior_std_libs_zlib_compress_object_test_case__test_decompresspickle() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "behavior"
# case = "compress_object_test_case__test_decompresspickle"
# subject = "cpython.test_zlib.CompressObjectTestCase.test_decompresspickle"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_zlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_zlib.py::CompressObjectTestCase::test_decompresspickle
"""Auto-ported test: CompressObjectTestCase::test_decompresspickle (CPython 3.12 oracle)."""


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
for proto in range(pickle.HIGHEST_PROTOCOL + 1):
    try:
        pickle.dumps(zlib.decompressobj(), proto)
        raise AssertionError('expected (TypeError, pickle.PicklingError)')
    except (TypeError, pickle.PicklingError):
        pass
print("CompressObjectTestCase::test_decompresspickle: ok")
"###);
    assert_output(&out, r###"CompressObjectTestCase::test_decompresspickle: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/zlib/compress_object_test_case__test_dictionary_streaming.py`.
#[test]
fn test_gen_behavior_std_libs_zlib_compress_object_test_case__test_dictionary_streaming() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "behavior"
# case = "compress_object_test_case__test_dictionary_streaming"
# subject = "cpython.test_zlib.CompressObjectTestCase.test_dictionary_streaming"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_zlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_zlib.py::CompressObjectTestCase::test_dictionary_streaming
"""Auto-ported test: CompressObjectTestCase::test_dictionary_streaming (CPython 3.12 oracle)."""


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
co = zlib.compressobj(zdict=HAMLET_SCENE)
do = zlib.decompressobj(zdict=HAMLET_SCENE)
piece = HAMLET_SCENE[1000:1500]
d0 = co.compress(piece) + co.flush(zlib.Z_SYNC_FLUSH)
d1 = co.compress(piece[100:]) + co.flush(zlib.Z_SYNC_FLUSH)
d2 = co.compress(piece[:-100]) + co.flush(zlib.Z_SYNC_FLUSH)

assert do.decompress(d0) == piece

assert do.decompress(d1) == piece[100:]

assert do.decompress(d2) == piece[:-100]
print("CompressObjectTestCase::test_dictionary_streaming: ok")
"###);
    assert_output(&out, r###"CompressObjectTestCase::test_dictionary_streaming: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/zlib/compress_object_test_case__test_empty_flush.py`.
#[test]
fn test_gen_behavior_std_libs_zlib_compress_object_test_case__test_empty_flush() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "behavior"
# case = "compress_object_test_case__test_empty_flush"
# subject = "cpython.test_zlib.CompressObjectTestCase.test_empty_flush"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_zlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_zlib.py::CompressObjectTestCase::test_empty_flush
"""Auto-ported test: CompressObjectTestCase::test_empty_flush (CPython 3.12 oracle)."""


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
co = zlib.compressobj(zlib.Z_BEST_COMPRESSION)

assert co.flush()
dco = zlib.decompressobj()

assert dco.flush() == b''
print("CompressObjectTestCase::test_empty_flush: ok")
"###);
    assert_output(&out, r###"CompressObjectTestCase::test_empty_flush: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/zlib/compress_object_test_case__test_flush_with_freed_input.py`.
#[test]
fn test_gen_behavior_std_libs_zlib_compress_object_test_case__test_flush_with_freed_input() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "behavior"
# case = "compress_object_test_case__test_flush_with_freed_input"
# subject = "cpython.test_zlib.CompressObjectTestCase.test_flush_with_freed_input"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_zlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_zlib.py::CompressObjectTestCase::test_flush_with_freed_input
"""Auto-ported test: CompressObjectTestCase::test_flush_with_freed_input (CPython 3.12 oracle)."""


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
input1 = b'abcdefghijklmnopqrstuvwxyz'
input2 = b'QWERTYUIOPASDFGHJKLZXCVBNM'
data = zlib.compress(input1)
dco = zlib.decompressobj()
dco.decompress(data, 1)
del data
data = zlib.compress(input2)

assert dco.flush() == input1[1:]
print("CompressObjectTestCase::test_flush_with_freed_input: ok")
"###);
    assert_output(&out, r###"CompressObjectTestCase::test_flush_with_freed_input: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/zlib/compress_object_test_case__test_flushes.py`.
#[test]
fn test_gen_behavior_std_libs_zlib_compress_object_test_case__test_flushes() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "behavior"
# case = "compress_object_test_case__test_flushes"
# subject = "cpython.test_zlib.CompressObjectTestCase.test_flushes"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_zlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_zlib.py::CompressObjectTestCase::test_flushes
"""Auto-ported test: CompressObjectTestCase::test_flushes (CPython 3.12 oracle)."""


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
sync_opt = ['Z_NO_FLUSH', 'Z_SYNC_FLUSH', 'Z_FULL_FLUSH', 'Z_PARTIAL_FLUSH']
if ZLIB_RUNTIME_VERSION_TUPLE >= (1, 2, 5, 3):
    sync_opt.append('Z_BLOCK')
sync_opt = [getattr(zlib, opt) for opt in sync_opt if hasattr(zlib, opt)]
data = HAMLET_SCENE * 8
for sync in sync_opt:
    for level in range(10):
        obj = zlib.compressobj(level)
        a = obj.compress(data[:3000])
        b = obj.flush(sync)
        c = obj.compress(data[3000:])
        d = obj.flush()

        assert zlib.decompress(b''.join([a, b, c, d])) == data
        del obj
print("CompressObjectTestCase::test_flushes: ok")
"###);
    assert_output(&out, r###"CompressObjectTestCase::test_flushes: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/zlib/compress_object_test_case__test_keywords.py`.
#[test]
fn test_gen_behavior_std_libs_zlib_compress_object_test_case__test_keywords() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "behavior"
# case = "compress_object_test_case__test_keywords"
# subject = "cpython.test_zlib.CompressObjectTestCase.test_keywords"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_zlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_zlib.py::CompressObjectTestCase::test_keywords
"""Auto-ported test: CompressObjectTestCase::test_keywords (CPython 3.12 oracle)."""


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
level = 2
method = zlib.DEFLATED
wbits = -12
memLevel = 9
strategy = zlib.Z_FILTERED
co = zlib.compressobj(level=level, method=method, wbits=wbits, memLevel=memLevel, strategy=strategy, zdict=b'')
do = zlib.decompressobj(wbits=wbits, zdict=b'')
try:
    co.compress(data=HAMLET_SCENE)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
try:
    do.decompress(data=zlib.compress(HAMLET_SCENE))
    raise AssertionError('expected TypeError')
except TypeError:
    pass
x = co.compress(HAMLET_SCENE) + co.flush()
y = do.decompress(x, max_length=len(HAMLET_SCENE)) + do.flush()

assert HAMLET_SCENE == y
print("CompressObjectTestCase::test_keywords: ok")
"###);
    assert_output(&out, r###"CompressObjectTestCase::test_keywords: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/zlib/compress_object_test_case__test_maxlen_custom.py`.
#[test]
fn test_gen_behavior_std_libs_zlib_compress_object_test_case__test_maxlen_custom() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "behavior"
# case = "compress_object_test_case__test_maxlen_custom"
# subject = "cpython.test_zlib.CompressObjectTestCase.test_maxlen_custom"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_zlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_zlib.py::CompressObjectTestCase::test_maxlen_custom
"""Auto-ported test: CompressObjectTestCase::test_maxlen_custom (CPython 3.12 oracle)."""


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
data = HAMLET_SCENE * 10
compressed = zlib.compress(data, 1)
dco = zlib.decompressobj()

assert dco.decompress(compressed, CustomInt()) == data[:100]
print("CompressObjectTestCase::test_maxlen_custom: ok")
"###);
    assert_output(&out, r###"CompressObjectTestCase::test_maxlen_custom: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/zlib/compress_object_test_case__test_maxlen_large.py`.
#[test]
fn test_gen_behavior_std_libs_zlib_compress_object_test_case__test_maxlen_large() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "behavior"
# case = "compress_object_test_case__test_maxlen_large"
# subject = "cpython.test_zlib.CompressObjectTestCase.test_maxlen_large"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_zlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_zlib.py::CompressObjectTestCase::test_maxlen_large
"""Auto-ported test: CompressObjectTestCase::test_maxlen_large (CPython 3.12 oracle)."""


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
data = HAMLET_SCENE * 10

assert len(data) > zlib.DEF_BUF_SIZE
compressed = zlib.compress(data, 1)
dco = zlib.decompressobj()

assert dco.decompress(compressed, sys.maxsize) == data
print("CompressObjectTestCase::test_maxlen_large: ok")
"###);
    assert_output(&out, r###"CompressObjectTestCase::test_maxlen_large: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/zlib/compress_object_test_case__test_maxlenmisc.py`.
#[test]
fn test_gen_behavior_std_libs_zlib_compress_object_test_case__test_maxlenmisc() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "behavior"
# case = "compress_object_test_case__test_maxlenmisc"
# subject = "cpython.test_zlib.CompressObjectTestCase.test_maxlenmisc"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_zlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_zlib.py::CompressObjectTestCase::test_maxlenmisc
"""Auto-ported test: CompressObjectTestCase::test_maxlenmisc (CPython 3.12 oracle)."""


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
dco = zlib.decompressobj()

try:
    dco.decompress(b'', -1)
    raise AssertionError('expected ValueError')
except ValueError:
    pass

assert b'' == dco.unconsumed_tail
print("CompressObjectTestCase::test_maxlenmisc: ok")
"###);
    assert_output(&out, r###"CompressObjectTestCase::test_maxlenmisc: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/zlib/compress_test_case__test_custom_bufsize.py`.
#[test]
fn test_gen_behavior_std_libs_zlib_compress_test_case__test_custom_bufsize() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "behavior"
# case = "compress_test_case__test_custom_bufsize"
# subject = "cpython.test_zlib.CompressTestCase.test_custom_bufsize"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_zlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_zlib.py::CompressTestCase::test_custom_bufsize
"""Auto-ported test: CompressTestCase::test_custom_bufsize (CPython 3.12 oracle)."""


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
data = HAMLET_SCENE * 10
compressed = zlib.compress(data, 1)

assert zlib.decompress(compressed, 15, CustomInt()) == data
print("CompressTestCase::test_custom_bufsize: ok")
"###);
    assert_output(&out, r###"CompressTestCase::test_custom_bufsize: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/zlib/compress_test_case__test_keywords.py`.
#[test]
fn test_gen_behavior_std_libs_zlib_compress_test_case__test_keywords() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "behavior"
# case = "compress_test_case__test_keywords"
# subject = "cpython.test_zlib.CompressTestCase.test_keywords"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_zlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_zlib.py::CompressTestCase::test_keywords
"""Auto-ported test: CompressTestCase::test_keywords (CPython 3.12 oracle)."""


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
x = zlib.compress(HAMLET_SCENE, level=3)

assert zlib.decompress(x) == HAMLET_SCENE
try:
    zlib.compress(data=HAMLET_SCENE, level=3)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

assert zlib.decompress(x, wbits=zlib.MAX_WBITS, bufsize=zlib.DEF_BUF_SIZE) == HAMLET_SCENE
print("CompressTestCase::test_keywords: ok")
"###);
    assert_output(&out, r###"CompressTestCase::test_keywords: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/zlib/compress_test_case__test_speech.py`.
#[test]
fn test_gen_behavior_std_libs_zlib_compress_test_case__test_speech() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "behavior"
# case = "compress_test_case__test_speech"
# subject = "cpython.test_zlib.CompressTestCase.test_speech"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_zlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_zlib.py::CompressTestCase::test_speech
"""Auto-ported test: CompressTestCase::test_speech (CPython 3.12 oracle)."""


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
x = zlib.compress(HAMLET_SCENE)

assert zlib.decompress(x) == HAMLET_SCENE
print("CompressTestCase::test_speech: ok")
"###);
    assert_output(&out, r###"CompressTestCase::test_speech: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/zlib/compress_test_case__test_speech128.py`.
#[test]
fn test_gen_behavior_std_libs_zlib_compress_test_case__test_speech128() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "behavior"
# case = "compress_test_case__test_speech128"
# subject = "cpython.test_zlib.CompressTestCase.test_speech128"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_zlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_zlib.py::CompressTestCase::test_speech128
"""Auto-ported test: CompressTestCase::test_speech128 (CPython 3.12 oracle)."""


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
data = HAMLET_SCENE * 128
x = zlib.compress(data)
if not HW_ACCELERATED:

    assert zlib.compress(bytearray(data)) == x
for ob in (x, bytearray(x)):

    assert zlib.decompress(ob) == data
print("CompressTestCase::test_speech128: ok")
"###);
    assert_output(&out, r###"CompressTestCase::test_speech128: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/zlib/compressobj_copy_forks_state.py`.
#[test]
fn test_gen_behavior_std_libs_zlib_compressobj_copy_forks_state() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "behavior"
# case = "compressobj_copy_forks_state"
# subject = "zlib.compressobj"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zlib.compressobj: copy() forks a compressor at the copy point so both share the pre-copy prefix then diverge with independent follow-up input, each decompressing to its own concatenation"""
import zlib

_TEXT = b"the quick brown fox jumps over the lazy dog " * 64
_c0 = zlib.compressobj(zlib.Z_BEST_COMPRESSION)
_prefix = _c0.compress(_TEXT)
_c1 = _c0.copy()
_alt = b"ZZZ" * 50
_s0 = _prefix + _c0.compress(_TEXT) + _c0.flush()
_s1 = _prefix + _c1.compress(_alt) + _c1.flush()
assert zlib.decompress(_s0) == _TEXT + _TEXT, "copy: original keeps compressing"
assert zlib.decompress(_s1) == _TEXT + _alt, "copy: fork diverges from prefix"

print("compressobj_copy_forks_state OK")
"###);
    assert_output(&out, r###"compressobj_copy_forks_state OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/zlib/compressobj_full_param_set_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_zlib_compressobj_full_param_set_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "behavior"
# case = "compressobj_full_param_set_roundtrip"
# subject = "zlib.compressobj"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zlib.compressobj: compressobj accepts the full positional parameter set (level, method, negative wbits, memLevel, strategy) and the matching raw decompressor recovers the input"""
import zlib

_TEXT = b"the quick brown fox jumps over the lazy dog " * 64
_co = zlib.compressobj(2, zlib.DEFLATED, -12, 9, zlib.Z_FILTERED)
_blob = _co.compress(_TEXT) + _co.flush()
_dco = zlib.decompressobj(-12)
assert _dco.decompress(_blob) + _dco.flush() == _TEXT, "full-param round-trip"

print("compressobj_full_param_set_roundtrip OK")
"###);
    assert_output(&out, r###"compressobj_full_param_set_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/zlib/crc32_exact_values_and_seed_chaining.py`.
#[test]
fn test_gen_behavior_std_libs_zlib_crc32_exact_values_and_seed_chaining() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "behavior"
# case = "crc32_exact_values_and_seed_chaining"
# subject = "zlib.crc32"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zlib.crc32: crc32 returns the IEEE 802.3 unsigned 32-bit value, seeds with 0, passes a seed through for empty input, matches documented values for explicit seeds, chains incrementally to equal the one-shot over the concatenation, and stays unsigned for a large seed"""
import zlib

# Exact values (unsigned) and known small inputs.
assert zlib.crc32(b"abcdefghijklmnop") == 2486878355, "crc32 abc..p"
assert zlib.crc32(b"spam") == 1138425661, "crc32 spam"
assert zlib.crc32(b"hello") == 907060870, "crc32 hello"
assert zlib.crc32(b"\x00") == 3523407757, "crc32 nul"

# Start value is the identity/seed: crc32 seeds with 0.
assert zlib.crc32(b"") == 0, "crc32 empty default = 0"
assert zlib.crc32(b"") == zlib.crc32(b"", 0), "crc32 default seed is 0"
# Empty input returns the supplied seed unchanged.
assert zlib.crc32(b"", 432) == 432, "crc32 empty passes seed through"

# Explicit seed produces documented values.
assert zlib.crc32(b"penguin", 0) == 3854672160, "crc32 penguin seed 0"
assert zlib.crc32(b"penguin", 1) == 1136044692, "crc32 penguin seed 1"

# Seed chaining equals one-shot over the concatenation.
_part = zlib.crc32(b"hel")
assert zlib.crc32(b"lo", _part) == zlib.crc32(b"hello"), "crc32 incremental"

# Large seed (0xFFFFFFFF) is accepted and result stays unsigned 32-bit.
assert 0 <= zlib.crc32(b"abc", 4294967295) <= 0xFFFFFFFF, "crc32 big seed unsigned"

print("crc32_exact_values_and_seed_chaining OK")
"###);
    assert_output(&out, r###"crc32_exact_values_and_seed_chaining OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/zlib/crc32_matches_binascii_crc32.py`.
#[test]
fn test_gen_behavior_std_libs_zlib_crc32_matches_binascii_crc32() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "behavior"
# case = "crc32_matches_binascii_crc32"
# subject = "zlib.crc32"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zlib.crc32: zlib.crc32 and binascii.crc32 produce identical checksums over the same input"""
import binascii
import zlib

assert binascii.crc32(b"abcdefghijklmnop") == zlib.crc32(b"abcdefghijklmnop"), "binascii parity abc..p"
assert binascii.crc32(b"spam") == zlib.crc32(b"spam"), "binascii parity spam"
assert binascii.crc32(b"") == zlib.crc32(b""), "binascii parity empty"

print("crc32_matches_binascii_crc32 OK")
"###);
    assert_output(&out, r###"crc32_matches_binascii_crc32 OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/zlib/decompressobj_eof_flag_flips_at_stream_end.py`.
#[test]
fn test_gen_behavior_std_libs_zlib_decompressobj_eof_flag_flips_at_stream_end() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "behavior"
# case = "decompressobj_eof_flag_flips_at_stream_end"
# subject = "zlib.decompressobj"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zlib.decompressobj: the decompressobj eof flag is False before input and mid-stream, and flips to True only once the full stream has been fed"""
import zlib

_stream = zlib.compress(b"foo")
_dco = zlib.decompressobj()
assert _dco.eof is False, "eof False before any input"
_dco.decompress(_stream[:-2])
assert _dco.eof is False, "eof False mid-stream"
_dco.decompress(_stream[-2:])
assert _dco.eof is True, "eof True after full stream"

print("decompressobj_eof_flag_flips_at_stream_end OK")
"###);
    assert_output(&out, r###"decompressobj_eof_flag_flips_at_stream_end OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/zlib/decompressobj_max_length_caps_via_unconsumed_tail.py`.
#[test]
fn test_gen_behavior_std_libs_zlib_decompressobj_max_length_caps_via_unconsumed_tail() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "behavior"
# case = "decompressobj_max_length_caps_via_unconsumed_tail"
# subject = "zlib.decompressobj"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zlib.decompressobj: a max_length argument caps output bytes, the remaining input reappears in unconsumed_tail, and resuming from unconsumed_tail reconstructs the full payload"""
import zlib

_src = zlib.compress(b"abcdefghijklmnopqrstuvwxyz")
_dco = zlib.decompressobj()
_head = _dco.decompress(_src, 5)
assert _head == b"abcde", "max_length caps output to 5 bytes"
assert len(_dco.unconsumed_tail) > 0, "leftover input in unconsumed_tail"
_tail = _dco.decompress(_dco.unconsumed_tail)
assert _head + _tail == b"abcdefghijklmnopqrstuvwxyz", "resume from unconsumed_tail"

print("decompressobj_max_length_caps_via_unconsumed_tail OK")
"###);
    assert_output(&out, r###"decompressobj_max_length_caps_via_unconsumed_tail OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/zlib/decompressobj_unused_data_captures_trailing_bytes.py`.
#[test]
fn test_gen_behavior_std_libs_zlib_decompressobj_unused_data_captures_trailing_bytes() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "behavior"
# case = "decompressobj_unused_data_captures_trailing_bytes"
# subject = "zlib.decompressobj"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zlib.decompressobj: bytes appended after a complete deflate stream land in unused_data (not in the output), and unconsumed_tail is empty without a max_length cap"""
import zlib

_packed = zlib.compress(b"abcdefghijklmnopqrstuvwxyz") + b"0123456789"
_dco = zlib.decompressobj()
assert _dco.decompress(_packed) == b"abcdefghijklmnopqrstuvwxyz", "stops at stream end"
assert _dco.unused_data == b"0123456789", "trailing bytes in unused_data"
assert _dco.unconsumed_tail == b"", "no unconsumed tail without max_length"

print("decompressobj_unused_data_captures_trailing_bytes OK")
"###);
    assert_output(&out, r###"decompressobj_unused_data_captures_trailing_bytes OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/zlib/exception_test_case__test_badargs.py`.
#[test]
fn test_gen_behavior_std_libs_zlib_exception_test_case__test_badargs() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "behavior"
# case = "exception_test_case__test_badargs"
# subject = "cpython.test_zlib.ExceptionTestCase.test_badargs"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_zlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_zlib.py::ExceptionTestCase::test_badargs
"""Auto-ported test: ExceptionTestCase::test_badargs (CPython 3.12 oracle)."""


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

try:
    zlib.adler32()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    zlib.crc32()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    zlib.compress()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    zlib.decompress()
    raise AssertionError('expected TypeError')
except TypeError:
    pass
for arg in (42, None, '', 'abc', (), []):

    try:
        zlib.adler32(arg)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

    try:
        zlib.crc32(arg)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

    try:
        zlib.compress(arg)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

    try:
        zlib.decompress(arg)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
print("ExceptionTestCase::test_badargs: ok")
"###);
    assert_output(&out, r###"ExceptionTestCase::test_badargs: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/zlib/exception_test_case__test_badcompressobj.py`.
#[test]
fn test_gen_behavior_std_libs_zlib_exception_test_case__test_badcompressobj() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "behavior"
# case = "exception_test_case__test_badcompressobj"
# subject = "cpython.test_zlib.ExceptionTestCase.test_badcompressobj"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_zlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_zlib.py::ExceptionTestCase::test_badcompressobj
"""Auto-ported test: ExceptionTestCase::test_badcompressobj (CPython 3.12 oracle)."""


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

try:
    zlib.compressobj(1, zlib.DEFLATED, 0)
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    zlib.compressobj(1, zlib.DEFLATED, zlib.MAX_WBITS + 1)
    raise AssertionError('expected ValueError')
except ValueError:
    pass
print("ExceptionTestCase::test_badcompressobj: ok")
"###);
    assert_output(&out, r###"ExceptionTestCase::test_badcompressobj: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/zlib/exception_test_case__test_baddecompressobj.py`.
#[test]
fn test_gen_behavior_std_libs_zlib_exception_test_case__test_baddecompressobj() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "behavior"
# case = "exception_test_case__test_baddecompressobj"
# subject = "cpython.test_zlib.ExceptionTestCase.test_baddecompressobj"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_zlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_zlib.py::ExceptionTestCase::test_baddecompressobj
"""Auto-ported test: ExceptionTestCase::test_baddecompressobj (CPython 3.12 oracle)."""


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

try:
    zlib.decompressobj(-1)
    raise AssertionError('expected ValueError')
except ValueError:
    pass
print("ExceptionTestCase::test_baddecompressobj: ok")
"###);
    assert_output(&out, r###"ExceptionTestCase::test_baddecompressobj: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/zlib/exception_test_case__test_badlevel.py`.
#[test]
fn test_gen_behavior_std_libs_zlib_exception_test_case__test_badlevel() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "behavior"
# case = "exception_test_case__test_badlevel"
# subject = "cpython.test_zlib.ExceptionTestCase.test_badlevel"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_zlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_zlib.py::ExceptionTestCase::test_badlevel
"""Auto-ported test: ExceptionTestCase::test_badlevel (CPython 3.12 oracle)."""


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

try:
    zlib.compress(b'ERROR', 10)
    raise AssertionError('expected zlib.error')
except zlib.error:
    pass
print("ExceptionTestCase::test_badlevel: ok")
"###);
    assert_output(&out, r###"ExceptionTestCase::test_badlevel: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/zlib/exception_test_case__test_decompressobj_badflush.py`.
#[test]
fn test_gen_behavior_std_libs_zlib_exception_test_case__test_decompressobj_badflush() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "behavior"
# case = "exception_test_case__test_decompressobj_badflush"
# subject = "cpython.test_zlib.ExceptionTestCase.test_decompressobj_badflush"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_zlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_zlib.py::ExceptionTestCase::test_decompressobj_badflush
"""Auto-ported test: ExceptionTestCase::test_decompressobj_badflush (CPython 3.12 oracle)."""


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

try:
    zlib.decompressobj().flush(0)
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    zlib.decompressobj().flush(-1)
    raise AssertionError('expected ValueError')
except ValueError:
    pass
print("ExceptionTestCase::test_decompressobj_badflush: ok")
"###);
    assert_output(&out, r###"ExceptionTestCase::test_decompressobj_badflush: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/zlib/higher_level_smaller_or_equal.py`.
#[test]
fn test_gen_behavior_std_libs_zlib_higher_level_smaller_or_equal() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "behavior"
# case = "higher_level_smaller_or_equal"
# subject = "zlib.compress"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zlib.compress: for repetitive data level 9 produces a stream no larger than level 1, and both decompress back to the original"""
import zlib

_compressible = b"abc" * 5000
_c1 = zlib.compress(_compressible, level=1)
_c9 = zlib.compress(_compressible, level=9)
assert len(_c9) <= len(_c1), f"level 9 ({len(_c9)}) <= level 1 ({len(_c1)})"
assert zlib.decompress(_c1) == _compressible, "level 1 round-trip"
assert zlib.decompress(_c9) == _compressible, "level 9 round-trip"

print("higher_level_smaller_or_equal OK")
"###);
    assert_output(&out, r###"higher_level_smaller_or_equal OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/zlib/preset_dictionary_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_zlib_preset_dictionary_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "behavior"
# case = "preset_dictionary_roundtrip"
# subject = "zlib.compressobj"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zlib.compressobj: a preset zdict supplied to both compressobj and decompressobj round-trips the payload, while a decompressor without the dict raises zlib.error"""
import zlib

_TEXT = b"the quick brown fox jumps over the lazy dog " * 64
_zdict = b"the quick brown fox lazy dog"
_co = zlib.compressobj(zdict=_zdict)
_cd = _co.compress(_TEXT) + _co.flush()
_dco = zlib.decompressobj(zdict=_zdict)
assert _dco.decompress(_cd) + _dco.flush() == _TEXT, "zdict round-trip"

_no_dict = zlib.decompressobj()
_raised = False
try:
    _no_dict.decompress(_cd)
except zlib.error:
    _raised = True
assert _raised, "missing zdict raises zlib.error"

print("preset_dictionary_roundtrip OK")
"###);
    assert_output(&out, r###"preset_dictionary_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/zlib/raw_deflate_negative_wbits_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_zlib_raw_deflate_negative_wbits_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "behavior"
# case = "raw_deflate_negative_wbits_roundtrip"
# subject = "zlib.compress"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zlib.compress: compress with wbits=-15 (raw DEFLATE, no zlib header/trailer) and decompress with wbits=-15 round-trips the input"""
import zlib

_data = b"stream test data " * 100
_raw_comp = zlib.compress(_data, wbits=-15)
_raw_decomp = zlib.decompress(_raw_comp, wbits=-15)
assert _raw_decomp == _data, "raw deflate (wbits=-15)"

print("raw_deflate_negative_wbits_roundtrip OK")
"###);
    assert_output(&out, r###"raw_deflate_negative_wbits_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/zlib/streaming_compress_concatenates.py`.
#[test]
fn test_gen_behavior_std_libs_zlib_streaming_compress_concatenates() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "behavior"
# case = "streaming_compress_concatenates"
# subject = "zlib.compressobj"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zlib.compressobj: feeding a payload to compressobj in successive chunks then flushing yields a stream that one-shot decompress recovers to the full input"""
import zlib

_data = b"stream test data " * 100
_comp = zlib.compressobj(level=6)
_parts = [_data[:50], _data[50:150], _data[150:]]
_compressed = b""
for _part in _parts:
    _compressed += _comp.compress(_part)
_compressed += _comp.flush()
assert zlib.decompress(_compressed) == _data, "streaming compress"

print("streaming_compress_concatenates OK")
"###);
    assert_output(&out, r###"streaming_compress_concatenates OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/zlib/streaming_decompress_chunked.py`.
#[test]
fn test_gen_behavior_std_libs_zlib_streaming_decompress_chunked() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "behavior"
# case = "streaming_decompress_chunked"
# subject = "zlib.decompressobj"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zlib.decompressobj: feeding a compressed stream to decompressobj in fixed-size chunks then flushing reassembles the original input"""
import zlib

_original = b"decomp test " * 50
_compressed = zlib.compress(_original)
_decomp = zlib.decompressobj()
_result = b""
_chunk = 20
for _i in range(0, len(_compressed), _chunk):
    _result += _decomp.decompress(_compressed[_i:_i + _chunk])
_result += _decomp.flush()
assert _result == _original, "streaming decompress"

print("streaming_decompress_chunked OK")
"###);
    assert_output(&out, r###"streaming_decompress_chunked OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/zlib/version_constants_shape_and_values.py`.
#[test]
fn test_gen_behavior_std_libs_zlib_version_constants_shape_and_values() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "behavior"
# case = "version_constants_shape_and_values"
# subject = "zlib.ZLIB_VERSION"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zlib.ZLIB_VERSION: ZLIB_VERSION and ZLIB_RUNTIME_VERSION are non-empty strings sharing a major version, and the named method/strategy/flush constants (DEFLATED, MAX_WBITS, Z_BEST_COMPRESSION, Z_FILTERED, Z_SYNC_FLUSH, DEF_BUF_SIZE) hold their documented int values"""
import zlib

# Version strings are present, non-empty, and share a major version. Exact
# numbers vary by build, so only structural shape is asserted.
assert isinstance(zlib.ZLIB_VERSION, str), "ZLIB_VERSION is str"
assert isinstance(zlib.ZLIB_RUNTIME_VERSION, str), "ZLIB_RUNTIME_VERSION is str"
assert len(zlib.ZLIB_VERSION) > 0, "ZLIB_VERSION non-empty"
assert zlib.ZLIB_RUNTIME_VERSION[0] == zlib.ZLIB_VERSION[0], "major version matches"

# Named method/strategy/flush constants are ints with documented values.
assert zlib.DEFLATED == 8, "DEFLATED == 8"
assert zlib.MAX_WBITS == 15, "MAX_WBITS == 15"
assert zlib.Z_BEST_COMPRESSION == 9, "Z_BEST_COMPRESSION == 9"
assert zlib.Z_FILTERED == 1, "Z_FILTERED == 1"
assert zlib.Z_SYNC_FLUSH == 2, "Z_SYNC_FLUSH == 2"
assert isinstance(zlib.DEF_BUF_SIZE, int) and zlib.DEF_BUF_SIZE > 0, "DEF_BUF_SIZE positive int"

print("version_constants_shape_and_values OK")
"###);
    assert_output(&out, r###"version_constants_shape_and_values OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/zlib/version_test_case__test_library_version.py`.
#[test]
fn test_gen_behavior_std_libs_zlib_version_test_case__test_library_version() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "behavior"
# case = "version_test_case__test_library_version"
# subject = "cpython.test_zlib.VersionTestCase.test_library_version"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_zlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_zlib.py::VersionTestCase::test_library_version
"""Auto-ported test: VersionTestCase::test_library_version (CPython 3.12 oracle)."""


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

assert zlib.ZLIB_RUNTIME_VERSION[0] == zlib.ZLIB_VERSION[0]
print("VersionTestCase::test_library_version: ok")
"###);
    assert_output(&out, r###"VersionTestCase::test_library_version: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/zlib/z_sync_flush_yields_decodable_chunk.py`.
#[test]
fn test_gen_behavior_std_libs_zlib_z_sync_flush_yields_decodable_chunk() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "behavior"
# case = "z_sync_flush_yields_decodable_chunk"
# subject = "zlib.compressobj"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zlib.compressobj: flushing a compressor with Z_SYNC_FLUSH emits a chunk that a decompressor can decode mid-stream without the final flush"""
import zlib

_co = zlib.compressobj(zlib.Z_BEST_COMPRESSION)
_dco = zlib.decompressobj()
_chunk = b"sync-flush payload " * 32
_first = _co.compress(_chunk)
_second = _co.flush(zlib.Z_SYNC_FLUSH)
assert _dco.decompress(_first + _second) == _chunk, "Z_SYNC_FLUSH chunk decodable"

print("z_sync_flush_yields_decodable_chunk OK")
"###);
    assert_output(&out, r###"z_sync_flush_yields_decodable_chunk OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/zlib/zlib_decompressor_test__test_constructor.py`.
#[test]
fn test_gen_behavior_std_libs_zlib_zlib_decompressor_test__test_constructor() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "behavior"
# case = "zlib_decompressor_test__test_constructor"
# subject = "cpython.test_zlib.ZlibDecompressorTest.test_Constructor"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_zlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_zlib.py::ZlibDecompressorTest::test_Constructor
"""Auto-ported test: ZlibDecompressorTest::test_Constructor (CPython 3.12 oracle)."""


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
TEXT = HAMLET_SCENE
DATA = zlib.compress(HAMLET_SCENE)
BAD_DATA = b'Not a valid deflate block'
BIG_TEXT = DATA * (128 * 1024 // len(DATA) + 1)
BIG_DATA = zlib.compress(BIG_TEXT)

try:
    zlib._ZlibDecompressor('ASDA')
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    zlib._ZlibDecompressor(-15, 'notbytes')
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    zlib._ZlibDecompressor(-15, b'bytes', 5)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("ZlibDecompressorTest::test_Constructor: ok")
"###);
    assert_output(&out, r###"ZlibDecompressorTest::test_Constructor: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/zlib/zlib_decompressor_test__test_decompress.py`.
#[test]
fn test_gen_behavior_std_libs_zlib_zlib_decompressor_test__test_decompress() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "behavior"
# case = "zlib_decompressor_test__test_decompress"
# subject = "cpython.test_zlib.ZlibDecompressorTest.testDecompress"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_zlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_zlib.py::ZlibDecompressorTest::testDecompress
"""Auto-ported test: ZlibDecompressorTest::testDecompress (CPython 3.12 oracle)."""


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
TEXT = HAMLET_SCENE
DATA = zlib.compress(HAMLET_SCENE)
BAD_DATA = b'Not a valid deflate block'
BIG_TEXT = DATA * (128 * 1024 // len(DATA) + 1)
BIG_DATA = zlib.compress(BIG_TEXT)
zlibd = zlib._ZlibDecompressor()

try:
    zlibd.decompress()
    raise AssertionError('expected TypeError')
except TypeError:
    pass
text = zlibd.decompress(DATA)

assert text == TEXT
print("ZlibDecompressorTest::testDecompress: ok")
"###);
    assert_output(&out, r###"ZlibDecompressorTest::testDecompress: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/zlib/zlib_decompressor_test__test_decompress_unused_data.py`.
#[test]
fn test_gen_behavior_std_libs_zlib_zlib_decompressor_test__test_decompress_unused_data() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "behavior"
# case = "zlib_decompressor_test__test_decompress_unused_data"
# subject = "cpython.test_zlib.ZlibDecompressorTest.testDecompressUnusedData"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_zlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_zlib.py::ZlibDecompressorTest::testDecompressUnusedData
"""Auto-ported test: ZlibDecompressorTest::testDecompressUnusedData (CPython 3.12 oracle)."""


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
TEXT = HAMLET_SCENE
DATA = zlib.compress(HAMLET_SCENE)
BAD_DATA = b'Not a valid deflate block'
BIG_TEXT = DATA * (128 * 1024 // len(DATA) + 1)
BIG_DATA = zlib.compress(BIG_TEXT)
zlibd = zlib._ZlibDecompressor()
unused_data = b'this is unused data'
text = zlibd.decompress(DATA + unused_data)

assert text == TEXT

assert zlibd.unused_data == unused_data
print("ZlibDecompressorTest::testDecompressUnusedData: ok")
"###);
    assert_output(&out, r###"ZlibDecompressorTest::testDecompressUnusedData: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/zlib/zlib_decompressor_test__test_decompressor_chunks_maxsize.py`.
#[test]
fn test_gen_behavior_std_libs_zlib_zlib_decompressor_test__test_decompressor_chunks_maxsize() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "behavior"
# case = "zlib_decompressor_test__test_decompressor_chunks_maxsize"
# subject = "cpython.test_zlib.ZlibDecompressorTest.testDecompressorChunksMaxsize"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_zlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_zlib.py::ZlibDecompressorTest::testDecompressorChunksMaxsize
"""Auto-ported test: ZlibDecompressorTest::testDecompressorChunksMaxsize (CPython 3.12 oracle)."""


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
TEXT = HAMLET_SCENE
DATA = zlib.compress(HAMLET_SCENE)
BAD_DATA = b'Not a valid deflate block'
BIG_TEXT = DATA * (128 * 1024 // len(DATA) + 1)
BIG_DATA = zlib.compress(BIG_TEXT)
zlibd = zlib._ZlibDecompressor()
max_length = 100
out = []
len_ = len(BIG_DATA) - 64
out.append(zlibd.decompress(BIG_DATA[:len_], max_length=max_length))

assert not zlibd.needs_input

assert len(out[-1]) == max_length
out.append(zlibd.decompress(b'', max_length=max_length))

assert not zlibd.needs_input

assert len(out[-1]) == max_length
out.append(zlibd.decompress(BIG_DATA[len_:], max_length=max_length))

assert len(out[-1]) <= max_length
while not zlibd.eof:
    out.append(zlibd.decompress(b'', max_length=max_length))

    assert len(out[-1]) <= max_length
out = b''.join(out)

assert out == BIG_TEXT

assert zlibd.unused_data == b''
print("ZlibDecompressorTest::testDecompressorChunksMaxsize: ok")
"###);
    assert_output(&out, r###"ZlibDecompressorTest::testDecompressorChunksMaxsize: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/zlib/zlib_decompressor_test__test_decompressor_inputbuf_1.py`.
#[test]
fn test_gen_behavior_std_libs_zlib_zlib_decompressor_test__test_decompressor_inputbuf_1() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "behavior"
# case = "zlib_decompressor_test__test_decompressor_inputbuf_1"
# subject = "cpython.test_zlib.ZlibDecompressorTest.test_decompressor_inputbuf_1"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_zlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_zlib.py::ZlibDecompressorTest::test_decompressor_inputbuf_1
"""Auto-ported test: ZlibDecompressorTest::test_decompressor_inputbuf_1 (CPython 3.12 oracle)."""


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
TEXT = HAMLET_SCENE
DATA = zlib.compress(HAMLET_SCENE)
BAD_DATA = b'Not a valid deflate block'
BIG_TEXT = DATA * (128 * 1024 // len(DATA) + 1)
BIG_DATA = zlib.compress(BIG_TEXT)
zlibd = zlib._ZlibDecompressor()
out = []

assert zlibd.decompress(DATA[:100], max_length=0) == b''
out.append(zlibd.decompress(b'', 2))
out.append(zlibd.decompress(DATA[100:105], 15))
out.append(zlibd.decompress(DATA[105:]))

assert b''.join(out) == TEXT
print("ZlibDecompressorTest::test_decompressor_inputbuf_1: ok")
"###);
    assert_output(&out, r###"ZlibDecompressorTest::test_decompressor_inputbuf_1: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/zlib/zlib_decompressor_test__test_decompressor_inputbuf_2.py`.
#[test]
fn test_gen_behavior_std_libs_zlib_zlib_decompressor_test__test_decompressor_inputbuf_2() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "behavior"
# case = "zlib_decompressor_test__test_decompressor_inputbuf_2"
# subject = "cpython.test_zlib.ZlibDecompressorTest.test_decompressor_inputbuf_2"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_zlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_zlib.py::ZlibDecompressorTest::test_decompressor_inputbuf_2
"""Auto-ported test: ZlibDecompressorTest::test_decompressor_inputbuf_2 (CPython 3.12 oracle)."""


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
TEXT = HAMLET_SCENE
DATA = zlib.compress(HAMLET_SCENE)
BAD_DATA = b'Not a valid deflate block'
BIG_TEXT = DATA * (128 * 1024 // len(DATA) + 1)
BIG_DATA = zlib.compress(BIG_TEXT)
zlibd = zlib._ZlibDecompressor()
out = []

assert zlibd.decompress(DATA[:200], max_length=0) == b''
out.append(zlibd.decompress(b''))
out.append(zlibd.decompress(DATA[200:280], 2))
out.append(zlibd.decompress(DATA[280:300], 2))
out.append(zlibd.decompress(DATA[300:]))

assert b''.join(out) == TEXT
print("ZlibDecompressorTest::test_decompressor_inputbuf_2: ok")
"###);
    assert_output(&out, r###"ZlibDecompressorTest::test_decompressor_inputbuf_2: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/zlib/zlib_decompressor_test__test_decompressor_inputbuf_3.py`.
#[test]
fn test_gen_behavior_std_libs_zlib_zlib_decompressor_test__test_decompressor_inputbuf_3() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "behavior"
# case = "zlib_decompressor_test__test_decompressor_inputbuf_3"
# subject = "cpython.test_zlib.ZlibDecompressorTest.test_decompressor_inputbuf_3"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_zlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_zlib.py::ZlibDecompressorTest::test_decompressor_inputbuf_3
"""Auto-ported test: ZlibDecompressorTest::test_decompressor_inputbuf_3 (CPython 3.12 oracle)."""


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
TEXT = HAMLET_SCENE
DATA = zlib.compress(HAMLET_SCENE)
BAD_DATA = b'Not a valid deflate block'
BIG_TEXT = DATA * (128 * 1024 // len(DATA) + 1)
BIG_DATA = zlib.compress(BIG_TEXT)
zlibd = zlib._ZlibDecompressor()
out = []
out.append(zlibd.decompress(DATA[:200], 5))
out.append(zlibd.decompress(DATA[200:300], 5))
out.append(zlibd.decompress(DATA[300:]))

assert b''.join(out) == TEXT
print("ZlibDecompressorTest::test_decompressor_inputbuf_3: ok")
"###);
    assert_output(&out, r###"ZlibDecompressorTest::test_decompressor_inputbuf_3: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/zlib/zlib_decompressor_test__test_eof_error.py`.
#[test]
fn test_gen_behavior_std_libs_zlib_zlib_decompressor_test__test_eof_error() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "behavior"
# case = "zlib_decompressor_test__test_eof_error"
# subject = "cpython.test_zlib.ZlibDecompressorTest.testEOFError"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_zlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_zlib.py::ZlibDecompressorTest::testEOFError
"""Auto-ported test: ZlibDecompressorTest::testEOFError (CPython 3.12 oracle)."""


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
TEXT = HAMLET_SCENE
DATA = zlib.compress(HAMLET_SCENE)
BAD_DATA = b'Not a valid deflate block'
BIG_TEXT = DATA * (128 * 1024 // len(DATA) + 1)
BIG_DATA = zlib.compress(BIG_TEXT)
zlibd = zlib._ZlibDecompressor()
text = zlibd.decompress(DATA)

try:
    zlibd.decompress(b'anything')
    raise AssertionError('expected EOFError')
except EOFError:
    pass

try:
    zlibd.decompress(b'')
    raise AssertionError('expected EOFError')
except EOFError:
    pass
print("ZlibDecompressorTest::testEOFError: ok")
"###);
    assert_output(&out, r###"ZlibDecompressorTest::testEOFError: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/zlib/zlib_decompressor_test__test_failure.py`.
#[test]
fn test_gen_behavior_std_libs_zlib_zlib_decompressor_test__test_failure() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "behavior"
# case = "zlib_decompressor_test__test_failure"
# subject = "cpython.test_zlib.ZlibDecompressorTest.test_failure"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_zlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_zlib.py::ZlibDecompressorTest::test_failure
"""Auto-ported test: ZlibDecompressorTest::test_failure (CPython 3.12 oracle)."""


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
TEXT = HAMLET_SCENE
DATA = zlib.compress(HAMLET_SCENE)
BAD_DATA = b'Not a valid deflate block'
BIG_TEXT = DATA * (128 * 1024 // len(DATA) + 1)
BIG_DATA = zlib.compress(BIG_TEXT)
zlibd = zlib._ZlibDecompressor()

try:
    zlibd.decompress(BAD_DATA * 30)
    raise AssertionError('expected Exception')
except Exception:
    pass

try:
    zlibd.decompress(BAD_DATA * 30)
    raise AssertionError('expected Exception')
except Exception:
    pass
print("ZlibDecompressorTest::test_failure: ok")
"###);
    assert_output(&out, r###"ZlibDecompressorTest::test_failure: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/zlib/zlib_decompressor_test__test_pickle.py`.
#[test]
fn test_gen_behavior_std_libs_zlib_zlib_decompressor_test__test_pickle() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "behavior"
# case = "zlib_decompressor_test__test_pickle"
# subject = "cpython.test_zlib.ZlibDecompressorTest.testPickle"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_zlib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_zlib.py::ZlibDecompressorTest::testPickle
"""Auto-ported test: ZlibDecompressorTest::testPickle (CPython 3.12 oracle)."""


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
TEXT = HAMLET_SCENE
DATA = zlib.compress(HAMLET_SCENE)
BAD_DATA = b'Not a valid deflate block'
BIG_TEXT = DATA * (128 * 1024 // len(DATA) + 1)
BIG_DATA = zlib.compress(BIG_TEXT)
for proto in range(pickle.HIGHEST_PROTOCOL + 1):
    try:
        pickle.dumps(zlib._ZlibDecompressor(), proto)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
print("ZlibDecompressorTest::testPickle: ok")
"###);
    assert_output(&out, r###"ZlibDecompressorTest::testPickle: ok
"###);
}
