# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "buffer"
# dimension = "behavior"
# case = "test_python_buffer_protocol__test_basic"
# subject = "cpython.test_buffer.TestPythonBufferProtocol.test_basic"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_buffer.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_buffer.py::TestPythonBufferProtocol::test_basic
"""Auto-ported test: TestPythonBufferProtocol::test_basic (CPython 3.12 oracle)."""


import contextlib
import unittest
from test import support
from test.support import os_helper
import inspect
from itertools import permutations, product
from random import randrange, sample, choice
import warnings
import sys, array, io, os
from decimal import Decimal
from fractions import Fraction


try:
    from _testbuffer import *
except ImportError:
    ndarray = None

try:
    import struct
except ImportError:
    struct = None

try:
    import ctypes
except ImportError:
    ctypes = None

try:
    with os_helper.EnvironmentVarGuard() as os.environ, warnings.catch_warnings():
        from numpy import ndarray as numpy_array
except ImportError:
    numpy_array = None

try:
    import _testcapi
except ImportError:
    _testcapi = None

SHORT_TEST = True

NATIVE = {'?': 0, 'c': 0, 'b': 0, 'B': 0, 'h': 0, 'H': 0, 'i': 0, 'I': 0, 'l': 0, 'L': 0, 'n': 0, 'N': 0, 'e': 0, 'f': 0, 'd': 0, 'P': 0}

if numpy_array:
    del NATIVE['n']
    del NATIVE['N']

if struct:
    try:
        struct.pack('Q', 2 ** 64 - 1)
        NATIVE['q'] = 0
        NATIVE['Q'] = 0
    except struct.error:
        pass

STANDARD = {'?': (0, 2), 'c': (0, 1 << 8), 'b': (-(1 << 7), 1 << 7), 'B': (0, 1 << 8), 'h': (-(1 << 15), 1 << 15), 'H': (0, 1 << 16), 'i': (-(1 << 31), 1 << 31), 'I': (0, 1 << 32), 'l': (-(1 << 31), 1 << 31), 'L': (0, 1 << 32), 'q': (-(1 << 63), 1 << 63), 'Q': (0, 1 << 64), 'e': (-65519, 65520), 'f': (-(1 << 63), 1 << 63), 'd': (-(1 << 1023), 1 << 1023)}

def native_type_range(fmt):
    """Return range of a native type."""
    if fmt == 'c':
        lh = (0, 256)
    elif fmt == '?':
        lh = (0, 2)
    elif fmt == 'e':
        lh = (-65519, 65520)
    elif fmt == 'f':
        lh = (-(1 << 63), 1 << 63)
    elif fmt == 'd':
        lh = (-(1 << 1023), 1 << 1023)
    else:
        for exp in (128, 127, 64, 63, 32, 31, 16, 15, 8, 7):
            try:
                struct.pack(fmt, (1 << exp) - 1)
                break
            except struct.error:
                pass
        lh = (-(1 << exp), 1 << exp) if exp & 1 else (0, 1 << exp)
    return lh

fmtdict = {'': NATIVE, '@': NATIVE, '<': STANDARD, '>': STANDARD, '=': STANDARD, '!': STANDARD}

if struct:
    for fmt in fmtdict['@']:
        fmtdict['@'][fmt] = native_type_range(fmt)

MEMORYVIEW = NATIVE.copy()

ARRAY = NATIVE.copy()

for k in NATIVE:
    if not k in 'bBhHiIlLfd':
        del ARRAY[k]

BYTEFMT = NATIVE.copy()

for k in NATIVE:
    if not k in 'Bbc':
        del BYTEFMT[k]

fmtdict['m'] = MEMORYVIEW

fmtdict['@m'] = MEMORYVIEW

fmtdict['a'] = ARRAY

fmtdict['b'] = BYTEFMT

fmtdict['@b'] = BYTEFMT

MODE = 0

MULT = 1

cap = {'ndarray': (['', '@', '<', '>', '=', '!'], ['', '1', '2', '3']), 'array': (['a'], ['']), 'numpy': ([''], ['']), 'memoryview': (['@m', 'm'], ['']), 'bytefmt': (['@b', 'b'], [''])}

