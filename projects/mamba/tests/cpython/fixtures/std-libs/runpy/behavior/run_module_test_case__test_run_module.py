# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "runpy"
# dimension = "behavior"
# case = "run_module_test_case__test_run_module"
# subject = "cpython.test_runpy.RunModuleTestCase.test_run_module"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_runpy.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_runpy.py::RunModuleTestCase::test_run_module
"""Auto-ported test: RunModuleTestCase::test_run_module (CPython 3.12 oracle)."""


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

def _add_pkg_dir(pkg_dir, namespace=False):
    os.mkdir(pkg_dir)
    if namespace:
        return None
    pkg_fname = os.path.join(pkg_dir, '__init__.py')
    create_empty_file(pkg_fname)
    return pkg_fname

def _add_relative_modules(base_dir, source, depth):
    if depth <= 1:
        raise ValueError('Relative module test needs depth > 1')
    pkg_name = '__runpy_pkg__'
    module_dir = base_dir
    for i in range(depth):
        parent_dir = module_dir
        module_dir = os.path.join(module_dir, pkg_name)
    sibling_fname = os.path.join(module_dir, 'sibling.py')
    create_empty_file(sibling_fname)
    if verbose > 1:
        print('  Added sibling module:', sibling_fname)
    uncle_dir = os.path.join(parent_dir, 'uncle')
    _add_pkg_dir(uncle_dir)
    if verbose > 1:
        print('  Added uncle package:', uncle_dir)
    cousin_dir = os.path.join(uncle_dir, 'cousin')
    _add_pkg_dir(cousin_dir)
    if verbose > 1:
        print('  Added cousin package:', cousin_dir)
    nephew_fname = os.path.join(cousin_dir, 'nephew.py')
    create_empty_file(nephew_fname)
    if verbose > 1:
        print('  Added nephew module:', nephew_fname)

def _check_module(depth, alter_sys=False, *, namespace=False, parent_namespaces=False):
    pkg_dir, mod_fname, mod_name, mod_spec = _make_pkg(example_source, depth, namespace=namespace, parent_namespaces=parent_namespaces)
    forget(mod_name)
    expected_ns = example_namespace.copy()
    expected_ns.update({'__name__': mod_name, '__file__': mod_fname, '__cached__': mod_spec.cached, '__package__': mod_name.rpartition('.')[0], '__spec__': mod_spec})
    if alter_sys:
        expected_ns.update({'run_argv0': mod_fname, 'run_name_in_sys_modules': True, 'module_in_sys_modules': True})

    def create_ns(init_globals):
        return run_module(mod_name, init_globals, alter_sys=alter_sys)
    try:
        if verbose > 1:
            print('Running from source:', mod_name)
        check_code_execution(create_ns, expected_ns)
        importlib.invalidate_caches()
        __import__(mod_name)
        os.remove(mod_fname)
        if not sys.dont_write_bytecode:
            make_legacy_pyc(mod_fname)
            unload(mod_name)
            importlib.invalidate_caches()
            if verbose > 1:
                print('Running from compiled:', mod_name)
            _fix_ns_for_legacy_pyc(expected_ns, alter_sys)
            check_code_execution(create_ns, expected_ns)
    finally:
        _del_pkg(pkg_dir)
    if verbose > 1:
        print('Module executed successfully')

def _check_package(depth, alter_sys=False, *, namespace=False, parent_namespaces=False):
    pkg_dir, mod_fname, mod_name, mod_spec = _make_pkg(example_source, depth, '__main__', namespace=namespace, parent_namespaces=parent_namespaces)
    pkg_name = mod_name.rpartition('.')[0]
    forget(mod_name)
    expected_ns = example_namespace.copy()
    expected_ns.update({'__name__': mod_name, '__file__': mod_fname, '__cached__': importlib.util.cache_from_source(mod_fname), '__package__': pkg_name, '__spec__': mod_spec})
    if alter_sys:
        expected_ns.update({'run_argv0': mod_fname, 'run_name_in_sys_modules': True, 'module_in_sys_modules': True})

    def create_ns(init_globals):
        return run_module(pkg_name, init_globals, alter_sys=alter_sys)
    try:
        if verbose > 1:
            print('Running from source:', pkg_name)
        check_code_execution(create_ns, expected_ns)
        importlib.invalidate_caches()
        __import__(mod_name)
        os.remove(mod_fname)
        if not sys.dont_write_bytecode:
            make_legacy_pyc(mod_fname)
            unload(mod_name)
            if verbose > 1:
                print('Running from compiled:', pkg_name)
            importlib.invalidate_caches()
            _fix_ns_for_legacy_pyc(expected_ns, alter_sys)
            check_code_execution(create_ns, expected_ns)
    finally:
        _del_pkg(pkg_dir)
    if verbose > 1:
        print('Package executed successfully')

