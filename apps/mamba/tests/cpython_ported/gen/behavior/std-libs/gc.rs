use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/gc/gc_tests__test_bug21435.py`.
#[test]
fn test_gen_behavior_std_libs_gc_gc_tests__test_bug21435() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gc"
# dimension = "behavior"
# case = "gc_tests__test_bug21435"
# subject = "cpython.test_gc.GCTests.test_bug21435"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_gc.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_gc.py::GCTests::test_bug21435
"""Auto-ported test: GCTests::test_bug21435 (CPython 3.12 oracle)."""


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
gc.collect()

class A:
    pass

class B:

    def __init__(self, x):
        self.x = x

    def __del__(self):
        self.attr = None

def do_work():
    a = A()
    b = B(A())
    a.attr = b
    b.attr = a
do_work()
gc.collect()
print("GCTests::test_bug21435: ok")
"###);
    assert_output(&out, r###"GCTests::test_bug21435: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/gc/gc_tests__test_collect_generations.py`.
#[test]
fn test_gen_behavior_std_libs_gc_gc_tests__test_collect_generations() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gc"
# dimension = "behavior"
# case = "gc_tests__test_collect_generations"
# subject = "cpython.test_gc.GCTests.test_collect_generations"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_gc.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_gc.py::GCTests::test_collect_generations
"""Auto-ported test: GCTests::test_collect_generations (CPython 3.12 oracle)."""


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
gc.collect()
x = []
gc.collect(0)
a, b, c = gc.get_count()
gc.collect(1)
d, e, f = gc.get_count()
gc.collect(2)
g, h, i = gc.get_count()

assert (b, c) == (1, 0)

assert (e, f) == (0, 1)