def randrange_fmt(mode, char, obj):
    """Return random item for a type specified by a mode and a single
       format character."""
    x = randrange(*fmtdict[mode][char])
    if char == 'c':
        x = bytes([x])
        if obj == 'numpy' and x == b'\x00':
            x = b'\x01'
    if char == '?':
        x = bool(x)
    if char in 'efd':
        x = struct.pack(char, x)
        x = struct.unpack(char, x)[0]
    return x

def gen_item(fmt, obj):
    """Return single random item."""
    mode, chars = fmt.split('#')
    x = []
    for c in chars:
        x.append(randrange_fmt(mode, c, obj))
    return x[0] if len(x) == 1 else tuple(x)

def gen_items(n, fmt, obj):
    """Return a list of random items (or a scalar)."""
    if n == 0:
        return gen_item(fmt, obj)
    lst = [0] * n
    for i in range(n):
        lst[i] = gen_item(fmt, obj)
    return lst

def struct_items(n, obj):
    mode = choice(cap[obj][MODE])
    xfmt = mode + '#'
    fmt = mode.strip('amb')
    nmemb = randrange(2, 10)
    for _ in range(nmemb):
        char = choice(tuple(fmtdict[mode]))
        multiplier = choice(cap[obj][MULT])
        xfmt += char * int(multiplier if multiplier else 1)
        fmt += multiplier + char
    items = gen_items(n, xfmt, obj)
    item = gen_item(xfmt, obj)
    return (fmt, items, item)

def randitems(n, obj='ndarray', mode=None, char=None):
    """Return random format, items, item."""
    if mode is None:
        mode = choice(cap[obj][MODE])
    if char is None:
        char = choice(tuple(fmtdict[mode]))
    multiplier = choice(cap[obj][MULT])
    fmt = mode + '#' + char * int(multiplier if multiplier else 1)
    items = gen_items(n, fmt, obj)
    item = gen_item(fmt, obj)
    fmt = mode.strip('amb') + multiplier + char
    return (fmt, items, item)

def iter_mode(n, obj='ndarray'):
    """Iterate through supported mode/char combinations."""
    for mode in cap[obj][MODE]:
        for char in fmtdict[mode]:
            yield randitems(n, obj, mode, char)

def iter_format(nitems, testobj='ndarray'):
    """Yield (format, items, item) for all possible modes and format
       characters plus one random compound format string."""
    for t in iter_mode(nitems, testobj):
        yield t
    if testobj != 'ndarray':
        return
    yield struct_items(nitems, testobj)

def is_byte_format(fmt):
    return 'c' in fmt or 'b' in fmt or 'B' in fmt

def is_memoryview_format(fmt):
    """format suitable for memoryview"""
    x = len(fmt)
    return (x == 1 or (x == 2 and fmt[0] == '@')) and fmt[x - 1] in MEMORYVIEW

NON_BYTE_FORMAT = [c for c in fmtdict['@'] if not is_byte_format(c)]

def atomp(lst):
    """Tuple items (representing structs) are regarded as atoms."""
    return not isinstance(lst, list)

def listp(lst):
    return isinstance(lst, list)

def prod(lst):
    """Product of list elements."""
    if len(lst) == 0:
        return 0
    x = lst[0]
    for v in lst[1:]:
        x *= v
    return x

def strides_from_shape(ndim, shape, itemsize, layout):
    """Calculate strides of a contiguous array. Layout is 'C' or
       'F' (Fortran)."""
    if ndim == 0:
        return ()
    if layout == 'C':
        strides = list(shape[1:]) + [itemsize]
        for i in range(ndim - 2, -1, -1):
            strides[i] *= strides[i + 1]
    else:
        strides = [itemsize] + list(shape[:-1])
        for i in range(1, ndim):
            strides[i] *= strides[i - 1]
    return strides

