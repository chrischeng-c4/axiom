# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipimport_support"
# dimension = "behavior"
# case = "zip_support_tests__test_doctest_issue4197"
# subject = "cpython.test_zipimport_support.ZipSupportTests.test_doctest_issue4197"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_zipimport_support.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_zipimport_support.py::ZipSupportTests::test_doctest_issue4197
"""Auto-ported test: ZipSupportTests::test_doctest_issue4197 (CPython 3.12 oracle)."""


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
test_src = inspect.getsource(test_doctest)
test_src = test_src.replace('from test.test_doctest import test_doctest', 'import test_zipped_doctest as test_doctest')
test_src = test_src.replace('test.test_doctest.test_doctest', 'test_zipped_doctest')
test_src = test_src.replace('test.test_doctest.sample_doctest', 'sample_zipped_doctest')
sample_sources = {}
for mod in [sample_doctest, sample_doctest_no_doctests, sample_doctest_no_docstrings]:
    src = inspect.getsource(mod)
    src = src.replace('test.test_doctest.test_doctest', 'test_zipped_doctest')
    mod_name = mod.__name__.split('.')[-1]
    mod_name = mod_name.replace('sample_', 'sample_zipped_')
    sample_sources[mod_name] = src
with os_helper.temp_dir() as d:
    script_name = make_script(d, 'test_zipped_doctest', test_src)
    zip_name, run_name = make_zip_script(d, 'test_zip', script_name)
    with zipfile.ZipFile(zip_name, 'a') as z:
        for mod_name, src in sample_sources.items():
            z.writestr(mod_name + '.py', src)
    if verbose:
        with zipfile.ZipFile(zip_name, 'r') as zip_file:
            print('Contents of %r:' % zip_name)
            zip_file.printdir()
    os.remove(script_name)
    sys.path.insert(0, zip_name)
    import test_zipped_doctest
    try:
        known_good_tests = [test_zipped_doctest.SampleClass, test_zipped_doctest.SampleClass.NestedClass, test_zipped_doctest.SampleClass.NestedClass.__init__, test_zipped_doctest.SampleClass.__init__, test_zipped_doctest.SampleClass.a_classmethod, test_zipped_doctest.SampleClass.a_property, test_zipped_doctest.SampleClass.a_staticmethod, test_zipped_doctest.SampleClass.double, test_zipped_doctest.SampleClass.get, test_zipped_doctest.SampleNewStyleClass, test_zipped_doctest.SampleNewStyleClass.__init__, test_zipped_doctest.SampleNewStyleClass.double, test_zipped_doctest.SampleNewStyleClass.get, test_zipped_doctest.sample_func, test_zipped_doctest.test_DocTest, test_zipped_doctest.test_DocTestParser, test_zipped_doctest.test_DocTestRunner.basics, test_zipped_doctest.test_DocTestRunner.exceptions, test_zipped_doctest.test_DocTestRunner.option_directives, test_zipped_doctest.test_DocTestRunner.optionflags, test_zipped_doctest.test_DocTestRunner.verbose_flag, test_zipped_doctest.test_Example, test_zipped_doctest.test_debug, test_zipped_doctest.test_testsource, test_zipped_doctest.test_trailing_space_in_test, test_zipped_doctest.test_DocTestSuite, test_zipped_doctest.test_DocTestFinder]
        fail_due_to_missing_data_files = [test_zipped_doctest.test_DocFileSuite, test_zipped_doctest.test_testfile, test_zipped_doctest.test_unittest_reportflags]
        for obj in known_good_tests:
            _run_object_doctest(obj, test_zipped_doctest)
    finally:
        del sys.modules['test_zipped_doctest']
print("ZipSupportTests::test_doctest_issue4197: ok")
