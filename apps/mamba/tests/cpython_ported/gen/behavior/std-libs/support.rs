use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/support/test_support__test_captured_stderr.py`.
#[test]
fn test_gen_behavior_std_libs_support_test_support__test_captured_stderr() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "support"
# dimension = "behavior"
# case = "test_support__test_captured_stderr"
# subject = "cpython.test_support.TestSupport.test_captured_stderr"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_support.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_support.py::TestSupport::test_captured_stderr
"""Auto-ported test: TestSupport::test_captured_stderr (CPython 3.12 oracle)."""


import errno
import importlib
import io
import os
import shutil
import socket
import stat
import subprocess
import sys
import sysconfig
import tempfile
import textwrap
import unittest
import warnings
from test import support
from test.support import import_helper
from test.support import os_helper
from test.support import script_helper
from test.support import socket_helper
from test.support import warnings_helper


TESTFN = os_helper.TESTFN


# --- test body ---
with support.captured_stderr() as stderr:
    print('hello', file=sys.stderr)

assert stderr.getvalue() == 'hello\n'
print("TestSupport::test_captured_stderr: ok")
"###);
    assert_output(&out, r###"TestSupport::test_captured_stderr: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/support/test_support__test_captured_stdout.py`.
#[test]
fn test_gen_behavior_std_libs_support_test_support__test_captured_stdout() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "support"
# dimension = "behavior"
# case = "test_support__test_captured_stdout"
# subject = "cpython.test_support.TestSupport.test_captured_stdout"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_support.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_support.py::TestSupport::test_captured_stdout
"""Auto-ported test: TestSupport::test_captured_stdout (CPython 3.12 oracle)."""


import errno
import importlib
import io
import os
import shutil
import socket
import stat
import subprocess
import sys
import sysconfig
import tempfile
import textwrap
import unittest
import warnings
from test import support
from test.support import import_helper
from test.support import os_helper
from test.support import script_helper
from test.support import socket_helper
from test.support import warnings_helper


TESTFN = os_helper.TESTFN


# --- test body ---
with support.captured_stdout() as stdout:
    print('hello')