def _ca(items, s):
    """Convert flat item list to the nested list representation of a
       multidimensional C array with shape 's'."""
    if atomp(items):
        return items
    if len(s) == 0:
        return items[0]
    lst = [0] * s[0]
    stride = len(items) // s[0] if s[0] else 0
    for i in range(s[0]):
        start = i * stride
        lst[i] = _ca(items[start:start + stride], s[1:])
    return lst

def _fa(items, s):
    """Convert flat item list to the nested list representation of a
       multidimensional Fortran array with shape 's'."""
    if atomp(items):
        return items
    if len(s) == 0:
        return items[0]
    lst = [0] * s[0]
    stride = s[0]
    for i in range(s[0]):
        lst[i] = _fa(items[i::stride], s[1:])
    return lst

def carray(items, shape):
    if listp(items) and (not 0 in shape) and (prod(shape) != len(items)):
        raise ValueError('prod(shape) != len(items)')
    return _ca(items, shape)

def farray(items, shape):
    if listp(items) and (not 0 in shape) and (prod(shape) != len(items)):
        raise ValueError('prod(shape) != len(items)')
    return _fa(items, shape)

def indices(shape):
    """Generate all possible tuples of indices."""
    iterables = [range(v) for v in shape]
    return product(*iterables)

def getindex(ndim, ind, strides):
    """Convert multi-dimensional index to the position in the flat list."""
    ret = 0
    for i in range(ndim):
        ret += strides[i] * ind[i]
    return ret

def transpose(src, shape):
    """Transpose flat item list that is regarded as a multi-dimensional
       matrix defined by shape: dest...[k][j][i] = src[i][j][k]...  """
    if not shape:
        return src
    ndim = len(shape)
    sstrides = strides_from_shape(ndim, shape, 1, 'C')
    dstrides = strides_from_shape(ndim, shape[::-1], 1, 'C')
    dest = [0] * len(src)
    for ind in indices(shape):
        fr = getindex(ndim, ind, sstrides)
        to = getindex(ndim, ind[::-1], dstrides)
        dest[to] = src[fr]
    return dest

def _flatten(lst):
    """flatten list"""
    if lst == []:
        return lst
    if atomp(lst):
        return [lst]
    return _flatten(lst[0]) + _flatten(lst[1:])

def flatten(lst):
    """flatten list or return scalar"""
    if atomp(lst):
        return lst
    return _flatten(lst)

def slice_shape(lst, slices):
    """Get the shape of lst after slicing: slices is a list of slice
       objects."""
    if atomp(lst):
        return []
    return [len(lst[slices[0]])] + slice_shape(lst[0], slices[1:])

def multislice(lst, slices):
    """Multi-dimensional slicing: slices is a list of slice objects."""
    if atomp(lst):
        return lst
    return [multislice(sublst, slices[1:]) for sublst in lst[slices[0]]]

def m_assign(llst, rlst, lslices, rslices):
    """Multi-dimensional slice assignment: llst and rlst are the operands,
       lslices and rslices are lists of slice objects. llst and rlst must
       have the same structure.

       For a two-dimensional example, this is not implemented in Python:

         llst[0:3:2, 0:3:2] = rlst[1:3:1, 1:3:1]

       Instead we write:

         lslices = [slice(0,3,2), slice(0,3,2)]
         rslices = [slice(1,3,1), slice(1,3,1)]
         multislice_assign(llst, rlst, lslices, rslices)
    """
    if atomp(rlst):
        return rlst
    rlst = [m_assign(l, r, lslices[1:], rslices[1:]) for l, r in zip(llst[lslices[0]], rlst[rslices[0]])]
    llst[lslices[0]] = rlst
    return llst

def cmp_structure(llst, rlst, lslices, rslices):
    """Compare the structure of llst[lslices] and rlst[rslices]."""
    lshape = slice_shape(llst, lslices)
    rshape = slice_shape(rlst, rslices)
    if len(lshape) != len(rshape):
        return -1
    for i in range(len(lshape)):
        if lshape[i] != rshape[i]:
            return -1
        if lshape[i] == 0:
            return 0
    return 0

