# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipimport_support"
# dimension = "behavior"
# case = "zip_support_tests__test_pdb_issue4201"
# subject = "cpython.test_zipimport_support.ZipSupportTests.test_pdb_issue4201"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_zipimport_support.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_zipimport_support.py::ZipSupportTests::test_pdb_issue4201
"""Auto-ported test: ZipSupportTests::test_pdb_issue4201 (CPython 3.12 oracle)."""


import test.support
import os
import os.path
import sys
import textwrap
import zipfile
import zipimport
import doctest
import inspect
import linecache
import unittest
from test.support import os_helper
from test.support.script_helper import spawn_python, kill_python, assert_python_ok, make_script, make_zip_script
from test.test_doctest import test_doctest, sample_doctest, sample_doctest_no_doctests, sample_doctest_no_docstrings


verbose = test.support.verbose

def _run_object_doctest(obj, module):
    finder = doctest.DocTestFinder(verbose=verbose, recurse=False)
    runner = doctest.DocTestRunner(verbose=verbose)
    try:
        name = '%s.%s' % (obj.__module__, obj.__qualname__)
    except AttributeError:
        name = module.__name__
    for example in finder.find(obj, name, module):
        runner.run(example)
    f, t = (runner.failures, runner.tries)
    if f:
        raise test.support.TestFailed('%d of %d doctests failed' % (f, t))
    if verbose:
        print('doctest (%s) ... %d tests with zero failures' % (module.__name__, t))
    return (f, t)

def tearDownModule():
    test.support.reap_children()


# --- test body ---
linecache.clearcache()
zipimport._zip_directory_cache.clear()
self_path = sys.path[:]
self_meta_path = sys.meta_path[:]
self_path_hooks = sys.path_hooks[:]
sys.path_importer_cache.clear()
test_src = textwrap.dedent('                    def f():\n                        pass\n\n                    import pdb\n                    pdb.Pdb(nosigint=True).runcall(f)\n                    ')
with os_helper.temp_dir() as d:
    script_name = make_script(d, 'script', test_src)
    p = spawn_python(script_name)
    p.stdin.write(b'l\n')
    data = kill_python(p)

    assert os.path.normcase(script_name.encode('utf-8')) in data
    zip_name, run_name = make_zip_script(d, 'test_zip', script_name, '__main__.py')
    p = spawn_python(zip_name)
    p.stdin.write(b'l\n')
    data = kill_python(p)

    assert os.path.normcase(run_name.encode('utf-8')) in data
print("ZipSupportTests::test_pdb_issue4201: ok")