assert stdout.getvalue() == 'hello\n'
print("TestSupport::test_captured_stdout: ok")
"###);
    assert_output(&out, r###"TestSupport::test_captured_stdout: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/support/test_support__test_clean_import.py`.
#[test]
fn test_gen_behavior_std_libs_support_test_support__test_clean_import() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "support"
# dimension = "behavior"
# case = "test_support__test_clean_import"
# subject = "cpython.test_support.TestSupport.test_CleanImport"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_support.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_support.py::TestSupport::test_CleanImport
"""Auto-ported test: TestSupport::test_CleanImport (CPython 3.12 oracle)."""


import errno
import importlib
import io
import os
import shutil
import socket
import stat
import subprocess
import sys
import sysconfig
import tempfile
import textwrap
import unittest
import warnings
from test import support
from test.support import import_helper
from test.support import os_helper
from test.support import script_helper
from test.support import socket_helper
from test.support import warnings_helper


TESTFN = os_helper.TESTFN


# --- test body ---
import importlib
with import_helper.CleanImport('pprint'):
    importlib.import_module('pprint')
print("TestSupport::test_CleanImport: ok")
"###);
    assert_output(&out, r###"TestSupport::test_CleanImport: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/support/test_support__test_gc_collect.py`.
#[test]
fn test_gen_behavior_std_libs_support_test_support__test_gc_collect() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "support"
# dimension = "behavior"
# case = "test_support__test_gc_collect"
# subject = "cpython.test_support.TestSupport.test_gc_collect"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_support.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_support.py::TestSupport::test_gc_collect
"""Auto-ported test: TestSupport::test_gc_collect (CPython 3.12 oracle)."""


import errno
import importlib
import io
import os
import shutil
import socket
import stat
import subprocess
import sys
import sysconfig
import tempfile
import textwrap
import unittest
import warnings
from test import support
from test.support import import_helper
from test.support import os_helper
from test.support import script_helper
from test.support import socket_helper
from test.support import warnings_helper


TESTFN = os_helper.TESTFN


# --- test body ---
support.gc_collect()
print("TestSupport::test_gc_collect: ok")
"###);
    assert_output(&out, r###"TestSupport::test_gc_collect: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/support/test_support__test_get_recursion_depth.py`.
#[test]
fn test_gen_behavior_std_libs_support_test_support__test_get_recursion_depth() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "support"
# dimension = "behavior"
# case = "test_support__test_get_recursion_depth"
# subject = "cpython.test_support.TestSupport.test_get_recursion_depth"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_support.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_support.py::TestSupport::test_get_recursion_depth
"""Auto-ported test: TestSupport::test_get_recursion_depth (CPython 3.12 oracle)."""


import errno
import importlib
import io
import os
import shutil
import socket
import stat
import subprocess
import sys
import sysconfig
import tempfile
import textwrap
import unittest
import warnings
from test import support
from test.support import import_helper
from test.support import os_helper
from test.support import script_helper
from test.support import socket_helper
from test.support import warnings_helper


TESTFN = os_helper.TESTFN


# --- test body ---
code = textwrap.dedent('\n            from test import support\n            import sys\n\n            def check(cond):\n                if not cond:\n                    raise AssertionError("test failed")\n\n            # depth 1\n            check(support.get_recursion_depth() == 1)\n\n            # depth 2\n            def test_func():\n                check(support.get_recursion_depth() == 2)\n            test_func()\n\n            def test_recursive(depth, limit):\n                if depth >= limit:\n                    # cannot call get_recursion_depth() at this depth,\n                    # it can raise RecursionError\n                    return\n                get_depth = support.get_recursion_depth()\n                print(f"test_recursive: {depth}/{limit}: "\n                      f"get_recursion_depth() says {get_depth}")\n                check(get_depth == depth)\n                test_recursive(depth + 1, limit)\n\n            # depth up to 25\n            with support.infinite_recursion(max_depth=25):\n                limit = sys.getrecursionlimit()\n                print(f"test with sys.getrecursionlimit()={limit}")\n                test_recursive(2, limit)\n\n            # depth up to 500\n            with support.infinite_recursion(max_depth=500):\n                limit = sys.getrecursionlimit()\n                print(f"test with sys.getrecursionlimit()={limit}")\n                test_recursive(2, limit)\n        ')
script_helper.assert_python_ok('-c', code)
print("TestSupport::test_get_recursion_depth: ok")
"###);
    assert_output(&out, r###"TestSupport::test_get_recursion_depth: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/support/test_support__test_import_fresh_module.py`.
#[test]
fn test_gen_behavior_std_libs_support_test_support__test_import_fresh_module() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "support"
# dimension = "behavior"
# case = "test_support__test_import_fresh_module"
# subject = "cpython.test_support.TestSupport.test_import_fresh_module"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_support.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_support.py::TestSupport::test_import_fresh_module
"""Auto-ported test: TestSupport::test_import_fresh_module (CPython 3.12 oracle)."""


import errno
import importlib
import io
import os
import shutil
import socket
import stat
import subprocess
import sys
import sysconfig
import tempfile
import textwrap
import unittest
import warnings
from test import support
from test.support import import_helper
from test.support import os_helper
from test.support import script_helper
from test.support import socket_helper
from test.support import warnings_helper


TESTFN = os_helper.TESTFN


# --- test body ---
import_helper.import_fresh_module('ftplib')
print("TestSupport::test_import_fresh_module: ok")
"###);
    assert_output(&out, r###"TestSupport::test_import_fresh_module: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/support/test_support__test_temp_dir_forked_child.py`.
#[test]
fn test_gen_behavior_std_libs_support_test_support__test_temp_dir_forked_child() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "support"
# dimension = "behavior"
# case = "test_support__test_temp_dir_forked_child"
# subject = "cpython.test_support.TestSupport.test_temp_dir__forked_child"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_support.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_support.py::TestSupport::test_temp_dir__forked_child
"""Auto-ported test: TestSupport::test_temp_dir__forked_child (CPython 3.12 oracle)."""


import errno
import importlib
import io
import os
import shutil
import socket
import stat
import subprocess
import sys
import sysconfig
import tempfile
import textwrap
import unittest
import warnings
from test import support
from test.support import import_helper
from test.support import os_helper
from test.support import script_helper
from test.support import socket_helper
from test.support import warnings_helper


TESTFN = os_helper.TESTFN


# --- test body ---
"""Test that a forked child process does not remove the directory."""
script_helper.assert_python_ok('-c', textwrap.dedent('\n            import os\n            from test import support\n            from test.support import os_helper\n            with os_helper.temp_cwd() as temp_path:\n                pid = os.fork()\n                if pid != 0:\n                    # parent process\n\n                    # wait for the child to terminate\n                    support.wait_process(pid, exitcode=0)\n\n                    # Make sure that temp_path is still present. When the child\n                    # process leaves the \'temp_cwd\'-context, the __exit__()-\n                    # method of the context must not remove the temporary\n                    # directory.\n                    if not os.path.isdir(temp_path):\n                        raise AssertionError("Child removed temp_path.")\n        '))
print("TestSupport::test_temp_dir__forked_child: ok")
"###);
    assert_output(&out, r###"TestSupport::test_temp_dir__forked_child: ok
"###);
}