def multislice_assign(llst, rlst, lslices, rslices):
    """Return llst after assigning: llst[lslices] = rlst[rslices]"""
    if cmp_structure(llst, rlst, lslices, rslices) < 0:
        raise ValueError('lvalue and rvalue have different structures')
    return m_assign(llst, rlst, lslices, rslices)

def verify_structure(memlen, itemsize, ndim, shape, strides, offset):
    """Verify that the parameters represent a valid array within
       the bounds of the allocated memory:
           char *mem: start of the physical memory block
           memlen: length of the physical memory block
           offset: (char *)buf - mem
    """
    if offset % itemsize:
        return False
    if offset < 0 or offset + itemsize > memlen:
        return False
    if any((v % itemsize for v in strides)):
        return False
    if ndim <= 0:
        return ndim == 0 and (not shape) and (not strides)
    if 0 in shape:
        return True
    imin = sum((strides[j] * (shape[j] - 1) for j in range(ndim) if strides[j] <= 0))
    imax = sum((strides[j] * (shape[j] - 1) for j in range(ndim) if strides[j] > 0))
    return 0 <= offset + imin and offset + imax + itemsize <= memlen

def get_item(lst, indices):
    for i in indices:
        lst = lst[i]
    return lst

def memory_index(indices, t):
    """Location of an item in the underlying memory."""
    memlen, itemsize, ndim, shape, strides, offset = t
    p = offset
    for i in range(ndim):
        p += strides[i] * indices[i]
    return p

def is_overlapping(t):
    """The structure 't' is overlapping if at least one memory location
       is visited twice while iterating through all possible tuples of
       indices."""
    memlen, itemsize, ndim, shape, strides, offset = t
    visited = 1 << memlen
    for ind in indices(shape):
        i = memory_index(ind, t)
        bit = 1 << i
        if visited & bit:
            return True
        visited |= bit
    return False

def rand_structure(itemsize, valid, maxdim=5, maxshape=16, shape=()):
    """Return random structure:
           (memlen, itemsize, ndim, shape, strides, offset)
       If 'valid' is true, the returned structure is valid, otherwise invalid.
       If 'shape' is given, use that instead of creating a random shape.
    """
    if not shape:
        ndim = randrange(maxdim + 1)
        if ndim == 0:
            if valid:
                return (itemsize, itemsize, ndim, (), (), 0)
            else:
                nitems = randrange(1, 16 + 1)
                memlen = nitems * itemsize
                offset = -itemsize if randrange(2) == 0 else memlen
                return (memlen, itemsize, ndim, (), (), offset)
        minshape = 2
        n = randrange(100)
        if n >= 95 and valid:
            minshape = 0
        elif n >= 90:
            minshape = 1
        shape = [0] * ndim
        for i in range(ndim):
            shape[i] = randrange(minshape, maxshape + 1)
    else:
        ndim = len(shape)
    maxstride = 5
    n = randrange(100)
    zero_stride = True if n >= 95 and n & 1 else False
    strides = [0] * ndim
    strides[ndim - 1] = itemsize * randrange(-maxstride, maxstride + 1)
    if not zero_stride and strides[ndim - 1] == 0:
        strides[ndim - 1] = itemsize
    for i in range(ndim - 2, -1, -1):
        maxstride *= shape[i + 1] if shape[i + 1] else 1
        if zero_stride:
            strides[i] = itemsize * randrange(-maxstride, maxstride + 1)
        else:
            strides[i] = (1, -1)[randrange(2)] * itemsize * randrange(1, maxstride + 1)
    imin = imax = 0
    if not 0 in shape:
        imin = sum((strides[j] * (shape[j] - 1) for j in range(ndim) if strides[j] <= 0))
        imax = sum((strides[j] * (shape[j] - 1) for j in range(ndim) if strides[j] > 0))
    nitems = imax - imin
    if valid:
        offset = -imin * itemsize
        memlen = offset + (imax + 1) * itemsize
    else:
        memlen = (-imin + imax) * itemsize
        offset = -imin - itemsize if randrange(2) == 0 else memlen
    return (memlen, itemsize, ndim, shape, strides, offset)

