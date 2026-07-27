use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/resource/resource_test__test_args.py`.
#[test]
fn test_gen_behavior_std_libs_resource_resource_test__test_args() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "resource"
# dimension = "behavior"
# case = "resource_test__test_args"
# subject = "cpython.test_resource.ResourceTest.test_args"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_resource.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_resource.py::ResourceTest::test_args
"""Auto-ported test: ResourceTest::test_args (CPython 3.12 oracle)."""


import contextlib
import sys
import unittest
from test import support
from test.support import import_helper
from test.support import os_helper
import time


resource = import_helper.import_module('resource')


# --- test body ---

try:
    resource.getrlimit()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    resource.getrlimit(42, 42)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    resource.setrlimit()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    resource.setrlimit(42, 42, 42)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("ResourceTest::test_args: ok")
"###);
    assert_output(&out, r###"ResourceTest::test_args: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/resource/resource_test__test_freebsd_contants.py`.
#[test]
fn test_gen_behavior_std_libs_resource_resource_test__test_freebsd_contants() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "resource"
# dimension = "behavior"
# case = "resource_test__test_freebsd_contants"
# subject = "cpython.test_resource.ResourceTest.test_freebsd_contants"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_resource.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_resource.py::ResourceTest::test_freebsd_contants
"""Auto-ported test: ResourceTest::test_freebsd_contants (CPython 3.12 oracle)."""


import contextlib
import sys
import unittest
from test import support
from test.support import import_helper
from test.support import os_helper
import time


resource = import_helper.import_module('resource')


# --- test body ---
for attr in ['SWAP', 'SBSIZE', 'NPTS']:
    with contextlib.suppress(AttributeError):

        assert isinstance(getattr(resource, 'RLIMIT_' + attr), int)
print("ResourceTest::test_freebsd_contants: ok")
"###);
    assert_output(&out, r###"ResourceTest::test_freebsd_contants: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/resource/resource_test__test_fsize_enforced.py`.
#[test]
fn test_gen_behavior_std_libs_resource_resource_test__test_fsize_enforced() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "resource"
# dimension = "behavior"
# case = "resource_test__test_fsize_enforced"
# subject = "cpython.test_resource.ResourceTest.test_fsize_enforced"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_resource.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_resource.py::ResourceTest::test_fsize_enforced
"""Auto-ported test: ResourceTest::test_fsize_enforced (CPython 3.12 oracle)."""


import contextlib
import sys
import unittest
from test import support
from test.support import import_helper
from test.support import os_helper
import time


resource = import_helper.import_module('resource')


# --- test body ---
try:
    cur, max = resource.getrlimit(resource.RLIMIT_FSIZE)
except AttributeError:
    pass
else:
    try:
        try:
            resource.setrlimit(resource.RLIMIT_FSIZE, (1024, max))
            limit_set = True
        except ValueError:
            limit_set = False
        f = open(os_helper.TESTFN, 'wb')
        try:
            f.write(b'X' * 1024)
            try:
                f.write(b'Y')
                f.flush()
                for i in range(5):
                    time.sleep(0.1)
                    f.flush()
            except OSError:
                if not limit_set:
                    raise
            if limit_set:
                resource.setrlimit(resource.RLIMIT_FSIZE, (cur, max))
        finally:
            f.close()
    finally:
        if limit_set:
            resource.setrlimit(resource.RLIMIT_FSIZE, (cur, max))
        os_helper.unlink(os_helper.TESTFN)
print("ResourceTest::test_fsize_enforced: ok")
"###);
    assert_output(&out, r###"ResourceTest::test_fsize_enforced: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/resource/resource_test__test_fsize_toobig.py`.
#[test]
fn test_gen_behavior_std_libs_resource_resource_test__test_fsize_toobig() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "resource"
# dimension = "behavior"
# case = "resource_test__test_fsize_toobig"
# subject = "cpython.test_resource.ResourceTest.test_fsize_toobig"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_resource.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_resource.py::ResourceTest::test_fsize_toobig
"""Auto-ported test: ResourceTest::test_fsize_toobig (CPython 3.12 oracle)."""


import contextlib
import sys
import unittest
from test import support
from test.support import import_helper
from test.support import os_helper
import time


resource = import_helper.import_module('resource')


# --- test body ---
too_big = 10 ** 50
try:
    cur, max = resource.getrlimit(resource.RLIMIT_FSIZE)
except AttributeError:
    pass
else:
    try:
        resource.setrlimit(resource.RLIMIT_FSIZE, (too_big, max))
    except (OverflowError, ValueError):
        pass
    try:
        resource.setrlimit(resource.RLIMIT_FSIZE, (max, too_big))
    except (OverflowError, ValueError):
        pass
print("ResourceTest::test_fsize_toobig: ok")
"###);
    assert_output(&out, r###"ResourceTest::test_fsize_toobig: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/resource/resource_test__test_pagesize.py`.
#[test]
fn test_gen_behavior_std_libs_resource_resource_test__test_pagesize() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "resource"
# dimension = "behavior"
# case = "resource_test__test_pagesize"
# subject = "cpython.test_resource.ResourceTest.test_pagesize"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_resource.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_resource.py::ResourceTest::test_pagesize
"""Auto-ported test: ResourceTest::test_pagesize (CPython 3.12 oracle)."""


import contextlib
import sys
import unittest
from test import support
from test.support import import_helper
from test.support import os_helper
import time


resource = import_helper.import_module('resource')


# --- test body ---
pagesize = resource.getpagesize()

assert isinstance(pagesize, int)

assert pagesize >= 0
print("ResourceTest::test_pagesize: ok")
"###);
    assert_output(&out, r###"ResourceTest::test_pagesize: ok
"###);
}