assert (h, i) == (0, 0)
print("GCTests::test_collect_generations: ok")
"###);
    assert_output(&out, r###"GCTests::test_collect_generations: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/gc/gc_tests__test_del.py`.
#[test]
fn test_gen_behavior_std_libs_gc_gc_tests__test_del() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gc"
# dimension = "behavior"
# case = "gc_tests__test_del"
# subject = "cpython.test_gc.GCTests.test_del"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_gc.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_gc.py::GCTests::test_del
"""Auto-ported test: GCTests::test_del (CPython 3.12 oracle)."""


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
thresholds = gc.get_threshold()
gc.enable()
gc.set_threshold(1)

class A:

    def __del__(self):
        dir(self)
a = A()
del a
gc.disable()
gc.set_threshold(*thresholds)
print("GCTests::test_del: ok")
"###);
    assert_output(&out, r###"GCTests::test_del: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/gc/gc_tests__test_del_newclass.py`.
#[test]
fn test_gen_behavior_std_libs_gc_gc_tests__test_del_newclass() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gc"
# dimension = "behavior"
# case = "gc_tests__test_del_newclass"
# subject = "cpython.test_gc.GCTests.test_del_newclass"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_gc.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_gc.py::GCTests::test_del_newclass
"""Auto-ported test: GCTests::test_del_newclass (CPython 3.12 oracle)."""


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
thresholds = gc.get_threshold()
gc.enable()
gc.set_threshold(1)

class A(object):

    def __del__(self):
        dir(self)
a = A()
del a
gc.disable()
gc.set_threshold(*thresholds)
print("GCTests::test_del_newclass: ok")
"###);
    assert_output(&out, r###"GCTests::test_del_newclass: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/gc/gc_tests__test_freeze.py`.
#[test]
fn test_gen_behavior_std_libs_gc_gc_tests__test_freeze() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gc"
# dimension = "behavior"
# case = "gc_tests__test_freeze"
# subject = "cpython.test_gc.GCTests.test_freeze"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_gc.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_gc.py::GCTests::test_freeze
"""Auto-ported test: GCTests::test_freeze (CPython 3.12 oracle)."""


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
gc.freeze()

assert gc.get_freeze_count() > 0
gc.unfreeze()

assert gc.get_freeze_count() == 0
print("GCTests::test_freeze: ok")
"###);
    assert_output(&out, r###"GCTests::test_freeze: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/gc/gc_tests__test_get_count.py`.
#[test]
fn test_gen_behavior_std_libs_gc_gc_tests__test_get_count() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gc"
# dimension = "behavior"
# case = "gc_tests__test_get_count"
# subject = "cpython.test_gc.GCTests.test_get_count"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_gc.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_gc.py::GCTests::test_get_count
"""Auto-ported test: GCTests::test_get_count (CPython 3.12 oracle)."""


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
gc.collect()
a, b, c = gc.get_count()
x = []
d, e, f = gc.get_count()

assert (b, c) == (0, 0)

assert (e, f) == (0, 0)

assert a < 5

assert d > a
print("GCTests::test_get_count: ok")
"###);
    assert_output(&out, r###"GCTests::test_get_count: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/gc/gc_tests__test_get_referents.py`.
#[test]
fn test_gen_behavior_std_libs_gc_gc_tests__test_get_referents() {
    let out = jit_capture(r###"# /// script
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
"###);
    assert_output(&out, r###"GCTests::test_get_referents: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/gc/gc_tests__test_get_stats.py`.
#[test]
fn test_gen_behavior_std_libs_gc_gc_tests__test_get_stats() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gc"
# dimension = "behavior"
# case = "gc_tests__test_get_stats"
# subject = "cpython.test_gc.GCTests.test_get_stats"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_gc.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_gc.py::GCTests::test_get_stats
"""Auto-ported test: GCTests::test_get_stats (CPython 3.12 oracle)."""


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
stats = gc.get_stats()

assert len(stats) == 3
for st in stats:

    assert isinstance(st, dict)

    assert set(st) == {'collected', 'collections', 'uncollectable'}

    assert st['collected'] >= 0

    assert st['collections'] >= 0

    assert st['uncollectable'] >= 0
if gc.isenabled():
    pass
    gc.disable()
old = gc.get_stats()
gc.collect(0)
new = gc.get_stats()

assert new[0]['collections'] == old[0]['collections'] + 1

assert new[1]['collections'] == old[1]['collections']

assert new[2]['collections'] == old[2]['collections']
gc.collect(2)
new = gc.get_stats()

assert new[0]['collections'] == old[0]['collections'] + 1

assert new[1]['collections'] == old[1]['collections']

assert new[2]['collections'] == old[2]['collections'] + 1
print("GCTests::test_get_stats: ok")
"###);
    assert_output(&out, r###"GCTests::test_get_stats: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/gc/gc_tests__test_is_tracked.py`.
#[test]
fn test_gen_behavior_std_libs_gc_gc_tests__test_is_tracked() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gc"
# dimension = "behavior"
# case = "gc_tests__test_is_tracked"
# subject = "cpython.test_gc.GCTests.test_is_tracked"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_gc.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_gc.py::GCTests::test_is_tracked
"""Auto-ported test: GCTests::test_is_tracked (CPython 3.12 oracle)."""


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

assert not gc.is_tracked(None)

assert not gc.is_tracked(1)

assert not gc.is_tracked(1.0)

assert not gc.is_tracked(1.0 + 5j)

assert not gc.is_tracked(True)

assert not gc.is_tracked(False)

assert not gc.is_tracked(b'a')

assert not gc.is_tracked('a')

assert not gc.is_tracked(bytearray(b'a'))

assert not gc.is_tracked(type)

assert not gc.is_tracked(int)

assert not gc.is_tracked(object)

assert not gc.is_tracked(object())

class UserClass:
    pass

class UserInt(int):
    pass

class UserClassSlots:
    __slots__ = ()

class UserFloatSlots(float):
    __slots__ = ()

class UserIntSlots(int):
    __slots__ = ()

assert gc.is_tracked(gc)

assert gc.is_tracked(UserClass)

assert gc.is_tracked(UserClass())

assert gc.is_tracked(UserInt())

assert gc.is_tracked([])

assert gc.is_tracked(set())

assert gc.is_tracked(UserClassSlots())

assert gc.is_tracked(UserFloatSlots())

assert gc.is_tracked(UserIntSlots())
print("GCTests::test_is_tracked: ok")
"###);
    assert_output(&out, r###"GCTests::test_is_tracked: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/gc/gc_tests__test_trashcan.py`.
#[test]
fn test_gen_behavior_std_libs_gc_gc_tests__test_trashcan() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gc"
# dimension = "behavior"
# case = "gc_tests__test_trashcan"
# subject = "cpython.test_gc.GCTests.test_trashcan"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_gc.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_gc.py::GCTests::test_trashcan
"""Auto-ported test: GCTests::test_trashcan (CPython 3.12 oracle)."""


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
class Ouch:
    n = 0

    def __del__(self):
        Ouch.n = Ouch.n + 1
        if Ouch.n % 17 == 0:
            gc.collect()
gc.enable()
N = 150
for count in range(2):
    t = []
    for i in range(N):
        t = [t, Ouch()]
    u = []
    for i in range(N):
        u = [u, Ouch()]
    v = {}
    for i in range(N):
        v = {1: v, 2: Ouch()}
gc.disable()
print("GCTests::test_trashcan: ok")
"###);
    assert_output(&out, r###"GCTests::test_trashcan: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/gc/gc_tests__test_trashcan_threads.py`.
#[test]
fn test_gen_behavior_std_libs_gc_gc_tests__test_trashcan_threads() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gc"
# dimension = "behavior"
# case = "gc_tests__test_trashcan_threads"
# subject = "cpython.test_gc.GCTests.test_trashcan_threads"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_gc.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_gc.py::GCTests::test_trashcan_threads
"""Auto-ported test: GCTests::test_trashcan_threads (CPython 3.12 oracle)."""


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
NESTING = 60
N_THREADS = 2

def sleeper_gen():
    """A generator that releases the GIL when closed or dealloc'ed."""
    try:
        yield
    finally:
        time.sleep(1e-06)

class C(list):
    inits = []
    dels = []

    def __init__(self, alist):
        self[:] = alist
        C.inits.append(None)

    def __del__(self):
        C.dels.append(None)
        g = sleeper_gen()
        next(g)

def make_nested():
    """Create a sufficiently nested container object so that the
            trashcan mechanism is invoked when deallocating it."""
    x = C([])
    for i in range(NESTING):
        x = [C([x])]
    del x

def run_thread():
    """Exercise make_nested() in a loop."""
    while not exit:
        make_nested()
old_switchinterval = sys.getswitchinterval()
sys.setswitchinterval(1e-05)
try:
    exit = []
    threads = []
    for i in range(N_THREADS):
        t = threading.Thread(target=run_thread)
        threads.append(t)
    with threading_helper.start_threads(threads, lambda: exit.append(1)):
        time.sleep(1.0)
finally:
    sys.setswitchinterval(old_switchinterval)
gc.collect()

assert len(C.inits) == len(C.dels)
print("GCTests::test_trashcan_threads: ok")
"###);
    assert_output(&out, r###"GCTests::test_trashcan_threads: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/gc/python_finalization_tests__test_ast_fini.py`.
#[test]
fn test_gen_behavior_std_libs_gc_python_finalization_tests__test_ast_fini() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gc"
# dimension = "behavior"
# case = "python_finalization_tests__test_ast_fini"
# subject = "cpython.test_gc.PythonFinalizationTests.test_ast_fini"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_gc.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_gc.py::PythonFinalizationTests::test_ast_fini
"""Auto-ported test: PythonFinalizationTests::test_ast_fini (CPython 3.12 oracle)."""


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
code = textwrap.dedent('\n            import ast\n            import codecs\n            from test import support\n\n            # Small AST tree to keep their AST types alive\n            tree = ast.parse("def f(x, y): return 2*x-y")\n\n            # Store the tree somewhere to survive until the last GC collection\n            support.late_deletion(tree)\n        ')
assert_python_ok('-c', code)
print("PythonFinalizationTests::test_ast_fini: ok")
"###);
    assert_output(&out, r###"PythonFinalizationTests::test_ast_fini: ok
"###);
}