def randslice_from_slicelen(slicelen, listlen):
    """Create a random slice of len slicelen that fits into listlen."""
    maxstart = listlen - slicelen
    start = randrange(maxstart + 1)
    maxstep = (listlen - start) // slicelen if slicelen else 1
    step = randrange(1, maxstep + 1)
    stop = start + slicelen * step
    s = slice(start, stop, step)
    _, _, _, control = slice_indices(s, listlen)
    if control != slicelen:
        raise RuntimeError
    return s

def randslice_from_shape(ndim, shape):
    """Create two sets of slices for an array x with shape 'shape'
       such that shapeof(x[lslices]) == shapeof(x[rslices])."""
    lslices = [0] * ndim
    rslices = [0] * ndim
    for n in range(ndim):
        l = shape[n]
        slicelen = randrange(1, l + 1) if l > 0 else 0
        lslices[n] = randslice_from_slicelen(slicelen, l)
        rslices[n] = randslice_from_slicelen(slicelen, l)
    return (tuple(lslices), tuple(rslices))

def rand_aligned_slices(maxdim=5, maxshape=16):
    """Create (lshape, rshape, tuple(lslices), tuple(rslices)) such that
       shapeof(x[lslices]) == shapeof(y[rslices]), where x is an array
       with shape 'lshape' and y is an array with shape 'rshape'."""
    ndim = randrange(1, maxdim + 1)
    minshape = 2
    n = randrange(100)
    if n >= 95:
        minshape = 0
    elif n >= 90:
        minshape = 1
    all_random = True if randrange(100) >= 80 else False
    lshape = [0] * ndim
    rshape = [0] * ndim
    lslices = [0] * ndim
    rslices = [0] * ndim
    for n in range(ndim):
        small = randrange(minshape, maxshape + 1)
        big = randrange(minshape, maxshape + 1)
        if big < small:
            big, small = (small, big)
        if all_random:
            start = randrange(-small, small + 1)
            stop = randrange(-small, small + 1)
            step = (1, -1)[randrange(2)] * randrange(1, small + 2)
            s_small = slice(start, stop, step)
            _, _, _, slicelen = slice_indices(s_small, small)
        else:
            slicelen = randrange(1, small + 1) if small > 0 else 0
            s_small = randslice_from_slicelen(slicelen, small)
        s_big = randslice_from_slicelen(slicelen, big)
        if randrange(2) == 0:
            rshape[n], lshape[n] = (big, small)
            rslices[n], lslices[n] = (s_big, s_small)
        else:
            rshape[n], lshape[n] = (small, big)
            rslices[n], lslices[n] = (s_small, s_big)
    return (lshape, rshape, tuple(lslices), tuple(rslices))

