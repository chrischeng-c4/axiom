# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "runpy"
# dimension = "behavior"
# case = "execution_layer_test_case__test_run_module_code"
# subject = "cpython.test_runpy.ExecutionLayerTestCase.test_run_module_code"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_runpy.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_runpy.py::ExecutionLayerTestCase::test_run_module_code
"""Auto-ported test: ExecutionLayerTestCase::test_run_module_code (CPython 3.12 oracle)."""


import contextlib
import importlib.machinery, importlib.util
import os.path
import pathlib
import py_compile
import re
import signal
import subprocess
import sys
import tempfile
import textwrap
import unittest
import warnings
from test.support import no_tracing, verbose, requires_subprocess, requires_resource
from test.support.import_helper import forget, make_legacy_pyc, unload
from test.support.os_helper import create_empty_file, temp_dir, FakePath
from test.support.script_helper import make_script, make_zip_script
import runpy
from runpy import _run_code, _run_module_code, run_module, run_path


example_source = "# Check basic code execution\nresult = ['Top level assignment']\ndef f():\n    result.append('Lower level reference')\nf()\ndel f\n# Check the sys module\nimport sys\nrun_argv0 = sys.argv[0]\nrun_name_in_sys_modules = __name__ in sys.modules\nmodule_in_sys_modules = (run_name_in_sys_modules and\n                         globals() is sys.modules[__name__].__dict__)\n# Check nested operation\nimport runpy\nnested = runpy._run_module_code('x=1\\n', mod_name='<run>')\n"

implicit_namespace = {'__name__': None, '__file__': None, '__cached__': None, '__package__': None, '__doc__': None, '__spec__': None}

example_namespace = {'sys': sys, 'runpy': runpy, 'result': ['Top level assignment', 'Lower level reference'], 'run_argv0': sys.argv[0], 'run_name_in_sys_modules': False, 'module_in_sys_modules': False, 'nested': dict(implicit_namespace, x=1, __name__='<run>', __loader__=None)}

example_namespace.update(implicit_namespace)

class CodeExecutionMixin:
    CHECKED_SPEC_ATTRIBUTES = ['name', 'parent', 'origin', 'cached', 'has_location', 'submodule_search_locations']

    def assertNamespaceMatches(self, result_ns, expected_ns):
        """Check two namespaces match.

           Ignores any unspecified interpreter created names
        """
        result_ns = result_ns.copy()
        expected_ns = expected_ns.copy()
        for k in list(result_ns):
            if k.startswith('__') and k.endswith('__'):
                if k not in expected_ns:
                    result_ns.pop(k)
                if k not in expected_ns['nested']:
                    result_ns['nested'].pop(k)
        result_spec = result_ns.pop('__spec__')
        expected_spec = expected_ns.pop('__spec__')
        if expected_spec is None:
            self.assertIsNone(result_spec)
        else:
            if expected_spec.loader is not None:
                self.assertEqual(type(result_spec.loader), type(expected_spec.loader))
            for attr in self.CHECKED_SPEC_ATTRIBUTES:
                k = '__spec__.' + attr
                actual = (k, getattr(result_spec, attr))
                expected = (k, getattr(expected_spec, attr))
                self.assertEqual(actual, expected)
        self.assertEqual(set(result_ns), set(expected_ns))
        for k in result_ns:
            actual = (k, result_ns[k])
            expected = (k, expected_ns[k])
            self.assertEqual(actual, expected)

    def check_code_execution(self, create_namespace, expected_namespace):
        """Check that an interface runs the example code correctly

           First argument is a callable accepting the initial globals and
           using them to create the actual namespace
           Second argument is the expected result
        """
        sentinel = object()
        expected_ns = expected_namespace.copy()
        run_name = expected_ns['__name__']
        saved_argv0 = sys.argv[0]
        saved_mod = sys.modules.get(run_name, sentinel)
        result_ns = create_namespace(None)
        self.assertNamespaceMatches(result_ns, expected_ns)
        self.assertIs(sys.argv[0], saved_argv0)
        self.assertIs(sys.modules.get(run_name, sentinel), saved_mod)
        initial_ns = {'sentinel': sentinel}
        expected_ns['sentinel'] = sentinel
        result_ns = create_namespace(initial_ns)
        self.assertIsNot(result_ns, initial_ns)
        self.assertNamespaceMatches(result_ns, expected_ns)
        self.assertIs(sys.argv[0], saved_argv0)
        self.assertIs(sys.modules.get(run_name, sentinel), saved_mod)


# --- test body ---
CHECKED_SPEC_ATTRIBUTES = ['name', 'parent', 'origin', 'cached', 'has_location', 'submodule_search_locations']

def assertNamespaceMatches(result_ns, expected_ns):
    """Check two namespaces match.

           Ignores any unspecified interpreter created names
        """
    result_ns = result_ns.copy()
    expected_ns = expected_ns.copy()
    for k in list(result_ns):
        if k.startswith('__') and k.endswith('__'):
            if k not in expected_ns:
                result_ns.pop(k)
            if k not in expected_ns['nested']:
                result_ns['nested'].pop(k)
    result_spec = result_ns.pop('__spec__')
    expected_spec = expected_ns.pop('__spec__')
    if expected_spec is None:

        assert result_spec is None
    else:
        if expected_spec.loader is not None:

            assert type(result_spec.loader) == type(expected_spec.loader)
        for attr in CHECKED_SPEC_ATTRIBUTES:
            k = '__spec__.' + attr
            actual = (k, getattr(result_spec, attr))
            expected = (k, getattr(expected_spec, attr))

            assert actual == expected

    assert set(result_ns) == set(expected_ns)
    for k in result_ns:
        actual = (k, result_ns[k])
        expected = (k, expected_ns[k])

        assert actual == expected

def check_code_execution(create_namespace, expected_namespace):
    """Check that an interface runs the example code correctly

           First argument is a callable accepting the initial globals and
           using them to create the actual namespace
           Second argument is the expected result
        """
    sentinel = object()
    expected_ns = expected_namespace.copy()
    run_name = expected_ns['__name__']
    saved_argv0 = sys.argv[0]
    saved_mod = sys.modules.get(run_name, sentinel)
    result_ns = create_namespace(None)
    assertNamespaceMatches(result_ns, expected_ns)

    assert sys.argv[0] is saved_argv0

    assert sys.modules.get(run_name, sentinel) is saved_mod
    initial_ns = {'sentinel': sentinel}
    expected_ns['sentinel'] = sentinel
    result_ns = create_namespace(initial_ns)

    assert result_ns is not initial_ns
    assertNamespaceMatches(result_ns, expected_ns)

    assert sys.argv[0] is saved_argv0

    assert sys.modules.get(run_name, sentinel) is saved_mod
mod_name = '<Nonsense>'
mod_fname = 'Some other nonsense'
mod_loader = "Now you're just being silly"
mod_package = ''
mod_spec = importlib.machinery.ModuleSpec(mod_name, origin=mod_fname, loader=mod_loader)
expected_ns = example_namespace.copy()
expected_ns.update({'__name__': mod_name, '__file__': mod_fname, '__loader__': mod_loader, '__package__': mod_package, '__spec__': mod_spec, 'run_argv0': mod_fname, 'run_name_in_sys_modules': True, 'module_in_sys_modules': True})

def create_ns(init_globals):
    return _run_module_code(example_source, init_globals, mod_name, mod_spec)
check_code_execution(create_ns, expected_ns)
print("ExecutionLayerTestCase::test_run_module_code: ok")
