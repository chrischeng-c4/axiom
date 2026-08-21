# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pkgutil"
# dimension = "behavior"
# case = "nested_namespace_package_test__test_nested"
# subject = "cpython.test_pkgutil.NestedNamespacePackageTest.test_nested"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_pkgutil.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_pkgutil.py::NestedNamespacePackageTest::test_nested
"""Auto-ported test: NestedNamespacePackageTest::test_nested (CPython 3.12 oracle)."""


from pathlib import Path
from test.support.import_helper import unload, CleanImport
from test.support.warnings_helper import check_warnings, ignore_warnings
import unittest
import sys
import importlib
from importlib.util import spec_from_file_location
import pkgutil
import os
import os.path
import tempfile
import shutil
import zipfile
from test.support.import_helper import DirsOnSysPath
from test.support.os_helper import FakePath
from test.test_importlib.util import uncache


def tearDownModule():
    import zipimport
    import importlib
    zipimport._zip_directory_cache.clear()
    importlib.invalidate_caches()


# --- test body ---
def create_module(name, contents):
    base, final = name.rsplit('.', 1)
    base_path = os.path.join(self_basedir, base.replace('.', os.path.sep))
    os.makedirs(base_path, exist_ok=True)
    with open(os.path.join(base_path, final + '.py'), 'w') as f:
        f.write(contents)
self_basedir = tempfile.mkdtemp()
self_old_path = sys.path[:]
pkgutil_boilerplate = 'import pkgutil; __path__ = pkgutil.extend_path(__path__, __name__)'
create_module('a.pkg.__init__', pkgutil_boilerplate)
create_module('b.pkg.__init__', pkgutil_boilerplate)
create_module('a.pkg.subpkg.__init__', pkgutil_boilerplate)
create_module('b.pkg.subpkg.__init__', pkgutil_boilerplate)
create_module('a.pkg.subpkg.c', 'c = 1')
create_module('b.pkg.subpkg.d', 'd = 2')
sys.path.insert(0, os.path.join(self_basedir, 'a'))
sys.path.insert(0, os.path.join(self_basedir, 'b'))
import pkg
pass

assert len(pkg.__path__) == 2
import pkg.subpkg
pass

assert len(pkg.subpkg.__path__) == 2
from pkg.subpkg.c import c
from pkg.subpkg.d import d

assert c == 1

assert d == 2
print("NestedNamespacePackageTest::test_nested: ok")