def randitems_from_structure(fmt, t):
    """Return a list of random items for structure 't' with format
       'fmtchar'."""
    memlen, itemsize, _, _, _, _ = t
    return gen_items(memlen // itemsize, '#' + fmt, 'numpy')

def ndarray_from_structure(items, fmt, t, flags=0):
    """Return ndarray from the tuple returned by rand_structure()"""
    memlen, itemsize, ndim, shape, strides, offset = t
    return ndarray(items, shape=shape, strides=strides, format=fmt, offset=offset, flags=ND_WRITABLE | flags)

def numpy_array_from_structure(items, fmt, t):
    """Return numpy_array from the tuple returned by rand_structure()"""
    memlen, itemsize, ndim, shape, strides, offset = t
    buf = bytearray(memlen)
    for j, v in enumerate(items):
        struct.pack_into(fmt, buf, j * itemsize, v)
    return numpy_array(buffer=buf, shape=shape, strides=strides, dtype=fmt, offset=offset)

def cast_items(exporter, fmt, itemsize, shape=None):
    """Interpret the raw memory of 'exporter' as a list of items with
       size 'itemsize'. If shape=None, the new structure is assumed to
       be 1-D with n * itemsize = bytelen. If shape is given, the usual
       constraint for contiguous arrays prod(shape) * itemsize = bytelen
       applies. On success, return (items, shape). If the constraints
       cannot be met, return (None, None). If a chunk of bytes is interpreted
       as NaN as a result of float conversion, return ('nan', None)."""
    bytelen = exporter.nbytes
    if shape:
        if prod(shape) * itemsize != bytelen:
            return (None, shape)
    elif shape == []:
        if exporter.ndim == 0 or itemsize != bytelen:
            return (None, shape)
    else:
        n, r = divmod(bytelen, itemsize)
        shape = [n]
        if r != 0:
            return (None, shape)
    mem = exporter.tobytes()
    byteitems = [mem[i:i + itemsize] for i in range(0, len(mem), itemsize)]
    items = []
    for v in byteitems:
        item = struct.unpack(fmt, v)[0]
        if item != item:
            return ('nan', shape)
        items.append(item)
    return (items, shape) if shape != [] else (items[0], shape)

def gencastshapes():
    """Generate shapes to test casting."""
    for n in range(32):
        yield [n]
    ndim = randrange(4, 6)
    minshape = 1 if randrange(100) > 80 else 2
    yield [randrange(minshape, 5) for _ in range(ndim)]
    ndim = randrange(2, 4)
    minshape = 1 if randrange(100) > 80 else 2
    yield [randrange(minshape, 5) for _ in range(ndim)]

def genslices(n):
    """Generate all possible slices for a single dimension."""
    return product(range(-n, n + 1), range(-n, n + 1), range(-n, n + 1))

def genslices_ndim(ndim, shape):
    """Generate all possible slice tuples for 'shape'."""
    iterables = [genslices(shape[n]) for n in range(ndim)]
    return product(*iterables)

def rslice(n, allow_empty=False):
    """Generate random slice for a single dimension of length n.
       If zero=True, the slices may be empty, otherwise they will
       be non-empty."""
    minlen = 0 if allow_empty or n == 0 else 1
    slicelen = randrange(minlen, n + 1)
    return randslice_from_slicelen(slicelen, n)

def rslices(n, allow_empty=False):
    """Generate random slices for a single dimension."""
    for _ in range(5):
        yield rslice(n, allow_empty)

def rslices_ndim(ndim, shape, iterations=5):
    """Generate random slice tuples for 'shape'."""
    for _ in range(iterations):
        yield tuple((rslice(shape[n]) for n in range(ndim)))
    for _ in range(iterations):
        yield tuple((rslice(shape[n], allow_empty=True) for n in range(ndim)))
    yield tuple((slice(0, 1, 0) for _ in range(ndim)))

def rpermutation(iterable, r=None):
    pool = tuple(iterable)
    r = len(pool) if r is None else r
    yield tuple(sample(pool, r))

def ndarray_print(nd):
    """Print ndarray for debugging."""
    try:
        x = nd.tolist()
    except (TypeError, NotImplementedError):
        x = nd.tobytes()
    if isinstance(nd, ndarray):
        offset = nd.offset
        flags = nd.flags
    else:
        offset = 'unknown'
        flags = 'unknown'
    print("ndarray(%s, shape=%s, strides=%s, suboffsets=%s, offset=%s, format='%s', itemsize=%s, flags=%s)" % (x, nd.shape, nd.strides, nd.suboffsets, offset, nd.format, nd.itemsize, flags))
    sys.stdout.flush()

ITERATIONS = 100

MAXDIM = 5

MAXSHAPE = 10

if SHORT_TEST:
    ITERATIONS = 10
    MAXDIM = 3
    MAXSHAPE = 4
    genslices = rslices
    genslices_ndim = rslices_ndim
    permutations = rpermutation


# --- test body ---
class MyBuffer:

    def __buffer__(self, flags):
        return memoryview(b'hello')
mv = memoryview(MyBuffer())

assert mv.tobytes() == b'hello'

assert bytes(MyBuffer()) == b'hello'
print("TestPythonBufferProtocol::test_basic: ok")
