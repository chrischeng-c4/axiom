# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "modulefinder"
# dimension = "behavior"
# case = "module_finder_test__test_replace_paths"
# subject = "cpython.test_modulefinder.ModuleFinderTest.test_replace_paths"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_modulefinder.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_modulefinder.py::ModuleFinderTest::test_replace_paths
"""Auto-ported test: ModuleFinderTest::test_replace_paths (CPython 3.12 oracle)."""


import os
import errno
import importlib.machinery
import py_compile
import shutil
import unittest
import tempfile
from test import support
import modulefinder


maybe_test = ['a.module', ['a', 'a.module', 'sys', 'b'], ['c'], ['b.something'], 'a/__init__.py\na/module.py\n                                from b import something\n                                from c import something\nb/__init__.py\n                                from sys import *\n']

maybe_test_new = ['a.module', ['a', 'a.module', 'sys', 'b', '__future__'], ['c'], ['b.something'], 'a/__init__.py\na/module.py\n                                from b import something\n                                from c import something\nb/__init__.py\n                                from __future__ import absolute_import\n                                from sys import *\n']

package_test = ['a.module', ['a', 'a.b', 'a.c', 'a.module', 'mymodule', 'sys'], ['blahblah', 'c'], [], 'mymodule.py\na/__init__.py\n                                import blahblah\n                                from a import b\n                                import c\na/module.py\n                                import sys\n                                from a import b as x\n                                from a.c import sillyname\na/b.py\na/c.py\n                                from a.module import x\n                                import mymodule as sillyname\n                                from sys import version_info\n']

absolute_import_test = ['a.module', ['a', 'a.module', 'b', 'b.x', 'b.y', 'b.z', '__future__', 'sys', 'gc'], ['blahblah', 'z'], [], 'mymodule.py\na/__init__.py\na/module.py\n                                from __future__ import absolute_import\n                                import sys # sys\n                                import blahblah # fails\n                                import gc # gc\n                                import b.x # b.x\n                                from b import y # b.y\n                                from b.z import * # b.z.*\na/gc.py\na/sys.py\n                                import mymodule\na/b/__init__.py\na/b/x.py\na/b/y.py\na/b/z.py\nb/__init__.py\n                                import z\nb/unused.py\nb/x.py\nb/y.py\nb/z.py\n']

relative_import_test = ['a.module', ['__future__', 'a', 'a.module', 'a.b', 'a.b.y', 'a.b.z', 'a.b.c', 'a.b.c.moduleC', 'a.b.c.d', 'a.b.c.e', 'a.b.x', 'gc'], [], [], 'mymodule.py\na/__init__.py\n                                from .b import y, z # a.b.y, a.b.z\na/module.py\n                                from __future__ import absolute_import # __future__\n                                import gc # gc\na/gc.py\na/sys.py\na/b/__init__.py\n                                from ..b import x # a.b.x\n                                #from a.b.c import moduleC\n                                from .c import moduleC # a.b.moduleC\na/b/x.py\na/b/y.py\na/b/z.py\na/b/g.py\na/b/c/__init__.py\n                                from ..c import e # a.b.c.e\na/b/c/moduleC.py\n                                from ..c import d # a.b.c.d\na/b/c/d.py\na/b/c/e.py\na/b/c/x.py\n']

relative_import_test_2 = ['a.module', ['a', 'a.module', 'a.sys', 'a.b', 'a.b.y', 'a.b.z', 'a.b.c', 'a.b.c.d', 'a.b.c.e', 'a.b.c.moduleC', 'a.b.c.f', 'a.b.x', 'a.another'], [], [], 'mymodule.py\na/__init__.py\n                                from . import sys # a.sys\na/another.py\na/module.py\n                                from .b import y, z # a.b.y, a.b.z\na/gc.py\na/sys.py\na/b/__init__.py\n                                from .c import moduleC # a.b.c.moduleC\n                                from .c import d # a.b.c.d\na/b/x.py\na/b/y.py\na/b/z.py\na/b/c/__init__.py\n                                from . import e # a.b.c.e\na/b/c/moduleC.py\n                                #\n                                from . import f   # a.b.c.f\n                                from .. import x  # a.b.x\n                                from ... import another # a.another\na/b/c/d.py\na/b/c/e.py\na/b/c/f.py\n']

