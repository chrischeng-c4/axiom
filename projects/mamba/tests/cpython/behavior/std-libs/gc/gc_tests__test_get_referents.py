# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gc"
# dimension = "behavior"
# case = "gc_tests__test_get_referents"
# subject = "cpython.test_gc.GCTests.test_get_referents"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_gc.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_gc.py::GCTests::test_get_referents
"""Auto-ported test: GCTests::test_get_referents (CPython 3.12 oracle)."""


import unittest
import unittest.mock
from test.support import verbose, refcount_test, cpython_only, requires_subprocess
from test.support.import_helper import import_module
from test.support.os_helper import temp_dir, TESTFN, unlink
from test.support.script_helper import assert_python_ok, make_script
from test.support import threading_helper
import gc
import sys
import sysconfig
import textwrap
import threading
import time
import weakref


try:
    from _testcapi import with_tp_del
except ImportError:

    def with_tp_del(cls):

        class C(object):

            def __new__(cls, *args, **kwargs):
                raise TypeError('requires _testcapi.with_tp_del')
        return C

try:
    from _testcapi import ContainerNoGC
except ImportError:
    ContainerNoGC = None

class C1055820(object):

    def __init__(self, i):
        self.i = i
        self.loop = self

class GC_Detector(object):

    def __init__(self):
        self.gc_happened = False

        def it_happened(ignored):
            self.gc_happened = True
        self.wr = weakref.ref(C1055820(666), it_happened)

@with_tp_del
class Uncollectable(object):
    """Create a reference cycle with multiple __del__ methods.

    An object in a reference cycle will never have zero references,
    and so must be garbage collected.  If one or more objects in the
    cycle have __del__ methods, the gc refuses to guess an order,
    and leaves the cycle uncollected."""

    def __init__(self, partner=None):
        if partner is None:
            self.partner = Uncollectable(partner=self)
        else:
            self.partner = partner

    def __tp_del__(self):
        pass

if sysconfig.get_config_vars().get('PY_CFLAGS', ''):
    BUILD_WITH_NDEBUG = '-DNDEBUG' in sysconfig.get_config_vars()['PY_CFLAGS']
else:
    BUILD_WITH_NDEBUG = not hasattr(sys, 'gettotalrefcount')

def setUpModule():
    global enabled, debug
    enabled = gc.isenabled()
    gc.disable()
    assert not gc.isenabled()
    debug = gc.get_debug()
    gc.set_debug(debug & ~gc.DEBUG_LEAK)
    gc.collect()

def tearDownModule():
    gc.set_debug(debug)
    if verbose:
        print('restoring automatic collection')
    gc.enable()
    assert gc.isenabled()
    if not enabled:
        gc.disable()


# --- test body ---
alist = [1, 3, 5]
got = gc.get_referents(alist)
got.sort()

assert got == alist
atuple = tuple(alist)
got = gc.get_referents(atuple)
got.sort()

assert got == alist
adict = {1: 3, 5: 7}
expected = [1, 3, 5, 7]
got = gc.get_referents(adict)
got.sort()

assert got == expected
got = gc.get_referents([1, 2], {3: 4}, (0, 0, 0))
got.sort()

assert got == [0, 0] + list(range(5))

assert gc.get_referents(1, 'a', 4j) == []
print("GCTests::test_get_referents: ok")