def _check_relative_imports(depth, run_name=None):
    contents = '\\\nfrom __future__ import absolute_import\nfrom . import sibling\nfrom ..uncle.cousin import nephew\n'
    pkg_dir, mod_fname, mod_name, mod_spec = _make_pkg(contents, depth)
    if run_name is None:
        expected_name = mod_name
    else:
        expected_name = run_name
    try:
        _add_relative_modules(pkg_dir, contents, depth)
        pkg_name = mod_name.rpartition('.')[0]
        if verbose > 1:
            print('Running from source:', mod_name)
        d1 = run_module(mod_name, run_name=run_name)

        assert d1['__name__'] == expected_name

        assert d1['__package__'] == pkg_name

        assert 'sibling' in d1

        assert 'nephew' in d1
        del d1
        importlib.invalidate_caches()
        __import__(mod_name)
        os.remove(mod_fname)
        if not sys.dont_write_bytecode:
            make_legacy_pyc(mod_fname)
            unload(mod_name)
            if verbose > 1:
                print('Running from compiled:', mod_name)
            importlib.invalidate_caches()
            d2 = run_module(mod_name, run_name=run_name)

            assert d2['__name__'] == expected_name

            assert d2['__package__'] == pkg_name

            assert 'sibling' in d2

            assert 'nephew' in d2
            del d2
    finally:
        _del_pkg(pkg_dir)
    if verbose > 1:
        print('Module executed successfully')

def _del_pkg(top):
    for entry in list(sys.modules):
        if entry.startswith('__runpy_pkg__'):
            del sys.modules[entry]
    if verbose > 1:
        print('  Removed sys.modules entries')
    del sys.path[0]
    if verbose > 1:
        print('  Removed sys.path entry')
    for root, dirs, files in os.walk(top, topdown=False):
        for name in files:
            try:
                os.remove(os.path.join(root, name))
            except OSError as ex:
                if verbose > 1:
                    print(ex)
        for name in dirs:
            fullname = os.path.join(root, name)
            try:
                os.rmdir(fullname)
            except OSError as ex:
                if verbose > 1:
                    print(ex)
    try:
        os.rmdir(top)
        if verbose > 1:
            print('  Removed package tree')
    except OSError as ex:
        if verbose > 1:
            print(ex)

def _fix_ns_for_legacy_pyc(ns, alter_sys):
    char_to_add = 'c'
    ns['__file__'] += char_to_add
    ns['__cached__'] = ns['__file__']
    spec = ns['__spec__']
    new_spec = importlib.util.spec_from_file_location(spec.name, ns['__file__'])
    ns['__spec__'] = new_spec
    if alter_sys:
        ns['run_argv0'] += char_to_add

def _make_pkg(source, depth, mod_base='runpy_test', *, namespace=False, parent_namespaces=False):
    if (namespace or parent_namespaces) and (not depth):
        raise RuntimeError("Can't mark top level module as a namespace package")
    pkg_name = '__runpy_pkg__'
    test_fname = mod_base + os.extsep + 'py'
    pkg_dir = sub_dir = os.path.realpath(tempfile.mkdtemp())
    if verbose > 1:
        print('  Package tree in:', sub_dir)
    sys.path.insert(0, pkg_dir)
    if verbose > 1:
        print('  Updated sys.path:', sys.path[0])
    if depth:
        namespace_flags = [parent_namespaces] * depth
        namespace_flags[-1] = namespace
        for namespace_flag in namespace_flags:
            sub_dir = os.path.join(sub_dir, pkg_name)
            pkg_fname = _add_pkg_dir(sub_dir, namespace_flag)
            if verbose > 1:
                print('  Next level in:', sub_dir)
            if verbose > 1:
                print('  Created:', pkg_fname)
    mod_fname = os.path.join(sub_dir, test_fname)
    with open(mod_fname, 'w') as mod_file:
        mod_file.write(source)
    if verbose > 1:
        print('  Created:', mod_fname)
    mod_name = (pkg_name + '.') * depth + mod_base
    mod_spec = importlib.util.spec_from_file_location(mod_name, mod_fname)
    return (pkg_dir, mod_fname, mod_name, mod_spec)

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

def expect_import_error(mod_name):
    try:
        run_module(mod_name)
    except ImportError:
        pass
    else:

        raise AssertionError('Expected import error for ' + mod_name)
for depth in range(4):
    if verbose > 1:
        print('Testing package depth:', depth)
    _check_module(depth)
print("RunModuleTestCase::test_run_module: ok")