relative_import_test_3 = ['a.module', ['a', 'a.module'], ['a.bar'], [], 'a/__init__.py\n                                def foo(): pass\na/module.py\n                                from . import foo\n                                from . import bar\n']

relative_import_test_4 = ['a.module', ['a', 'a.module'], [], [], 'a/__init__.py\n                                def foo(): pass\na/module.py\n                                from . import *\n']

bytecode_test = ['a', ['a'], [], [], '']

syntax_error_test = ['a.module', ['a', 'a.module', 'b'], ['b.module'], [], 'a/__init__.py\na/module.py\n                                import b.module\nb/__init__.py\nb/module.py\n                                ?  # SyntaxError: invalid syntax\n']

same_name_as_bad_test = ['a.module', ['a', 'a.module', 'b', 'b.c'], ['c'], [], 'a/__init__.py\na/module.py\n                                import c\n                                from b import c\nb/__init__.py\nb/c.py\n']

coding_default_utf8_test = ['a_utf8', ['a_utf8', 'b_utf8'], [], [], "a_utf8.py\n                                # use the default of utf8\n                                print('Unicode test A code point 2090 ₐ that is not valid in cp1252')\n                                import b_utf8\nb_utf8.py\n                                # use the default of utf8\n                                print('Unicode test B code point 2090 ₐ that is not valid in cp1252')\n"]

coding_explicit_utf8_test = ['a_utf8', ['a_utf8', 'b_utf8'], [], [], "a_utf8.py\n                                # coding=utf8\n                                print('Unicode test A code point 2090 ₐ that is not valid in cp1252')\n                                import b_utf8\nb_utf8.py\n                                # use the default of utf8\n                                print('Unicode test B code point 2090 ₐ that is not valid in cp1252')\n"]

coding_explicit_cp1252_test = ['a_cp1252', ['a_cp1252', 'b_utf8'], [], [], b"a_cp1252.py\n                                # coding=cp1252\n                                # 0xe2 is not allowed in utf8\n                                print('CP1252 test P\xe2t\xe9')\n                                import b_utf8\n" + "b_utf8.py\n                                # use the default of utf8\n                                print('Unicode test A code point 2090 ₐ that is not valid in cp1252')\n".encode('utf-8')]

def open_file(path):
    dirname = os.path.dirname(path)
    try:
        os.makedirs(dirname)
    except OSError as e:
        if e.errno != errno.EEXIST:
            raise
    return open(path, 'wb')

def create_package(test_dir, source):
    ofi = None
    try:
        for line in source.splitlines():
            if type(line) != bytes:
                line = line.encode('utf-8')
            if line.startswith(b' ') or line.startswith(b'\t'):
                ofi.write(line.strip() + b'\n')
            else:
                if ofi:
                    ofi.close()
                if type(line) == bytes:
                    line = line.decode('utf-8')
                ofi = open_file(os.path.join(test_dir, line.strip()))
    finally:
        if ofi:
            ofi.close()


# --- test body ---
def _do_test(info, report=False, debug=0, replace_paths=[], modulefinder_class=modulefinder.ModuleFinder):
    import_this, modules, missing, maybe_missing, source = info
    create_package(self_test_dir, source)
    mf = modulefinder_class(path=self_test_path, debug=debug, replace_paths=replace_paths)
    mf.import_hook(import_this)
    if report:
        mf.report()
    modules = sorted(set(modules))
    found = sorted(mf.modules)

    assert found == modules
    bad, maybe = mf.any_missing_maybe()

    assert bad == missing

    assert maybe == maybe_missing
self_test_dir = tempfile.mkdtemp()
self_test_path = [self_test_dir, os.path.dirname(tempfile.__file__)]
old_path = os.path.join(self_test_dir, 'a', 'module.py')
new_path = os.path.join(self_test_dir, 'a', 'spam.py')
with support.captured_stdout() as output:
    _do_test(maybe_test, debug=2, replace_paths=[(old_path, new_path)])
output = output.getvalue()
expected = 'co_filename %r changed to %r' % (old_path, new_path)

assert expected in output
print("ModuleFinderTest::test_replace_paths: ok")
